use anyhow::Result;
use betfair_rs::{api_client::BetfairApiClient, config::Config};
use std::env;

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
    
    // Get sport ID from command line argument or use Soccer (1) as default
    let sport_id = env::args()
        .nth(1)
        .unwrap_or_else(|| "1".to_string());
    
    // First, let's get the sport name
    println!("Fetching sport information...");
    let sports = client.list_sports().await?;
    let sport_name = sports
        .iter()
        .find(|s| s.event_type.id == sport_id)
        .map(|s| s.event_type.name.clone())
        .unwrap_or_else(|| format!("Sport ID {}", sport_id));
    
    println!("Sport: {} (ID: {})\n", sport_name, sport_id);
    
    // List competitions for this sport
    println!("Fetching competitions for {}...\n", sport_name);
    match client.list_competitions(vec![sport_id.clone()]).await {
        Ok(competitions) => {
            if competitions.is_empty() {
                println!("No competitions found for this sport.");
            } else {
                println!("Found {} competitions:", competitions.len());
                println!("{:<10} {:<40} {:<15} {}", "ID", "Name", "Region", "Market Count");
                println!("{}", "-".repeat(80));
                
                for comp in competitions.iter().take(10) {
                    println!(
                        "{:<10} {:<40} {:<15} {}", 
                        comp.competition.id,
                        comp.competition.name.chars().take(39).collect::<String>(),
                        comp.competition_region.as_ref().unwrap_or(&"-".to_string()),
                        comp.market_count
                    );
                }
                
                if competitions.len() > 10 {
                    println!("... and {} more competitions", competitions.len() - 10);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching competitions: {}", e);
        }
    }
    
    println!();
    
    // List events for this sport
    println!("Fetching events for {}...\n", sport_name);
    match client.list_events(vec![sport_id.clone()]).await {
        Ok(events) => {
            if events.is_empty() {
                println!("No events found for this sport.");
            } else {
                println!("Found {} events:", events.len());
                println!("{:<10} {:<50} {:<15} {}", "ID", "Name", "Country", "Market Count");
                println!("{}", "-".repeat(90));
                
                for event in events.iter().take(20) {
                    println!(
                        "{:<10} {:<50} {:<15} {}", 
                        event.event.id,
                        event.event.name.chars().take(49).collect::<String>(),
                        event.event.country_code.as_ref().unwrap_or(&"-".to_string()),
                        event.market_count
                    );
                }
                
                if events.len() > 20 {
                    println!("... and {} more events", events.len() - 20);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching events: {}", e);
        }
    }
    
    println!("\nTip: Run with a sport ID as argument to see events for that sport");
    println!("Example: cargo run --example list_events 2  (for Tennis)");
    
    Ok(())
}