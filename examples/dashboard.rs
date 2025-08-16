use anyhow::Result;
use betfair_rs::{
    config::Config,
    dto::{
        account::GetAccountFundsRequest,
        common::{MarketProjection, OrderType, PersistenceType, Side},
        order::{
            CancelInstruction, CancelOrdersRequest, LimitOrder, ListCurrentOrdersRequest,
            PlaceInstruction, PlaceOrdersRequest,
        },
        ListMarketCatalogueRequest, MarketFilter,
    },
    orderbook::Orderbook,
    UnifiedBetfairClient,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState,
    },
    Frame, Terminal,
};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use tracing::info;

#[derive(Debug, Clone)]
enum AppMode {
    Browse,
    Order,
    #[allow(dead_code)]
    Manage,
    #[allow(dead_code)]
    Search,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
enum OrderField {
    Price,
    Size,
}

#[derive(Debug, Clone, PartialEq)]
enum Panel {
    MarketBrowser,
    OrderBook,
    ActiveOrders,
    OrderEntry,
}

#[derive(Debug, Clone)]
struct Market {
    id: String,
    name: String,
    #[allow(dead_code)]
    event_name: String,
    #[allow(dead_code)]
    market_start_time: Option<String>,
    total_matched: f64,
    runners: Vec<Runner>,
}

#[derive(Debug, Clone)]
struct Runner {
    id: u64,
    name: String,
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    last_price_traded: Option<f64>,
    #[allow(dead_code)]
    total_matched: Option<f64>,
}

#[derive(Debug, Clone)]
struct Order {
    bet_id: String,
    market_id: String,
    #[allow(dead_code)]
    selection_id: i64,
    side: Side,
    price: f64,
    size: f64,
    size_matched: f64,
    status: String,
    #[allow(dead_code)]
    placed_date: String,
    // Additional details for display
    competition_name: String,
    event_name: String,
    market_type: String,
    runner_name: String,
}

#[derive(Debug, Clone)]
struct RunnerOrderBook {
    runner_id: u64,
    runner_name: String,
    bids: Vec<(f64, f64)>, // (price, size)
    asks: Vec<(f64, f64)>, // (price, size)
    #[allow(dead_code)]
    last_traded: Option<f64>,
    #[allow(dead_code)]
    total_matched: f64,
    is_streaming: bool,           // Flag to indicate if data is from streaming
    last_update: Option<Instant>, // Track when this runner was last updated
    prev_best_bid: Option<f64>,   // Track previous best bid for change detection
    prev_best_ask: Option<f64>,   // Track previous best ask for change detection
}

#[derive(Debug, Clone)]
struct OrderBookData {
    #[allow(dead_code)]
    market_id: String,
    runners: Vec<RunnerOrderBook>,
}

struct App {
    mode: AppMode,
    active_panel: Panel,

    // Market browser state
    sports: Vec<(String, String, u32)>, // (id, name, market_count)
    selected_sport: Option<usize>,
    competitions: Vec<(String, String, u32)>, // (id, name, market_count)
    selected_competition: Option<usize>,
    events: Vec<(String, String, u32)>, // (id, name, market_count)
    selected_event: Option<usize>,
    markets: Vec<Market>,
    selected_market: Option<usize>,
    market_browser_scroll_offset: usize, // Viewport scroll offset for market browser

    // Order book state
    current_orderbook: Option<OrderBookData>,
    selected_runner: Option<usize>,
    streaming_orderbooks: Arc<RwLock<HashMap<String, HashMap<String, Orderbook>>>>, // market_id -> runner_id -> orderbook
    last_streaming_update: Option<Instant>, // Track when we last received streaming data

    // Active orders state
    active_orders: Vec<Order>,
    selected_order: Option<usize>,
    active_orders_scroll_offset: usize, // Viewport scroll offset for active orders

    // Order entry state
    order_market_id: String,
    order_selection_id: String,
    order_runner_name: String,
    order_side: Side,
    order_price: String,
    order_size: String,
    order_field_focus: OrderField,

    // Account state
    available_balance: f64,
    exposure: f64,
    total_orders: usize,

    // Connection state
    api_connected: bool,
    streaming_connected: bool,
    #[allow(dead_code)]
    last_update: Instant,
    current_streaming_market: Option<String>, // Track which market we're streaming

    // Search state
    #[allow(dead_code)]
    search_query: String,

    // UI state
    status_message: String,
    error_message: Option<String>,

    // Unified client
    client: Option<UnifiedBetfairClient>,
}

impl App {
    fn new() -> Self {
        Self {
            mode: AppMode::Browse,
            active_panel: Panel::MarketBrowser,

            sports: vec![],
            selected_sport: None,
            competitions: vec![],
            selected_competition: None,
            events: vec![],
            selected_event: None,
            markets: vec![],
            selected_market: None,
            market_browser_scroll_offset: 0,

            current_orderbook: None,
            selected_runner: None,
            streaming_orderbooks: Arc::new(RwLock::new(HashMap::new())),
            last_streaming_update: None,

            active_orders: vec![],
            selected_order: None,
            active_orders_scroll_offset: 0,

            order_market_id: String::new(),
            order_selection_id: String::new(),
            order_runner_name: String::new(),
            order_side: Side::Back,
            order_price: String::new(),
            order_size: String::new(),
            order_field_focus: OrderField::Price,

            available_balance: 0.0,
            exposure: 0.0,
            total_orders: 0,

            api_connected: false,
            streaming_connected: false,
            last_update: Instant::now(),
            current_streaming_market: None,

            search_query: String::new(),

            status_message: "Initializing...".to_string(),
            error_message: None,

            client: None,
        }
    }

    async fn init(&mut self) -> Result<()> {
        // Load configuration and create clients
        let config = Config::new()?;

        // Initialize unified client
        let mut client = UnifiedBetfairClient::new(config);
        self.status_message = "Logging in to Betfair API...".to_string();

        let login_response = client.login().await?;
        if login_response.login_status != "SUCCESS" {
            return Err(anyhow::anyhow!(
                "Login failed: {}",
                login_response.login_status
            ));
        }

        self.api_connected = true;
        self.status_message = "Initializing streaming...".to_string();

        // Try to start the streaming client
        match client.start_streaming().await {
            Ok(()) => {
                // Get the shared orderbooks reference
                if let Some(orderbooks) = client.get_streaming_orderbooks() {
                    self.streaming_orderbooks = orderbooks;
                    self.streaming_connected = true;
                    self.status_message = "Connected to API and streaming".to_string();
                    info!("Streaming client connected successfully");
                } else {
                    self.streaming_connected = false;
                    self.status_message = "API connected (no streaming)".to_string();
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to start streaming client: {e}. Continuing without streaming."
                );
                self.streaming_connected = false;
            }
        }

        self.client = Some(client);

        // Load initial data
        self.load_sports().await?;
        self.load_account_info().await?;
        self.load_active_orders().await?;

        self.status_message = if self.streaming_connected {
            "Connected to Betfair (with streaming)".to_string()
        } else {
            "Connected to Betfair (polling mode)".to_string()
        };
        Ok(())
    }

    async fn load_sports(&mut self) -> Result<()> {
        if let Some(client) = &mut self.client {
            let sports = client.list_sports(None).await?;
            self.sports = sports
                .into_iter()
                .filter(|s| s.market_count > 0)
                .map(|s| (s.event_type.id, s.event_type.name, s.market_count as u32))
                .collect();
            self.sports.sort_by(|a, b| b.2.cmp(&a.2));
        }
        Ok(())
    }

    async fn load_competitions(&mut self, sport_id: &str) -> Result<()> {
        if let Some(client) = &mut self.client {
            let filter = MarketFilter {
                event_type_ids: Some(vec![sport_id.to_string()]),
                ..Default::default()
            };
            let competitions = client.list_competitions(Some(filter)).await?;
            self.competitions = competitions
                .into_iter()
                .map(|c| (c.competition.id, c.competition.name, c.market_count as u32))
                .collect();
            self.competitions.sort_by(|a, b| b.2.cmp(&a.2));
        }
        Ok(())
    }

    async fn load_events(&mut self, sport_id: &str, competition_id: Option<&str>) -> Result<()> {
        if let Some(client) = &mut self.client {
            let mut filter = MarketFilter {
                event_type_ids: Some(vec![sport_id.to_string()]),
                ..Default::default()
            };
            if let Some(comp_id) = competition_id {
                filter.competition_ids = Some(vec![comp_id.to_string()]);
            }
            let events = client.list_events(Some(filter)).await?;
            self.events = events
                .into_iter()
                .map(|e| (e.event.id, e.event.name, e.market_count as u32))
                .collect();
        }
        Ok(())
    }

