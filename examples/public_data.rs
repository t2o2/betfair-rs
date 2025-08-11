use betfair_rs::public_data::PublicDataClient;
use betfair_rs::config::Config;
use std::env;

fn main() -> anyhow::Result<()> {
    // Try to get app key from environment variable first, then config file
    let client = if let Ok(app_key) = env::var("BETFAIR_APP_KEY") {
        println!("Using app key from BETFAIR_APP_KEY environment variable");
        PublicDataClient::with_app_key(app_key)
    } else if let Ok(config) = Config::new() {
        println!("Using app key from config.toml");
        PublicDataClient::with_app_key(config.betfair.api_key)
    } else {
        println!("No BETFAIR_APP_KEY or config.toml found");
        println!("Note: This will likely fail as the endpoint requires authentication");
        PublicDataClient::new()
    };
    
    println!("Fetching all sports from Betfair public API...\n");
    
    match client.list_sports() {
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