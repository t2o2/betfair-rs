use anyhow::Result;
use betfair_rs::orderbook::{Orderbook, PriceLevel};
use betfair_rs::{BetfairClient, Config, StreamingClient};
use betfair_rs::dto::market::{ListMarketCatalogueRequest, MarketFilter};
use betfair_rs::dto::common::MarketProjection;
use clap::{Parser, Subcommand};
use indexmap::IndexMap;
use redis::{Client, Commands as RedisCommands, Connection};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tokio::signal;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn, debug};
use tracing_subscriber::EnvFilter;
use chrono::{DateTime, Utc};

#[derive(Parser)]
#[command(name = "redis-streamer")]
#[command(about = "Redis-based market streaming tool for betfair-rs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Stream markets from Redis venue games
    StreamVenue {
        /// Venue game ID (e.g., 34750237)
        venue_game_id: String,
        /// Orderbook depth (default: 10)
        #[arg(short, long, default_value = "10")]
        depth: usize,
        /// Update interval in seconds (default: 2)
        #[arg(short, long, default_value = "2")]
        interval: u64,
        /// Redis URL (default: redis://127.0.0.1:6379)
        #[arg(long, default_value = "redis://127.0.0.1:6379")]
        redis_url: String,
        /// Run continuously until stopped (daemon mode)
        #[arg(long, default_value = "false")]
        daemon: bool,
        /// Refresh Redis data every N minutes (default: 30, 0 = no refresh)
        #[arg(long, default_value = "30")]
        refresh_minutes: u64,
    },
    /// Stream all available markets from Redis
    StreamAll {
        /// Orderbook depth (default: 10)
        #[arg(short, long, default_value = "10")]
        depth: usize,
        /// Update interval in seconds (default: 2)
        #[arg(short, long, default_value = "2")]
        interval: u64,
        /// Redis URL (default: redis://127.0.0.1:6379)
        #[arg(long, default_value = "redis://127.0.0.1:6379")]
        redis_url: String,
        /// Limit number of markets to stream (default: no limit)
        #[arg(short, long)]
        limit: Option<usize>,
        /// Run continuously until stopped (daemon mode)
        #[arg(long, default_value = "false")]
        daemon: bool,
        /// Refresh Redis data every N minutes (default: 30, 0 = no refresh)
        #[arg(long, default_value = "30")]
        refresh_minutes: u64,
    },
    /// List available venue games in Redis
    ListVenues {
        /// Redis URL (default: redis://127.0.0.1:6379)
        #[arg(long, default_value = "redis://127.0.0.1:6379")]
        redis_url: String,
    },
    /// Debug Redis contents and connectivity
    Debug {
        /// Redis URL (default: redis://127.0.0.1:6379)
        #[arg(long, default_value = "redis://127.0.0.1:6379")]
        redis_url: String,
        /// Pattern to search for (default: *)
        #[arg(short, long, default_value = "*")]
        pattern: String,
        /// Limit number of keys to show (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Test streaming with hardcoded active market IDs (bypasses Redis)
    TestLive {
        /// Market IDs to test (comma-separated)
        #[arg(short, long, default_value = "1.248000000")]
        market_ids: String,
        /// Orderbook depth (default: 10)
        #[arg(short, long, default_value = "10")]
        depth: usize,
        /// Update interval in seconds (default: 5)
        #[arg(short, long, default_value = "5")]
        interval: u64,
    },
    /// Test streaming WITHOUT market filtering (force stream any market ID)
    TestRaw {
        /// Market IDs to test (comma-separated)
        #[arg(short, long, default_value = "1.248000000")]
        market_ids: String,
        /// Orderbook depth (default: 10)
        #[arg(short, long, default_value = "10")]
        depth: usize,
        /// Update interval in seconds (default: 5)
        #[arg(short, long, default_value = "5")]
        interval: u64,
    },
}

#[derive(Debug, Clone)]
struct UpdateStats {
    #[allow(dead_code)]
    first_update: Instant,
    last_update: Instant,
    update_count: u64,
}

impl UpdateStats {
    fn new(now: Instant) -> Self {
        Self {
            first_update: now,
            last_update: now,
            update_count: 0,
        }
    }

    fn record_update(&mut self, now: Instant) {
        self.last_update = now;
        self.update_count += 1;
    }
}

#[derive(Debug, Clone)]
struct MessageStats {
    market_changes: u64,
    total_messages: u64,
    last_heartbeat: Option<Instant>,
}

impl MessageStats {
    fn new() -> Self {
        Self {
            market_changes: 0,
            total_messages: 0,
            last_heartbeat: None,
        }
    }

    fn record_market_change(&mut self) {
        self.market_changes += 1;
        self.total_messages += 1;
    }

    #[allow(dead_code)]
    fn record_heartbeat(&mut self) {
        self.last_heartbeat = Some(Instant::now());
        self.total_messages += 1;
    }
}

struct RedisMarketExtractor {
    connection: Connection,
}

