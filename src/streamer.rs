use crate::connection_state::{ConnectionManager, ConnectionState};
use crate::msg_model::HeartbeatMessage;
use crate::msg_model::MarketChangeMessage;
use crate::msg_model::OrderChangeMessage;
use crate::orderbook::Orderbook;
use crate::retry::{RetryConfig, RetryPolicy};
use anyhow::Result;
use rustls_pki_types::ServerName;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use tracing::{debug, error, info, warn};

const STREAM_API_ENDPOINT: &str = "stream-api.betfair.com:443";
const STREAM_API_HOST: &str = "stream-api.betfair.com";

type OrderbookCallback = Arc<dyn Fn(String, HashMap<String, Orderbook>) + Send + Sync + 'static>;
type OrderUpdateCallback = Arc<dyn Fn(OrderChangeMessage) + Send + Sync + 'static>;

pub struct BetfairStreamer {
    app_key: String,
    ssoid: String,
    orderbook_callback: Option<OrderbookCallback>,
    orderupdate_callback: Option<OrderUpdateCallback>,
    message_sender: Option<mpsc::Sender<String>>,
    message_receiver: Option<mpsc::Receiver<String>>,
    subscribed_markets: HashSet<(String, usize)>,
    subscribed_to_orders: bool,
    last_message_ts: Arc<Mutex<Instant>>,
    heartbeat_threshold: Duration,
    is_resubscribing: Arc<Mutex<bool>>,
    orderbooks: HashMap<String, HashMap<String, Orderbook>>,
    connection_manager: ConnectionManager,
    _retry_policy: RetryPolicy,
}

impl BetfairStreamer {
    pub fn new(app_key: String, ssoid: String) -> Self {
        Self {
            app_key,
            ssoid,
            orderbook_callback: None,
            orderupdate_callback: None,
            message_sender: None,
            message_receiver: None,
            subscribed_markets: HashSet::new(),
            subscribed_to_orders: false,
            last_message_ts: Arc::new(Mutex::new(Instant::now() + Duration::from_secs(10))),
            heartbeat_threshold: Duration::from_secs(10),
            is_resubscribing: Arc::new(Mutex::new(false)),
            orderbooks: HashMap::new(),
            connection_manager: ConnectionManager::new(),
            _retry_policy: RetryPolicy::new(RetryConfig {
                max_attempts: 5,
                initial_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(60),
                multiplier: 2.0,
            }),
        }
    }

    pub fn set_orderbook_callback<F>(&mut self, callback: F)
    where
        F: Fn(String, HashMap<String, Orderbook>) + Send + Sync + 'static,
    {
        self.orderbook_callback = Some(Arc::new(callback));
    }

    pub fn set_orderupdate_callback<F>(&mut self, callback: F)
    where
        F: Fn(OrderChangeMessage) + Send + Sync + 'static,
    {
        self.orderupdate_callback = Some(Arc::new(callback));
    }

