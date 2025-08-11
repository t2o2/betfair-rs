use anyhow::Result;
use betfair_rs::{api_client::BetfairApiClient, config::Config};
use std::env;
use std::collections::HashMap;

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
    
    // Get sport name from command line argument
    let sport_query = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            println!("No sport name provided. Showing all sports overview.");
            println!("Usage: cargo run --example competitions_by_sport <SPORT_NAME>");
            println!("Examples: Soccer, Tennis, Basketball, Golf, Cricket, Boxing\n");
            "ALL".to_string()
        });
    
    // Fetch all sports
    let sports = client.list_sports().await?;
    
    if sport_query.to_uppercase() == "ALL" {
        // Show overview of all sports with competitions
        println!("SPORTS OVERVIEW");
        println!("{}", "=".repeat(100));
        println!("{:<10} {:<25} {:<15} {:<15} {:<30}", 
            "Sport ID", "Sport Name", "Total Markets", "Competitions", "Top Competition");
        println!("{}", "-".repeat(100));
        
        let mut total_markets = 0;
        let mut total_competitions = 0;
        
        for sport in sports.iter().filter(|s| s.market_count > 0) {
            // Get competitions for this sport
            let competitions = client.list_competitions(vec![sport.event_type.id.clone()]).await?;
            
            if !competitions.is_empty() {
                let top_comp = competitions.iter()
                    .max_by_key(|c| c.market_count)
                    .unwrap();
                
                let top_comp_name = if top_comp.competition.name.len() > 29 {
                    format!("{}...", top_comp.competition.name.chars().take(26).collect::<String>())
                } else {
                    top_comp.competition.name.clone()
                };
                
                println!("{:<10} {:<25} {:<15} {:<15} {:<30}",
                    sport.event_type.id,
                    sport.event_type.name.chars().take(24).collect::<String>(),
                    sport.market_count,
                    competitions.len(),
                    top_comp_name
                );
                
                total_markets += sport.market_count;
                total_competitions += competitions.len() as i32;
            }
        }
        
        println!("{}", "-".repeat(100));
        println!("{:<10} {:<25} {:<15} {:<15}",
            "", "TOTAL", total_markets, total_competitions);
        
        println!("\nTo see details for a specific sport, run:");
        println!("cargo run --example competitions_by_sport <SPORT_NAME>");
        
    } else {
        // Find the sport by name (case-insensitive partial match)
        let matching_sports: Vec<_> = sports.iter()
            .filter(|s| s.event_type.name.to_lowercase().contains(&sport_query.to_lowercase()))
            .collect();
        
        if matching_sports.is_empty() {
            println!("No sport found matching: {}", sport_query);
            println!("\nAvailable sports:");
            for sport in sports.iter().filter(|s| s.market_count > 0).take(20) {
                println!("  - {}", sport.event_type.name);
            }
            return Ok(());
        }
        
        // If multiple matches, show them and use the first one
        if matching_sports.len() > 1 {
            println!("Found {} sports matching '{}', using the first one:", matching_sports.len(), sport_query);
            for sport in &matching_sports {
                println!("  - {} (ID: {})", sport.event_type.name, sport.event_type.id);
            }
            println!();
        }
        
        let selected_sport = matching_sports[0];
        let sport_id = &selected_sport.event_type.id;
        let sport_name = &selected_sport.event_type.name;
        
        println!("SPORT: {} (ID: {})", sport_name, sport_id);
        println!("{}", "=".repeat(100));
        println!("Total Markets: {}\n", selected_sport.market_count);
        
        // Get all competitions for this sport
        let mut competitions = client.list_competitions(vec![sport_id.clone()]).await?;
        
        if competitions.is_empty() {
            println!("No competitions found for {}", sport_name);
            return Ok(());
        }
        
        // Sort by market count
        competitions.sort_by(|a, b| b.market_count.cmp(&a.market_count));
        
        // Group by region
        let mut by_region: HashMap<String, Vec<_>> = HashMap::new();
        for comp in &competitions {
            let region = comp.competition_region.clone()
                .unwrap_or_else(|| "International".to_string());
            by_region.entry(region).or_insert_with(Vec::new).push(comp.clone());
        }
        
        // Statistics
        println!("COMPETITION STATISTICS");
        println!("{}", "-".repeat(100));
        println!("Total Competitions: {}", competitions.len());
        println!("Total Markets: {}", competitions.iter().map(|c| c.market_count).sum::<i32>());
        println!("Average Markets per Competition: {:.1}", 
            competitions.iter().map(|c| c.market_count).sum::<i32>() as f64 / competitions.len() as f64);
        
        if let Some(max_comp) = competitions.first() {
            println!("Most Active Competition: {} ({} markets)", 
                max_comp.competition.name, max_comp.market_count);
        }
        
        if let Some(min_comp) = competitions.last() {
            println!("Least Active Competition: {} ({} markets)", 
                min_comp.competition.name, min_comp.market_count);
        }
        
        // Regional breakdown
        println!("\nREGIONAL BREAKDOWN");
        println!("{}", "-".repeat(100));
        println!("{:<25} {:<15} {:<15} {:<40}", "Region", "Competitions", "Total Markets", "Top Competition");
        println!("{}", "-".repeat(100));
        
        let mut regions: Vec<_> = by_region.keys().cloned().collect();
        regions.sort();
        
        for region in regions {
            let region_comps = &by_region[&region];
            let total_markets: i32 = region_comps.iter().map(|c| c.market_count).sum();
            let top_comp = region_comps.iter().max_by_key(|c| c.market_count).unwrap();
            
            let top_comp_name = if top_comp.competition.name.len() > 39 {
                format!("{}...", top_comp.competition.name.chars().take(36).collect::<String>())
            } else {
                top_comp.competition.name.clone()
            };
            
            println!("{:<25} {:<15} {:<15} {:<40}",
                region.chars().take(24).collect::<String>(),
                region_comps.len(),
                total_markets,
                top_comp_name
            );
        }
        
        // Active competitions (with markets)
        let active_competitions: Vec<_> = competitions.iter()
            .filter(|c| c.market_count > 0)
            .collect();
        
        println!("\nACTIVE COMPETITIONS (with markets)");
        println!("{}", "-".repeat(100));
        println!("{:<10} {:<50} {:<20} {:<10}", "ID", "Name", "Region", "Markets");
        println!("{}", "-".repeat(100));
        
        for (i, comp) in active_competitions.iter().enumerate() {
            if i >= 25 {
                println!("... and {} more active competitions", active_competitions.len() - 25);
                break;
            }
            
            let name = if comp.competition.name.len() > 49 {
                format!("{}...", comp.competition.name.chars().take(46).collect::<String>())
            } else {
                comp.competition.name.clone()
            };
            
            println!("{:<10} {:<50} {:<20} {:<10}",
                comp.competition.id,
                name,
                comp.competition_region.as_ref()
                    .unwrap_or(&"International".to_string())
                    .chars().take(19).collect::<String>(),
                comp.market_count
            );
        }
        
        // Inactive competitions (no markets currently)
        let inactive_competitions: Vec<_> = competitions.iter()
            .filter(|c| c.market_count == 0)
            .collect();
        
        if !inactive_competitions.is_empty() {
            println!("\nINACTIVE COMPETITIONS (no current markets)");
            println!("{}", "-".repeat(100));
            println!("Total: {} competitions", inactive_competitions.len());
            
            if inactive_competitions.len() <= 10 {
                for comp in &inactive_competitions {
                    println!("  - {} ({})", 
                        comp.competition.name,
                        comp.competition_region.as_ref().unwrap_or(&"International".to_string())
                    );
                }
            } else {
                for comp in inactive_competitions.iter().take(5) {
                    println!("  - {} ({})", 
                        comp.competition.name,
                        comp.competition_region.as_ref().unwrap_or(&"International".to_string())
                    );
                }
                println!("  ... and {} more", inactive_competitions.len() - 5);
            }
        }
        
        // Market distribution
        println!("\nMARKET DISTRIBUTION");
        println!("{}", "-".repeat(100));
        
        let high_market = competitions.iter().filter(|c| c.market_count >= 100).count();
        let medium_market = competitions.iter().filter(|c| c.market_count >= 10 && c.market_count < 100).count();
        let low_market = competitions.iter().filter(|c| c.market_count > 0 && c.market_count < 10).count();
        let no_market = competitions.iter().filter(|c| c.market_count == 0).count();
        
        println!("High activity (100+ markets):    {} competitions", high_market);
        println!("Medium activity (10-99 markets):  {} competitions", medium_market);
        println!("Low activity (1-9 markets):       {} competitions", low_market);
        println!("No current markets:               {} competitions", no_market);
    }
    
    Ok(())
}