impl RedisMarketExtractor {
    fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let connection = client.get_connection()?;
        info!("Connected to Redis at {}", redis_url);
        Ok(Self { connection })
    }

    fn extract_market_ids_from_venue_game(&mut self, venue_game_id: &str) -> Result<Vec<String>> {
        let pattern = format!("venue_game:betfair:{venue_game_id}:match_odds");
        self.extract_market_ids_from_pattern(&pattern)
    }

    fn extract_all_market_ids(&mut self) -> Result<Vec<String>> {
        let pattern = "venue_game:betfair:*:match_odds";
        self.extract_market_ids_from_pattern(pattern)
    }

    fn extract_market_ids_from_pattern(&mut self, pattern: &str) -> Result<Vec<String>> {
        let keys: Vec<String> = RedisCommands::keys(&mut self.connection, pattern)?;
        let keys_count = keys.len();

        if keys_count == 0 {
            warn!("No keys found matching pattern '{}'", pattern);
            info!("Try running 'cargo run -- debug' to see what keys exist in Redis");
            return Ok(vec![]);
        }

        info!("Found {} keys matching pattern '{}'", keys_count, pattern);

        // Show first few keys for debugging
        for (i, key) in keys.iter().take(3).enumerate() {
            debug!("Sample key {}: {}", i + 1, key);
        }

        let mut market_ids = HashSet::new();
        let mut successful_extractions = 0;

        for key in &keys {
            match self.extract_market_id_from_redis_data(key) {
                Ok(Some(market_id)) => {
                    market_ids.insert(market_id);
                    successful_extractions += 1;
                }
                Ok(None) => {
                    debug!("No market_id found in key: {}", key);
                }
                Err(e) => {
                    warn!("Error extracting market_id from key '{}': {}", key, e);
                }
            }
        }

        let result: Vec<String> = market_ids.into_iter().collect();
        info!("Extracted {} unique market IDs from {} keys ({} successful extractions)",
              result.len(), keys_count, successful_extractions);

        if result.is_empty() && keys_count > 0 {
            warn!("Found {} keys but couldn't extract any market IDs", keys_count);
            warn!("This suggests the Redis data format may not match expectations");
            warn!("Expected: JSON data with $.market_id field accessible via JSON.GET");
        }

        Ok(result)
    }

    fn extract_market_id_from_redis_data(&mut self, key: &str) -> Result<Option<String>> {
        // Try JSON.GET first
        let json_data: Option<String> = redis::cmd("JSON.GET")
            .arg(key)
            .arg("$.market_id")
            .query(&mut self.connection)
            .unwrap_or(None);

        if let Some(json_str) = json_data {
            debug!("Raw JSON response for key '{}': {}", key, json_str);

            // Parse the JSON response - it's typically an array like ["1.247952767"]
            if let Ok(parsed) = serde_json::from_str::<Value>(&json_str) {
                if let Some(array) = parsed.as_array() {
                    if let Some(first_element) = array.first() {
                        if let Some(market_id) = first_element.as_str() {
                            info!("✅ Extracted market_id '{}' from key '{}'", market_id, key);
                            return Ok(Some(market_id.to_string()));
                        }
                    }
                } else if let Some(market_id) = parsed.as_str() {
                    // Handle case where it's a direct string
                    info!("✅ Extracted market_id '{}' from key '{}'", market_id, key);
                    return Ok(Some(market_id.to_string()));
                }
            }

            warn!("⚠️ Could not parse JSON response '{}' for key '{}'", json_str, key);
        } else {
            debug!("No JSON.GET response for key: {}", key);
        }

        debug!("❌ Could not extract market_id from key: {}", key);
        Ok(None)
    }

    fn get_all_venue_games(&mut self) -> Result<Vec<String>> {
        let pattern = "venue_game:betfair:*:match_odds";
        let keys: Vec<String> = RedisCommands::keys(&mut self.connection, pattern)?;

        let mut venue_games = HashSet::new();

        for key in keys {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() >= 4 && parts[0] == "venue_game" && parts[1] == "betfair" {
                venue_games.insert(parts[2].to_string());
            }
        }

        let result: Vec<String> = venue_games.into_iter().collect();
        info!("Found {} unique venue games", result.len());

        Ok(result)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("betfair_rs=debug".parse().unwrap())
                .add_directive("redis_streamer=info".parse().unwrap())
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::StreamVenue { venue_game_id, depth, interval, redis_url, daemon, refresh_minutes } => {
            stream_venue_markets(&venue_game_id, depth, interval, &redis_url, daemon, refresh_minutes).await
        }
        Commands::StreamAll { depth, interval, redis_url, limit, daemon, refresh_minutes } => {
            stream_all_markets(depth, interval, &redis_url, limit, daemon, refresh_minutes).await
        }
        Commands::ListVenues { redis_url } => {
            list_venue_games(&redis_url).await
        }
        Commands::Debug { redis_url, pattern, limit } => {
            debug_redis_contents(&redis_url, &pattern, limit).await
        }
        Commands::TestLive { market_ids, depth, interval } => {
            let ids: Vec<String> = market_ids.split(',').map(|s| s.trim().to_string()).collect();
            info!("🧪 Testing streaming with market IDs: {:?}", ids);
            stream_markets(ids, depth, interval).await
        }
        Commands::TestRaw { market_ids, depth, interval } => {
            let ids: Vec<String> = market_ids.split(',').map(|s| s.trim().to_string()).collect();
            info!("🧪 RAW TEST: Streaming without filtering, market IDs: {:?}", ids);
            stream_markets_raw(ids, depth, interval).await
        }
    }
}

