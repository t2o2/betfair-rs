use anyhow::Result;
use tracing::info;
mod betfair;
mod config;
mod models;
use std::error::Error;
mod streamer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Betfair Trading Bot Starting...");

    let config = config::Config::new()?;
    let mut betfair_client = betfair::BetfairClient::new(config);
    betfair_client.login().await?;
    let token = betfair_client.get_session_token().await.unwrap();
    info!("Betfair session token: {}", token);

    betfair_client.start_listening().await?;
    info!("Betfair client started listening");

    betfair_client.subscribe_to_market("1.240911695".to_string()).await?;

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    info!("Betfair market subscribed");
    Ok(())
} 