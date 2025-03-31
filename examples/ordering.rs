use anyhow::Result;
use tracing::info;
use std::error::Error;
use betfair_rs::{config, betfair, order::OrderSide};

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

    let market_id = "1.240634817".to_string();
    let runner_id = 39674645;
    let side = OrderSide::Back;
    let price = 10.0;
    let size = 1.0;

    let order = betfair_rs::order::Order::new(market_id, runner_id, side, price, size);
    let order_response = client.place_order(order).await?;
    info!("Order response: {:?}", order_response);

    tokio::time::sleep(std::time::Duration::from_secs(120)).await;
    info!("Betfair market subscribed");
    Ok(())
} 