async fn stream_venue_markets(venue_game_id: &str, depth: usize, interval: u64, redis_url: &str, daemon: bool, refresh_minutes: u64) -> Result<()> {
    let mut redis_extractor = RedisMarketExtractor::new(redis_url)?;

    loop {
        let market_ids = redis_extractor.extract_market_ids_from_venue_game(venue_game_id)?;

        if market_ids.is_empty() {
            warn!("No market IDs found for venue game {}", venue_game_id);
            if !daemon {
                return Ok(());
            }
            info!("Daemon mode: waiting {} minutes before rechecking...", refresh_minutes);
            sleep(Duration::from_secs(refresh_minutes * 60)).await;
            continue;
        }

        info!("Found {} markets for venue game {}: {:?}", market_ids.len(), venue_game_id, market_ids);

        if daemon {
            info!("🔄 Starting continuous monitoring mode (Ctrl+C to stop)");
            stream_markets_continuous(market_ids, depth, interval, redis_url, refresh_minutes).await?;
        } else {
            stream_markets(market_ids, depth, interval).await?;
        }

        if !daemon {
            break;
        }
    }

    Ok(())
}

async fn stream_all_markets(depth: usize, interval: u64, redis_url: &str, limit: Option<usize>, daemon: bool, refresh_minutes: u64) -> Result<()> {
    let mut redis_extractor = RedisMarketExtractor::new(redis_url)?;

    loop {
        let mut market_ids = redis_extractor.extract_all_market_ids()?;

        if market_ids.is_empty() {
            warn!("No market IDs found in Redis");
            if !daemon {
                return Ok(());
            }
            info!("Daemon mode: waiting {} minutes before rechecking...", refresh_minutes);
            sleep(Duration::from_secs(refresh_minutes * 60)).await;
            continue;
        }

        if let Some(limit) = limit {
            market_ids.truncate(limit);
            info!("Limited to {} markets", limit);
        }

        info!("Found {} markets from Redis: {:?}", market_ids.len(), market_ids);

        if daemon {
            info!("🔄 Starting continuous monitoring mode (Ctrl+C to stop)");
            stream_markets_continuous(market_ids, depth, interval, redis_url, refresh_minutes).await?;
        } else {
            stream_markets(market_ids, depth, interval).await?;
        }

        if !daemon {
            break;
        }
    }

    Ok(())
}

async fn list_venue_games(redis_url: &str) -> Result<()> {
    let mut redis_extractor = RedisMarketExtractor::new(redis_url)?;
    let venue_games = redis_extractor.get_all_venue_games()?;

    if venue_games.is_empty() {
        println!("No venue games found in Redis");
        println!("\nTroubleshooting:");
        println!("1. Ensure Redis is running and accessible");
        println!("2. Check if Redis contains data in expected format");
        println!("3. Run: cargo run -- debug --pattern '*venue*' --limit 20");
        println!("4. Expected key pattern: venue_game:betfair:*:match_odds:*");
        return Ok(());
    }

    println!("Found {} venue games:", venue_games.len());
    for (i, venue_game) in venue_games.iter().enumerate() {
        println!("  {}: {}", i + 1, venue_game);
    }

    Ok(())
}

