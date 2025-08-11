use anyhow::Result;
use betfair_rs::{api_client::BetfairApiClient, config::Config, dto::{MarketFilter, ListMarketCatalogueRequest, common::{MarketStatus, Side, OrderType, PersistenceType}, account::GetAccountFundsRequest, order::{PlaceOrdersRequest, PlaceInstruction, LimitOrder, CancelOrdersRequest, CancelInstruction, ListCurrentOrdersRequest}}};
use clap::{Parser, Subcommand};
use chrono::{DateTime, Utc};

#[derive(Parser)]
#[command(name = "betfair-cli")]
#[command(about = "Betfair API CLI tool", long_about = None)]
#[command(after_help = "EXAMPLES:
    # List all available sports
    cargo run --example cli -- list-sports

    # List competitions for Soccer (sport ID 1)
    cargo run --example cli -- list-competitions -s 1

    # List events for Soccer in Premier League (competition ID 10932509)
    cargo run --example cli -- list-events -s 1 -c 10932509

    # List all markets for a specific event
    cargo run --example cli -- list-markets -s 1 -e 34433119

    # List markets filtered by competition and event
    cargo run --example cli -- list-markets -s 1 -c 10932509 -e 34433119
    
    # Get odds for a specific market
    cargo run --example cli -- get-odds -m 1.234567890
    
    # List runners for a specific market
    cargo run --example cli -- list-runners -m 1.234567890")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all sports
    ListSports,
    
    /// List competitions for a sport
    ListCompetitions {
        /// Sport ID
        #[arg(short, long)]
        sport: String,
    },
    
    /// List events 
    ListEvents {
        /// Sport ID
        #[arg(short, long)]
        sport: String,
        
        /// Competition ID (optional)
        #[arg(short, long)]
        competition: Option<String>,
    },
    
    /// List markets
    ListMarkets {
        /// Sport ID
        #[arg(short, long)]
        sport: String,
        
        /// Competition ID (optional)
        #[arg(short, long)]
        competition: Option<String>,
        
        /// Event ID (optional)
        #[arg(short, long)]
        event: Option<String>,
    },
    
    /// Get odds for a specific market
    GetOdds {
        /// Market ID
        #[arg(short, long)]
        market: String,
    },
    
    /// List runners for a specific market
    ListRunners {
        /// Market ID
        #[arg(short, long)]
        market: String,
    },
    
    /// Get account funds information
    GetAccount,
    
    /// Place an order on a market
    PlaceOrder {
        /// Market ID
        #[arg(short, long)]
        market: String,
        
        /// Selection/Runner ID
        #[arg(short = 'r', long)]
        runner: u64,
        
        /// Side (BACK or LAY)
        #[arg(short = 's', long)]
        side: String,
        
        /// Price/Odds (e.g., 2.5)
        #[arg(short, long)]
        price: f64,
        
        /// Stake amount in £
        #[arg(short = 'a', long)]
        amount: f64,
        
        /// Persistence type (LAPSE or PERSIST, default: LAPSE)
        #[arg(long, default_value = "LAPSE")]
        persistence: String,
    },
    
    /// Cancel an order
    CancelOrder {
        /// Market ID
        #[arg(short, long)]
        market: String,
        
        /// Bet ID to cancel
        #[arg(short, long)]
        bet_id: String,
    },
    
    /// List current orders
    ListOrders {
        /// Market ID (optional - if not provided, lists all orders)
        #[arg(short, long)]
        market: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
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
    
    match cli.command {
        Commands::ListSports => {
            list_sports(&mut client).await?;
        }
        Commands::ListCompetitions { sport } => {
            list_competitions(&mut client, &sport).await?;
        }
        Commands::ListEvents { sport, competition } => {
            list_events(&mut client, &sport, competition).await?;
        }
        Commands::ListMarkets { sport, competition, event } => {
            list_markets(&mut client, &sport, competition, event).await?;
        }
        Commands::GetOdds { market } => {
            get_odds(&mut client, &market).await?;
        }
        Commands::ListRunners { market } => {
            list_runners(&mut client, &market).await?;
        }
        Commands::GetAccount => {
            get_account(&mut client).await?;
        }
        Commands::PlaceOrder { market, runner, side, price, amount, persistence } => {
            place_order(&mut client, &market, runner, &side, price, amount, &persistence).await?;
        }
        Commands::CancelOrder { market, bet_id } => {
            cancel_order(&mut client, &market, &bet_id).await?;
        }
        Commands::ListOrders { market } => {
            list_orders(&mut client, market.as_deref()).await?;
        }
    }
    
    Ok(())
}

