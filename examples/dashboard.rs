use anyhow::Result;
use betfair_rs::{
    api_client::BetfairApiClient,
    betfair::BetfairClient,
    config::Config,
    dto::{
        MarketFilter, ListMarketCatalogueRequest,
        common::{Side, OrderType, PersistenceType},
        account::GetAccountFundsRequest,
        order::{
            PlaceOrdersRequest, PlaceInstruction, LimitOrder,
            CancelOrdersRequest, CancelInstruction, ListCurrentOrdersRequest
        }
    },
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
        Block, BorderType, Borders, Cell, List, ListItem,
        Paragraph, Row, Table, TableState
    },
    Frame, Terminal,
};
use std::{
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

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
    #[allow(dead_code)]
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
}

#[derive(Debug, Clone)]
struct OrderBookData {
    #[allow(dead_code)]
    market_id: String,
    #[allow(dead_code)]
    runner_id: String,
    bids: Vec<(f64, f64)>, // (price, size)
    asks: Vec<(f64, f64)>, // (price, size)
    #[allow(dead_code)]
    last_traded: Option<f64>,
    #[allow(dead_code)]
    total_matched: f64,
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
    
    // Order book state
    current_orderbook: Option<OrderBookData>,
    
    // Active orders state
    active_orders: Vec<Order>,
    selected_order: Option<usize>,
    
    // Order entry state
    order_market_id: String,
    order_selection_id: String,
    order_side: Side,
    order_price: String,
    order_size: String,
    
    // Account state
    available_balance: f64,
    exposure: f64,
    total_orders: usize,
    
    // Connection state
    api_connected: bool,
    #[allow(dead_code)]
    streaming_connected: bool,
    #[allow(dead_code)]
    last_update: Instant,
    
    // Search state
    #[allow(dead_code)]
    search_query: String,
    
    // UI state
    status_message: String,
    error_message: Option<String>,
    
    // Clients
    api_client: Option<BetfairApiClient>,
    streaming_client: Option<Arc<Mutex<BetfairClient>>>,
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
            
            current_orderbook: None,
            
            active_orders: vec![],
            selected_order: None,
            
            order_market_id: String::new(),
            order_selection_id: String::new(),
            order_side: Side::Back,
            order_price: String::new(),
            order_size: String::new(),
            
            available_balance: 0.0,
            exposure: 0.0,
            total_orders: 0,
            
            api_connected: false,
            streaming_connected: false,
            last_update: Instant::now(),
            
            search_query: String::new(),
            
            status_message: "Initializing...".to_string(),
            error_message: None,
            
