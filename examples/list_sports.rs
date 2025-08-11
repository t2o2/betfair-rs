use anyhow::Result;
use betfair_rs::{api_client::BetfairApiClient, config::Config};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::new()?;
    
    // Create API client
    let mut client = BetfairApiClient::new(config);
    
    // Login
    println!("Logging in to Betfair...");
    let login_response = client.login().await?;
    
    if login_response.login_status != "SUCCESS" {
        eprintln!("Login failed: {}", login_response.login_status);
        return Ok(());
    }
    
    println!("Login successful!\n");
    
    // Fetch all sports
    println!("Fetching all sports...\n");
    
    match client.list_sports().await {
        Ok(sports) => {
            println!("Found {} sports:", sports.len());
            println!("{:<10} {:<30} {}", "ID", "Name", "Market Count");
            println!("{}", "-".repeat(60));
            
            for sport in sports {
                println!(
                    "{:<10} {:<30} {}", 
                    sport.event_type.id, 
                    sport.event_type.name,
                    sport.market_count
                );
            }
        }
        Err(e) => {
            eprintln!("Error fetching sports: {}", e);
        }
    }
    
    Ok(())
}