    pub async fn connect_betfair_tls_stream(&mut self) -> Result<()> {
        self.connection_manager
            .set_state(ConnectionState::Connecting)
            .await;
        info!("TLS connect starting");

        let auth_msg = format!(
            "{{\"op\": \"authentication\",\"id\":1, \"appKey\": \"{}\", \"session\": \"{}\"}}\r\n",
            self.app_key, self.ssoid
        );
        info!("{auth_msg}");
        let tcp_stream = TcpStream::connect(STREAM_API_ENDPOINT).await?;

        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));
        let domain = ServerName::try_from(STREAM_API_HOST)
            .map_err(|e| anyhow::anyhow!("Invalid DNS name: {e}"))?
            .to_owned();

        let tls_stream = connector
            .connect(domain, tcp_stream)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to establish TLS connection: {e}"))?;

        let (reader, mut writer) = tokio::io::split(tls_stream);

        // Set up channels for message passing
        let (tx_write, mut rx_write) = mpsc::channel::<String>(100);
        let (tx_read, rx_read) = mpsc::channel::<String>(100);
        self.message_sender = Some(tx_write);
        self.message_receiver = Some(rx_read);

        // Spawn writer task
        tokio::spawn(async move {
            while let Some(message) = rx_write.recv().await {
                if let Err(e) = writer.write_all(message.as_bytes()).await {
                    eprintln!("Error writing to stream: {e}");
                    break;
                }
            }
        });
        // Spawn reader task
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        line = line.strip_suffix("\r\n").unwrap_or(&line).to_string();
                        info!("Raw message: {}", line);

                        if let Err(e) = tx_read.send(line.clone()).await {
                            error!("Error sending message to main task: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error reading from stream: {}", e);
                        break;
                    }
                }
            }
        });

        // Send initial authentication message
        self.send_message(auth_msg).await?;

        self.connection_manager
            .set_state(ConnectionState::Connected)
            .await;
        info!("Successfully connected to Betfair streaming service");

        Ok(())
    }

    pub async fn send_message(&self, message: String) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            sender.send(message).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Message sender not initialized"))
        }
    }

    fn create_market_subscription_message(market_id: &str, levels: usize) -> String {
        format!(
            "{{\"op\": \"marketSubscription\", \"id\": 1, \"marketFilter\": {{ \"marketIds\":[\"{market_id}\"]}}, \"marketDataFilter\": {{ \"fields\": [\"EX_BEST_OFFERS\"], \"ladderLevels\": {levels}}}}}\r\n"
        )
    }

    pub fn create_order_subscription_message(filter_json: &str) -> String {
        format!(
            "{{\"op\":\"orderSubscription\",\"orderFilter\":{filter_json},\"segmentationEnabled\":true}}\r\n"
        )
    }

    pub async fn subscribe(&mut self, market_id: String, levels: usize) -> Result<()> {
        let sub_msg = Self::create_market_subscription_message(&market_id, levels);
        info!("Sending subscription: {}", sub_msg);

        self.send_message(sub_msg).await?;
        self.subscribed_markets.insert((market_id, levels));
        Ok(())
    }

    pub fn get_message_sender(&self) -> Option<mpsc::Sender<String>> {
        self.message_sender.clone()
    }

    pub async fn subscribe_to_orders(&mut self, filter_json: &str) -> Result<()> {
        let order_sub_msg = Self::create_order_subscription_message(filter_json);
        info!("Sending order subscription: {}", order_sub_msg);

        self.send_message(order_sub_msg).await?;
        self.subscribed_to_orders = true;
        Ok(())
    }

    async fn reconnect(&mut self) -> Result<()> {
        let attempts = self.connection_manager.get_reconnect_attempts().await;
        if attempts >= 5 {
            self.connection_manager
                .set_state(ConnectionState::Failed(format!(
                    "Failed to reconnect after {attempts} attempts"
                )))
                .await;
            return Err(anyhow::anyhow!("Max reconnection attempts exceeded"));
        }

        self.connection_manager
            .set_state(ConnectionState::Reconnecting)
            .await;
        warn!("Attempting to reconnect (attempt {})", attempts + 1);

        // Try to reconnect
        match self.connect_betfair_tls_stream().await {
            Ok(_) => {
                info!("Successfully reconnected to Betfair streaming service");

                // Resubscribe to all markets
                for (market_id, levels) in self.subscribed_markets.clone() {
                    if let Err(e) = self.subscribe(market_id, levels).await {
                        error!("Failed to resubscribe to market: {}", e);
                    }
                }

                // Resubscribe to orders if needed
                if self.subscribed_to_orders {
                    if let Err(e) = self.subscribe_to_orders("{}").await {
                        error!("Failed to resubscribe to orders: {}", e);
                    }
                }

                Ok(())
            }
            Err(e) => {
                error!("Reconnection failed: {}", e);
                Err(e)
            }
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let receiver = self.message_receiver.take();
        let Some(mut receiver) = receiver else {
            return Err(anyhow::anyhow!("Message receiver not initialized"));
        };

        // Clone necessary components for the heartbeat task
        let last_heartbeat = Arc::clone(&self.last_message_ts);
        let heartbeat_threshold = self.heartbeat_threshold;
        let is_resubscribing = Arc::clone(&self.is_resubscribing);
        let message_sender = self.message_sender.clone();
        let subscribed_markets = self.subscribed_markets.clone();

        // Spawn heartbeat monitoring task
        let heartbeat_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let elapsed = {
                    match last_heartbeat.lock() {
                        Ok(guard) => guard.elapsed(),
                        Err(e) => {
                            error!("Mutex lock poisoned: {}", e);
                            continue;
                        }
                    }
                };

                if elapsed > heartbeat_threshold {
                    let should_resubscribe = {
                        match is_resubscribing.lock() {
                            Ok(mut guard) => {
                                if !*guard {
                                    *guard = true;
                                    true
                                } else {
                                    false
                                }
                            }
                            Err(e) => {
                                error!("Mutex lock poisoned: {}", e);
                                false
                            }
                        }
                    };

                    if should_resubscribe {
                        if let Some(sender) = &message_sender {
                            // Resubscribe to all markets
                            info!("Resubscribing to {} markets", subscribed_markets.len());
                            for (market_id, levels) in &subscribed_markets {
                                let subscription_message =
                                    BetfairStreamer::create_market_subscription_message(
                                        market_id, *levels,
                                    );
                                info!("Sending subscription: {}", subscription_message);
                                if let Err(e) = sender.send(subscription_message).await {
                                    error!("Failed to send resubscription message: {}", e);
                                }
                            }
                        }

                        match is_resubscribing.lock() {
                            Ok(mut guard) => {
                                *guard = false;
                            }
                            Err(e) => {
                                error!("Mutex lock poisoned: {}", e);
                            }
                        }
                    }
                }
            }
        });

        loop {
            match receiver.recv().await {
                Some(message) => {
                    if let Err(e) = self.handle_message(message).await {
                        error!("Error handling message: {}", e);
                        // Continue processing other messages even if one fails
                    }
                }
                None => {
                    // Channel closed, indicating disconnection
                    warn!("Message channel closed, connection lost");
                    self.connection_manager
                        .set_state(ConnectionState::Disconnected)
                        .await;

                    // Attempt reconnection
                    match self.reconnect().await {
                        Ok(_) => {
                            info!("Reconnection successful, restarting message processing");
                            // Get new receiver after reconnection
                            if let Some(new_receiver) = self.message_receiver.take() {
                                receiver = new_receiver;
                                continue;
                            } else {
                                error!("Failed to get new receiver after reconnection");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to reconnect: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        heartbeat_handle.abort();

        Ok(())
    }

    async fn handle_message(&mut self, message: String) -> Result<()> {
        let parsed_message: Value = serde_json::from_str(&message)?;
        if let Some(op) = parsed_message.get("op").and_then(Value::as_str) {
            match op {
                "mcm" => {
                    if let Ok(market_change_message) =
                        serde_json::from_str::<MarketChangeMessage>(&message.to_string())
                    {
                        info!(
                            "MCM received: {} markets, {} changes",
                            market_change_message.market_changes.len(),
                            market_change_message
                                .market_changes
                                .iter()
                                .map(|mc| mc.runner_changes.as_ref().map_or(0, |rc| rc.len()))
                                .sum::<usize>()
                        );
                        debug!("MarketChangeMessage details: {:?}", &market_change_message);
                        self.parse_market_change_message(market_change_message);
                    } else if let Ok(heartbeat_message) =
                        serde_json::from_str::<HeartbeatMessage>(&message.to_string())
                    {
                        info!("Heartbeat received (id: {})", heartbeat_message.id);
                        debug!("HeartbeatMessage details: {:?}", heartbeat_message);
                    } else {
                        info!("Unknown MCM message: {}", parsed_message);
                    }
                    if let Ok(mut ts) = self.last_message_ts.lock() {
                        *ts = Instant::now();
                    }
                }
                "ocm" => {
                    if let Ok(order_change_message) =
                        serde_json::from_str::<OrderChangeMessage>(&message.to_string())
                    {
                        info!(
                            "OCM received: {} order changes",
                            order_change_message.order_changes.len()
                        );
                        debug!("OrderChangeMessage details: {:?}", &order_change_message);
                        self.parse_order_change_message(order_change_message);
                    } else if let Ok(heartbeat_message) =
                        serde_json::from_str::<HeartbeatMessage>(&message.to_string())
                    {
                        info!("Heartbeat received (id: {})", heartbeat_message.id);
                        debug!("HeartbeatMessage details: {:?}", &heartbeat_message);
                    } else {
                        info!("Unknown OCM message: {}", parsed_message);
                    }
                    if let Ok(mut ts) = self.last_message_ts.lock() {
                        *ts = Instant::now();
                    }
                }
                "heartbeat" => {
                    info!("Standalone heartbeat message received");
                    if let Ok(mut ts) = self.last_message_ts.lock() {
                        *ts = Instant::now();
                    }
                }
                "status" => {
                    // Parse status message for authentication response
                    let status_code = parsed_message.get("statusCode").and_then(Value::as_str);
                    let error_message = parsed_message.get("errorMessage").and_then(Value::as_str);
                    let connection_id = parsed_message.get("connectionId").and_then(Value::as_str);

                    match status_code {
                        Some("SUCCESS") => {
                            info!(
                                "Authentication successful - Connection ID: {:?}",
                                connection_id
                            );
                        }
                        Some("FAILURE") => {
                            error!("Authentication failed - Error: {:?}", error_message);
                        }
                        Some(code) => {
                            warn!("Status message with code '{}': {}", code, parsed_message);
                        }
                        None => {
                            info!("Status message (no code): {}", parsed_message);
                        }
                    }

                    if let Ok(mut ts) = self.last_message_ts.lock() {
                        *ts = Instant::now();
                    }
                }
                "connection" => {
                    info!("Connection message: {}", parsed_message);
                    if let Ok(mut ts) = self.last_message_ts.lock() {
                        *ts = Instant::now();
                    }
                }
                other => {
                    info!("Unknown message type '{}': {}", other, parsed_message);
                }
            }
        } else {
            info!("Message without 'op' field: {}", parsed_message);
        }

        Ok(())
    }

    fn parse_market_change_message(&mut self, market_change_message: MarketChangeMessage) {
        info!(
            "Parsing market change message with {} market changes",
            market_change_message.market_changes.len()
        );

        for market_change in market_change_message.market_changes {
            let market_id = market_change.id;
            info!("Processing market change for market {market_id}");

            let market_orderbooks = self.orderbooks.entry(market_id.clone()).or_default();

            if let Some(runner_changes) = market_change.runner_changes {
                info!(
                    "Market {market_id} has {} runner changes",
                    runner_changes.len()
                );

                for runner_change in runner_changes {
                    let runner_id = runner_change.id.to_string();
                    debug!("Processing runner change for runner {runner_id} in market {market_id}");

                    let orderbook = market_orderbooks.entry(runner_id.clone()).or_default();
                    let mut has_bid_updates = false;
                    let mut has_ask_updates = false;

                    if let Some(batb) = runner_change.available_to_back {
                        for level in batb {
                            if level.len() >= 3 {
                                let level_index = level[0] as usize;
                                let price = level[1];
                                let size = level[2];
                                orderbook.add_bid(level_index, price, size);
                                has_bid_updates = true;
                            }
                        }
                    }

                    if let Some(batl) = runner_change.available_to_lay {
                        for level in batl {
                            if level.len() >= 3 {
                                let level_index = level[0] as usize;
                                let price = level[1];
                                let size = level[2];
                                orderbook.add_ask(level_index, price, size);
                                has_ask_updates = true;
                            }
                        }
                    }

                    if has_bid_updates || has_ask_updates {
                        debug!("Updated orderbook for runner {runner_id}: bids={has_bid_updates}, asks={has_ask_updates}");
                    }

                    orderbook.set_ts(market_change_message.pt);
                    debug!("Orderbook for runner {}:", runner_id);
                    debug!("\n{}", orderbook.pretty_print());
                }
            } else {
                debug!("Market {market_id} has no runner changes");
            }

            info!(
                "Market {market_id} now has {} runners with orderbook data",
                market_orderbooks.len()
            );

            if let Some(callback) = &self.orderbook_callback {
                info!(
                    "Invoking orderbook callback for market {market_id} with {} runners",
                    market_orderbooks.len()
                );
                let market_id_clone = market_id.clone();
                let orderbooks_clone = market_orderbooks.clone();
                let callback_clone = callback.clone();
                tokio::spawn(async move {
                    callback_clone(market_id_clone, orderbooks_clone);
                });
            } else {
                warn!("No orderbook callback set for market {market_id} - data will not be propagated");
            }
        }
    }

    fn parse_order_change_message(&mut self, order_change_message: OrderChangeMessage) {
        if let Some(callback) = &self.orderupdate_callback {
            let callback_clone = callback.clone();
            let message_clone = order_change_message.clone();
            tokio::spawn(async move {
                callback_clone(message_clone);
            });
        }
    }
}
