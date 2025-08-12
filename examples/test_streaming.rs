use anyhow::Result;
use betfair_rs::{config::Config, streaming_client::StreamingClient};
use std::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting streaming test...");

    // Load configuration
    let config = Config::new()?;

    // Create and start streaming client
    let mut streaming_client = StreamingClient::new(config);
    
    info!("Starting streaming client...");
    streaming_client.start().await?;
    
    info!("Streaming client started successfully!");

    // Test market ID - replace with a valid market ID
    let market_id = "1.240634817".to_string();
    
    info!("Subscribing to market {}...", market_id);
    streaming_client.subscribe_to_market(market_id.clone(), 5).await?;
    
    info!("Subscribed! Waiting for orderbook updates...");

    // Get the orderbooks reference
    let orderbooks = streaming_client.get_orderbooks();
    
    // Monitor for updates for 30 seconds
    for i in 0..30 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        let obs = orderbooks.read().unwrap();
        if let Some(market_obs) = obs.get(&market_id) {
            info!("Update {}: Market has {} runners with orderbook data", i + 1, market_obs.len());
            
            for (runner_id, orderbook) in market_obs.iter() {
                if !orderbook.bids.is_empty() || !orderbook.asks.is_empty() {
                    info!("  Runner {}: {} bids, {} asks", 
                        runner_id, 
                        orderbook.bids.len(), 
                        orderbook.asks.len()
                    );
                    
                    if let Some(best_bid) = orderbook.bids.first() {
                        info!("    Best bid: {:.2} @ {:.2}", best_bid.price, best_bid.size);
                    }
                    if let Some(best_ask) = orderbook.asks.first() {
                        info!("    Best ask: {:.2} @ {:.2}", best_ask.price, best_ask.size);
                    }
                }
            }
        } else {
            info!("Update {}: No data yet for market {}", i + 1, market_id);
        }
    }
    
    info!("Test complete. Stopping streaming client...");
    streaming_client.stop().await?;
    
    info!("Streaming client stopped.");
    Ok(())
}