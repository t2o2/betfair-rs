use anyhow::Result;
use betfair_rs::{api_client::BetfairApiClient, config::Config, dto::MarketFilter};
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
    
    // Get country code from command line argument
    let country_code = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            println!("No country code provided. Using GB (Great Britain) as default.");
            println!("Usage: cargo run --example competitions_by_country <COUNTRY_CODE>");
            println!("Examples: GB, US, ES, FR, DE, IT, BR, AR\n");
            "GB".to_string()
        });
    
    println!("Fetching all competitions in country: {}\n", country_code);
    
    // Create filter for specific country
    let filter = MarketFilter {
        market_countries: Some(vec![country_code.clone()]),
        ..Default::default()
    };
    
    match client.list_competitions_filtered(filter).await {
        Ok(mut competitions) => {
            if competitions.is_empty() {
                println!("No competitions found for country: {}", country_code);
                println!("\nTry one of these country codes:");
                println!("GB - Great Britain");
                println!("US - United States");
                println!("ES - Spain");
                println!("FR - France");
                println!("DE - Germany");
                println!("IT - Italy");
                println!("BR - Brazil");
                println!("AU - Australia");
            } else {
                // Sort by market count
                competitions.sort_by(|a, b| b.market_count.cmp(&a.market_count));
                
                println!("Found {} competitions in {}:\n", competitions.len(), country_code);
                
                // Group by sport
                let mut by_sport: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
                
                // We need to identify sport for each competition
                // Since competition doesn't have sport info directly, we'll get all sports first
                let sports = client.list_sports().await?;
                
                // For each sport, get its competitions
                for sport in &sports {
                    let sport_comps = client.list_competitions(vec![sport.event_type.id.clone()]).await?;
                    for comp in &sport_comps {
                        // Check if this competition is in our filtered list
                        if competitions.iter().any(|c| c.competition.id == comp.competition.id) {
                            by_sport.entry(sport.event_type.name.clone())
                                .or_insert_with(Vec::new)
                                .push(comp.clone());
                        }
                    }
                }
                
                // Show competitions grouped by sport
                println!("Competitions by Sport:");
                println!("{}", "=".repeat(90));
                
                let mut sports_list: Vec<_> = by_sport.keys().cloned().collect();
                sports_list.sort();
                
                for sport_name in sports_list {
                    let sport_comps = &by_sport[&sport_name];
                    println!("\n{} ({} competitions)", sport_name, sport_comps.len());
                    println!("{}", "-".repeat(90));
                    println!("{:<10} {:<50} {:<15} {}", "ID", "Name", "Region", "Markets");
                    
                    for comp in sport_comps.iter().take(5) {
                        let name = if comp.competition.name.len() > 49 {
                            format!("{}...", comp.competition.name.chars().take(46).collect::<String>())
                        } else {
                            comp.competition.name.clone()
                        };
                        
                        println!(
                            "{:<10} {:<50} {:<15} {}",
                            comp.competition.id,
                            name,
                            comp.competition_region.as_ref().unwrap_or(&country_code),
                            comp.market_count
                        );
                    }
                    
                    if sport_comps.len() > 5 {
                        println!("           ... and {} more", sport_comps.len() - 5);
                    }
                }
                
                // Show top competitions overall
                println!("\n\nTop 10 Competitions by Market Count:");
                println!("{}", "=".repeat(90));
                println!("{:<10} {:<50} {:<15} {}", "ID", "Name", "Region", "Markets");
                println!("{}", "-".repeat(90));
                
                for comp in competitions.iter().take(10) {
                    let name = if comp.competition.name.len() > 49 {
                        format!("{}...", comp.competition.name.chars().take(46).collect::<String>())
                    } else {
                        comp.competition.name.clone()
                    };
                    
                    println!(
                        "{:<10} {:<50} {:<15} {}",
                        comp.competition.id,
                        name,
                        comp.competition_region.as_ref().unwrap_or(&country_code),
                        comp.market_count
                    );
                }
                
                println!("\nStatistics:");
                println!("- Total competitions: {}", competitions.len());
                println!("- Total markets: {}", competitions.iter().map(|c| c.market_count).sum::<i32>());
                println!("- Sports represented: {}", by_sport.len());
            }
        }
        Err(e) => {
            eprintln!("Error fetching competitions: {}", e);
        }
    }
    
    Ok(())
}