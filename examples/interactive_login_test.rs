use anyhow::Result;
use betfair_rs::{config::Config, UnifiedBetfairClient};
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file first
    dotenv::dotenv().ok();

    // Initialize logging (after loading .env so RUST_LOG is available)
    tracing_subscriber::fmt::init();

    // Get credentials from environment variables
    let username = env::var("BETFAIR_USERNAME").expect("BETFAIR_USERNAME must be set");
    let password = env::var("BETFAIR_PASSWORD").expect("BETFAIR_PASSWORD must be set");
    let api_key = env::var("BETFAIR_API_KEY").expect("BETFAIR_API_KEY must be set");

    // Create a minimal config with just the API key (other fields won't be used for interactive login)
    let config = Config {
        betfair: betfair_rs::config::BetfairConfig {
            username: String::new(), // Not used for interactive login
            password: String::new(), // Not used for interactive login
            api_key,
            pem_path: String::new(), // Not used for interactive login
        },
    };

    // Initialize unified client
    let mut client = UnifiedBetfairClient::new(config);

    info!("Attempting interactive login...");

    // Use interactive login instead of certificate login
    let _login_response = client.login_interactive(username, password).await?;

    info!("Login successful!");
    info!("Session token: {:?}", client.get_session_token());

    // Test a simple API call to verify the login worked
    info!("Testing API call - getting account details...");
    let account_details = client.get_account_details().await?;
    info!("Account details: {:?}", account_details);

    // Test getting sports list
    info!("Getting list of sports...");
    let sports = client.list_sports(None).await?;
    info!("Found {} sports", sports.len());
    for sport in sports.iter().take(5) {
        info!("Sport: {} ({})", sport.event_type.name, sport.event_type.id);
    }

    Ok(())
}