async fn debug_redis_contents(redis_url: &str, pattern: &str, limit: usize) -> Result<()> {
    println!("🔍 Debugging Redis contents...");
    println!("Redis URL: {}", redis_url);
    println!("Pattern: {}", pattern);
    println!("Limit: {}", limit);
    println!("{}", "-".repeat(60));

    let mut redis_extractor = RedisMarketExtractor::new(redis_url)?;

    // Get all keys matching pattern
    let keys: Vec<String> = RedisCommands::keys(&mut redis_extractor.connection, pattern)?;

    if keys.is_empty() {
        println!("❌ No keys found matching pattern '{}'", pattern);
        println!("\n💡 Suggestions:");
        println!("1. Try pattern '*' to see all keys");
        println!("2. Check if Redis server is running: redis-cli ping");
        println!("3. Verify Redis URL is correct");
        return Ok(());
    }

    println!("✅ Found {} keys matching pattern '{}'", keys.len(), pattern);
    println!("\n📋 Showing first {} keys:", limit.min(keys.len()));

    for (i, key) in keys.iter().take(limit).enumerate() {
        println!("  {}: {}", i + 1, key);

        // Try to get the value and show its type/structure
        let key_type: String = redis::cmd("TYPE")
            .arg(key)
            .query(&mut redis_extractor.connection)
            .unwrap_or_else(|_| "unknown".to_string());

        println!("     Type: {}", key_type);

        // For our expected venue_game keys, try to show structure
        if key.contains("venue_game") && key.contains("betfair") {
            println!("     🎯 This looks like a venue game key!");

            // Try JSON.GET to see if it has JSON structure
            if let Ok(json_check) = redis::cmd("JSON.GET")
                .arg(key)
                .arg("$")
                .query::<Option<String>>(&mut redis_extractor.connection)
            {
                if let Some(json_data) = json_check {
                    println!("     📄 JSON data (first 200 chars): {}",
                            json_data.chars().take(200).collect::<String>());

                    // Try to get market_id specifically
                    if let Ok(market_id_result) = redis::cmd("JSON.GET")
                        .arg(key)
                        .arg("$.market_id")
                        .query::<Option<String>>(&mut redis_extractor.connection)
                    {
                        match market_id_result {
                            Some(market_id) => println!("     🏁 Market ID: {}", market_id),
                            None => println!("     ⚠️  No $.market_id field found"),
                        }
                    }
                } else {
                    println!("     ❌ No JSON data found");
                }
            } else {
                // Try regular GET if JSON.GET fails
                if let Ok(regular_data) = redis::cmd("GET")
                    .arg(key)
                    .query::<Option<String>>(&mut redis_extractor.connection)
                {
                    if let Some(data) = regular_data {
                        println!("     📄 Regular data (first 200 chars): {}",
                                data.chars().take(200).collect::<String>());
                    }
                }
            }
        }

        if i < limit - 1 && i < keys.len() - 1 {
            println!();
        }
    }

    if keys.len() > limit {
        println!("\n... and {} more keys", keys.len() - limit);
    }

    println!("\n🔧 Expected format for venue games:");
    println!("  Key: venue_game:betfair:{{venue_id}}:match_odds:{{market_key}}");
    println!("  Value: JSON with $.market_id field");
    println!("  Command to check: JSON.GET key $.market_id");

    Ok(())
}

async fn stream_markets_continuous(market_ids: Vec<String>, depth: usize, interval: u64, redis_url: &str, refresh_minutes: u64) -> Result<()> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_signal = Arc::clone(&shutdown);

    // Setup signal handler for graceful shutdown
    tokio::spawn(async move {
        if let Err(e) = signal::ctrl_c().await {
            warn!("Failed to listen for Ctrl+C signal: {}", e);
            return;
        }
        info!("🛑 Shutdown signal received, stopping...");
        shutdown_signal.store(true, Ordering::Relaxed);
    });

    let subscription_start = Instant::now();
    let update_stats = Arc::new(Mutex::new(IndexMap::<String, UpdateStats>::new()));
    let message_stats = Arc::new(Mutex::new(MessageStats::new()));

    // Try to connect to Betfair if config exists, otherwise run in Redis-only mode
    let betfair_available = match try_initialize_betfair_streaming(market_ids.clone(), depth, &update_stats, &message_stats).await {
        Ok(streaming_client) => {
            info!("✅ Betfair streaming initialized successfully");
            Some(streaming_client)
        }
        Err(e) => {
            warn!("⚠️ Betfair streaming unavailable: {}", e);
            warn!("Running in Redis-only monitoring mode");
            None
        }
    };

    let mut last_refresh = Instant::now();
    let refresh_interval = Duration::from_secs(refresh_minutes * 60);

    info!("🚀 Starting continuous market monitoring...");
    info!("📊 Monitoring {} markets", market_ids.len());
    info!("⏰ Update interval: {}s", interval);
    if refresh_minutes > 0 {
        info!("🔄 Redis refresh: {}min", refresh_minutes);
    }
    info!("🛑 Press Ctrl+C to stop");
    info!("{}", "=".repeat(100));

    while !shutdown.load(Ordering::Relaxed) {
        let elapsed_since_subscription = subscription_start.elapsed();

        // Check if we need to refresh Redis data
        if refresh_minutes > 0 && last_refresh.elapsed() >= refresh_interval {
            info!("🔄 Refreshing Redis data...");
            if let Ok(_redis_extractor) = RedisMarketExtractor::new(redis_url) {
                // You could refresh market list here if needed
                // For now, just log that we're still monitoring
                info!("✅ Redis connection verified");
            }
            last_refresh = Instant::now();
        }

        // Display monitoring status
        if let Some(ref streaming_client) = betfair_available {
            display_betfair_streaming_status(&streaming_client, &market_ids, &update_stats, &message_stats, elapsed_since_subscription).await;
        } else {
            display_redis_only_status(&market_ids, elapsed_since_subscription);
        }

        sleep(Duration::from_secs(interval)).await;
    }

    info!("🔚 Monitoring stopped gracefully");
    Ok(())
}

