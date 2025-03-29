use anyhow::Result;
use tracing::info;
use std::collections::HashMap;
use std::error::Error;
mod betfair;
mod config;
mod msg_model;
mod model;
mod streamer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Betfair Trading Bot Starting...");

    let config = config::Config::new()?;
    let mut betfair_client = betfair::BetfairClient::new(config);
    betfair_client.login().await?;
    let token = betfair_client.get_session_token().await.unwrap();
    info!("Betfair session token: {}", token);

    betfair_client.set_orderbook_callback(orderbook_callback);

    betfair_client.connect().await?;
    betfair_client.subscribe_to_markets(vec!["1.241542802".to_string()], 10).await?;
    betfair_client.start_listening().await?;
    info!("Betfair client started listening");

    // Keep the program running
    tokio::time::sleep(std::time::Duration::from_secs(120)).await;
    info!("Betfair market subscribed");
    Ok(())
} 

fn orderbook_callback(market_id: String, orderbooks: HashMap<String, model::Orderbook>) {
    println!("\n=== Orderbook Update ===");
    println!("Market ID: {}", market_id);
    for (runner_id, orderbook) in orderbooks {
        println!("Runner ID: {}", runner_id);
        println!("Timestamp: {}", orderbook.ts);
        println!("{}", orderbook.pretty_print());
    }
    println!("=====================\n");
}