            api_client: None,
            streaming_client: None,
        }
    }
    
    async fn init(&mut self) -> Result<()> {
        // Load configuration and create clients
        let config = Config::new()?;
        
        // Initialize API client
        let mut api_client = BetfairApiClient::new(config.clone());
        self.status_message = "Logging in to Betfair API...".to_string();
        
        let login_response = api_client.login().await?;
        if login_response.login_status != "SUCCESS" {
            return Err(anyhow::anyhow!("Login failed: {}", login_response.login_status));
        }
        
        self.api_connected = true;
        self.api_client = Some(api_client);
        
        // Initialize streaming client (in background)
        let streaming_client = Arc::new(Mutex::new(BetfairClient::new(config.clone())));
        self.streaming_client = Some(streaming_client);
        
        // Load initial data
        self.load_sports().await?;
        self.load_account_info().await?;
        self.load_active_orders().await?;
        
        self.status_message = "Connected to Betfair".to_string();
        Ok(())
    }
    
    async fn load_sports(&mut self) -> Result<()> {
        if let Some(client) = &mut self.api_client {
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
        if let Some(client) = &mut self.api_client {
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
        if let Some(client) = &mut self.api_client {
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
        if let Some(client) = &mut self.api_client {
            let mut filter = MarketFilter {
                event_type_ids: Some(vec![sport_id.to_string()]),
                ..Default::default()
            };
            if let Some(ev_id) = event_id {
                filter.event_ids = Some(vec![ev_id.to_string()]);
            }
            
            let request = ListMarketCatalogueRequest {
                filter,
                market_projection: None,
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
                    runners: m.runners.unwrap_or_default().into_iter().map(|r| Runner {
                        id: r.selection_id as u64,
                        name: r.runner_name,
                        status: "Active".to_string(),
                        last_price_traded: None,
                        total_matched: None,
                    }).collect(),
                })
                .collect();
        }
        Ok(())
    }
    
    async fn load_orderbook(&mut self, market_id: &str) -> Result<()> {
        if let Some(client) = &mut self.api_client {
            let market_books = client.get_odds(market_id.to_string()).await?;
            
            if let Some(market_book) = market_books.first() {
                if let Some(runners) = &market_book.runners {
                    if let Some(runner) = runners.first() {
                        let mut bids = vec![];
                        let mut asks = vec![];
                        
                        if let Some(ex) = &runner.ex {
                            if let Some(back_prices) = &ex.available_to_back {
                                bids = back_prices.iter()
                                    .take(10)
                                    .map(|p| (p.price, p.size))
                                    .collect();
                            }
                            if let Some(lay_prices) = &ex.available_to_lay {
                                asks = lay_prices.iter()
                                    .take(10)
                                    .map(|p| (p.price, p.size))
                                    .collect();
                            }
                        }
                        
                        self.current_orderbook = Some(OrderBookData {
                            market_id: market_id.to_string(),
                            runner_id: runner.selection_id.to_string(),
                            bids,
                            asks,
                            last_traded: runner.last_price_traded,
                            total_matched: runner.total_matched.unwrap_or(0.0),
                        });
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn load_account_info(&mut self) -> Result<()> {
        if let Some(client) = &mut self.api_client {
            let funds = client.get_account_funds(GetAccountFundsRequest { wallet: None }).await?;
            self.available_balance = funds.available_to_bet_balance;
            self.exposure = funds.exposure;
        }
        Ok(())
    }
    
    async fn load_active_orders(&mut self) -> Result<()> {
        if let Some(client) = &mut self.api_client {
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
            self.active_orders = response.current_orders
                .into_iter()
                .map(|o| Order {
                    bet_id: o.bet_id,
                    market_id: o.market_id,
                    selection_id: o.selection_id,
                    side: o.side,
                    price: o.price_size.price,
                    size: o.price_size.size,
                    size_matched: o.size_matched.unwrap_or(0.0),
                    status: format!("{:?}", o.status),
                    placed_date: o.placed_date.unwrap_or_default(),
                })
                .collect();
            self.total_orders = self.active_orders.len();
        }
        Ok(())
    }
    
    async fn place_order(&mut self) -> Result<()> {
        if let Some(client) = &mut self.api_client {
            let price: f64 = self.order_price.parse()?;
            let size: f64 = self.order_size.parse()?;
            let selection_id: i64 = self.order_selection_id.parse()?;
            
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
                self.status_message = "Order placed successfully".to_string();
                self.load_active_orders().await?;
                self.load_account_info().await?;
                
                // Clear order form
                self.order_price.clear();
                self.order_size.clear();
            } else {
                self.error_message = Some(format!("Order failed: {}", response.status));
            }
        }
        Ok(())
    }
    
    async fn cancel_order(&mut self, bet_id: &str) -> Result<()> {
        if let Some(client) = &mut self.api_client {
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
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(2),  // Shortcuts bar
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
    
    // Render Status Bar
    render_status_bar(f, chunks[1], app);
    
    // Render Shortcuts Bar
    render_shortcuts_bar(f, chunks[2], app);
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
    
    let block = Block::default()
        .title(" Market Browser (Press key to select) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    
    let inner = block.inner(area);
    
    // Create list items based on current navigation level with selection keys
    let items: Vec<ListItem> = if app.selected_sport.is_none() {
        // Show sports
        app.sports.iter().enumerate().map(|(idx, (_id, name, count))| {
            let key = get_selection_key(idx);
            let is_selected = app.selected_sport == Some(idx);
            let style = if is_selected {
                Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("[{key}] {name:<38} [{count} markets]"))
                .style(style)
        }).collect()
    } else if app.selected_competition.is_none() && !app.competitions.is_empty() {
        // Show competitions
        app.competitions.iter().enumerate().map(|(idx, (_id, name, count))| {
            let key = get_selection_key(idx);
            let is_selected = app.selected_competition == Some(idx);
            let style = if is_selected {
                Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("[{key}] {name:<38} [{count} markets]"))
                .style(style)
        }).collect()
    } else if app.selected_event.is_none() && !app.events.is_empty() {
        // Show events
        app.events.iter().enumerate().map(|(idx, (_id, name, count))| {
            let key = get_selection_key(idx);
            let is_selected = app.selected_event == Some(idx);
            let style = if is_selected {
                Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("[{key}] {name:<38} [{count} markets]"))
                .style(style)
        }).collect()
    } else {
        // Show markets
        app.markets.iter().enumerate().map(|(idx, market)| {
            let key = get_selection_key(idx);
            let is_selected = app.selected_market == Some(idx);
            let style = if is_selected {
                Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("[{}] {:<28} £{:.0}", key, market.name, market.total_matched))
                .style(style)
        }).collect()
    };
    
    let list = List::new(items)
        .block(Block::default());
    
    f.render_widget(block, area);
    f.render_widget(list, inner);
}

fn render_order_book(f: &mut Frame, area: Rect, app: &App) {
    let is_active = matches!(app.active_panel, Panel::OrderBook);
    let border_color = if is_active { Color::Cyan } else { Color::Gray };
    
    let block = Block::default()
        .title(" Order Book ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    
    if let Some(orderbook) = &app.current_orderbook {
        let inner = block.inner(area);
        
        // Create order book display
        let mut rows = vec![];
        let max_levels = std::cmp::max(orderbook.bids.len(), orderbook.asks.len());
        
        for i in 0..max_levels {
            let bid_price = orderbook.bids.get(i).map(|(p, _)| format!("{p:.2}")).unwrap_or_default();
            let bid_size = orderbook.bids.get(i).map(|(_, s)| format!("{s:.0}")).unwrap_or_default();
            let ask_price = orderbook.asks.get(i).map(|(p, _)| format!("{p:.2}")).unwrap_or_default();
            let ask_size = orderbook.asks.get(i).map(|(_, s)| format!("{s:.0}")).unwrap_or_default();
            
            rows.push(Row::new(vec![
                Cell::from(bid_size).style(Style::default().fg(Color::Green)),
                Cell::from(bid_price).style(Style::default().fg(Color::Green)),
                Cell::from(ask_price).style(Style::default().fg(Color::Red)),
                Cell::from(ask_size).style(Style::default().fg(Color::Red)),
            ]));
        }
        
        let table = Table::new(
            rows,
            [
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]
        )
        .header(Row::new(vec!["Size", "Bid", "Ask", "Size"])
            .style(Style::default().add_modifier(Modifier::BOLD)))
        .block(Block::default());
        
        f.render_widget(block, area);
        f.render_widget(table, inner);
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
        .title(format!(" Active Orders ({}) ", app.active_orders.len()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    
    if !app.active_orders.is_empty() {
        let inner = block.inner(area);
        
        let rows: Vec<Row> = app.active_orders.iter().map(|order| {
            let side_color = match order.side {
                Side::Back => Color::Green,
                Side::Lay => Color::Red,
            };
            
            Row::new(vec![
                Cell::from(order.bet_id.chars().take(8).collect::<String>()),
                Cell::from(format!("{:?}", order.side)).style(Style::default().fg(side_color)),
                Cell::from(format!("{:.2}", order.price)),
                Cell::from(format!("£{:.2}", order.size)),
                Cell::from(format!("£{:.2}", order.size_matched)),
                Cell::from(order.status.clone()),
            ])
        }).collect();
        
        // Create a stateful table with proper selection
        let mut table_state = TableState::default();
        table_state.select(app.selected_order);
        
        let table = Table::new(
            rows,
            [
                Constraint::Length(8),
                Constraint::Length(5),
                Constraint::Length(7),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(10),
            ]
        )
        .header(Row::new(vec!["ID", "Side", "Price", "Size", "Matched", "Status"])
            .style(Style::default().add_modifier(Modifier::BOLD)))
        .block(Block::default())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                .fg(Color::Cyan)
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
            Constraint::Min(1),
        ])
        .split(inner);
    
    // Market ID
    let market_text = format!("Market: {}", app.order_market_id);
    f.render_widget(Paragraph::new(market_text), chunks[0]);
    
    // Side selection
    let side_text = format!("Side: {:?}", app.order_side);
    let side_color = match app.order_side {
        Side::Back => Color::Green,
        Side::Lay => Color::Red,
    };
    f.render_widget(
        Paragraph::new(side_text).style(Style::default().fg(side_color)),
        chunks[1]
    );
    
    // Price input
    let price_text = format!("Price: {}", app.order_price);
    f.render_widget(Paragraph::new(price_text), chunks[2]);
    
    // Size input
    let size_text = format!("Stake: £{}", app.order_size);
    f.render_widget(Paragraph::new(size_text), chunks[3]);
    
    // Instructions
    let instructions = if is_active {
        "Enter: Place | Tab: Next Field | B/L: Toggle Side | Esc: Cancel"
    } else {
        "Press 'o' to enter order mode"
    };
    f.render_widget(
        Paragraph::new(instructions)
            .style(Style::default().fg(Color::DarkGray)),
        chunks[4]
    );
    
    f.render_widget(block, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(15),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Min(1),
            Constraint::Length(30),
        ])
        .split(area);
    
    // Connection status
    let conn_status = if app.api_connected {
        Span::styled("[Connected]", Style::default().fg(Color::Green))
    } else {
        Span::styled("[Disconnected]", Style::default().fg(Color::Red))
    };
    f.render_widget(Paragraph::new(conn_status), chunks[0]);
    
    // Balance
    let balance = format!("Balance: £{:.2}", app.available_balance);
    f.render_widget(Paragraph::new(balance), chunks[1]);
    
    // Orders count
    let orders = format!("Orders: {}", app.total_orders);
    f.render_widget(Paragraph::new(orders), chunks[2]);
    
    // Status message or error
    let message = if let Some(err) = &app.error_message {
        Span::styled(err, Style::default().fg(Color::Red))
    } else {
        Span::raw(&app.status_message)
    };
    f.render_widget(Paragraph::new(message), chunks[3]);
    
    // Help
    let help = "Tab:Panel | O:Order | R:Refresh | ?:Help | Q:Quit";
    f.render_widget(
        Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Right),
        chunks[4]
    );
}

fn render_shortcuts_bar(f: &mut Frame, area: Rect, app: &App) {
    let shortcuts = match app.mode {
        AppMode::Browse => {
            match app.active_panel {
                Panel::MarketBrowser => {
                    vec![
                        ("1-9/a-z", "Select Item"),
                        ("Backspace", "Back"),
                        ("Tab", "Next Panel"),
                        ("o", "Order Mode"),
                        ("r", "Refresh"),
                        ("?", "Help"),
                        ("q", "Quit"),
                    ]
                },
                Panel::OrderBook => {
                    vec![
                        ("Tab", "Next Panel"),
                        ("Shift+Tab", "Prev Panel"),
                        ("o", "Order Mode"),
                        ("r", "Refresh"),
                        ("?", "Help"),
                        ("q", "Quit"),
                    ]
                },
                Panel::ActiveOrders => {
                    vec![
                        ("↑↓/jk", "Navigate"),
                        ("Enter", "Cancel Order"),
                        ("Tab", "Next Panel"),
                        ("o", "Order Mode"),
                        ("r", "Refresh"),
                        ("?", "Help"),
                        ("q", "Quit"),
                    ]
                },
                Panel::OrderEntry => {
                    vec![
                        ("Tab", "Next Panel"),
                        ("o", "Order Mode"),
                        ("r", "Refresh"),
                        ("?", "Help"),
                        ("q", "Quit"),
                    ]
                },
            }
        },
        AppMode::Order => {
            vec![
                ("Tab", "Next Field"),
                ("↑↓", "Adjust Value"),
                ("Enter", "Place Order"),
                ("Esc", "Cancel"),
                ("b/l", "Back/Lay"),
            ]
        },
        AppMode::Help => {
            vec![
                ("Esc", "Close Help"),
                ("q", "Quit"),
            ]
        },
        _ => vec![],
    };
    
    let shortcuts_text: Vec<Span> = shortcuts
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut spans = vec![
                Span::styled(*key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(":"),
                Span::styled(*desc, Style::default().fg(Color::Gray)),
            ];
            if i < shortcuts.len() - 1 {
                spans.push(Span::raw("  "));
            }
            spans
        })
        .collect();
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    
    let shortcuts_paragraph = Paragraph::new(Line::from(shortcuts_text))
        .block(block)
        .alignment(Alignment::Center);
    
    f.render_widget(shortcuts_paragraph, area);
}

async fn handle_direct_selection(app: &mut App, index: usize) -> Result<()> {
    // Determine which list we're working with and select the item
    if !app.markets.is_empty() {
        if index < app.markets.len() {
            app.selected_market = Some(index);
            // Load orderbook for selected market
            if let Some(market) = app.markets.get(index) {
                let market_id = market.id.clone();
                let order_market_id = market.id.clone();
                let order_selection_id = market.runners.first().map(|r| r.id.to_string()).unwrap_or_default();
                
                app.load_orderbook(&market_id).await?;
                app.order_market_id = order_market_id;
                app.order_selection_id = order_selection_id;
            }
        }
    } else if !app.events.is_empty() {
        if index < app.events.len() {
            app.selected_event = Some(index);
            // Load markets for selected event
            if let Some(event) = app.events.get(index) {
                let event_id = event.0.clone();
                let sport_id = app.selected_sport.and_then(|i| app.sports.get(i).map(|s| s.0.clone()));
                if let Some(sport_id) = sport_id {
                    app.load_markets(&sport_id, Some(&event_id)).await?;
                    app.selected_market = None;
                }
            }
        }
    } else if !app.competitions.is_empty() {
        if index < app.competitions.len() {
            app.selected_competition = Some(index);
            // Load events for selected competition
            if let Some(comp) = app.competitions.get(index) {
                let comp_id = comp.0.clone();
                let sport_id = app.selected_sport.and_then(|i| app.sports.get(i).map(|s| s.0.clone()));
                if let Some(sport_id) = sport_id {
                    app.load_events(&sport_id, Some(&comp_id)).await?;
                    app.selected_event = None;
                }
            }
        }
    } else if !app.sports.is_empty()
        && index < app.sports.len() {
            app.selected_sport = Some(index);
            // Load competitions for selected sport
            if let Some(sport) = app.sports.get(index) {
                let sport_id = sport.0.clone();
                app.load_competitions(&sport_id).await?;
                app.selected_competition = None;
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
                                        let order_selection_id = market.runners.first().map(|r| r.id.to_string()).unwrap_or_default();
                                        
                                        app.load_orderbook(&market_id).await?;
                                        app.order_market_id = order_market_id;
                                        app.order_selection_id = order_selection_id;
                                    }
                                }
                            } else if !app.events.is_empty() {
                                // We're at event level - load markets for selected event
                                if let Some(index) = app.selected_event {
                                    if let Some(event) = app.events.get(index) {
                                        let event_id = event.0.clone();
                                        let sport_id = app.selected_sport.and_then(|i| app.sports.get(i).map(|s| s.0.clone()));
                                        if let Some(sport_id) = sport_id {
                                            app.load_markets(&sport_id, Some(&event_id)).await?;
                                            app.selected_market = if !app.markets.is_empty() { Some(0) } else { None };
                                        }
                                    }
                                }
                            } else if !app.competitions.is_empty() {
                                // We're at competition level - load events for selected competition
                                if let Some(index) = app.selected_competition {
                                    if let Some(comp) = app.competitions.get(index) {
                                        let comp_id = comp.0.clone();
                                        let sport_id = app.selected_sport.and_then(|i| app.sports.get(i).map(|s| s.0.clone()));
                                        if let Some(sport_id) = sport_id {
                                            app.load_events(&sport_id, Some(&comp_id)).await?;
                                            app.selected_event = if !app.events.is_empty() { Some(0) } else { None };
                                        }
                                    }
                                }
                            } else if !app.sports.is_empty() {
                                // We're at sports level - load competitions for selected sport
                                if let Some(index) = app.selected_sport {
                                    if let Some(sport) = app.sports.get(index) {
                                        let sport_id = sport.0.clone();
                                        app.load_competitions(&sport_id).await?;
                                        app.selected_competition = if !app.competitions.is_empty() { Some(0) } else { None };
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
                    if app.selected_market.is_some() {
                        app.selected_market = None;
                        app.markets.clear();
                    } else if app.selected_event.is_some() {
                        app.selected_event = None;
                        app.events.clear();
                    } else if app.selected_competition.is_some() {
                        app.selected_competition = None;
                        app.competitions.clear();
                    } else if app.selected_sport.is_some() {
                        app.selected_sport = None;
                    }
                }
                // Number keys 1-9 for direct selection
                KeyCode::Char('1') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 0).await?;
                    }
                }
                KeyCode::Char('2') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 1).await?;
                    }
                }
                KeyCode::Char('3') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 2).await?;
                    }
                }
                KeyCode::Char('4') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 3).await?;
                    }
                }
                KeyCode::Char('5') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 4).await?;
                    }
                }
                KeyCode::Char('6') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 5).await?;
                    }
                }
                KeyCode::Char('7') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 6).await?;
                    }
                }
                KeyCode::Char('8') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 7).await?;
                    }
                }
                KeyCode::Char('9') => {
                    if app.active_panel == Panel::MarketBrowser {
                        handle_direct_selection(app, 8).await?;
                    }
                }
                // Letter keys a-z for items 10-35
                KeyCode::Char(c) if ('a'..='z').contains(&c) => {
                    if app.active_panel == Panel::MarketBrowser {
                        let index = 9 + (c as usize - 'a' as usize);
                        handle_direct_selection(app, index).await?;
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
                    // Simple input handling - in real app would need proper field selection
                    if app.order_price.len() < 10 {
                        app.order_price.push(c);
                    }
                }
                KeyCode::Backspace => {
                    app.order_price.pop();
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
    let refresh_interval = Duration::from_secs(30);
    
    loop {
        // Draw UI
        terminal.draw(|f| ui(f, &app))?;
        
        // Handle events with timeout for periodic refresh
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if handle_input(&mut app, key.code).await? {
                    break;
                }
            }
        }
        
        // Periodic refresh
        if last_refresh.elapsed() > refresh_interval {
            if let Err(e) = app.load_account_info().await {
                app.error_message = Some(format!("Refresh failed: {e}"));
            }
            if let Err(e) = app.load_active_orders().await {
                app.error_message = Some(format!("Refresh failed: {e}"));
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