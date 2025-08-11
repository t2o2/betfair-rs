use anyhow::Result;
use serde_json::json;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Betfair Account Funds Example Starting...");

    // Initialize the client
    let config = betfair_rs::config::Config::new()?;
    let mut client = betfair_rs::betfair::BetfairClient::new(config);

    // Login to Betfair
    client.login().await?;
    info!("Successfully logged in to Betfair");

    // Get account funds
    let account_funds = client.get_account_funds().await?;

    // Print account information
    info!("Account Funds Information:");
    info!(
        "Available to Bet Balance: {:.2}",
        account_funds.available_to_bet_balance
    );
    info!("Exposure: {:.2}", account_funds.exposure);
    info!("Exposure Limit: {:.2}", account_funds.exposure_limit);
    info!("Discount Rate: {:.2}", account_funds.discount_rate);
    info!("Points Balance: {:.2}", account_funds.points_balance);
    info!("Wallet: {}", account_funds.wallet);

    // Print as JSON for better readability
    info!(
        "\nAccount Funds (JSON format):\n{}",
        serde_json::to_string_pretty(&json!({
            "available_to_bet_balance": account_funds.available_to_bet_balance,
            "exposure": account_funds.exposure,
            "exposure_limit": account_funds.exposure_limit,
            "discount_rate": account_funds.discount_rate,
            "points_balance": account_funds.points_balance,
            "wallet": account_funds.wallet,
        }))?
    );

    info!("Account Funds Example Completed");
    Ok(())
}
