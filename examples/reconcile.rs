use anyhow::Result;
use betfair_rs::{betfair, config};
use serde_json::json;
use std::error::Error;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Betfair Order Reconciliation Starting...");

    // Initialize client and login
    let config = config::Config::new()?;
    let mut client = betfair::BetfairClient::new(config);
    client.login().await?;
    info!("Successfully logged in to Betfair");

    // Get list of recent orders to reconcile
    let bet_ids = vec![
        "384087398733".to_string(), // Example bet IDs to check
        "384086212322".to_string(),
        "382340338578".to_string(),
        "383109613524".to_string(),
    ];

    // Get current status of orders
    let order_statuses = client.get_order_status(bet_ids.clone()).await?;
    info!(
        "Order Statuses:\n{}",
        serde_json::to_string_pretty(&json!(order_statuses))?
    );

    // Process each order status
    for bet_id in bet_ids {
        let matching_status = order_statuses.get(&bet_id);

        if let Some(order_status) = matching_status {
            info!("\nReconciling order {}", bet_id);
            info!("Status: {}", order_status.order_status);
            info!("Size Matched: {}", order_status.size_matched.unwrap_or(0.0));
            info!(
                "Size Remaining: {}",
                order_status.size_remaining.unwrap_or(0.0)
            );
            info!("Price: {}", order_status.price_requested.unwrap_or(0.0));

            // Handle different order statuses
            match order_status.order_status.as_str() {
                "EXECUTION_COMPLETE" => {
                    info!("Order fully executed");
                }
                "EXECUTABLE" => {
                    info!("Order still active - consider canceling if too old");
                    // Optionally cancel old orders
                    // client.cancel_order(market_id.clone(), bet_id).await?;
                }
                "EXPIRED" => {
                    info!("Order expired without full execution");
                }
                _ => {
                    info!("Unexpected order status");
                }
            }
        } else {
            info!("No status found for bet ID: {}", bet_id);
        }
    }

    info!("Order reconciliation completed");
    Ok(())
}
