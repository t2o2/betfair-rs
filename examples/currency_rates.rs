use anyhow::Result;
use betfair_rs::{config::Config, BetfairClient};
use betfair_rs::dto::account::ListCurrencyRatesRequest;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration from config.toml
    let config = Config::new()?;

    // Initialize unified client
    let mut client = BetfairClient::new(config);

    info!("Logging in to Betfair API...");

    // Use certificate-based login
    let _login_response = client.login().await?;

    info!("Login successful!");

    // Get currency exchange rates
    info!("Fetching currency exchange rates from GBP...");
    let rates_request = ListCurrencyRatesRequest {
        from_currency: Some("GBP".to_string()),
    };

    match client.list_currency_rates(rates_request).await {
        Ok(rates) => {
            println!("\n{:-<50}", "");
            println!("Currency Exchange Rates (from GBP)");
            println!("{:-<50}", "");
            println!("{:<10} {:>15}", "Currency", "Rate");
            println!("{:-<50}", "");
            
            for rate in &rates {
                println!("{:<10} {:>15}", rate.currency_code, rate.rate);
            }
            println!("{:-<50}", "");
            info!("Total currencies: {}", rates.len());
        }
        Err(e) => {
            eprintln!("Failed to fetch currency rates: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
