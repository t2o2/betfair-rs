use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "betfair")]
#[command(about = "Betfair trading CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive terminal dashboard
    Dashboard,
    /// Stream real-time market data to console
    Stream {
        /// Market IDs to subscribe to
        #[arg(required = true)]
        market_ids: Vec<String>,
        /// Orderbook depth (default: 10)
        #[arg(short, long, default_value = "10")]
        depth: usize,
        /// Update interval in seconds (default: 2)
        #[arg(short, long, default_value = "2")]
        interval: u64,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dashboard => dashboard::run(),
        Commands::Stream {
            market_ids,
            depth,
            interval,
        } => streaming::run(market_ids, depth, interval),
    }
}

// Include the dashboard module
mod dashboard {
    include!("../../examples/dashboard.rs");

    pub fn run() -> anyhow::Result<()> {
        main()
    }
}

// Streaming module for console output
mod streaming {
    use anyhow::Result;
    use betfair_rs::orderbook::{Orderbook, PriceLevel};
    use betfair_rs::{BetfairApiClient, Config, StreamingClient};
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::sleep;
    use tracing::{info, warn};

    pub fn run(market_ids: Vec<String>, depth: usize, interval: u64) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async_run(market_ids, depth, interval))
    }

    async fn async_run(market_ids: Vec<String>, depth: usize, interval: u64) -> Result<()> {
        tracing_subscriber::fmt::init();

        let config = Config::new()?;
        let mut api_client = BetfairApiClient::new(config.clone());

        info!("Logging in to Betfair...");
        api_client.login().await?;
        let session_token = api_client
            .get_session_token()
            .ok_or_else(|| anyhow::anyhow!("Failed to get session token"))?;
        info!("Login successful");

        let mut streaming_client =
            StreamingClient::with_session_token(config.betfair.api_key.clone(), session_token);

        info!("Starting streaming client...");
        streaming_client.start().await?;

        info!(
            "Subscribing to {} market(s) in single batch subscription...",
            market_ids.len()
        );
        streaming_client
            .subscribe_to_markets(market_ids.clone(), depth)
            .await?;

        info!("Waiting for initial orderbook data...");
        sleep(Duration::from_secs(2)).await;

        let orderbooks = streaming_client.get_orderbooks();

        info!("Starting market monitoring (press Ctrl+C to stop)...");
        info!("{}", "=".repeat(100));

        loop {
            if let Ok(books) = orderbooks.read() {
                for market_id in &market_ids {
                    if let Some(market_books) = books.get(market_id) {
                        print_market_summary(market_id, market_books);
                    } else {
                        warn!("No orderbook data for market {}", market_id);
                    }
                }
            }

            if market_ids.len() > 1 {
                println!("\n{}", "=".repeat(100));
            }

            sleep(Duration::from_secs(interval)).await;
        }
    }

    fn print_market_summary(market_id: &str, market_books: &HashMap<String, Orderbook>) {
        println!("\n{}", "=".repeat(80));
        println!(
            "Market ID: {} | Time: {} | Selections: {}",
            market_id,
            chrono::Local::now().format("%H:%M:%S"),
            market_books.len()
        );
        println!("{}", "-".repeat(80));

        for (selection_id, orderbook) in market_books.iter() {
            println!("\nSelection ID: {selection_id}");

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
                    "\nSpread: {:.2} | Mid: {:.2}",
                    spread,
                    (best_bid.price + best_ask.price) / 2.0
                );
            }
        }
    }
}
