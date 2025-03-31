use anyhow::Result;
use tracing::info;
use std::error::Error;
use betfair_rs::{config, betfair, order::OrderSide};

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

    let order = betfair_rs::order::Order::new(market_id.clone(), runner_id, side, price, size);
    let order_response = client.place_order(order).await?;
    info!("Order response: {:?}", order_response);

    if let Some(bet_id) = order_response.instruction_reports[0].bet_id.clone() {
        info!("Placed order with bet ID: {}", bet_id);
        
        info!("Waiting 5 seconds before canceling order");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        
        let cancel_response = client.cancel_order(market_id, bet_id).await?;
        info!("Cancel order response: {:?}", cancel_response);
    } else {
        info!("No bet ID in order response, order may have failed");
    }

    info!("Betfair Trading Ordering Completed");

    Ok(())
} 
