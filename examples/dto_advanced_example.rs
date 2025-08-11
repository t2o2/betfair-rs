use anyhow::Result;
use betfair_rs::{api_client::BetfairApiClient, config::Config, dto::*};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Advanced DTO Example");

    // Load configuration and create client
    let config = Config::new()?;
    let mut client = BetfairApiClient::new(config);

    // Login
    info!("Logging in...");
    let login_response = client.login().await?;
    if login_response.session_token.is_none() {
        eprintln!("Login failed: {}", login_response.login_status);
        return Ok(());
    }
    info!("Login successful!");

    // Example 1: Complex market search with filters
    info!("\n=== Advanced Market Search ===");
    let market_filter = MarketFilter {
        text_query: Some("Football".to_string()),
        exchange_ids: None,
        event_type_ids: Some(vec!["1".to_string()]), // Football event type
        event_ids: None,
        competition_ids: None,
        market_ids: None,
        venues: None,
        bsp_only: Some(false),
        turn_in_play_enabled: Some(true),
        in_play_only: Some(false),
        market_betting_types: Some(vec!["ODDS".to_string()]),
        market_countries: Some(vec!["GB".to_string()]),
        market_type_codes: Some(vec!["MATCH_ODDS".to_string()]),
        market_start_time: None,
        with_orders: None,
    };

    let catalogue_request = ListMarketCatalogueRequest {
        filter: market_filter,
        market_projection: Some(vec![
            MarketProjection::Competition,
            MarketProjection::Event,
            MarketProjection::MarketStartTime,
            MarketProjection::MarketDescription,
            MarketProjection::RunnerDescription,
            MarketProjection::RunnerMetadata,
        ]),
        sort: Some(MarketSort::FirstToStart),
        max_results: Some(5),
        locale: Some("en".to_string()),
    };

    let markets = client.list_market_catalogue(catalogue_request).await?;

    for market in &markets {
        info!("Market: {} - {}", market.market_id, market.market_name);

        if let Some(event) = &market.event {
            info!("  Event: {} ({})", event.name, event.id);
            if let Some(venue) = &event.venue {
                info!("  Venue: {}", venue);
            }
        }

        if let Some(competition) = &market.competition {
            info!("  Competition: {} ({})", competition.name, competition.id);
        }

        if let Some(description) = &market.description {
            info!("  Market Type: {}", description.market_type);
            info!("  Betting Type: {}", description.betting_type);
            info!("  Turn In Play: {}", description.turn_in_play_enabled);
        }

        if let Some(runners) = &market.runners {
            info!("  Runners:");
            for runner in runners {
                info!(
                    "    {} - {} (Handicap: {})",
                    runner.selection_id, runner.runner_name, runner.handicap
                );
                if let Some(metadata) = &runner.metadata {
                    for (key, value) in metadata {
                        info!("      {}: {}", key, value);
                    }
                }
            }
        }
    }

    // Example 2: Get detailed market book with all projections
    if !markets.is_empty() {
        let market_id = markets[0].market_id.clone();
        info!("\n=== Detailed Market Book for {} ===", market_id);

        let market_book_request = ListMarketBookRequest {
            market_ids: vec![market_id.clone()],
            price_projection: Some(PriceProjectionDto {
                price_data: Some(vec![
                    PriceProjection::SpAvailable,
                    PriceProjection::SpTraded,
                    PriceProjection::ExTraded,
                    PriceProjection::ExBestOffers,
                ]),
                ex_best_offers_overrides: Some(ExBestOffersOverrides {
                    best_prices_depth: Some(3),
                    rollup_model: Some("STAKE".to_string()),
                    rollup_limit: Some(20),
                    rollup_liability_threshold: None,
                    rollup_liability_factor: None,
                }),
                virtualise: Some(true),
                rollover_stakes: Some(false),
            }),
            order_projection: Some(OrderProjection::All),
            match_projection: Some(MatchProjection::RolledUpByPrice),
            include_overall_position: Some(false),
            partition_matched_by_strategy_ref: Some(false),
            customer_strategy_refs: None,
            currency_code: Some("GBP".to_string()),
            locale: Some("en".to_string()),
            matched_since: None,
            bet_ids: None,
        };

        let market_books = client.list_market_book(market_book_request).await?;

        if let Some(market_book) = market_books.first() {
            info!("Market ID: {}", market_book.market_id);
            info!("Market Status: {:?}", market_book.status);
            info!("In Play: {:?}", market_book.inplay);
            info!("Total Matched: {:?}", market_book.total_matched);
            info!("Total Available: {:?}", market_book.total_available);

            if let Some(runners) = &market_book.runners {
                for runner in runners {
                    info!(
                        "\nRunner {} - Status: {}",
                        runner.selection_id, runner.status
                    );
                    info!("  Last Price Traded: {:?}", runner.last_price_traded);
                    info!("  Total Matched: {:?}", runner.total_matched);

                    // Exchange prices
                    if let Some(ex) = &runner.ex {
                        if let Some(back_prices) = &ex.available_to_back {
                            info!("  Back prices:");
                            for (i, price_size) in back_prices.iter().enumerate().take(3) {
                                info!(
                                    "    Level {}: {} @ £{}",
                                    i + 1,
                                    price_size.price,
                                    price_size.size
                                );
                            }
                        }
                        if let Some(lay_prices) = &ex.available_to_lay {
                            info!("  Lay prices:");
                            for (i, price_size) in lay_prices.iter().enumerate().take(3) {
                                info!(
                                    "    Level {}: {} @ £{}",
                                    i + 1,
                                    price_size.price,
                                    price_size.size
                                );
                            }
                        }
                        if let Some(traded) = &ex.traded_volume {
                            info!("  Traded volume:");
                            for price_size in traded.iter().take(5) {
                                info!("    {} @ £{}", price_size.price, price_size.size);
                            }
                        }
                    }

                    // Starting prices
                    if let Some(sp) = &runner.sp {
                        info!("  Starting Price Info:");
                        info!("    Near Price: {:?}", sp.near_price);
                        info!("    Far Price: {:?}", sp.far_price);
                        info!("    Actual SP: {:?}", sp.actual_sp);
                    }
                }
            }
        }
    }

    // Example 3: Complex order placement with all options
    info!("\n=== Complex Order Placement Example (DRY RUN) ===");
    info!("This is just showing the DTO structure, not placing actual orders");

    if !markets.is_empty() && markets[0].runners.is_some() {
        let market = &markets[0];
        let runner_id = market.runners.as_ref().unwrap()[0].selection_id;

        let place_request = PlaceOrdersRequest {
            market_id: market.market_id.clone(),
            instructions: vec![
                // Limit order example
                PlaceInstruction {
                    order_type: OrderType::Limit,
                    selection_id: runner_id,
                    handicap: Some(0.0),
                    side: Side::Back,
                    limit_order: Some(LimitOrder {
                        size: 2.0,
                        price: 1000.0, // Very high odds to avoid matching
                        persistence_type: PersistenceType::Lapse,
                        time_in_force: None,
                        min_fill_size: None,
                        bet_target_type: None,
                        bet_target_size: None,
                    }),
                    limit_on_close_order: None,
                    market_on_close_order: None,
                    customer_order_ref: Some("test_order_001".to_string()),
                },
                // Market on close order example
                PlaceInstruction {
                    order_type: OrderType::MarketOnClose,
                    selection_id: runner_id,
                    handicap: Some(0.0),
                    side: Side::Lay,
                    limit_order: None,
                    limit_on_close_order: None,
                    market_on_close_order: Some(MarketOnCloseOrder { liability: 10.0 }),
                    customer_order_ref: Some("test_moc_001".to_string()),
                },
            ],
            customer_ref: Some("example_customer_ref".to_string()),
            market_version: None,
            customer_strategy_ref: Some("example_strategy".to_string()),
            async_: Some(false),
        };

        info!("Would place orders:");
        for instruction in &place_request.instructions {
            info!(
                "  Order Type: {:?}, Selection: {}, Side: {:?}",
                instruction.order_type, instruction.selection_id, instruction.side
            );
            if let Some(limit) = &instruction.limit_order {
                info!("    Limit: {} @ £{}", limit.price, limit.size);
            }
            if let Some(moc) = &instruction.market_on_close_order {
                info!("    Market on Close Liability: £{}", moc.liability);
            }
        }
    }

    // Example 4: Query cleared orders with detailed filters
    info!("\n=== Querying Cleared Orders ===");
    let cleared_request = ListClearedOrdersRequest {
        bet_status: "SETTLED".to_string(),
        event_type_ids: Some(vec!["1".to_string()]), // Football
        event_ids: None,
        market_ids: None,
        runner_ids: None,
        bet_ids: None,
        customer_order_refs: None,
        customer_strategy_refs: None,
        side: None,
        settled_date_range: None,
        group_by: Some("MARKET".to_string()),
        include_item_description: Some(true),
        locale: Some("en".to_string()),
        from_record: Some(0),
        record_count: Some(5),
    };

    let cleared_response = client.list_cleared_orders(cleared_request).await?;
    info!(
        "Found {} cleared orders",
        cleared_response.cleared_orders.len()
    );

    for order in &cleared_response.cleared_orders {
        info!("\nCleared Order: {}", order.bet_id);
        info!(
            "  Market: {}, Selection: {}",
            order.market_id, order.selection_id
        );
        info!(
            "  Side: {:?}, Price: {}, Size Settled: {}",
            order.side, order.price_matched, order.size_settled
        );
        info!("  Profit: £{}", order.profit);

        if let Some(item_desc) = &order.item_description {
            if let Some(event_desc) = &item_desc.event_desc {
                info!("  Event: {}", event_desc);
            }
            if let Some(market_desc) = &item_desc.market_desc {
                info!("  Market: {}", market_desc);
            }
            if let Some(runner_desc) = &item_desc.runner_desc {
                info!("  Runner: {}", runner_desc);
            }
        }
    }

    info!("\n=== Advanced DTO Example completed successfully ===");
    Ok(())
}
