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
    
    // Get sport ID from command line argument or show menu
    let sport_id = if let Some(id) = env::args().nth(1) {
        id
    } else {
        // Show available sports
        println!("Fetching available sports...\n");
        let sports = client.list_sports().await?;
        
        println!("Available sports:");
        println!("{:<10} {:<30} {}", "ID", "Name", "Market Count");
        println!("{}", "-".repeat(60));
        
        let top_sports: Vec<_> = sports.iter()
            .filter(|s| s.market_count > 0)
            .take(15)
            .collect();
        
        for sport in &top_sports {
            println!(
                "{:<10} {:<30} {}", 
                sport.event_type.id, 
                sport.event_type.name,
                sport.market_count
            );
        }
        
        println!("\nUsing Soccer (ID: 1) as default. Run with sport ID to see other sports.");
        println!("Example: cargo run --example list_competitions 2  (for Tennis)\n");
        
        "1".to_string()
    };
    
    // Get sport name
    let sports = client.list_sports().await?;
    let sport_name = sports
        .iter()
        .find(|s| s.event_type.id == sport_id)
        .map(|s| s.event_type.name.clone())
        .unwrap_or_else(|| format!("Sport ID {}", sport_id));
    
    println!("Fetching competitions for: {} (ID: {})\n", sport_name, sport_id);
    
    // List all competitions for this sport
    match client.list_competitions(vec![sport_id.clone()]).await {
        Ok(mut competitions) => {
            if competitions.is_empty() {
                println!("No competitions found for {}.", sport_name);
            } else {
                // Sort by market count (descending)
                competitions.sort_by(|a, b| b.market_count.cmp(&a.market_count));
                
                println!("Found {} competitions for {}:\n", competitions.len(), sport_name);
                
                // Group by region
                let mut by_region: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
                for comp in &competitions {
                    let region = comp.competition_region.clone().unwrap_or_else(|| "International".to_string());
                    by_region.entry(region).or_insert_with(Vec::new).push(comp);
                }
                
                // Show summary by region
                println!("Summary by Region:");
                println!("{:<20} {}", "Region", "Competition Count");
                println!("{}", "-".repeat(40));
                
                let mut regions: Vec<_> = by_region.keys().cloned().collect();
                regions.sort();
                
                for region in &regions {
                    println!("{:<20} {}", region, by_region[region].len());
                }
                
                println!("\nTop Competitions by Market Count:");
                println!("{:<10} {:<50} {:<15} {}", "ID", "Name", "Region", "Markets");
                println!("{}", "-".repeat(90));
                
                for comp in competitions.iter().take(30) {
                    let name = if comp.competition.name.len() > 49 {
                        format!("{}...", comp.competition.name.chars().take(46).collect::<String>())
                    } else {
                        comp.competition.name.clone()
                    };
                    
                    println!(
                        "{:<10} {:<50} {:<15} {}", 
                        comp.competition.id,
                        name,
                        comp.competition_region.as_ref().unwrap_or(&"International".to_string()),
                        comp.market_count
                    );
                }
                
                if competitions.len() > 30 {
                    println!("\n... and {} more competitions", competitions.len() - 30);
                }
                
                // Show competitions with most markets
                println!("\nStatistics:");
                println!("- Total competitions: {}", competitions.len());
                println!("- Total markets: {}", competitions.iter().map(|c| c.market_count).sum::<i32>());
                println!("- Average markets per competition: {:.1}", 
                    competitions.iter().map(|c| c.market_count).sum::<i32>() as f64 / competitions.len() as f64);
                
                if let Some(max_comp) = competitions.first() {
                    println!("- Most active competition: {} ({} markets)", 
                        max_comp.competition.name, max_comp.market_count);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching competitions: {}", e);
        }
    }
    
    Ok(())
}