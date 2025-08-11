use anyhow::Result;
use betfair_rs::msg_model::OrderChangeMessage;
use betfair_rs::{betfair, config};
use std::error::Error;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Betfair Order Streaming Example Starting...");

    // Initialize client and login
    let config = config::Config::new()?;
    let mut client = betfair::BetfairClient::new(config);
    client.login().await?;
    let token = client.get_session_token().await.unwrap();
    info!("Betfair session token: {}", token);

    // Connect to streaming API
    client.connect().await?;

    // Set up order update callback
    client.set_orderupdate_callback(|order_change: OrderChangeMessage| {
        info!("Received order update:");
        info!("Clock: {}", order_change.clk);
        let is_initial_order_update = order_change.op == "ocm" && order_change.clk == "AAAAAAAAAAAAAA==";
        if is_initial_order_update {
            info!("Initial order update");
        }
        else {
            info!("Order update");
        }
        info!("Publish time: {}", order_change.pt);
        for order_change in order_change.oc {
            info!("Market ID: {}", order_change.id);
            for runner_change in order_change.orc {
                info!("Runner ID: {}", runner_change.id);
                if let Some(unmatched_orders) = runner_change.uo {
                    for order in unmatched_orders {
                        info!("Unmatched order: ID={}, Price={}, Size={}, Side={}, Status={}, Remaining={}, Matched={}, Lapsed={}, Cancelled={}, Voided={}", 
                            order.id, order.p, order.s, order.side, order.status,
                            order.sr, order.sm, order.sl, order.sc, order.sv);
                    }
                }
            }
        }
    })?;

    client.subscribe_to_orders().await?;

    client.start_listening().await?;

    tokio::time::sleep(std::time::Duration::from_secs(300)).await;
    info!("Streaming session completed");
    Ok(())
}
