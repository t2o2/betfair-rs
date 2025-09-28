use anyhow::Result;
use betfair_rs::dto::streaming::OrderFilter;
use betfair_rs::{BetfairClient, Config, StreamingClient};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::new()?;

    let mut api_client = BetfairClient::new(config.clone());

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

    let order_filter = OrderFilter {
        include_overall_position: Some(true),
        customer_strategy_refs: None,
        partition_matched_by_strategy_ref: Some(false),
    };

    info!("Subscribing to order updates...");
    streaming_client
        .subscribe_to_orders(Some(order_filter))
        .await?;

    info!("Waiting for order data...");
    sleep(Duration::from_secs(2)).await;

    let orders = streaming_client.get_orders();

    info!("Starting order monitoring (press Ctrl+C to stop)...");
    info!("{}", "=".repeat(80));

    loop {
        if let Ok(order_map) = orders.read() {
            if order_map.is_empty() {
                info!("No active orders found");
            } else {
                for (market_id, cache) in order_map.iter() {
                    print_order_summary(market_id, cache);
                }
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}

fn print_order_summary(market_id: &str, cache: &betfair_rs::order_cache::OrderCache) {
    println!("\n{}", "=".repeat(80));
    println!(
        "Market ID: {} | Last Update: {}",
        market_id,
        format_timestamp(cache.last_update)
    );
    println!("{}", "-".repeat(80));

    if cache.runners.is_empty() {
        println!("No runners with orders");
        return;
    }

    for (selection_id, runner) in cache.runners.iter() {
        println!("\nSelection ID: {selection_id}");
        if let Some(handicap) = runner.handicap {
            println!("  Handicap: {handicap}");
        }

        if runner.orders.is_empty() {
            println!("  No active orders");
        } else {
            println!("\n  Active Orders ({}):", runner.orders.len());
            println!(
                "  {:^15} {:^10} {:^10} {:^10} {:^15} {:^10}",
                "Bet ID", "Side", "Price", "Size", "Status", "Matched"
            );
            println!("  {}", "-".repeat(75));

            for order in runner.orders.values() {
                let matched = order.sm.unwrap_or(0.0);
                let status = match order.status.as_str() {
                    "E" => "Executable",
                    "EC" => "Exec Complete",
                    _ => &order.status,
                };

                println!(
                    "  {:^15} {:^10} {:^10.2} {:^10.2} {:^15} {:^10.2}",
                    &order.id[..order.id.len().min(15)],
                    order.side,
                    order.p,
                    order.s,
                    status,
                    matched
                );

                if let Some(remaining) = order.sr {
                    if remaining > 0.0 {
                        println!("    Remaining: {remaining:.2}");
                    }
                }

                if let Some(avg_price) = order.avp {
                    println!("    Average Price Matched: {avg_price:.2}");
                }
            }
        }

        if !runner.matched_backs.is_empty() {
            println!("\n  Matched Backs:");
            for (price, size) in runner.matched_backs.iter() {
                println!("    Price {price}: Size {size:.2}");
            }
            println!(
                "    Total Back Matched: {:.2}",
                runner.get_total_back_matched()
            );
        }

        if !runner.matched_lays.is_empty() {
            println!("\n  Matched Lays:");
            for (price, size) in runner.matched_lays.iter() {
                println!("    Price {price}: Size {size:.2}");
            }
            println!(
                "    Total Lay Matched: {:.2}",
                runner.get_total_lay_matched()
            );
        }
    }
}

fn format_timestamp(ts: i64) -> String {
    if ts == 0 {
        return "N/A".to_string();
    }

    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp_millis(ts);
    dt.map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Invalid".to_string())
}
