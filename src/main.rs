use anyhow::Result;

mod betfair;
mod config;
mod models;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    println!("Betfair Trading Bot Starting...");

    let config = config::Config::new()?;
    let mut betfair_client = betfair::BetfairClient::new(config);
    betfair_client.login().await.unwrap();

    Ok(())
} 