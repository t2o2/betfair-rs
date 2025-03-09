use anyhow::Result;
use tracing::info;
mod betfair;
mod config;
mod models;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Betfair Trading Bot Starting...");

    let config = config::Config::new()?;
    let mut betfair_client = betfair::BetfairClient::new(config);
    let token = betfair_client.login().await?;
    info!("Login token: {}", token);

    Ok(())
} 