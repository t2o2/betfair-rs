use anyhow::Result;
use tracing::info;
use std::collections::HashMap;
use std::error::Error;
use betfair_rs::{config, betfair, orderbook};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Betfair Trading Streamer Starting...");

    let config = config::Config::new()?;
    let mut client = betfair::BetfairClient::new(config);
    client.login().await?;
    let token = client.get_session_token().await.unwrap();
    info!("Betfair session token: {}", token);

    client.set_orderbook_callback(orderbook_callback);

    client.connect().await?;
    info!("Connected to Betfair streaming service");

    let market_ids = vec!["1.241529489".to_string()];
    client.subscribe_to_markets(market_ids.clone(), 3).await?;
    info!("Subscribed to markets: {:?}", market_ids);

    client.start_listening().await?;
    info!("Started listening for market updates");

    tokio::time::sleep(std::time::Duration::from_secs(120)).await;
    info!("Streaming session completed");
    Ok(())
} 

fn orderbook_callback(market_id: String, orderbooks: HashMap<String, orderbook::Orderbook>) {
    info!("\n=== Orderbook Update ===");
    info!("Market ID: {}", market_id);
    for (runner_id, orderbook) in orderbooks {
        info!("Runner ID: {}", runner_id);
        info!("Timestamp: {}", orderbook.ts);
        info!("{}", orderbook.pretty_print());
    }
    info!("=====================\n");
}