async fn list_sports(client: &mut BetfairApiClient) -> Result<()> {
    println!("Fetching all sports...\n");
    
    match client.list_sports(None).await {
        Ok(mut sports) => {
            // Sort by market count descending
            sports.sort_by(|a, b| b.market_count.cmp(&a.market_count));
            
            println!("Available Sports:");
            println!("{:<10} {:<30} {:<15}", "ID", "Name", "Market Count");
            println!("{}", "-".repeat(55));
            
            for sport in sports {
                println!(
                    "{:<10} {:<30} {:<15}",
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

async fn list_competitions(client: &mut BetfairApiClient, sport_id: &str) -> Result<()> {
    // Get sport name
    let sports = client.list_sports(None).await?;
    let sport_name = sports
        .iter()
        .find(|s| s.event_type.id == sport_id)
        .map(|s| s.event_type.name.clone())
        .unwrap_or_else(|| format!("Sport ID {}", sport_id));
    
    println!("Fetching competitions for: {}\n", sport_name);
    
    let filter = MarketFilter {
        event_type_ids: Some(vec![sport_id.to_string()]),
        ..Default::default()
    };
    
    match client.list_competitions(Some(filter)).await {
        Ok(mut competitions) => {
            if competitions.is_empty() {
                println!("No competitions found for sport ID: {}", sport_id);
            } else {
                // Sort by market count descending
                competitions.sort_by(|a, b| b.market_count.cmp(&a.market_count));
                
                println!("Competitions for {}:", sport_name);
                println!("{:<12} {:<40} {:<15} {:<10}", "ID", "Name", "Region", "Markets");
                println!("{}", "-".repeat(80));
                
                let display_count = std::cmp::min(50, competitions.len());
                for comp in competitions.iter().take(display_count) {
                    let name = if comp.competition.name.len() > 39 {
                        format!("{}...", comp.competition.name.chars().take(36).collect::<String>())
                    } else {
                        comp.competition.name.clone()
                    };
                    
                    println!(
                        "{:<12} {:<40} {:<15} {:<10}",
                        comp.competition.id,
                        name,
                        comp.competition_region.as_ref().unwrap_or(&"-".to_string()),
                        comp.market_count
                    );
                }
                
                if competitions.len() > display_count {
                    println!("\n... and {} more competitions", competitions.len() - display_count);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching competitions: {}", e);
        }
    }
    
    Ok(())
}

async fn list_events(client: &mut BetfairApiClient, sport_id: &str, competition_id: Option<String>) -> Result<()> {
    // Get sport name
    let sports = client.list_sports(None).await?;
    let sport_name = sports
        .iter()
        .find(|s| s.event_type.id == sport_id)
        .map(|s| s.event_type.name.clone())
        .unwrap_or_else(|| format!("Sport ID {}", sport_id));
    
    let mut title = format!("Events for: {}", sport_name);
    
    let mut filter = MarketFilter {
        event_type_ids: Some(vec![sport_id.to_string()]),
        ..Default::default()
    };
    
    // Add competition filter if provided
    if let Some(comp_id) = &competition_id {
        filter.competition_ids = Some(vec![comp_id.clone()]);
        
        // Get competition name
        let comp_filter = MarketFilter {
            competition_ids: Some(vec![comp_id.clone()]),
            ..Default::default()
        };
        let competitions = client.list_competitions(Some(comp_filter)).await?;
        if let Some(comp) = competitions.first() {
            title = format!("Events for: {} - {}", sport_name, comp.competition.name);
        }
    }
    
    println!("Fetching {}\n", title);
    
    match client.list_events(Some(filter)).await {
        Ok(mut events) => {
            if events.is_empty() {
                println!("No events found.");
                if competition_id.is_none() {
                    println!("Try adding a competition filter:");
                    println!("  cli list-events -s {} -c [COMPETITION_ID]", sport_id);
                }
            } else {
                // Sort by open date
                events.sort_by(|a, b| {
                    let empty = String::new();
                    let a_date = a.event.open_date.as_ref().unwrap_or(&empty);
                    let b_date = b.event.open_date.as_ref().unwrap_or(&empty);
                    a_date.cmp(b_date)
                });
                
                println!("Found {} events:", events.len());
                println!("{:<12} {:<45} {:<18} {:<8} {:<8}", "Event ID", "Name", "Date", "Country", "Markets");
                println!("{}", "-".repeat(90));
                
                let display_count = std::cmp::min(50, events.len());
                for event in events.iter().take(display_count) {
                    let name = if event.event.name.len() > 44 {
                        format!("{}...", event.event.name.chars().take(41).collect::<String>())
                    } else {
                        event.event.name.clone()
                    };
                    
                    let open_date = event.event.open_date
                        .as_ref()
                        .map(|d| {
                            if let Ok(dt) = d.parse::<DateTime<Utc>>() {
                                dt.format("%Y-%m-%d %H:%M").to_string()
                            } else {
                                d.chars().take(16).collect()
                            }
                        })
                        .unwrap_or_else(|| "-".to_string());
                    
                    println!(
                        "{:<12} {:<45} {:<18} {:<8} {:<8}", 
                        event.event.id,
                        name,
                        open_date,
                        event.event.country_code.as_ref().unwrap_or(&"-".to_string()),
                        event.market_count
                    );
                }
                
                if events.len() > display_count {
                    println!("\n... and {} more events", events.len() - display_count);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching events: {}", e);
        }
    }
    
    Ok(())
}

async fn list_markets(client: &mut BetfairApiClient, sport_id: &str, competition_id: Option<String>, event_id: Option<String>) -> Result<()> {
    // Get sport name
    let sports = client.list_sports(None).await?;
    let sport_name = sports
        .iter()
        .find(|s| s.event_type.id == sport_id)
        .map(|s| s.event_type.name.clone())
        .unwrap_or_else(|| format!("Sport ID {}", sport_id));
    
    let mut filter = MarketFilter {
        event_type_ids: Some(vec![sport_id.to_string()]),
        ..Default::default()
    };
    
    let mut title = format!("Markets for: {}", sport_name);
    
    // Add competition filter if provided
    if let Some(comp_id) = &competition_id {
        filter.competition_ids = Some(vec![comp_id.clone()]);
    }
    
    // Add event filter if provided (most specific)
    if let Some(ev_id) = &event_id {
        filter.event_ids = Some(vec![ev_id.clone()]);
        
        // Get event name
        let event_filter = MarketFilter {
            event_ids: Some(vec![ev_id.clone()]),
            ..Default::default()
        };
        let events = client.list_events(Some(event_filter)).await?;
        if let Some(event) = events.first() {
            title = format!("Markets for: {}", event.event.name);
        }
    }
    
    println!("Fetching {}\n", title);
    
    let request = ListMarketCatalogueRequest {
        filter,
        market_projection: None,
        sort: None,
        max_results: Some(100),
        locale: None,
    };
    
    match client.list_market_catalogue(request).await {
        Ok(markets) => {
            if markets.is_empty() {
                println!("No markets found.");
            } else {
                println!("Found {} markets:", markets.len());
                println!("{:<18} {:<45} {:<18}", "Market ID", "Market Name", "Start Time");
                println!("{}", "-".repeat(85));
                
                let display_count = std::cmp::min(50, markets.len());
                for market in markets.iter().take(display_count) {
                    let name = if market.market_name.len() > 44 {
                        format!("{}...", market.market_name.chars().take(41).collect::<String>())
                    } else {
                        market.market_name.clone()
                    };
                    
                    let start_time = market.market_start_time
                        .as_ref()
                        .map(|d| {
                            if let Ok(dt) = d.parse::<DateTime<Utc>>() {
                                dt.format("%Y-%m-%d %H:%M").to_string()
                            } else {
                                d.chars().take(16).collect()
                            }
                        })
                        .unwrap_or_else(|| "-".to_string());
                    
                    println!(
                        "{:<18} {:<45} {:<18}",
                        market.market_id,
                        name,
                        start_time
                    );
                }
                
                if markets.len() > display_count {
                    println!("\n... and {} more markets", markets.len() - 50);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching markets: {}", e);
        }
    }
    
    Ok(())
}

async fn get_odds(client: &mut BetfairApiClient, market_id: &str) -> Result<()> {
    println!("Fetching odds for market: {}\n", market_id);
    
    match client.get_odds(market_id.to_string()).await {
        Ok(market_books) => {
            if market_books.is_empty() {
                println!("No market data found for ID: {}", market_id);
                return Ok(());
            }
            
            let market = &market_books[0];
            
            // Display market info
            println!("Market Status: {:?}", market.status.as_ref().unwrap_or(&MarketStatus::Open));
            println!("In-Play: {}", market.inplay.unwrap_or(false));
            println!("Total Matched: £{:.2}", market.total_matched.unwrap_or(0.0));
            println!("Total Available: £{:.2}", market.total_available.unwrap_or(0.0));
            
            if let Some(runners) = &market.runners {
                println!("\n{:<12} {:<10} {:<12} {:<12} {:<12}", 
                    "Selection ID", "Status", "Last Traded", "Back Price", "Lay Price");
                println!("{}", "-".repeat(60));
                
                for runner in runners {
                    let last_price = runner.last_price_traded
                        .map(|p| format!("{:.2}", p))
                        .unwrap_or_else(|| "-".to_string());
                    
                    let (back_price, back_size) = if let Some(ex) = &runner.ex {
                        if let Some(back_prices) = &ex.available_to_back {
                            if !back_prices.is_empty() {
                                (
                                    format!("{:.2}", back_prices[0].price),
                                    format!("£{:.0}", back_prices[0].size)
                                )
                            } else {
                                ("-".to_string(), "-".to_string())
                            }
                        } else {
                            ("-".to_string(), "-".to_string())
                        }
                    } else {
                        ("-".to_string(), "-".to_string())
                    };
                    
                    let (lay_price, lay_size) = if let Some(ex) = &runner.ex {
                        if let Some(lay_prices) = &ex.available_to_lay {
                            if !lay_prices.is_empty() {
                                (
                                    format!("{:.2}", lay_prices[0].price),
                                    format!("£{:.0}", lay_prices[0].size)
                                )
                            } else {
                                ("-".to_string(), "-".to_string())
                            }
                        } else {
                            ("-".to_string(), "-".to_string())
                        }
                    } else {
                        ("-".to_string(), "-".to_string())
                    };
                    
                    println!("{:<12} {:<10} {:<12} {:<6} ({:<6}) {:<6} ({:<6})",
                        runner.selection_id,
                        format!("{:?}", runner.status),
                        last_price,
                        back_price,
                        back_size,
                        lay_price,
                        lay_size
                    );
                }
            } else {
                println!("No runner data available");
            }
        }
        Err(e) => {
            eprintln!("Error fetching odds: {}", e);
        }
    }
    
    Ok(())
}

async fn list_runners(client: &mut BetfairApiClient, market_id: &str) -> Result<()> {
    println!("Fetching runners for market: {}\n", market_id);
    
    match client.list_runners(market_id).await {
        Ok(markets) => {
            if let Some(market) = markets.first() {
                // Display market info
                println!("Market: {}", market.market_name);
                if let Some(event) = &market.event {
                    println!("Event: {}", event.name);
                    if let Some(open_date) = &event.open_date {
                        if let Ok(dt) = open_date.parse::<DateTime<Utc>>() {
                            println!("Start: {}", dt.format("%Y-%m-%d %H:%M UTC"));
                        }
                    }
                }
                
                if let Some(description) = &market.description {
                    println!("Type: {}", description.market_type);
                }
                
                // Display runners
                if let Some(runners) = &market.runners {
                    println!("\nRunners ({} total):\n", runners.len());
                    println!("{:<15} {:<40} {:<12} {:<10}", "Selection ID", "Runner Name", "Handicap", "Sort");
                    println!("{}", "-".repeat(80));
                    
                    for runner in runners {
                        let name = if runner.runner_name.len() > 39 {
                            format!("{}...", runner.runner_name.chars().take(36).collect::<String>())
                        } else {
                            runner.runner_name.clone()
                        };
                        
                        let handicap = if runner.handicap != 0.0 {
                            format!("{:+.1}", runner.handicap)
                        } else {
                            "-".to_string()
                        };
                        
                        println!("{:<15} {:<40} {:<12} {:<10}",
                            runner.selection_id,
                            name,
                            handicap,
                            runner.sort_priority
                        );
                        
                        // Display metadata if available
                        if let Some(metadata) = &runner.metadata {
                            if !metadata.is_empty() {
                                for (key, value) in metadata {
                                    println!("                  {}: {}", key, value);
                                }
                            }
                        }
                    }
                    
                    println!("\nHint: Use selection IDs to place orders on specific runners");
                } else {
                    println!("No runner data available");
                }
            } else {
                println!("Market not found");
            }
        }
        Err(e) => {
            eprintln!("Error fetching runners: {}", e);
        }
    }
    
    Ok(())
}

async fn get_account(client: &mut BetfairApiClient) -> Result<()> {
    println!("Fetching account information...\n");
    
    match client.get_account_funds(GetAccountFundsRequest { wallet: None }).await {
        Ok(funds) => {
            println!("Account Funds:");
            println!("{}", "-".repeat(50));
            println!("Available to Bet: £{:.2}", funds.available_to_bet_balance);
            println!("Exposure: £{:.2}", funds.exposure);
            println!("Exposure Limit: £{:.2}", funds.exposure_limit);
            println!("Retained Commission: £{:.2}", funds.retained_commission);
            
            if let Some(discount_rate) = funds.discount_rate {
                println!("Discount Rate: {:.2}%", discount_rate);
            }
            
            println!("Points Balance: {}", funds.points_balance);
            
            if let Some(wallet) = &funds.wallet {
                println!("Wallet: {}", wallet);
            }
        }
        Err(e) => {
            eprintln!("Error fetching account funds: {}", e);
        }
    }
    
    // Also fetch account details
    println!("\nFetching account details...\n");
    match client.get_account_details().await {
        Ok(details) => {
            println!("Account Details:");
            println!("{}", "-".repeat(50));
            
            if let Some(currency_code) = &details.currency_code {
                println!("Currency: {}", currency_code);
            }
            
            if let Some(country_code) = &details.country_code {
                println!("Country: {}", country_code);
            }
            
            if let Some(timezone) = &details.timezone {
                println!("Timezone: {}", timezone);
            }
            
            if let Some(locale_code) = &details.locale_code {
                println!("Locale: {}", locale_code);
            }
            
            if let Some(region) = &details.region {
                println!("Region: {}", region);
            }
            
            if let Some(first_name) = &details.first_name {
                println!("First Name: {}", first_name);
            }
            
            if let Some(last_name) = &details.last_name {
                println!("Last Name: {}", last_name);
            }
            
            if let Some(discount_rate) = details.discount_rate {
                println!("Discount Rate: {:.2}%", discount_rate);
            }
            
            if let Some(points_balance) = details.points_balance {
                println!("Points Balance: {}", points_balance);
            }
        }
        Err(e) => {
            eprintln!("Error fetching account details: {}", e);
        }
    }
    
    Ok(())
}

async fn place_order(
    client: &mut BetfairApiClient,
    market_id: &str,
    selection_id: u64,
    side: &str,
    price: f64,
    size: f64,
    persistence: &str,
) -> Result<()> {
    println!("Placing order...\n");
    
    // Parse side
    let order_side = match side.to_uppercase().as_str() {
        "BACK" => Side::Back,
        "LAY" => Side::Lay,
        _ => {
            eprintln!("Error: Invalid side: {}. Must be BACK or LAY", side);
            return Ok(());
        }
    };
    
    // Parse persistence type
    let persistence_type = match persistence.to_uppercase().as_str() {
        "LAPSE" => PersistenceType::Lapse,
        "PERSIST" => PersistenceType::Persist,
        _ => {
            eprintln!("Error: Invalid persistence: {}. Must be LAPSE or PERSIST", persistence);
            return Ok(());
        }
    };
    
    // Create place instruction
    let instruction = PlaceInstruction {
        order_type: OrderType::Limit,
        selection_id: selection_id as i64,
        handicap: Some(0.0),
        side: order_side.clone(),
        limit_order: Some(LimitOrder {
            size,
            price,
            persistence_type,
            time_in_force: None,
            min_fill_size: None,
            bet_target_type: None,
            bet_target_size: None,
        }),
        limit_on_close_order: None,
        market_on_close_order: None,
        customer_order_ref: None,
    };
    
    let request = PlaceOrdersRequest {
        market_id: market_id.to_string(),
        instructions: vec![instruction],
        customer_ref: None,
        market_version: None,
        customer_strategy_ref: None,
        async_: None,
    };
    
    match client.place_orders(request).await {
        Ok(response) => {
            println!("Order Response:");
            println!("{}", "-".repeat(50));
            println!("Status: {}", response.status);
            
            if let Some(error_code) = &response.error_code {
                println!("Error Code: {:?}", error_code);
            }
            
            println!("Market ID: {}", response.market_id);
            
            if let Some(reports) = &response.instruction_reports {
                for report in reports {
                    println!("\nOrder Report:");
                    println!("  Status: {:?}", report.status);
                    
                    if let Some(bet_id) = &report.bet_id {
                        println!("  Bet ID: {}", bet_id);
                        println!("\nHint: Use this Bet ID to cancel the order: cli cancel-order -m {} -b {}", market_id, bet_id);
                    }
                    
                    if let Some(placed_date) = &report.placed_date {
                        println!("  Placed: {}", placed_date);
                    }
                    
                    if let Some(avg_price) = report.average_price_matched {
                        println!("  Avg Price Matched: {:.2}", avg_price);
                    }
                    
                    if let Some(size_matched) = report.size_matched {
                        println!("  Size Matched: £{:.2}", size_matched);
                    }
                    
                    if let Some(error_code) = &report.error_code {
                        println!("  Error: {:?}", error_code);
                    }
                    
                    println!("  Selection: {}", report.instruction.selection_id);
                    println!("  Side: {:?}", report.instruction.side);
                    if let Some(limit_order) = &report.instruction.limit_order {
                        println!("  Price: {:.2}", limit_order.price);
                        println!("  Size: £{:.2}", limit_order.size);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error placing order: {}", e);
        }
    }
    
    Ok(())
}

async fn cancel_order(
    client: &mut BetfairApiClient,
    market_id: &str,
    bet_id: &str,
) -> Result<()> {
    println!("Cancelling order...\n");
    
    let instruction = CancelInstruction {
        bet_id: bet_id.to_string(),
        size_reduction: None,
    };
    
    let request = CancelOrdersRequest {
        market_id: market_id.to_string(),
        instructions: vec![instruction],
        customer_ref: None,
    };
    
    match client.cancel_orders(request).await {
        Ok(response) => {
            println!("Cancel Response:");
            println!("{}", "-".repeat(50));
            println!("Status: {}", response.status);
            
            if let Some(error_code) = &response.error_code {
                println!("Error Code: {:?}", error_code);
            }
            
            println!("Market ID: {}", response.market_id);
            
            if let Some(reports) = &response.instruction_reports {
                for report in reports {
                    println!("\nCancel Report:");
                    println!("  Status: {:?}", report.status);
                    
                    if let Some(size_cancelled) = report.size_cancelled {
                        println!("  Size Cancelled: £{:.2}", size_cancelled);
                    }
                    
                    if let Some(cancelled_date) = &report.cancelled_date {
                        println!("  Cancelled: {}", cancelled_date);
                    }
                    
                    if let Some(error_code) = &report.error_code {
                        println!("  Error: {:?}", error_code);
                    }
                    
                    println!("  Bet ID: {}", report.instruction.bet_id);
                }
            }
        }
        Err(e) => {
            eprintln!("Error cancelling order: {}", e);
        }
    }
    
    Ok(())
}

async fn list_orders(client: &mut BetfairApiClient, market_id: Option<&str>) -> Result<()> {
    println!("Fetching current orders...\n");
    
    let request = ListCurrentOrdersRequest {
        bet_ids: None,
        market_ids: market_id.map(|m| vec![m.to_string()]),
        order_projection: None,
        customer_order_refs: None,
        customer_strategy_refs: None,
        date_range: None,
        order_by: None,
        sort_dir: None,
        from_record: None,
        record_count: None,
    };
    
    match client.list_current_orders(request).await {
        Ok(response) => {
            let orders = &response.current_orders;
                if orders.is_empty() {
                    println!("No current orders found");
                } else {
                    println!("Current Orders ({} total):\n", orders.len());
                    println!("{:<12} {:<15} {:<12} {:<6} {:<8} {:<8} {:<10} {:<10} {:<10}", 
                        "Bet ID", "Market ID", "Selection", "Side", "Price", "Size", "Matched", "Status", "Type");
                    println!("{}", "-".repeat(100));
                    
                    for order in orders {
                        let bet_id = if order.bet_id.len() > 10 {
                            format!("{}...", &order.bet_id[..10])
                        } else {
                            order.bet_id.clone()
                        };
                        
                        let market_id = if order.market_id.len() > 13 {
                            format!("{}...", &order.market_id[..13])
                        } else {
                            order.market_id.clone()
                        };
                        
                        println!("{:<12} {:<15} {:<12} {:<6} {:<8.2} £{:<7.2} £{:<9.2} {:<10} {:<10}",
                            bet_id,
                            market_id,
                            order.selection_id,
                            format!("{:?}", order.side),
                            order.price_size.price,
                            order.price_size.size,
                            order.size_matched.unwrap_or(0.0),
                            format!("{:?}", order.status),
                            format!("{:?}", order.order_type)
                        );
                        
                        if let Some(size_matched) = order.size_matched {
                            if size_matched > 0.0 {
                            if let Some(avg_price) = order.average_price_matched {
                                    println!("                  Average Price Matched: {:.2}", avg_price);
                                }
                            }
                        }
                        
                        if let Some(size_remaining) = order.size_remaining {
                            if size_remaining > 0.0 {
                                println!("                  Remaining: £{:.2}", size_remaining);
                            }
                        }
                        
                        if let Some(size_cancelled) = order.size_cancelled {
                            if size_cancelled > 0.0 {
                                println!("                  Cancelled: £{:.2}", size_cancelled);
                            }
                        }
                        
                        if let Some(placed_date) = &order.placed_date {
                            if let Ok(dt) = placed_date.parse::<DateTime<Utc>>() {
                                println!("                  Placed: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
                            }
                        }
                    }
                    
                    println!("\nHint: Use bet IDs to cancel specific orders");
                    println!("Hint: Use: cli cancel-order -m [MARKET_ID] -b [BET_ID]");
                }
        }
        Err(e) => {
            eprintln!("Error fetching orders: {}", e);
        }
    }
    
    Ok(())
}