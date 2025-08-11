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
    
    // Get filter type from command line
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        show_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "sports" => {
            // List all sports
            println!("Fetching all sports...\n");
            let sports = client.list_sports(None).await?;
            
            println!("Found {} sports:", sports.len());
            for sport in sports.iter().take(10) {
                println!("  {} - {} ({} markets)", 
                    sport.event_type.id, 
                    sport.event_type.name, 
                    sport.market_count
                );
            }
            if sports.len() > 10 {
                println!("  ... and {} more", sports.len() - 10);
            }
        }
        
        "sport" => {
            // List competitions for a specific sport
            let sport_id = args.get(2).unwrap_or(&"1".to_string()).clone();
            
            println!("Fetching competitions for sport ID {}...\n", sport_id);
            
            let filter = MarketFilter {
                event_type_ids: Some(vec![sport_id.clone()]),
                ..Default::default()
            };
            
            let competitions = client.list_competitions(Some(filter)).await?;
            
            println!("Found {} competitions:", competitions.len());
            for comp in competitions.iter().take(10) {
                println!("  {} - {} ({} markets)", 
                    comp.competition.id,
                    comp.competition.name,
                    comp.market_count
                );
            }
            if competitions.len() > 10 {
                println!("  ... and {} more", competitions.len() - 10);
            }
        }
        
        "country" => {
            // List competitions for a specific country
            let country_code = args.get(2).unwrap_or(&"GB".to_string()).clone();
            
            println!("Fetching competitions in country {}...\n", country_code);
            
            let filter = MarketFilter {
                market_countries: Some(vec![country_code.clone()]),
                ..Default::default()
            };
            
            let competitions = client.list_competitions(Some(filter)).await?;
            
            if competitions.is_empty() {
                println!("No competitions found for country: {}", country_code);
                println!("\nTry: GB, US, ES, FR, DE, IT, BR, AU");
            } else {
                println!("Found {} competitions in {}:", competitions.len(), country_code);
                
                // Group by sport (we'll need to fetch sports to identify them)
                let sports = client.list_sports(None).await?;
                let mut by_sport = std::collections::HashMap::new();
                
                for sport in &sports {
                    let sport_filter = MarketFilter {
                        event_type_ids: Some(vec![sport.event_type.id.clone()]),
                        market_countries: Some(vec![country_code.clone()]),
                        ..Default::default()
                    };
                    
                    let sport_comps = client.list_competitions(Some(sport_filter)).await?;
                    if !sport_comps.is_empty() {
                        by_sport.insert(sport.event_type.name.clone(), sport_comps);
                    }
                }
                
                for (sport_name, comps) in by_sport.iter() {
                    println!("\n{} ({} competitions):", sport_name, comps.len());
                    for comp in comps.iter().take(5) {
                        println!("  - {} ({} markets)", comp.competition.name, comp.market_count);
                    }
                    if comps.len() > 5 {
                        println!("  ... and {} more", comps.len() - 5);
                    }
                }
            }
        }
        
        "combined" => {
            // Combined filter: sport + country
            let sport_id = args.get(2).unwrap_or(&"1".to_string()).clone();
            let country_code = args.get(3).unwrap_or(&"GB".to_string()).clone();
            
            println!("Fetching competitions for sport {} in country {}...\n", sport_id, country_code);
            
            let filter = MarketFilter {
                event_type_ids: Some(vec![sport_id.clone()]),
                market_countries: Some(vec![country_code.clone()]),
                ..Default::default()
            };
            
            let competitions = client.list_competitions(Some(filter)).await?;
            
            if competitions.is_empty() {
                println!("No competitions found for sport {} in country {}", sport_id, country_code);
            } else {
                println!("Found {} competitions:", competitions.len());
                for comp in &competitions {
                    println!("  {} - {} ({} markets)",
                        comp.competition.id,
                        comp.competition.name,
                        comp.market_count
                    );
                }
            }
        }
        
        "events" => {
            // List events with optional filters
            let sport_id = args.get(2);
            let competition_id = args.get(3);
            
            let mut filter = MarketFilter::default();
            
            if let Some(sid) = sport_id {
                filter.event_type_ids = Some(vec![sid.clone()]);
                println!("Fetching events for sport {}...", sid);
            } else {
                println!("Fetching all events (this may take a while)...");
            }
            
            if let Some(cid) = competition_id {
                filter.competition_ids = Some(vec![cid.clone()]);
                println!("Filtered by competition {}...", cid);
            }
            
            println!();
            
            let events = client.list_events(Some(filter)).await?;
            
            println!("Found {} events:", events.len());
            for event in events.iter().take(20) {
                println!("  {} - {} ({} markets)",
                    event.event.id,
                    event.event.name,
                    event.market_count
                );
            }
            if events.len() > 20 {
                println!("  ... and {} more", events.len() - 20);
            }
        }
        
        "inplay" => {
            // List in-play events only
            println!("Fetching in-play events...\n");
            
            let filter = MarketFilter {
                in_play_only: Some(true),
                ..Default::default()
            };
            
            let events = client.list_events(Some(filter)).await?;
            
            if events.is_empty() {
                println!("No in-play events currently available");
            } else {
                println!("Found {} in-play events:", events.len());
                
                // Group by sport
                let sports = client.list_sports(None).await?;
                for sport in &sports {
                    let sport_filter = MarketFilter {
                        event_type_ids: Some(vec![sport.event_type.id.clone()]),
                        in_play_only: Some(true),
                        ..Default::default()
                    };
                    
                    let sport_events = client.list_events(Some(sport_filter)).await?;
                    if !sport_events.is_empty() {
                        println!("\n{} ({} events):", sport.event_type.name, sport_events.len());
                        for event in sport_events.iter().take(5) {
                            println!("  - {} ({} markets)", 
                                event.event.name, 
                                event.market_count
                            );
                        }
                        if sport_events.len() > 5 {
                            println!("  ... and {} more", sport_events.len() - 5);
                        }
                    }
                }
            }
        }
        
        _ => {
            println!("Unknown filter type: {}", args[1]);
            show_usage();
        }
    }
    
    Ok(())
}

fn show_usage() {
    println!("Betfair API Filter Demo");
    println!("=======================\n");
    println!("Usage: cargo run --example filter_demo <filter_type> [options]\n");
    println!("Filter types:");
    println!("  sports                    - List all sports");
    println!("  sport <id>               - List competitions for a sport (default: 1=Soccer)");
    println!("  country <code>           - List competitions in a country (default: GB)");
    println!("  combined <sport> <country> - Combined sport + country filter");
    println!("  events [sport] [comp]    - List events with optional filters");
    println!("  inplay                   - List in-play events only");
    println!("\nExamples:");
    println!("  cargo run --example filter_demo sports");
    println!("  cargo run --example filter_demo sport 2         # Tennis");
    println!("  cargo run --example filter_demo country US      # USA competitions");
    println!("  cargo run --example filter_demo combined 1 GB   # Soccer in GB");
    println!("  cargo run --example filter_demo events 1        # Soccer events");
    println!("  cargo run --example filter_demo inplay          # Live events");
}