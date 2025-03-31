use anyhow::Result;
use tracing::info;
use std::error::Error;
use betfair_rs::{config, betfair, order::{OrderSide}};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Betfair Trading Ordering Starting...");

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

    // First place a persistent order
    info!("Placing a persistent order...");
    let order = betfair_rs::order::Order::new(market_id.clone(), runner_id, side.clone(), price, size, None);
    let order_response = client.place_order(order).await?;
    info!("Persistent order response:\n{}", serde_json::to_string_pretty(&json!(order_response))?);

    if let Some(bet_id) = order_response.instruction_reports[0].bet_id.clone() {
        info!("Placed persistent order with bet ID: {}", bet_id);
        
        // Check order status immediately after placing
        if let Some(status) = client.get_order_status(bet_id.clone()).await? {
            info!("Initial persistent order status:\n{}", serde_json::to_string_pretty(&json!(status))?);
        }
        
        info!("Waiting 5 seconds before checking status again");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        
        // Check order status again
        if let Some(status) = client.get_order_status(bet_id.clone()).await? {
            info!("Persistent order status after 5 seconds:\n{}", serde_json::to_string_pretty(&json!(status))?);
        }
        
        info!("Waiting 5 more seconds before canceling order");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        
        let cancel_response = client.cancel_order(market_id.clone(), bet_id.clone()).await?;
        info!("Cancel persistent order response:\n{}", serde_json::to_string_pretty(&json!(cancel_response))?);
        
        // Check final order status after cancellation
        if let Some(status) = client.get_order_status(bet_id).await? {
            info!("Final persistent order status after cancellation:\n{}", serde_json::to_string_pretty(&json!(status))?);
        }
    } else {
        info!("No bet ID in persistent order response, order may have failed");
    }

    info!("\nPlacing a FOK order...");
    let order = betfair_rs::order::Order::new(market_id.clone(), runner_id, side, price, size, Some(betfair_rs::order::TimeInForceType::FillOrKill));
    let order_response = client.place_order(order).await?;
    info!("FOK order response:\n{}", serde_json::to_string_pretty(&json!(order_response))?);

    if let Some(bet_id) = order_response.instruction_reports[0].bet_id.clone() {
        info!("Placed FOK order with bet ID: {}", bet_id);
        
        // Check order status immediately after placing
        if let Some(status) = client.get_order_status(bet_id.clone()).await? {
            info!("Initial FOK order status:\n{}", serde_json::to_string_pretty(&json!(status))?);
        }
        
        info!("Waiting 5 seconds before checking status again");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        
        // Check order status again
        if let Some(status) = client.get_order_status(bet_id.clone()).await? {
            info!("FOK order status after 5 seconds:\n{}", serde_json::to_string_pretty(&json!(status))?);
        }
        
        info!("Waiting 5 more seconds before checking final status");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        
        // Check final order status
        if let Some(status) = client.get_order_status(bet_id).await? {
            info!("Final FOK order status:\n{}", serde_json::to_string_pretty(&json!(status))?);
        }
    } else {
        info!("No bet ID in FOK order response, order may have failed");
    }

    info!("Betfair Trading Ordering Completed");

    Ok(())
} 