    async fn load_markets(&mut self, sport_id: &str, event_id: Option<&str>) -> Result<()> {
        if let Some(client) = &mut self.client {
            let mut filter = MarketFilter {
                event_type_ids: Some(vec![sport_id.to_string()]),
                ..Default::default()
            };
            if let Some(ev_id) = event_id {
                filter.event_ids = Some(vec![ev_id.to_string()]);
            }

            let request = ListMarketCatalogueRequest {
                filter,
                market_projection: Some(vec![
                    MarketProjection::Event,
                    MarketProjection::RunnerDescription,
                    MarketProjection::MarketDescription,
                ]),
                sort: None,
                max_results: Some(50),
                locale: None,
            };

            let markets_data = client.list_market_catalogue(request).await?;
            self.markets = markets_data
                .into_iter()
                .map(|m| Market {
                    id: m.market_id,
                    name: m.market_name,
                    event_name: m.event.map(|e| e.name).unwrap_or_default(),
                    market_start_time: m.market_start_time,
                    total_matched: m.total_matched.unwrap_or(0.0),
                    runners: m
                        .runners
                        .unwrap_or_default()
                        .into_iter()
                        .map(|r| Runner {
                            id: r.selection_id as u64,
                            name: r.runner_name,
                            status: "Active".to_string(),
                            last_price_traded: None,
                            total_matched: None,
                        })
                        .collect(),
                })
                .collect();
        }
        Ok(())
    }