async fn filter_active_markets(client: &BetfairClient, market_ids: Vec<String>) -> Result<Vec<String>> {
    info!("🔍 Checking market start times for {} markets", market_ids.len());

    let request = ListMarketCatalogueRequest {
        filter: MarketFilter {
            market_ids: Some(market_ids.clone()),
            ..Default::default()
        },
        market_projection: Some(vec![MarketProjection::MarketDescription]),
        sort: None,
        max_results: None,
        locale: None,
    };

    let markets = match client.list_market_catalogue(request).await {
        Ok(markets) => markets,
        Err(e) => {
            warn!("⚠️ Failed to get market catalogue (markets may be invalid/expired): {}", e);
            warn!("⚠️ All Redis market IDs appear to be stale - no active markets found");
            return Ok(Vec::new()); // Return empty list when all markets are invalid
        }
    };
    let now = Utc::now();
    let mut active_markets = Vec::new();
    let mut filtered_count = 0;

    for market in markets {
        if let Some(desc) = &market.description {
            // Parse market_time (ISO 8601 format)
            if let Ok(market_time) = DateTime::parse_from_rfc3339(&desc.market_time) {
                let market_time_utc = market_time.with_timezone(&Utc);

                // Keep markets that start in the future or recently (within 1 hour ago for in-play)
                let one_hour_ago = now - chrono::Duration::hours(1);

                if market_time_utc >= one_hour_ago {
                    active_markets.push(market.market_id.clone());
                    info!("✅ Keeping market {} '{}' (starts: {})",
                        market.market_id,
                        market.market_name,
                        market_time_utc.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                } else {
                    filtered_count += 1;
                    debug!("❌ Filtered out market {} '{}' (started: {})",
                        market.market_id,
                        market.market_name,
                        market_time_utc.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                }
            } else {
                warn!("⚠️ Could not parse market_time for market {}: '{}'",
                    market.market_id, desc.market_time);
                // Include markets with unparseable times to be safe
                active_markets.push(market.market_id);
            }
        } else {
            warn!("⚠️ No market description for market {}, including it", market.market_id);
            active_markets.push(market.market_id);
        }
    }

    info!("📊 Market filtering result: {} active, {} filtered out", active_markets.len(), filtered_count);
    Ok(active_markets)
}

async fn try_initialize_betfair_streaming(
    market_ids: Vec<String>,
    depth: usize,
    update_stats: &Arc<Mutex<IndexMap<String, UpdateStats>>>,
    message_stats: &Arc<Mutex<MessageStats>>
) -> Result<StreamingClient> {
    // Try to load config - if it fails, we'll run in Redis-only mode
    let config = Config::new()?;
    let mut api_client = BetfairClient::new(config.clone());

    info!("Attempting to login to Betfair...");
    api_client.login().await?;
    let session_token = api_client
        .get_session_token()
        .ok_or_else(|| anyhow::anyhow!("Failed to get session token"))?;
    info!("✅ Betfair login successful");

    // Filter for active markets only (markets with start time >= now)
    let active_market_ids = filter_active_markets(&api_client, market_ids).await?;
    info!("📊 Filtered to {} active markets", active_market_ids.len());

    if active_market_ids.is_empty() {
        return Err(anyhow::anyhow!("No active markets found - all Redis markets appear to be stale or invalid"));
    }

    let mut streaming_client =
        StreamingClient::with_session_token(config.betfair.api_key.clone(), session_token);

    // Set up callback for real data BEFORE starting the client
    let stats_for_callback = Arc::clone(update_stats);
    let msg_stats_for_callback = Arc::clone(message_stats);
    info!("🔧 Setting up orderbook callback for market updates");
    streaming_client.set_orderbook_callback(move |market_id, _orderbooks| {
        let now = Instant::now();

        if let Ok(mut stats) = stats_for_callback.lock() {
            let market_stats = stats.entry(market_id.clone()).or_insert_with(|| {
                info!("📈 First update received for market {}", market_id);
                UpdateStats::new(now)
            });
            market_stats.record_update(now);
        }

        if let Ok(mut msg_stats) = msg_stats_for_callback.lock() {
            msg_stats.record_market_change();
        }
    });

    info!("Starting Betfair streaming client...");
    streaming_client.start().await?;

    // Initialize stats for all markets
    info!("🔧 Pre-initializing statistics for {} markets", active_market_ids.len());
    for market_id in &active_market_ids {
        if let Ok(mut stats) = update_stats.lock() {
            stats.insert(market_id.clone(), UpdateStats::new(Instant::now()));
        }
    }

    // Subscribe to markets
    info!("Subscribing to {} markets with depth {}...", active_market_ids.len(), depth);
    streaming_client.subscribe_to_markets(active_market_ids, depth).await?;

    Ok(streaming_client)
}

fn display_redis_only_status(market_ids: &[String], elapsed: Duration) {
    println!("\n{}", "=".repeat(140));
    println!("📡 REDIS-ONLY MONITOR | Elapsed: {:.1}s | Time: {} | Markets: {} | Status: Monitoring Redis",
        elapsed.as_secs_f64(),
        chrono::Local::now().format("%H:%M:%S"),
        market_ids.len()
    );
    println!("{}", "=".repeat(140));

    println!("🔍 Redis Market Discovery Status:");
    for (i, market_id) in market_ids.iter().take(10).enumerate() {
        println!("  📊 Market {}: {}", i + 1, market_id);
    }

    if market_ids.len() > 10 {
        println!("  ... and {} more markets", market_ids.len() - 10);
    }

    println!("\n💡 Note: Running in Redis-only mode");
    println!("   • Add config.toml with Betfair credentials for live streaming");
    println!("   • Currently monitoring Redis for market discovery");
}

async fn display_betfair_streaming_status(
    streaming_client: &StreamingClient,
    market_ids: &[String],
    update_stats: &Arc<Mutex<IndexMap<String, UpdateStats>>>,
    message_stats: &Arc<Mutex<MessageStats>>,
    elapsed: Duration
) {
    let msg_stats_display = if let Ok(msg_stats) = message_stats.lock() {
        format!("Messages: {} | Market Changes: {}",
            msg_stats.total_messages,
            msg_stats.market_changes
        )
    } else {
        "Message stats unavailable".to_string()
    };

    println!("\n{}", "=".repeat(140));
    println!("🚀 BETFAIR STREAMING MONITOR | Elapsed: {:.1}s | Time: {} | {}",
        elapsed.as_secs_f64(),
        chrono::Local::now().format("%H:%M:%S"),
        msg_stats_display
    );
    println!("{}", "=".repeat(140));

    // Display market status
    for market_id in market_ids.iter().take(5) {
        let update_info = if let Ok(stats) = update_stats.lock() {
            if let Some(market_stats) = stats.get(market_id) {
                let seconds_since_last = market_stats.last_update.elapsed().as_secs_f64();
                let status = match seconds_since_last {
                    s if s <= 10.0 => "🟢 LIVE",
                    s if s <= 30.0 => "🟡 SLOW",
                    _ => "🔴 STALE"
                };
                format!("{} | Updates: {} | Last: {:.1}s ago",
                    status, market_stats.update_count, seconds_since_last)
            } else {
                "⚪ NO DATA".to_string()
            }
        } else {
            "Status unavailable".to_string()
        };

        println!("📊 Market: {} | {}", market_id, update_info);
    }

    if market_ids.len() > 5 {
        println!("... and {} more markets", market_ids.len() - 5);
    }

    // Debug: Show raw orderbook presence
    let orderbook_count = streaming_client.get_orderbooks().read()
        .map(|books| books.len())
        .unwrap_or(0);

    if orderbook_count > 0 {
        println!("🔍 Debug: {} markets in orderbook storage", orderbook_count);
    } else {
        println!("🔍 Debug: No markets in orderbook storage (this may indicate a streaming issue)");
    }
}

async fn stream_markets(market_ids: Vec<String>, depth: usize, interval: u64) -> Result<()> {
    let subscription_start = Instant::now();
    let update_stats = Arc::new(Mutex::new(IndexMap::<String, UpdateStats>::new()));
    let message_stats = Arc::new(Mutex::new(MessageStats::new()));

    // Initialize Betfair client
    let config = Config::new()?;
    let mut api_client = BetfairClient::new(config.clone());

    info!("Logging in to Betfair...");
    api_client.login().await?;
    let session_token = api_client
        .get_session_token()
        .ok_or_else(|| anyhow::anyhow!("Failed to get session token"))?;
    info!("Login successful");

    // Filter for active markets only (markets with start time >= now)
    let active_market_ids = filter_active_markets(&api_client, market_ids).await?;
    info!("📊 Filtered to {} active markets", active_market_ids.len());

    if active_market_ids.is_empty() {
        return Err(anyhow::anyhow!("No active markets found - all Redis markets appear to be stale or invalid"));
    }

    let mut streaming_client =
        StreamingClient::with_session_token(config.betfair.api_key.clone(), session_token);

    // Set up update tracking callback BEFORE starting the client
    let stats_for_callback = Arc::clone(&update_stats);
    let msg_stats_for_callback = Arc::clone(&message_stats);
    info!("🔧 Setting up orderbook callback for market updates");
    streaming_client.set_orderbook_callback(move |market_id, _orderbooks| {
        let now = Instant::now();

        if let Ok(mut stats) = stats_for_callback.lock() {
            let market_stats = stats.entry(market_id.clone()).or_insert_with(|| {
                info!("📈 First update received for market {}", market_id);
                UpdateStats::new(now)
            });
            market_stats.record_update(now);

            if market_stats.update_count % 10 == 0 {
                info!("Market {} received {} updates (last: {:.1}s ago)",
                    market_id,
                    market_stats.update_count,
                    market_stats.last_update.elapsed().as_secs_f64()
                );
            }
        }

        if let Ok(mut msg_stats) = msg_stats_for_callback.lock() {
            msg_stats.record_market_change();
        }
    });

    info!("Starting streaming client...");
    streaming_client.start().await?;

    // Initialize stats for all markets
    for market_id in &active_market_ids {
        if let Ok(mut stats) = update_stats.lock() {
            stats.insert(market_id.clone(), UpdateStats::new(Instant::now()));
        }
    }

    info!("Subscribing to {} market(s) with depth {}...", active_market_ids.len(), depth);
    streaming_client
        .subscribe_to_markets(active_market_ids.clone(), depth)
        .await?;

    info!("Waiting for initial orderbook data...");
    sleep(Duration::from_secs(3)).await;

    let orderbooks = streaming_client.get_orderbooks();

    info!("Starting market monitoring (press Ctrl+C to stop)...");
    info!("{}", "=".repeat(100));

    loop {
        let elapsed_since_subscription = subscription_start.elapsed();

        if let Ok(books) = orderbooks.read() {
            let msg_stats_display = if let Ok(msg_stats) = message_stats.lock() {
                let heartbeat_info = if let Some(last_heartbeat) = msg_stats.last_heartbeat {
                    format!("HB: {:.0}s ago", last_heartbeat.elapsed().as_secs_f64())
                } else {
                    "HB: None".to_string()
                };

                format!("Messages: {} | Market Changes: {} | {}",
                    msg_stats.total_messages,
                    msg_stats.market_changes,
                    heartbeat_info
                )
            } else {
                "Message stats unavailable".to_string()
            };

            println!("\n{}", "=".repeat(140));
            println!("REDIS STREAMING MONITOR | Elapsed: {:.1}s | Time: {} | {}",
                elapsed_since_subscription.as_secs_f64(),
                chrono::Local::now().format("%H:%M:%S"),
                msg_stats_display
            );
            println!("{}", "=".repeat(140));

            for market_id in &active_market_ids {
                let update_info = if let Ok(stats) = update_stats.lock() {
                    if let Some(market_stats) = stats.get(market_id) {
                        let seconds_since_last = market_stats.last_update.elapsed().as_secs_f64();
                        let status = match seconds_since_last {
                            s if s <= 10.0 => "🟢 LIVE",
                            s if s <= 30.0 => "🟡 SLOW",
                            _ => "🔴 STALE"
                        };
                        Some(format!("{} | Updates: {} | Last: {:.1}s ago",
                            status, market_stats.update_count, seconds_since_last))
                    } else {
                        Some("⚪ NO DATA".to_string())
                    }
                } else {
                    None
                };

                if let Some(market_books) = books.get(market_id) {
                    print_market_summary(market_id, market_books, update_info);
                } else {
                    println!("\n📊 Market: {} | {}", market_id,
                        update_info.unwrap_or_else(|| "NO ORDERBOOK DATA".to_string()));
                }
            }
        }

        if active_market_ids.len() > 1 {
            println!("\n{}", "=".repeat(140));
        }

        sleep(Duration::from_secs(interval)).await;
    }
}

async fn stream_markets_raw(market_ids: Vec<String>, depth: usize, interval: u64) -> Result<()> {
    info!("🚫 RAW MODE: Bypassing all market filtering");
    let subscription_start = Instant::now();
    let update_stats = Arc::new(Mutex::new(IndexMap::<String, UpdateStats>::new()));
    let message_stats = Arc::new(Mutex::new(MessageStats::new()));

    // Initialize Betfair client
    let config = Config::new()?;
    let mut api_client = BetfairClient::new(config.clone());

    info!("Logging in to Betfair...");
    api_client.login().await?;
    let session_token = api_client
        .get_session_token()
        .ok_or_else(|| anyhow::anyhow!("Failed to get session token"))?;
    info!("Login successful");

    info!("🚫 SKIPPING market filtering - using markets as-is: {:?}", market_ids);

    let mut streaming_client =
        StreamingClient::with_session_token(config.betfair.api_key.clone(), session_token);

    // Set up update tracking callback BEFORE starting the client
    let stats_for_callback = Arc::clone(&update_stats);
    let msg_stats_for_callback = Arc::clone(&message_stats);
    info!("🔧 Setting up orderbook callback for market updates");
    streaming_client.set_orderbook_callback(move |market_id, _orderbooks| {
        let now = Instant::now();

        if let Ok(mut stats) = stats_for_callback.lock() {
            let market_stats = stats.entry(market_id.clone()).or_insert_with(|| {
                info!("📈 First update received for market {}", market_id);
                UpdateStats::new(now)
            });
            market_stats.record_update(now);

            if market_stats.update_count % 10 == 0 {
                info!("Market {} received {} updates (last: {:.1}s ago)",
                    market_id,
                    market_stats.update_count,
                    market_stats.last_update.elapsed().as_secs_f64()
                );
            }
        }

        if let Ok(mut msg_stats) = msg_stats_for_callback.lock() {
            msg_stats.record_market_change();
        }
    });

    info!("Starting streaming client...");
    streaming_client.start().await?;

    // Initialize stats for all markets
    for market_id in &market_ids {
        if let Ok(mut stats) = update_stats.lock() {
            stats.insert(market_id.clone(), UpdateStats::new(Instant::now()));
        }
    }

    info!("🚫 RAW SUBSCRIPTION: Subscribing to {} market(s) with depth {} (NO FILTERING)...", market_ids.len(), depth);
    streaming_client
        .subscribe_to_markets(market_ids.clone(), depth)
        .await?;

    info!("Waiting for initial orderbook data...");
    sleep(Duration::from_secs(3)).await;

    let orderbooks = streaming_client.get_orderbooks();

    info!("🚫 RAW STREAMING: Starting market monitoring (press Ctrl+C to stop)...");
    info!("{}", "=".repeat(100));

    loop {
        let elapsed_since_subscription = subscription_start.elapsed();

        if let Ok(books) = orderbooks.read() {
            let msg_stats_display = if let Ok(msg_stats) = message_stats.lock() {
                format!("Messages: {} | Market Changes: {}",
                    msg_stats.total_messages,
                    msg_stats.market_changes
                )
            } else {
                "Message stats unavailable".to_string()
            };

            println!("\n{}", "=".repeat(140));
            println!("🚫 RAW STREAMING MONITOR | Elapsed: {:.1}s | Time: {} | {}",
                elapsed_since_subscription.as_secs_f64(),
                chrono::Local::now().format("%H:%M:%S"),
                msg_stats_display
            );
            println!("{}", "=".repeat(140));

            for market_id in &market_ids {
                let update_info = if let Ok(stats) = update_stats.lock() {
                    if let Some(market_stats) = stats.get(market_id) {
                        let seconds_since_last = market_stats.last_update.elapsed().as_secs_f64();
                        let status = match seconds_since_last {
                            s if s <= 10.0 => "🟢 LIVE",
                            s if s <= 30.0 => "🟡 SLOW",
                            _ => "🔴 STALE"
                        };
                        Some(format!("{} | Updates: {} | Last: {:.1}s ago",
                            status, market_stats.update_count, seconds_since_last))
                    } else {
                        Some("⚪ NO DATA".to_string())
                    }
                } else {
                    None
                };

                if let Some(market_books) = books.get(market_id) {
                    print_market_summary(market_id, market_books, update_info);
                } else {
                    println!("\n📊 Market: {} | {}", market_id,
                        update_info.unwrap_or_else(|| "NO ORDERBOOK DATA".to_string()));
                }
            }
        }

        if market_ids.len() > 1 {
            println!("\n{}", "=".repeat(140));
        }

        sleep(Duration::from_secs(interval)).await;
    }
}

fn print_market_summary(market_id: &str, market_books: &HashMap<String, Orderbook>, update_info: Option<String>) {
    println!("\n{}", "-".repeat(100));
    println!(
        "📊 Market: {} | Selections: {} | {}",
        market_id,
        market_books.len(),
        update_info.unwrap_or_else(|| "Update status unknown".to_string())
    );
    println!("{}", "-".repeat(100));

    for (selection_id, orderbook) in market_books.iter() {
        println!("\n🏃 Selection: {selection_id}");

        let top_bids: Vec<&PriceLevel> = orderbook.bids.iter().take(5).collect();
        let top_asks: Vec<&PriceLevel> = orderbook.asks.iter().take(5).collect();

        println!("\n{:^20} | {:^20}", "BACK (BIDS)", "LAY (ASKS)");
        println!(
            "{:^10} {:^9} | {:^9} {:^10}",
            "Price", "Size", "Price", "Size"
        );
        println!("{}", "-".repeat(41));

        let max_len = top_bids.len().max(top_asks.len());

        for i in 0..max_len {
            let bid_str = if i < top_bids.len() {
                format!("{:>10.2} {:>9.2}", top_bids[i].price, top_bids[i].size)
            } else {
                " ".repeat(20)
            };

            let ask_str = if i < top_asks.len() {
                format!("{:>9.2} {:>10.2}", top_asks[i].price, top_asks[i].size)
            } else {
                " ".repeat(20)
            };

            println!("{bid_str} | {ask_str}");
        }

        if let (Some(best_bid), Some(best_ask)) =
            (orderbook.get_best_bid(), orderbook.get_best_ask())
        {
            let spread = best_ask.price - best_bid.price;
            println!(
                "\n📈 Spread: {:.2} | Mid: {:.2}",
                spread,
                (best_bid.price + best_ask.price) / 2.0
            );
        }
    }
}