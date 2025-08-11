use anyhow::Result;
use betfair_rs::{api_client::BetfairApiClient, config::Config, dto::*};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Betfair API Client Example");

    // Load configuration
    let config = Config::new()?;

    // Create API client
    let mut client = BetfairApiClient::new(config);

    // Login
    info!("Logging in to Betfair...");
    let login_response = client.login().await?;
    match login_response.session_token {
        Some(_) => info!("Login successful!"),
        None => {
            eprintln!("Login failed: {}", login_response.login_status);
            return Ok(());
        }
    }

    // Example 1: Search for markets
    info!("\n=== Searching for Premier League markets ===");
    let markets = client
        .search_markets("Premier League".to_string(), Some(5))
        .await?;
    for market in &markets {
        info!("Market: {} - {}", market.market_id, market.market_name);
        if let Some(start_time) = &market.market_start_time {
            info!("  Start time: {}", start_time);
        }
    }

    // Example 2: Get market prices
    if !markets.is_empty() {
        let market_id = markets[0].market_id.clone();
        info!("\n=== Getting prices for market {} ===", market_id);

        let market_books = client.get_market_prices(vec![market_id.clone()]).await?;
        if let Some(market_book) = market_books.first() {
            info!("Market status: {:?}", market_book.status);
            info!("Total matched: {:?}", market_book.total_matched);

            if let Some(runners) = &market_book.runners {
                for runner in runners {
                    info!("Runner {} - Status: {}", runner.selection_id, runner.status);
                    if let Some(ex) = &runner.ex {
                        if let Some(back_prices) = &ex.available_to_back {
                            if let Some(best_back) = back_prices.first() {
                                info!(
                                    "  Best back price: {} @ {}",
                                    best_back.price, best_back.size
                                );
                            }
                        }
                        if let Some(lay_prices) = &ex.available_to_lay {
                            if let Some(best_lay) = lay_prices.first() {
                                info!("  Best lay price: {} @ {}", best_lay.price, best_lay.size);
                            }
                        }
                    }
                }
            }
        }
    }

    // Example 3: Place a simple order (commented out to avoid actual betting)
    /*
    if !markets.is_empty() {
        let market_id = markets[0].market_id.clone();
        info!("\n=== Placing a test order ===");

        // Place a small back bet at low odds (unlikely to match)
        let order_response = client.place_simple_order(
            market_id.clone(),
            123456789, // selection_id - you'd need to get this from the market
            Side::Back,
            1000.0,    // price (very high odds, unlikely to match)
            2.0,       // size (minimum stake)
        ).await?;

        info!("Order response status: {}", order_response.status);
        if let Some(reports) = &order_response.instruction_reports {
            for report in reports {
                info!("Order status: {}", report.status);
                if let Some(bet_id) = &report.bet_id {
                    info!("Bet ID: {}", bet_id);

                    // Cancel the order immediately
                    info!("Cancelling order...");
                    let cancel_response = client.cancel_bet(market_id.clone(), bet_id.clone()).await?;
                    info!("Cancel response status: {}", cancel_response.status);
                }
            }
        }
    }
    */

    // Example 4: Get account funds
    info!("\n=== Getting account funds ===");
    let funds = client
        .get_account_funds(GetAccountFundsRequest { wallet: None })
        .await?;
    info!("Available balance: £{}", funds.available_to_bet_balance);
    info!("Exposure: £{}", funds.exposure);
    info!("Exposure limit: £{}", funds.exposure_limit);

    // Example 5: Get account details
    info!("\n=== Getting account details ===");
    let details = client.get_account_details().await?;
    if let Some(currency) = &details.currency_code {
        info!("Currency: {}", currency);
    }
    if let Some(country) = &details.country_code {
        info!("Country: {}", country);
    }
    if let Some(timezone) = &details.timezone {
        info!("Timezone: {}", timezone);
    }

    // Example 6: List current orders
    info!("\n=== Listing current orders ===");
    let current_orders_response = client
        .list_current_orders(ListCurrentOrdersRequest {
            bet_ids: None,
            market_ids: None,
            order_projection: Some(OrderProjection::All),
            customer_order_refs: None,
            customer_strategy_refs: None,
            date_range: None,
            order_by: None,
            sort_dir: None,
            from_record: None,
            record_count: Some(10),
        })
        .await?;

    info!(
        "Found {} current orders",
        current_orders_response.current_orders.len()
    );
    for order in &current_orders_response.current_orders {
        info!(
            "Order {} - Market: {}, Selection: {}, Status: {:?}",
            order.bet_id, order.market_id, order.selection_id, order.status
        );
        info!(
            "  Price: {} @ {}, Side: {:?}",
            order.price_size.price, order.price_size.size, order.side
        );
    }

    info!("\n=== Example completed successfully ===");
    Ok(())
}