    async fn load_orderbook(&mut self, market_id: &str) -> Result<()> {
        // Clear current orderbook and reset selection
        self.current_orderbook = None;
        self.selected_runner = None;

        // First, get runner names from the market catalog
        let mut runner_names = HashMap::new();
        if let Some(client) = &mut self.client {
            let filter = MarketFilter {
                market_ids: Some(vec![market_id.to_string()]),
                ..Default::default()
            };
            let catalog_request = ListMarketCatalogueRequest {
                filter,
                market_projection: Some(vec![
                    MarketProjection::RunnerDescription,
                    MarketProjection::MarketDescription,
                ]),
                sort: None,
                max_results: Some(1),
                locale: None,
            };

            if let Ok(markets) = client.list_market_catalogue(catalog_request).await {
                if let Some(market) = markets.first() {
                    if let Some(runners) = &market.runners {
                        for runner in runners {
                            runner_names.insert(
                                runner.selection_id.to_string(),
                                runner.runner_name.clone(),
                            );
                        }
                    }
                }
            }
        }

        // Try to use streaming if available
        if self.streaming_connected {
            // Check if we need to subscribe to a different market
            let needs_subscription = if let Some(prev_market) = &self.current_streaming_market {
                prev_market != market_id
            } else {
                true
            };

            // Subscribe to the market if needed
            if needs_subscription {
                if let Some(client) = &self.client {
                    if let Err(e) = client.subscribe_to_market(market_id.to_string(), 5).await {
                        self.error_message =
                            Some(format!("Failed to subscribe to market stream: {e}"));
                        self.streaming_connected = false;
                    } else {
                        self.current_streaming_market = Some(market_id.to_string());
                        self.status_message = format!("Streaming market {market_id}");

                        // Give streaming more time to populate initial data
                        tokio::time::sleep(Duration::from_millis(2000)).await;
                    }
                }
            } else {
                // Already subscribed to this market, just wait a bit for fresh data
                self.status_message = format!("Refreshing market {market_id}");
                tokio::time::sleep(Duration::from_millis(500)).await;
            }

            // Read from streaming orderbooks - retry a few times if no data yet
            let mut retries = 0;
            while retries < 3 {
                let orderbooks = self.streaming_orderbooks.read().unwrap();
                if let Some(market_orderbooks) = orderbooks.get(market_id) {
                    if !market_orderbooks.is_empty() {
                let mut runner_books = vec![];

                for (runner_id_str, orderbook) in market_orderbooks {
                    let runner_id: u64 = runner_id_str.parse().unwrap_or(0);

                    // Convert streaming orderbook to our format
                    let bids: Vec<(f64, f64)> = orderbook
                        .bids
                        .iter()
                        .take(10)
                        .map(|level| (level.price, level.size))
                        .collect();

                    let asks: Vec<(f64, f64)> = orderbook
                        .asks
                        .iter()
                        .take(10)
                        .map(|level| (level.price, level.size))
                        .collect();

                    let runner_name = runner_names
                        .get(runner_id_str)
                        .cloned()
                        .unwrap_or_else(|| format!("Runner {runner_id}"));

                    runner_books.push(RunnerOrderBook {
                        runner_id,
                        runner_name,
                        bids,
                        asks,
                        last_traded: None,
                        total_matched: 0.0,
                        is_streaming: true,
                        last_update: None,
                        prev_best_bid: None,
                        prev_best_ask: None,
                    });
                }

                // Sort runners by ID for consistent ordering
                runner_books.sort_by_key(|r| r.runner_id);

                self.current_orderbook = Some(OrderBookData {
                    market_id: market_id.to_string(),
                    runners: runner_books,
                });

                if self.selected_runner.is_none()
                    && !self.current_orderbook.as_ref().unwrap().runners.is_empty()
                {
                    self.selected_runner = Some(0);
                }

                        return Ok(());
                    }
                }
                drop(orderbooks); // Release lock before sleeping
                
                if retries < 2 {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                retries += 1;
            }
            
            // If we still don't have streaming data after retries, fall back to polling
            self.status_message = format!("No streaming data for market {market_id}, using polling");
        }

        // Fall back to polling if streaming not available or failed
        if let Some(client) = &mut self.client {
            let market_books = client.get_odds(market_id.to_string()).await?;

            if let Some(market_book) = market_books.first() {
                if let Some(runners) = &market_book.runners {
                    let mut runner_books = vec![];

                    for runner in runners {
                        let mut bids = vec![];
                        let mut asks = vec![];

                        if let Some(ex) = &runner.ex {
                            if let Some(back_prices) = &ex.available_to_back {
                                bids = back_prices
                                    .iter()
                                    .take(10)
                                    .map(|p| (p.price, p.size))
                                    .collect();
                            }
                            if let Some(lay_prices) = &ex.available_to_lay {
                                asks = lay_prices
                                    .iter()
                                    .take(10)
                                    .map(|p| (p.price, p.size))
                                    .collect();
                            }
                        }

                        let runner_name = runner_names
                            .get(&runner.selection_id.to_string())
                            .cloned()
                            .or_else(|| {
                                self.markets
                                    .iter()
                                    .find(|m| m.id == market_id)
                                    .and_then(|m| {
                                        m.runners
                                            .iter()
                                            .find(|r| r.id as i64 == runner.selection_id)
                                    })
                                    .map(|r| r.name.clone())
                            })
                            .unwrap_or_else(|| format!("Runner {}", runner.selection_id));

                        runner_books.push(RunnerOrderBook {
                            runner_id: runner.selection_id as u64,
                            runner_name,
                            bids,
                            asks,
                            last_traded: runner.last_price_traded,
                            total_matched: runner.total_matched.unwrap_or(0.0),
                            is_streaming: false,
                            last_update: None,
                            prev_best_bid: None,
                            prev_best_ask: None,
                        });
                    }

                    // Sort runners by ID for consistent ordering
                    runner_books.sort_by_key(|r| r.runner_id);

                    self.current_orderbook = Some(OrderBookData {
                        market_id: market_id.to_string(),
                        runners: runner_books,
                    });

                    if self.selected_runner.is_none()
                        && !self.current_orderbook.as_ref().unwrap().runners.is_empty()
                    {
                        self.selected_runner = Some(0);
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_account_info(&mut self) -> Result<()> {
        if let Some(client) = &mut self.client {
            let funds = client
                .get_account_funds(GetAccountFundsRequest { wallet: None })
                .await?;
            self.available_balance = funds.available_to_bet_balance;
            self.exposure = funds.exposure;
        }
        Ok(())
    }

    async fn load_active_orders(&mut self) -> Result<()> {
        // Reset scroll when loading new orders
        self.reset_active_orders_scroll();

        if let Some(client) = &mut self.client {
            let request = ListCurrentOrdersRequest {
                bet_ids: None,
                market_ids: None,
                order_projection: None,
                customer_order_refs: None,
                customer_strategy_refs: None,
                date_range: None,
                order_by: None,
                sort_dir: None,
                from_record: None,
                record_count: Some(50),
            };

            let response = client.list_current_orders(request).await?;

            // Collect unique market IDs to fetch market details
            let market_ids: Vec<String> = response
                .current_orders
                .iter()
                .map(|o| o.market_id.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            // Fetch market details for enrichment
            let mut market_details = std::collections::HashMap::new();
            for market_id in market_ids {
                let filter = MarketFilter {
                    market_ids: Some(vec![market_id.clone()]),
                    ..Default::default()
                };
                let request = ListMarketCatalogueRequest {
                    filter,
                    market_projection: Some(vec![
                        MarketProjection::Competition,
                        MarketProjection::Event,
                        MarketProjection::MarketDescription,
                        MarketProjection::RunnerDescription,
                    ]),
                    sort: None,
                    max_results: Some(1),
                    locale: None,
                };
                if let Ok(markets) = client.list_market_catalogue(request).await {
                    if let Some(market) = markets.first() {
                        market_details.insert(market_id, market.clone());
                    }
                }
            }

            self.active_orders = response
                .current_orders
                .into_iter()
                .map(|o| {
                    let market = market_details.get(&o.market_id);
                    let competition_name = market
                        .and_then(|m| m.competition.as_ref())
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    let event_name = market
                        .and_then(|m| m.event.as_ref())
                        .map(|e| e.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    let market_type = market
                        .and_then(|m| m.description.as_ref())
                        .map(|d| d.market_type.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    let runner_name = market
                        .and_then(|m| m.runners.as_ref())
                        .and_then(|runners| {
                            runners.iter().find(|r| r.selection_id == o.selection_id)
                        })
                        .map(|r| r.runner_name.clone())
                        .unwrap_or_else(|| format!("Runner {}", o.selection_id));

                    Order {
                        bet_id: o.bet_id,
                        market_id: o.market_id,
                        selection_id: o.selection_id,
                        side: o.side,
                        price: o.price_size.price,
                        size: o.price_size.size,
                        size_matched: o.size_matched.unwrap_or(0.0),
                        status: format!("{:?}", o.status),
                        placed_date: o.placed_date.unwrap_or_default(),
                        competition_name,
                        event_name,
                        market_type,
                        runner_name,
                    }
                })
                .collect();
            self.total_orders = self.active_orders.len();
        }
        Ok(())
    }

    async fn place_order(&mut self) -> Result<()> {
        // Validate inputs first
        if self.order_market_id.is_empty() {
            self.error_message = Some("No market selected".to_string());
            return Ok(());
        }

        if self.order_selection_id.is_empty() {
            self.error_message =
                Some("No runner selected. Press 1-9 in Order Book to select".to_string());
            return Ok(());
        }

        if self.order_price.is_empty() {
            self.error_message = Some("Price is required".to_string());
            return Ok(());
        }

        if self.order_size.is_empty() {
            self.error_message = Some("Stake is required".to_string());
            return Ok(());
        }

        if let Some(client) = &mut self.client {
            let price: f64 = match self.order_price.parse() {
                Ok(p) => p,
                Err(_) => {
                    self.error_message = Some("Invalid price format".to_string());
                    return Ok(());
                }
            };

            let size: f64 = match self.order_size.parse() {
                Ok(s) => s,
                Err(_) => {
                    self.error_message = Some("Invalid stake format".to_string());
                    return Ok(());
                }
            };

            let selection_id: i64 = match self.order_selection_id.parse() {
                Ok(id) => id,
                Err(_) => {
                    self.error_message =
                        Some(format!("Invalid selection ID: {}", self.order_selection_id));
                    return Ok(());
                }
            };

            // Validate price and size ranges
            if !(1.01..=1000.0).contains(&price) {
                self.error_message = Some("Price must be between 1.01 and 1000".to_string());
                return Ok(());
            }

            if size < 2.0 {
                self.error_message = Some("Minimum stake is £2".to_string());
                return Ok(());
            }

            let instruction = PlaceInstruction {
                order_type: OrderType::Limit,
                selection_id,
                handicap: Some(0.0),
                side: self.order_side.clone(),
                limit_order: Some(LimitOrder {
                    size,
                    price,
                    persistence_type: PersistenceType::Lapse,
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
                market_id: self.order_market_id.clone(),
                instructions: vec![instruction],
                customer_ref: None,
                market_version: None,
                customer_strategy_ref: None,
                async_: None,
            };

            let response = client.place_orders(request).await?;

            if response.status == "SUCCESS" {
                self.status_message = format!(
                    "Order placed: {} £{:.2} @ {:.2} on {}",
                    if matches!(self.order_side, Side::Back) {
                        "Back"
                    } else {
                        "Lay"
                    },
                    size,
                    price,
                    self.order_runner_name
                );
                self.load_active_orders().await?;
                self.load_account_info().await?;

                // Clear order form
                self.order_price.clear();
                self.order_size.clear();
                self.error_message = None;
            } else {
                // Check for specific error details
                let error_detail = if let Some(instruction_reports) = response.instruction_reports {
                    if let Some(report) = instruction_reports.first() {
                        if let Some(error_code) = &report.error_code {
                            format!("{}: {:?}", response.status, error_code)
                        } else {
                            format!("{}: Order rejected", response.status)
                        }
                    } else {
                        response.status.clone()
                    }
                } else {
                    response.status.clone()
                };
                self.error_message = Some(format!("Order failed: {error_detail}"));
            }
        } else {
            self.error_message = Some("Not connected to API".to_string());
        }
        Ok(())
    }

    async fn cancel_order(&mut self, bet_id: &str) -> Result<()> {
        if let Some(client) = &mut self.client {
            if let Some(order) = self.active_orders.iter().find(|o| o.bet_id == bet_id) {
                let instruction = CancelInstruction {
                    bet_id: bet_id.to_string(),
                    size_reduction: None,
                };

                let request = CancelOrdersRequest {
                    market_id: order.market_id.clone(),
                    instructions: vec![instruction],
                    customer_ref: None,
                };

                let response = client.cancel_orders(request).await?;

                if response.status == "SUCCESS" {
                    self.status_message = "Order cancelled successfully".to_string();
                    self.load_active_orders().await?;
                    self.load_account_info().await?;
                } else {
                    self.error_message = Some(format!("Cancel failed: {}", response.status));
                }
            }
        }
        Ok(())
    }

    fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::MarketBrowser => Panel::OrderBook,
            Panel::OrderBook => Panel::ActiveOrders,
            Panel::ActiveOrders => Panel::OrderEntry,
            Panel::OrderEntry => Panel::MarketBrowser,
        };
    }

    fn prev_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::MarketBrowser => Panel::OrderEntry,
            Panel::OrderBook => Panel::MarketBrowser,
            Panel::ActiveOrders => Panel::OrderBook,
            Panel::OrderEntry => Panel::ActiveOrders,
        };
    }

    fn update_market_browser_scroll(&mut self, viewport_height: usize) {
        // Update scroll offset based on selected item
        let selected_index = if !self.markets.is_empty() {
            self.selected_market
        } else if !self.events.is_empty() {
            self.selected_event
        } else if !self.competitions.is_empty() {
            self.selected_competition
        } else {
            self.selected_sport
        };

        if let Some(selected) = selected_index {
            if selected < self.market_browser_scroll_offset {
                // Selected item is above viewport
                self.market_browser_scroll_offset = selected;
            } else if selected >= self.market_browser_scroll_offset + viewport_height {
                // Selected item is below viewport
                self.market_browser_scroll_offset = selected.saturating_sub(viewport_height - 1);
            }
        }
    }

    fn update_active_orders_scroll(&mut self, viewport_height: usize) {
        // Update scroll offset based on selected order
        if let Some(selected) = self.selected_order {
            if selected < self.active_orders_scroll_offset {
                // Selected item is above viewport
                self.active_orders_scroll_offset = selected;
            } else if selected >= self.active_orders_scroll_offset + viewport_height {
                // Selected item is below viewport
                self.active_orders_scroll_offset = selected.saturating_sub(viewport_height - 1);
            }
        }
    }

    fn reset_market_browser_scroll(&mut self) {
        self.market_browser_scroll_offset = 0;
    }

    fn reset_active_orders_scroll(&mut self) {
        self.active_orders_scroll_offset = 0;
    }

    fn update_orderbook_from_streaming(&mut self) {
        // Update current orderbook display from streaming data if available
        if self.streaming_connected {
            if let Some(market_id) = &self.current_streaming_market {
                // Check for market-level update time
                if let Some(client) = &self.client {
                    if let Some(market_update_time) = client.get_market_last_update_time(market_id)
                    {
                        self.last_streaming_update = Some(market_update_time);
                    }
                }

                let orderbooks = self.streaming_orderbooks.read().unwrap();
                if let Some(market_orderbooks) = orderbooks.get(market_id) {
                    // Update status to show we're receiving streaming data
                    self.status_message =
                        format!("Streaming {} runners active", market_orderbooks.len());
                    if let Some(current_ob) = &mut self.current_orderbook {
                        // Only update if we have the same market
                        if current_ob.market_id == *market_id {
                            let mut data_updated = false;
                            // Update existing runners with streaming data
                            for runner in &mut current_ob.runners {
                                let runner_id_str = runner.runner_id.to_string();
                                if let Some(streaming_ob) = market_orderbooks.get(&runner_id_str) {
                                    // Check if data actually changed
                                    let new_bids: Vec<(f64, f64)> = streaming_ob
                                        .bids
                                        .iter()
                                        .take(10)
                                        .map(|level| (level.price, level.size))
                                        .collect();
                                    let new_asks: Vec<(f64, f64)> = streaming_ob
                                        .asks
                                        .iter()
                                        .take(10)
                                        .map(|level| (level.price, level.size))
                                        .collect();

                                    if new_bids != runner.bids || new_asks != runner.asks {
                                        data_updated = true;
                                        // Track previous best prices for change detection
                                        runner.prev_best_bid = runner.bids.first().map(|(p, _)| *p);
                                        runner.prev_best_ask = runner.asks.first().map(|(p, _)| *p);
                                        runner.bids = new_bids;
                                        runner.asks = new_asks;
                                        runner.last_update = Some(Instant::now());
                                    }
                                    runner.is_streaming = true;
                                }
                            }

                            if data_updated {
                                self.last_streaming_update = Some(Instant::now());
                            }
                        }
                    }
                } else {
                    // No streaming data available
                    if let Some(market_id) = &self.current_streaming_market {
                        self.status_message =
                            format!("Waiting for streaming data for market {market_id}");
                    }
                }
            }
        } else {
            // Streaming not connected
            if self.current_orderbook.is_some() {
                self.status_message = "Streaming not connected - showing static data".to_string();
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(2), // Shortcuts bar
        ])
        .split(f.area());

    // Main area split into 2x2 grid
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[1]);

    // Render Market Browser
    render_market_browser(f, left_chunks[0], app);

    // Render Order Book
    render_order_book(f, right_chunks[0], app);

    // Render Active Orders
    render_active_orders(f, left_chunks[1], app);

    // Render Order Entry
    render_order_entry(f, right_chunks[1], app);

    // Render Shortcuts Bar with status info
    render_shortcuts_bar(f, chunks[1], app);
}

fn get_selection_key(index: usize) -> String {
    if index < 9 {
        (index + 1).to_string()
    } else if index < 35 {
        ((b'a' + (index - 9) as u8) as char).to_string()
    } else {
        " ".to_string()
    }
}

fn render_market_browser(f: &mut Frame, area: Rect, app: &App) {
    let is_active = matches!(app.active_panel, Panel::MarketBrowser);
    let border_color = if is_active { Color::Cyan } else { Color::Gray };

    // Build breadcrumb
    let mut breadcrumb = String::from(" Market Browser");
    if let Some(sport_idx) = app.selected_sport {
        if let Some((_id, name, _count)) = app.sports.get(sport_idx) {
            breadcrumb.push_str(&format!(" > {name}"));
        }
    }
    if let Some(comp_idx) = app.selected_competition {
        if let Some((_id, name, _count)) = app.competitions.get(comp_idx) {
            breadcrumb.push_str(&format!(" > {name}"));
        }
    }
    if let Some(event_idx) = app.selected_event {
        if let Some((_id, name, _count)) = app.events.get(event_idx) {
            breadcrumb.push_str(&format!(" > {name}"));
        }
    }
    breadcrumb.push(' ');

    let block = Block::default()
        .title(breadcrumb)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);

    // Calculate visible area
    let visible_height = inner.height as usize;

    // Create list items based on current navigation level with selection keys
    // Priority: markets > events > competitions > sports
    let all_items: Vec<ListItem> = if !app.markets.is_empty() {
        // Show markets
        app.markets
            .iter()
            .enumerate()
            .map(|(idx, market)| {
                let key = get_selection_key(idx);
                let is_selected = app.selected_market == Some(idx);
                let style = if is_selected {
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!(
                    "[{}] {:<28} £{:.0}",
                    key, market.name, market.total_matched
                ))
                .style(style)
            })
            .collect()
    } else if !app.events.is_empty() {
        // Show events
        app.events
            .iter()
            .enumerate()
            .map(|(idx, (_id, name, count))| {
                let key = get_selection_key(idx);
                let is_selected = app.selected_event == Some(idx);
                let style = if is_selected {
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("[{key}] {name:<38} [{count} markets]")).style(style)
            })
            .collect()
    } else if !app.competitions.is_empty() {
        // Show competitions
        app.competitions
            .iter()
            .enumerate()
            .map(|(idx, (_id, name, count))| {
                let key = get_selection_key(idx);
                let is_selected = app.selected_competition == Some(idx);
                let style = if is_selected {
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("[{key}] {name:<38} [{count} markets]")).style(style)
            })
            .collect()
    } else {
        // Show sports (default)
        app.sports
            .iter()
            .enumerate()
            .map(|(idx, (_id, name, count))| {
                let key = get_selection_key(idx);
                let is_selected = app.selected_sport == Some(idx);
                let style = if is_selected {
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("[{key}] {name:<38} [{count} markets]")).style(style)
            })
            .collect()
    };

    // Apply scrolling - only show items within the viewport
    let visible_items: Vec<ListItem> = all_items
        .into_iter()
        .skip(app.market_browser_scroll_offset)
        .take(visible_height)
        .collect();

    let list = List::new(visible_items).block(Block::default());

    f.render_widget(block, area);
    f.render_widget(list, inner);
}

fn render_order_book(f: &mut Frame, area: Rect, app: &App) {
    let is_active = matches!(app.active_panel, Panel::OrderBook);
    let border_color = if is_active { Color::Cyan } else { Color::Gray };

    // Include selected market name and streaming status in title
    let title = if let Some(market_idx) = app.selected_market {
        if let Some(market) = app.markets.get(market_idx) {
            let streaming_indicator =
                if app.streaming_connected && app.current_streaming_market.is_some() {
                    if let Some(last_update) = app.last_streaming_update {
                        let elapsed = last_update.elapsed();
                        if elapsed.as_secs() < 2 {
                            " [LIVE •]" // Solid dot indicates recent update
                        } else if elapsed.as_secs() < 10 {
                            " [LIVE ◦]" // Hollow dot indicates connected but no recent updates
                        } else {
                            " [LIVE ?]" // Question mark indicates possibly stale
                        }
                    } else {
                        " [LIVE -]" // Dash indicates waiting for first update
                    }
                } else {
                    " [POLL]"
                };
            format!(" Order Book - {}{} ", market.name, streaming_indicator)
        } else {
            " Order Book ".to_string()
        }
    } else {
        " Order Book ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    if let Some(orderbook) = &app.current_orderbook {
        let inner = block.inner(area);

        // Add update timestamp if streaming
        let mut update_info = String::new();
        if app.streaming_connected {
            if let Some(last_update) = app.last_streaming_update {
                let elapsed = last_update.elapsed();
                update_info = format!("Last update: {:.1}s ago | ", elapsed.as_secs_f32());
            } else {
                update_info = "Waiting for streaming data... | ".to_string();
            }
        }

        // Split area into sections for each runner
        let num_runners = orderbook.runners.len().min(4); // Show max 4 runners
        if num_runners == 0 {
            let text = Paragraph::new(format!("{update_info}No runners available"))
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(text, area);
            return;
        }

        let runner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                (0..num_runners)
                    .map(|_| Constraint::Ratio(1, num_runners as u32))
                    .collect::<Vec<_>>(),
            )
            .split(inner);

        // Render each runner's orderbook
        for (idx, (runner, chunk)) in orderbook
            .runners
            .iter()
            .zip(runner_chunks.iter())
            .enumerate()
        {
            let is_selected = app.selected_runner == Some(idx);
            let runner_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Create runner header with selection key - show both name and ID
            let key = get_selection_key(idx);
            let runner_title = format!(
                "[{}] {} (ID: {})",
                key, runner.runner_name, runner.runner_id
            );

            // Create order book rows for this runner
            let mut rows = vec![];
            let max_levels = std::cmp::max(runner.bids.len(), runner.asks.len()).min(3); // Show max 3 levels per runner

            for i in 0..max_levels {
                let bid_price = runner
                    .bids
                    .get(i)
                    .map(|(p, _)| format!("{p:.2}"))
                    .unwrap_or_default();
                let bid_size = runner
                    .bids
                    .get(i)
                    .map(|(_, s)| format!("{s:.0}"))
                    .unwrap_or_default();
                let ask_price = runner
                    .asks
                    .get(i)
                    .map(|(p, _)| format!("{p:.2}"))
                    .unwrap_or_default();
                let ask_size = runner
                    .asks
                    .get(i)
                    .map(|(_, s)| format!("{s:.0}"))
                    .unwrap_or_default();

                // Highlight best prices that have recently changed
                let bid_style = if i == 0 && runner.is_streaming {
                    if let (Some(last_update), Some(curr_price), Some(prev_price)) = (
                        runner.last_update,
                        runner.bids.first().map(|(p, _)| *p),
                        runner.prev_best_bid,
                    ) {
                        if last_update.elapsed().as_millis() < 500 {
                            if curr_price > prev_price {
                                Style::default()
                                    .fg(Color::LightGreen)
                                    .add_modifier(Modifier::BOLD)
                            } else if curr_price < prev_price {
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::DIM)
                            } else {
                                Style::default().fg(Color::Green)
                            }
                        } else {
                            Style::default().fg(Color::Green)
                        }
                    } else {
                        Style::default().fg(Color::Green)
                    }
                } else {
                    Style::default().fg(Color::Green)
                };

                let ask_style = if i == 0 && runner.is_streaming {
                    if let (Some(last_update), Some(curr_price), Some(prev_price)) = (
                        runner.last_update,
                        runner.asks.first().map(|(p, _)| *p),
                        runner.prev_best_ask,
                    ) {
                        if last_update.elapsed().as_millis() < 500 {
                            if curr_price < prev_price {
                                Style::default()
                                    .fg(Color::LightRed)
                                    .add_modifier(Modifier::BOLD)
                            } else if curr_price > prev_price {
                                Style::default().fg(Color::Red).add_modifier(Modifier::DIM)
                            } else {
                                Style::default().fg(Color::Red)
                            }
                        } else {
                            Style::default().fg(Color::Red)
                        }
                    } else {
                        Style::default().fg(Color::Red)
                    }
                } else {
                    Style::default().fg(Color::Red)
                };

                rows.push(Row::new(vec![
                    Cell::from(bid_size).style(bid_style),
                    Cell::from(bid_price).style(bid_style),
                    Cell::from(ask_price).style(ask_style),
                    Cell::from(ask_size).style(ask_style),
                ]));
            }

            let runner_block = Block::default()
                .title(runner_title)
                .borders(Borders::TOP)
                .border_style(runner_style);

            if rows.is_empty() {
                let no_prices = Paragraph::new("No prices available")
                    .block(runner_block)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::DarkGray));
                f.render_widget(no_prices, *chunk);
            } else {
                let table = Table::new(
                    rows,
                    [
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                    ],
                )
                .header(
                    Row::new(vec!["Size", "Back", "Lay", "Size"])
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .block(runner_block);

                f.render_widget(table, *chunk);
            }
        }

        f.render_widget(block, area);
    } else {
        let text = Paragraph::new("Select a market to view order book")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(text, area);
    }
}

fn render_active_orders(f: &mut Frame, area: Rect, app: &App) {
    let is_active = matches!(app.active_panel, Panel::ActiveOrders);
    let border_color = if is_active { Color::Cyan } else { Color::Gray };

    let block = Block::default()
        .title(format!(
            " Active Orders ({}) - Press 'c' to cancel selected ",
            app.active_orders.len()
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    if !app.active_orders.is_empty() {
        let inner = block.inner(area);

        // Calculate visible area (accounting for header)
        let visible_height = inner.height.saturating_sub(1) as usize; // Subtract 1 for header

        let all_rows: Vec<Row> = app
            .active_orders
            .iter()
            .enumerate()
            .map(|(index, order)| {
                let side_color = match order.side {
                    Side::Back => Color::Green,
                    Side::Lay => Color::Red,
                };

                let key = get_selection_key(index);

                // Truncate long names to fit
                let comp_name: String = order.competition_name.chars().take(15).collect();
                let event_name: String = order.event_name.chars().take(20).collect();
                let runner_name: String = order.runner_name.chars().take(15).collect();

                Row::new(vec![
                    Cell::from(format!("[{key}]")).style(Style::default().fg(Color::Yellow)),
                    Cell::from(comp_name),
                    Cell::from(event_name),
                    Cell::from(order.market_type.clone()),
                    Cell::from(runner_name),
                    Cell::from(format!("{:?}", order.side)).style(Style::default().fg(side_color)),
                    Cell::from(format!("{:.2}", order.price)),
                    Cell::from(format!("£{:.2}", order.size)),
                    Cell::from(format!("£{:.2}", order.size_matched)),
                    Cell::from(order.status.clone()),
                ])
            })
            .collect();

        // Apply scrolling - only show rows within the viewport
        let visible_rows: Vec<Row> = all_rows
            .into_iter()
            .skip(app.active_orders_scroll_offset)
            .take(visible_height)
            .collect();

        // Create a stateful table with proper selection
        // Adjust selection index for viewport
        let mut table_state = TableState::default();
        if let Some(selected) = app.selected_order {
            if selected >= app.active_orders_scroll_offset
                && selected < app.active_orders_scroll_offset + visible_height
            {
                table_state.select(Some(selected - app.active_orders_scroll_offset));
            }
        }

        let table = Table::new(
            visible_rows,
            [
                Constraint::Length(4),  // Key
                Constraint::Length(15), // Competition
                Constraint::Length(20), // Event
                Constraint::Length(12), // Market Type
                Constraint::Length(15), // Runner
                Constraint::Length(5),  // Side
                Constraint::Length(7),  // Price
                Constraint::Length(8),  // Size
                Constraint::Length(8),  // Matched
                Constraint::Length(10), // Status
            ],
        )
        .header(
            Row::new(vec![
                "Key",
                "Competition",
                "Event",
                "Market",
                "Runner",
                "Side",
                "Price",
                "Size",
                "Matched",
                "Status",
            ])
            .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(Block::default())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                .fg(Color::Yellow),
        )
        .highlight_symbol("► ");

        f.render_widget(block, area);
        f.render_stateful_widget(table, inner, &mut table_state);
    } else {
        let text = Paragraph::new("No active orders")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(text, area);
    }
}

fn render_order_entry(f: &mut Frame, area: Rect, app: &App) {
    let is_active = matches!(app.active_panel, Panel::OrderEntry);
    let border_color = if is_active { Color::Cyan } else { Color::Gray };

    let block = Block::default()
        .title(" Order Entry ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);

    // Market ID
    let market_text = format!("Market: {}", app.order_market_id);
    f.render_widget(Paragraph::new(market_text), chunks[0]);

    // Runner selection
    let runner_text = if app.order_runner_name.is_empty() {
        "Runner: (Press 1-9 in Order Book to select)".to_string()
    } else {
        format!(
            "Runner: {} (ID: {})",
            app.order_runner_name, app.order_selection_id
        )
    };
    let runner_style = if app.order_runner_name.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Cyan)
    };
    f.render_widget(Paragraph::new(runner_text).style(runner_style), chunks[1]);

    // Side selection
    let side_text = format!("Side: {:?}", app.order_side);
    let side_color = match app.order_side {
        Side::Back => Color::Green,
        Side::Lay => Color::Red,
    };
    f.render_widget(
        Paragraph::new(side_text).style(Style::default().fg(side_color)),
        chunks[2],
    );

    // Price input
    let price_text = format!("Price: {}", app.order_price);
    let price_style = if app.order_field_focus == OrderField::Price {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    f.render_widget(Paragraph::new(price_text).style(price_style), chunks[3]);

    // Size input
    let size_text = format!("Stake: £{}", app.order_size);
    let size_style = if app.order_field_focus == OrderField::Size {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    f.render_widget(Paragraph::new(size_text).style(size_style), chunks[4]);

    // Instructions
    let instructions = if is_active {
        "Enter: Place | Tab: Next Field | B/L: Toggle Side | Esc: Cancel"
    } else {
        "Press 'o' to enter order mode | Select runner in Order Book (1-9)"
    };
    f.render_widget(
        Paragraph::new(instructions).style(Style::default().fg(Color::DarkGray)),
        chunks[5],
    );

    f.render_widget(block, area);
}

fn render_shortcuts_bar(f: &mut Frame, area: Rect, app: &App) {
    // Split the bar into left (status) and right (shortcuts) sections
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(60), // Status info (increased for error messages)
            Constraint::Min(1),     // Shortcuts
        ])
        .split(area);

    // Render status info on the left
    let mut status_parts = vec![];

    // Show error or status message first if present
    if let Some(err) = &app.error_message {
        status_parts.push(Span::styled(
            err,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
        status_parts.push(Span::raw(" | "));
    } else if !app.status_message.is_empty() && app.status_message != "Connected to Betfair" {
        status_parts.push(Span::styled(
            &app.status_message,
            Style::default().fg(Color::Green),
        ));
        status_parts.push(Span::raw(" | "));
    }

    status_parts.extend(vec![
        if app.api_connected {
            Span::styled("●", Style::default().fg(Color::Green))
        } else {
            Span::styled("●", Style::default().fg(Color::Red))
        },
        if app.streaming_connected {
            Span::styled(
                " LIVE",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(" POLL", Style::default().fg(Color::DarkGray))
        },
        Span::raw(format!(" £{:.2}", app.available_balance)),
        Span::raw(format!(" [{} orders]", app.total_orders)),
    ]);

    let status_line = Line::from(status_parts);
    f.render_widget(
        Paragraph::new(status_line).style(Style::default().bg(Color::Black)),
        chunks[0],
    );

    // Render shortcuts on the right
    let shortcuts = match app.mode {
        AppMode::Browse => match app.active_panel {
            Panel::MarketBrowser => {
                vec![
                    ("↑↓/jk", "Navigate"),
                    ("1-9/a-z", "Quick Select"),
                    ("Enter", "Select"),
                    ("Backspace", "Back"),
                    ("Tab", "Next Panel"),
                    ("o", "Order"),
                    ("q", "Quit"),
                ]
            }
            Panel::OrderBook => {
                vec![
                    ("↑↓/jk", "Navigate"),
                    ("1-9", "Quick Select"),
                    ("Tab", "Next Panel"),
                    ("o", "Order"),
                    ("r", "Refresh"),
                    ("q", "Quit"),
                ]
            }
            Panel::ActiveOrders => {
                vec![
                    ("↑↓/jk", "Navigate"),
                    ("1-9/a-z", "Quick Select"),
                    ("c/Enter", "Cancel"),
                    ("Tab", "Next Panel"),
                    ("o", "Order"),
                    ("q", "Quit"),
                ]
            }
            Panel::OrderEntry => {
                vec![
                    ("↑↓", "Switch Field"),
                    ("Tab", "Next Panel"),
                    ("o", "Order"),
                    ("r", "Refresh"),
                    ("q", "Quit"),
                ]
            }
        },
        AppMode::Order => {
            vec![
                ("↑↓/Tab", "Switch Field"),
                ("Enter", "Place Order"),
                ("Esc", "Cancel"),
                ("b/l", "Back/Lay"),
            ]
        }
        AppMode::Help => {
            vec![("Esc", "Close Help"), ("q", "Quit")]
        }
        _ => vec![],
    };

    let shortcuts_text: Vec<Span> = shortcuts
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut spans = vec![
                Span::styled(
                    *key,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(":"),
                Span::styled(*desc, Style::default().fg(Color::Gray)),
            ];
            if i < shortcuts.len() - 1 {
                spans.push(Span::raw("  "));
            }
            spans
        })
        .collect();

    let shortcuts_paragraph = Paragraph::new(Line::from(shortcuts_text))
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Right);

    f.render_widget(shortcuts_paragraph, chunks[1]);
}

fn handle_runner_selection(app: &mut App, index: usize) {
    if let Some(orderbook) = &app.current_orderbook {
        if index < orderbook.runners.len() {
            app.selected_runner = Some(index);
            let runner = &orderbook.runners[index];
            app.order_selection_id = runner.runner_id.to_string();
            app.order_runner_name = runner.runner_name.clone();
        }
    }
}

async fn handle_direct_selection(app: &mut App, index: usize) -> Result<()> {
    // Determine which list we're working with and select the item
    if !app.markets.is_empty() {
        if index < app.markets.len() {
            app.selected_market = Some(index);
            app.update_market_browser_scroll(20);
            // Load orderbook for selected market
            if let Some(market) = app.markets.get(index) {
                let market_id = market.id.clone();
                let order_market_id = market.id.clone();

                app.load_orderbook(&market_id).await?;
                app.order_market_id = order_market_id;

                // Set first runner as default
                if let Some(orderbook) = &app.current_orderbook {
                    if let Some(first_runner) = orderbook.runners.first() {
                        app.order_selection_id = first_runner.runner_id.to_string();
                        app.order_runner_name = first_runner.runner_name.clone();
                        app.selected_runner = Some(0);
                    }
                }
            }
        }
    } else if !app.events.is_empty() {
        if index < app.events.len() {
            app.selected_event = Some(index);
            app.update_market_browser_scroll(20);
            // Load markets for selected event
            if let Some(event) = app.events.get(index) {
                let event_id = event.0.clone();
                let sport_id = app
                    .selected_sport
                    .and_then(|i| app.sports.get(i).map(|s| s.0.clone()));
                if let Some(sport_id) = sport_id {
                    app.load_markets(&sport_id, Some(&event_id)).await?;
                    app.selected_market = None;
                    app.reset_market_browser_scroll();
                }
            }
        }
    } else if !app.competitions.is_empty() {
        if index < app.competitions.len() {
            app.selected_competition = Some(index);
            app.update_market_browser_scroll(20);
            // Load events for selected competition
            if let Some(comp) = app.competitions.get(index) {
                let comp_id = comp.0.clone();
                let sport_id = app
                    .selected_sport
                    .and_then(|i| app.sports.get(i).map(|s| s.0.clone()));
                if let Some(sport_id) = sport_id {
                    app.load_events(&sport_id, Some(&comp_id)).await?;
                    app.selected_event = None;
                    app.reset_market_browser_scroll();
                }
            }
        }
    } else if !app.sports.is_empty() && index < app.sports.len() {
        app.selected_sport = Some(index);
        app.update_market_browser_scroll(20);
        // Load competitions for selected sport
        if let Some(sport) = app.sports.get(index) {
            let sport_id = sport.0.clone();
            app.load_competitions(&sport_id).await?;
            app.selected_competition = None;
            app.reset_market_browser_scroll();
        }
    }
    Ok(())
}

async fn handle_input(app: &mut App, key: KeyCode) -> Result<bool> {
    match app.mode {
        AppMode::Browse => {
            match key {
                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
                KeyCode::Tab => app.next_panel(),
                KeyCode::BackTab => app.prev_panel(),
                KeyCode::Char('o') => {
                    app.mode = AppMode::Order;
                    app.active_panel = Panel::OrderEntry;
                    app.error_message = None; // Clear any old errors
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    app.load_account_info().await?;
                    app.load_active_orders().await?;
                    if let Some(market) = app.selected_market {
                        let market_id = app.markets.get(market).map(|m| m.id.clone());
                        if let Some(market_id) = market_id {
                            app.load_orderbook(&market_id).await?;
                        }
                    }
                    app.status_message = "Refreshed".to_string();
                }
                KeyCode::Char('?') => app.mode = AppMode::Help,
                // Arrow key navigation (Up/Down for moving within panel)
                KeyCode::Up | KeyCode::Char('k') => {
                    match app.active_panel {
                        Panel::MarketBrowser => {
                            // Navigate up in the currently active list
                            if !app.markets.is_empty() {
                                if let Some(selected) = app.selected_market {
                                    if selected > 0 {
                                        app.selected_market = Some(selected - 1);
                                        app.update_market_browser_scroll(20); // Assume ~20 visible items
                                    }
                                }
                            } else if !app.events.is_empty() {
                                if let Some(selected) = app.selected_event {
                                    if selected > 0 {
                                        app.selected_event = Some(selected - 1);
                                        app.update_market_browser_scroll(20);
                                    }
                                }
                            } else if !app.competitions.is_empty() {
                                if let Some(selected) = app.selected_competition {
                                    if selected > 0 {
                                        app.selected_competition = Some(selected - 1);
                                        app.update_market_browser_scroll(20);
                                    }
                                }
                            } else if !app.sports.is_empty() {
                                if let Some(selected) = app.selected_sport {
                                    if selected > 0 {
                                        app.selected_sport = Some(selected - 1);
                                        app.update_market_browser_scroll(20);
                                    }
                                }
                            }
                        }
                        Panel::OrderBook => {
                            if let Some(orderbook) = &app.current_orderbook {
                                if !orderbook.runners.is_empty() {
                                    if let Some(selected) = app.selected_runner {
                                        if selected > 0 {
                                            app.selected_runner = Some(selected - 1);
                                            handle_runner_selection(app, selected - 1);
                                        }
                                    } else {
                                        app.selected_runner = Some(0);
                                        handle_runner_selection(app, 0);
                                    }
                                }
                            }
                        }
                        Panel::ActiveOrders => {
                            if !app.active_orders.is_empty() {
                                if let Some(selected) = app.selected_order {
                                    if selected > 0 {
                                        app.selected_order = Some(selected - 1);
                                        app.update_active_orders_scroll(10); // Assume ~10 visible items
                                    }
                                } else {
                                    app.selected_order = Some(0);
                                    app.update_active_orders_scroll(10);
                                }
                            }
                        }
                        Panel::OrderEntry => {
                            // In order entry, up/down can switch between price and size fields
                            app.order_field_focus = match app.order_field_focus {
                                OrderField::Size => OrderField::Price,
                                OrderField::Price => OrderField::Size,
                            };
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    match app.active_panel {
                        Panel::MarketBrowser => {
                            // Navigate down in the currently active list
                            if !app.markets.is_empty() {
                                if let Some(selected) = app.selected_market {
                                    if selected < app.markets.len() - 1 {
                                        app.selected_market = Some(selected + 1);
                                        app.update_market_browser_scroll(20);
                                    }
                                } else {
                                    app.selected_market = Some(0);
                                    app.update_market_browser_scroll(20);
                                }
                            } else if !app.events.is_empty() {
                                if let Some(selected) = app.selected_event {
                                    if selected < app.events.len() - 1 {
                                        app.selected_event = Some(selected + 1);
                                        app.update_market_browser_scroll(20);
                                    }
                                } else {
                                    app.selected_event = Some(0);
                                    app.update_market_browser_scroll(20);
                                }
                            } else if !app.competitions.is_empty() {
                                if let Some(selected) = app.selected_competition {
                                    if selected < app.competitions.len() - 1 {
                                        app.selected_competition = Some(selected + 1);
                                        app.update_market_browser_scroll(20);
                                    }
                                } else {
                                    app.selected_competition = Some(0);
                                    app.update_market_browser_scroll(20);
                                }
                            } else if !app.sports.is_empty() {
                                if let Some(selected) = app.selected_sport {
                                    if selected < app.sports.len() - 1 {
                                        app.selected_sport = Some(selected + 1);
                                        app.update_market_browser_scroll(20);
                                    }
                                } else {
                                    app.selected_sport = Some(0);
                                    app.update_market_browser_scroll(20);
                                }
                            }
                        }
                        Panel::OrderBook => {
                            if let Some(orderbook) = &app.current_orderbook {
                                if !orderbook.runners.is_empty() {
                                    if let Some(selected) = app.selected_runner {
                                        if selected < orderbook.runners.len() - 1 {
                                            app.selected_runner = Some(selected + 1);
                                            handle_runner_selection(app, selected + 1);
                                        }
                                    } else {
                                        app.selected_runner = Some(0);
                                        handle_runner_selection(app, 0);
                                    }
                                }
                            }
                        }
                        Panel::ActiveOrders => {
                            if !app.active_orders.is_empty() {
                                if let Some(selected) = app.selected_order {
                                    if selected < app.active_orders.len() - 1 {
                                        app.selected_order = Some(selected + 1);
                                        app.update_active_orders_scroll(10);
                                    }
                                } else {
                                    app.selected_order = Some(0);
                                    app.update_active_orders_scroll(10);
                                }
                            }
                        }
                        Panel::OrderEntry => {
                            // In order entry, up/down can switch between price and size fields
                            app.order_field_focus = match app.order_field_focus {
                                OrderField::Price => OrderField::Size,
                                OrderField::Size => OrderField::Price,
                            };
                        }
                    }
                }
                KeyCode::Enter => {
                    match app.active_panel {
                        Panel::MarketBrowser => {
                            // Handle navigation through hierarchy based on current level
                            if !app.markets.is_empty() {
                                // We're at market level - load orderbook for selected market
                                if let Some(index) = app.selected_market {
                                    if let Some(market) = app.markets.get(index) {
                                        let market_id = market.id.clone();
                                        let order_market_id = market.id.clone();

                                        app.load_orderbook(&market_id).await?;
                                        app.order_market_id = order_market_id;

                                        // Set first runner as default
                                        if let Some(orderbook) = &app.current_orderbook {
                                            if let Some(first_runner) = orderbook.runners.first() {
                                                app.order_selection_id =
                                                    first_runner.runner_id.to_string();
                                                app.order_runner_name =
                                                    first_runner.runner_name.clone();
                                                app.selected_runner = Some(0);
                                            }
                                        }
                                    }
                                }
                            } else if !app.events.is_empty() {
                                // We're at event level - load markets for selected event
                                if let Some(index) = app.selected_event {
                                    if let Some(event) = app.events.get(index) {
                                        let event_id = event.0.clone();
                                        let sport_id = app
                                            .selected_sport
                                            .and_then(|i| app.sports.get(i).map(|s| s.0.clone()));
                                        if let Some(sport_id) = sport_id {
                                            app.load_markets(&sport_id, Some(&event_id)).await?;
                                            app.selected_market = if !app.markets.is_empty() {
                                                Some(0)
                                            } else {
                                                None
                                            };
                                        }
                                    }
                                }
                            } else if !app.competitions.is_empty() {
                                // We're at competition level - load events for selected competition
                                if let Some(index) = app.selected_competition {
                                    if let Some(comp) = app.competitions.get(index) {
                                        let comp_id = comp.0.clone();
                                        let sport_id = app
                                            .selected_sport
                                            .and_then(|i| app.sports.get(i).map(|s| s.0.clone()));
                                        if let Some(sport_id) = sport_id {
                                            app.load_events(&sport_id, Some(&comp_id)).await?;
                                            app.selected_event = if !app.events.is_empty() {
                                                Some(0)
                                            } else {
                                                None
                                            };
                                        }
                                    }
                                }
                            } else if !app.sports.is_empty() {
                                // We're at sports level - load competitions for selected sport
                                if let Some(index) = app.selected_sport {
                                    if let Some(sport) = app.sports.get(index) {
                                        let sport_id = sport.0.clone();
                                        app.load_competitions(&sport_id).await?;
                                        app.selected_competition = if !app.competitions.is_empty() {
                                            Some(0)
                                        } else {
                                            None
                                        };
                                    }
                                }
                            }
                        }
                        Panel::ActiveOrders => {
                            // Cancel selected order
                            if let Some(index) = app.selected_order {
                                let bet_id = app.active_orders.get(index).map(|o| o.bet_id.clone());
                                if let Some(bet_id) = bet_id {
                                    app.cancel_order(&bet_id).await?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Backspace | KeyCode::Esc => {
                    // Navigate back in hierarchy
                    if !app.markets.is_empty() {
                        // We're viewing markets - go back to events
                        app.markets.clear();
                        app.selected_market = None;
                        app.current_orderbook = None;
                        app.order_market_id.clear();
                        app.reset_market_browser_scroll();
                        app.status_message = "Back to events".to_string();
                        // Events should already be loaded, just need to display them
                    } else if !app.events.is_empty() {
                        // We're viewing events - go back to competitions
                        app.events.clear();
                        app.selected_event = None;
                        app.reset_market_browser_scroll();
                        app.status_message = "Back to competitions".to_string();
                        // Competitions should already be loaded
                    } else if !app.competitions.is_empty() {
                        // We're viewing competitions - go back to sports
                        app.competitions.clear();
                        app.selected_competition = None;
                        app.reset_market_browser_scroll();
                        app.status_message = "Back to sports".to_string();
                        // Sports list remains
                    } else if app.selected_sport.is_some() {
                        // We're at sport level but haven't loaded anything yet
                        // OR we've come back to sports list
                        if app.selected_competition.is_some() {
                            // Clear the selection that might be lingering
                            app.selected_competition = None;
                        } else {
                            // Just deselect the sport
                            app.selected_sport = None;
                            app.status_message = "Select a sport".to_string();
                        }
                    }
                }
                // Number keys 1-9 for direct selection
                KeyCode::Char('1') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 0).await?;
                    } else if app.active_panel == Panel::ActiveOrders {
                        if !app.active_orders.is_empty() {
                            app.selected_order = Some(0);
                            app.update_active_orders_scroll(10);
                        } else {
                            app.selected_order = None;
                        }
                    } else if app.active_panel == Panel::OrderBook {
                        handle_runner_selection(app, 0);
                    }
                }
                KeyCode::Char('2') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 1).await?;
                    } else if app.active_panel == Panel::ActiveOrders {
                        if app.active_orders.len() > 1 {
                            app.selected_order = Some(1);
                            app.update_active_orders_scroll(10);
                        } else {
                            app.selected_order = None;
                        }
                    } else if app.active_panel == Panel::OrderBook {
                        handle_runner_selection(app, 1);
                    }
                }
                KeyCode::Char('3') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 2).await?;
                    } else if app.active_panel == Panel::ActiveOrders {
                        if app.active_orders.len() > 2 {
                            app.selected_order = Some(2);
                            app.update_active_orders_scroll(10);
                        } else {
                            app.selected_order = None;
                        }
                    } else if app.active_panel == Panel::OrderBook {
                        handle_runner_selection(app, 2);
                    }
                }
                KeyCode::Char('4') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 3).await?;
                    } else if app.active_panel == Panel::ActiveOrders {
                        if app.active_orders.len() > 3 {
                            app.selected_order = Some(3);
                            app.update_active_orders_scroll(10);
                        } else {
                            app.selected_order = None;
                        }
                    } else if app.active_panel == Panel::OrderBook {
                        handle_runner_selection(app, 3);
                    }
                }
                KeyCode::Char('5') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 4).await?;
                    } else if app.active_panel == Panel::ActiveOrders {
                        if app.active_orders.len() > 4 {
                            app.selected_order = Some(4);
                            app.update_active_orders_scroll(10);
                        } else {
                            app.selected_order = None;
                        }
                    } else if app.active_panel == Panel::OrderBook {
                        handle_runner_selection(app, 4);
                    }
                }
                KeyCode::Char('6') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 5).await?;
                    } else if app.active_panel == Panel::ActiveOrders {
                        if app.active_orders.len() > 5 {
                            app.selected_order = Some(5);
                            app.update_active_orders_scroll(10);
                        } else {
                            app.selected_order = None;
                        }
                    } else if app.active_panel == Panel::OrderBook {
                        handle_runner_selection(app, 5);
                    }
                }
                KeyCode::Char('7') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 6).await?;
                    } else if app.active_panel == Panel::ActiveOrders {
                        if app.active_orders.len() > 6 {
                            app.selected_order = Some(6);
                            app.update_active_orders_scroll(10);
                        } else {
                            app.selected_order = None;
                        }
                    } else if app.active_panel == Panel::OrderBook {
                        handle_runner_selection(app, 6);
                    }
                }
                KeyCode::Char('8') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 7).await?;
                    } else if app.active_panel == Panel::ActiveOrders {
                        if app.active_orders.len() > 7 {
                            app.selected_order = Some(7);
                            app.update_active_orders_scroll(10);
                        } else {
                            app.selected_order = None;
                        }
                    } else if app.active_panel == Panel::OrderBook {
                        handle_runner_selection(app, 7);
                    }
                }
                KeyCode::Char('9') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 8).await?;
                    } else if app.active_panel == Panel::ActiveOrders {
                        if app.active_orders.len() > 8 {
                            app.selected_order = Some(8);
                            app.update_active_orders_scroll(10);
                        } else {
                            app.selected_order = None;
                        }
                    } else if app.active_panel == Panel::OrderBook {
                        handle_runner_selection(app, 8);
                    }
                }
                // 'c' key - cancel order in Active Orders panel, or select item 12 in Market Browser
                KeyCode::Char('c') => {
                    if app.active_panel == Panel::ActiveOrders {
                        if let Some(index) = app.selected_order {
                            let bet_id = app.active_orders.get(index).map(|o| o.bet_id.clone());
                            if let Some(bet_id) = bet_id {
                                app.cancel_order(&bet_id).await?;
                            }
                        }
                    } else if app.active_panel == Panel::MarketBrowser {
                        // 'c' selects item 12 (index 11) in the market browser
                        let index = 11; // 9 + ('c' - 'a') = 9 + 2 = 11
                        handle_direct_selection(app, index).await?;
                    }
                }
                KeyCode::Char('C') => {
                    if app.active_panel == Panel::ActiveOrders {
                        if let Some(index) = app.selected_order {
                            let bet_id = app.active_orders.get(index).map(|o| o.bet_id.clone());
                            if let Some(bet_id) = bet_id {
                                app.cancel_order(&bet_id).await?;
                            }
                        }
                    }
                }
                // Letter keys a-z (except 'c') for items 10-35
                KeyCode::Char(c) if c.is_ascii_lowercase() && c != 'c' => {
                    if app.active_panel == Panel::MarketBrowser {
                        let index = 9 + (c as usize - 'a' as usize);
                        handle_direct_selection(app, index).await?;
                    } else if app.active_panel == Panel::ActiveOrders {
                        let index = 9 + (c as usize - 'a' as usize);
                        if app.active_orders.len() > index {
                            app.selected_order = Some(index);
                            app.update_active_orders_scroll(10);
                        } else {
                            app.selected_order = None;
                        }
                    }
                }
                _ => {}
            }
        }
        AppMode::Order => {
            match key {
                KeyCode::Esc => {
                    app.mode = AppMode::Browse;
                    app.active_panel = Panel::MarketBrowser;
                    app.error_message = None; // Clear error on exit
                }
                KeyCode::Tab | KeyCode::Down | KeyCode::Char('j') => {
                    // Toggle between Price and Size fields (down moves forward)
                    app.order_field_focus = match app.order_field_focus {
                        OrderField::Price => OrderField::Size,
                        OrderField::Size => OrderField::Price,
                    };
                }
                KeyCode::BackTab | KeyCode::Up | KeyCode::Char('k') => {
                    // Toggle between Price and Size fields (up moves backward)
                    app.order_field_focus = match app.order_field_focus {
                        OrderField::Price => OrderField::Size,
                        OrderField::Size => OrderField::Price,
                    };
                }
                KeyCode::Enter => {
                    if !app.order_price.is_empty() && !app.order_size.is_empty() {
                        app.place_order().await?;
                        app.mode = AppMode::Browse;
                    }
                }
                KeyCode::Char('b') | KeyCode::Char('B') => {
                    app.order_side = Side::Back;
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    app.order_side = Side::Lay;
                }
                KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                    // Add input to the focused field
                    match app.order_field_focus {
                        OrderField::Price => {
                            if app.order_price.len() < 10 {
                                app.order_price.push(c);
                            }
                        }
                        OrderField::Size => {
                            if app.order_size.len() < 10 {
                                app.order_size.push(c);
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    // Remove last character from focused field
                    match app.order_field_focus {
                        OrderField::Price => {
                            app.order_price.pop();
                        }
                        OrderField::Size => {
                            app.order_size.pop();
                        }
                    }
                }
                _ => {}
            }
        }
        AppMode::Help => {
            if key == KeyCode::Esc || key == KeyCode::Char('q') {
                app.mode = AppMode::Browse;
            }
        }
        _ => {}
    }

    Ok(false)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Initialize app (login, load initial data)
    if let Err(e) = app.init().await {
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        eprintln!("Failed to initialize: {e}");
        return Err(e);
    }

    // Main loop
    let mut last_refresh = Instant::now();
    let mut last_streaming_update = Instant::now();
    let refresh_interval = Duration::from_secs(30);
    let streaming_update_interval = Duration::from_millis(100); // Update streaming data every 100ms

    loop {
        // Update orderbook from streaming data if available
        if app.streaming_connected && last_streaming_update.elapsed() > streaming_update_interval {
            app.update_orderbook_from_streaming();
            last_streaming_update = Instant::now();
        }

        // Draw UI
        terminal.draw(|f| ui(f, &app))?;

        // Handle events with timeout for periodic refresh
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if handle_input(&mut app, key.code).await? {
                    break;
                }
            }
        }

        // Periodic refresh for account and orders
        if last_refresh.elapsed() > refresh_interval {
            if let Err(e) = app.load_account_info().await {
                app.error_message = Some(format!("Refresh failed: {e}"));
            }
            if let Err(e) = app.load_active_orders().await {
                app.error_message = Some(format!("Refresh failed: {e}"));
            }

            // If not streaming, also refresh orderbook
            if !app.streaming_connected && app.selected_market.is_some() {
                if let Some(market) = app.selected_market.and_then(|idx| app.markets.get(idx)) {
                    let market_id = market.id.clone();
                    if let Err(e) = app.load_orderbook(&market_id).await {
                        app.error_message = Some(format!("Orderbook refresh failed: {e}"));
                    }
                }
            }

            last_refresh = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
