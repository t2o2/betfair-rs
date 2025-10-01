use anyhow::Result;
use betfair_rs::orderbook::{Orderbook, PriceLevel};
use betfair_rs::{BetfairClient, Config, StreamingClient};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

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

    let market_id = "1.247749878";
    let orderbook_depth = 10;

    info!(
        "Subscribing to market {} with depth {}...",
        market_id, orderbook_depth
    );
    streaming_client
        .subscribe_to_market(market_id.to_string(), orderbook_depth)
        .await?;

    info!("Waiting for initial orderbook data...");
    sleep(Duration::from_secs(2)).await;

    let orderbooks = streaming_client.get_orderbooks();

    info!("Starting orderbook monitoring (press Ctrl+C to stop)...");
    info!("{}", "=".repeat(80));

    loop {
        if let Ok(books) = orderbooks.read() {
            if let Some(market_books) = books.get(market_id) {
                print_orderbook_summary(market_id, market_books);
            } else {
                warn!("No orderbook data for market {}", market_id);
            }
        }

        sleep(Duration::from_secs(2)).await;
    }
}

fn print_orderbook_summary(market_id: &str, market_books: &HashMap<String, Orderbook>) {
    println!("\n{}", "=".repeat(80));
    println!(
        "Market ID: {} | Time: {}",
        market_id,
        chrono::Local::now().format("%H:%M:%S")
    );
    println!("{}", "-".repeat(80));

    for (selection_id, orderbook) in market_books.iter() {
        println!("\nSelection ID: {selection_id}");

        // Get top 5 bids and asks
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
                (best_bid.price + best_ask.price) / Decimal::from(2)
            );
        }
    }
}
