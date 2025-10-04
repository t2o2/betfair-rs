use crate::config::Config;
use crate::connection_state::{ConnectionManager, ConnectionState};
use crate::dto::streaming::{MarketDefinition, OrderChangeMessage, OrderFilter};
use crate::order_cache::OrderCache;
use crate::orderbook::Orderbook;
use crate::streamer::BetfairStreamer;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Type alias for orderbook callback function
type OrderbookCallback =
    Arc<dyn Fn(String, HashMap<String, Orderbook>, Option<MarketDefinition>) + Send + Sync + 'static>;
type OrderUpdateCallback = Arc<dyn Fn(OrderChangeMessage) + Send + Sync + 'static>;

/// A non-blocking streaming client for Betfair market data
pub struct StreamingClient {
    api_key: String,
    session_token: Option<String>,
    streaming_task: Option<JoinHandle<()>>,
    command_sender: Option<mpsc::Sender<StreamingCommand>>,
    orderbooks: Arc<RwLock<HashMap<String, HashMap<String, Orderbook>>>>,
    orders: Arc<RwLock<HashMap<String, OrderCache>>>,
    is_connected: Arc<RwLock<bool>>,
    last_update_times: Arc<RwLock<HashMap<String, Instant>>>,
    custom_orderbook_callback: Option<OrderbookCallback>,
    custom_order_callback: Option<OrderUpdateCallback>,
    connection_manager: ConnectionManager,
    subscribed_markets: Arc<RwLock<HashMap<String, usize>>>,
    subscribed_to_orders: Arc<RwLock<bool>>,
    order_filter: Arc<RwLock<Option<OrderFilter>>>,
    enable_reconnection: bool,
}

#[derive(Debug)]
enum StreamingCommand {
    Subscribe(String, usize),           // market_id, levels
    SubscribeBatch(Vec<String>, usize), // market_ids, levels
    Unsubscribe(String),
    SubscribeOrders(Option<OrderFilter>), // order subscription with optional filter
    Stop,
}

impl StreamingClient {
    /// Create a new streaming client with API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            session_token: None,
            streaming_task: None,
            command_sender: None,
            orderbooks: Arc::new(RwLock::new(HashMap::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            is_connected: Arc::new(RwLock::new(false)),
            last_update_times: Arc::new(RwLock::new(HashMap::new())),
            custom_orderbook_callback: None,
            custom_order_callback: None,
            connection_manager: ConnectionManager::new(),
            subscribed_markets: Arc::new(RwLock::new(HashMap::new())),
            subscribed_to_orders: Arc::new(RwLock::new(false)),
            order_filter: Arc::new(RwLock::new(None)),
            enable_reconnection: true,
        }
    }

    /// Create a new streaming client with API key and existing session token
    pub fn with_session_token(api_key: String, session_token: String) -> Self {
        Self {
            api_key,
            session_token: Some(session_token),
            streaming_task: None,
            command_sender: None,
            orderbooks: Arc::new(RwLock::new(HashMap::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            is_connected: Arc::new(RwLock::new(false)),
            last_update_times: Arc::new(RwLock::new(HashMap::new())),
            custom_orderbook_callback: None,
            custom_order_callback: None,
            connection_manager: ConnectionManager::new(),
            subscribed_markets: Arc::new(RwLock::new(HashMap::new())),
            subscribed_to_orders: Arc::new(RwLock::new(false)),
            order_filter: Arc::new(RwLock::new(None)),
            enable_reconnection: true,
        }
    }

    /// Create from Config for backward compatibility
    pub fn from_config(config: Config) -> Self {
        Self::new(config.betfair.api_key.clone())
    }

    /// Set or update the session token
    pub fn set_session_token(&mut self, token: String) {
        self.session_token = Some(token);
    }

    /// Enable or disable automatic reconnection
    pub fn set_reconnection_enabled(&mut self, enabled: bool) {
        self.enable_reconnection = enabled;
    }

    /// Get the current connection state
    pub async fn get_connection_state(&self) -> ConnectionState {
        self.connection_manager.get_state().await
    }

    /// Get the number of reconnection attempts
    pub async fn get_reconnect_attempts(&self) -> u32 {
        self.connection_manager.get_reconnect_attempts().await
    }

    /// Get a reference to the shared orderbooks
    pub fn get_orderbooks(&self) -> Arc<RwLock<HashMap<String, HashMap<String, Orderbook>>>> {
        self.orderbooks.clone()
    }

    /// Get the last update time for a market
    pub fn get_last_update_time(&self, market_id: &str) -> Option<Instant> {
        self.last_update_times.read().ok()?.get(market_id).copied()
    }

    /// Set a custom orderbook callback that will be called immediately when new data arrives
    pub fn set_orderbook_callback<F>(&mut self, callback: F)
    where
        F: Fn(String, HashMap<String, Orderbook>, Option<MarketDefinition>) + Send + Sync + 'static,
    {
        self.custom_orderbook_callback = Some(Arc::new(callback));
    }

    pub fn set_order_callback<F>(&mut self, callback: F)
    where
        F: Fn(OrderChangeMessage) + Send + Sync + 'static,
    {
        self.custom_order_callback = Some(Arc::new(callback));
    }

    pub fn get_orders(&self) -> Arc<RwLock<HashMap<String, OrderCache>>> {
        self.orders.clone()
    }

    /// Initialize and start the streaming client in a background task with reconnection support
    pub async fn start(&mut self) -> Result<()> {
        // Ensure we have a session token
        let session_token = self
            .session_token
            .as_ref()
            .ok_or_else(|| {
                anyhow::anyhow!("Session token not set. Call set_session_token() first.")
            })?
            .clone();

        // Create command channel
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<StreamingCommand>(100);
        self.command_sender = Some(cmd_tx.clone());

        // Clone necessary data for the task
        let api_key = self.api_key.clone();
        let orderbooks = self.orderbooks.clone();
        let orders = self.orders.clone();
        let is_connected = self.is_connected.clone();
        let last_update_times = self.last_update_times.clone();
        let custom_orderbook_callback = self.custom_orderbook_callback.clone();
        let custom_order_callback = self.custom_order_callback.clone();
        let connection_manager = self.connection_manager.clone();
        let subscribed_markets = self.subscribed_markets.clone();
        let subscribed_to_orders = self.subscribed_to_orders.clone();
        let order_filter = self.order_filter.clone();
        let enable_reconnection = self.enable_reconnection;

        // Create a oneshot channel to signal when ready (only used once on first connection)
        let (ready_tx, ready_rx) = oneshot::channel();
        let ready_tx = Arc::new(RwLock::new(Some(ready_tx)));

        // Create a shared message sender that will be updated on each connection
        let active_message_sender = Arc::new(RwLock::new(None::<mpsc::Sender<String>>));
        let active_message_sender_clone = active_message_sender.clone();

        // Spawn command handler task that works across reconnections
        let cmd_sender_ref = active_message_sender.clone();
        let subscribed_markets_ref = subscribed_markets.clone();
        let subscribed_to_orders_ref = subscribed_to_orders.clone();
        let order_filter_ref = order_filter.clone();
        let orderbooks_ref = orderbooks.clone();
        let last_update_times_ref = last_update_times.clone();

        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                let sender = {
                    if let Ok(guard) = cmd_sender_ref.read() {
                        guard.clone()
                    } else {
                        None
                    }
                };

                if sender.is_none() {
                    warn!("No active message sender, command will be queued");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                let sender = sender.unwrap();

                match cmd {
                    StreamingCommand::Subscribe(market_id, levels) => {
                        info!("Processing subscription for market {market_id} with {levels} levels");

                        if let Ok(mut markets) = subscribed_markets_ref.write() {
                            markets.insert(market_id.clone(), levels);
                        }

                        if let Ok(mut obs) = orderbooks_ref.write() {
                            obs.remove(&market_id);
                        }
                        if let Ok(mut times) = last_update_times_ref.write() {
                            times.remove(&market_id);
                        }

                        let sub_msg = Self::create_market_subscription_message(&market_id, levels);
                        if let Err(e) = sender.send(sub_msg).await {
                            error!("Failed to send subscription: {e}");
                        }
                    }
                    StreamingCommand::SubscribeBatch(market_ids, levels) => {
                        info!("Processing batch subscription for {} markets", market_ids.len());

                        if let Ok(mut markets) = subscribed_markets_ref.write() {
                            for market_id in &market_ids {
                                markets.insert(market_id.clone(), levels);
                            }
                        }

                        if let Ok(mut obs) = orderbooks_ref.write() {
                            for market_id in &market_ids {
                                obs.remove(market_id);
                            }
                        }
                        if let Ok(mut times) = last_update_times_ref.write() {
                            for market_id in &market_ids {
                                times.remove(market_id);
                            }
                        }

                        let sub_msg = Self::create_batch_market_subscription_message(&market_ids, levels);
                        if let Err(e) = sender.send(sub_msg).await {
                            error!("Failed to send batch subscription: {e}");
                        }
                    }
                    StreamingCommand::Unsubscribe(market_id) => {
                        if let Ok(mut markets) = subscribed_markets_ref.write() {
                            markets.remove(&market_id);
                        }
                        if let Ok(mut obs) = orderbooks_ref.write() {
                            obs.remove(&market_id);
                        }
                        if let Ok(mut times) = last_update_times_ref.write() {
                            times.remove(&market_id);
                        }
                    }
                    StreamingCommand::SubscribeOrders(filter) => {
                        info!("Processing order subscription");

                        if let Ok(mut subscribed) = subscribed_to_orders_ref.write() {
                            *subscribed = true;
                        }
                        if let Ok(mut filter_state) = order_filter_ref.write() {
                            *filter_state = filter.clone();
                        }

                        let filter_json = if let Some(f) = filter {
                            serde_json::to_string(&f).unwrap_or_else(|_| "{}".to_string())
                        } else {
                            "{}".to_string()
                        };

                        let sub_msg = format!(
                            "{{\"op\":\"orderSubscription\",\"orderFilter\":{filter_json},\"segmentationEnabled\":true}}\r\n"
                        );

                        if let Err(e) = sender.send(sub_msg).await {
                            error!("Failed to send order subscription: {e}");
                        }
                    }
                    StreamingCommand::Stop => {
                        info!("Stopping streaming client");
                        break;
                    }
                }
            }
        });

        // Spawn the streaming task with reconnection loop
        let handle = tokio::spawn(async move {
            let mut first_start = true;
            let mut reconnect_attempt = 0u32;

            loop {
                if !first_start && enable_reconnection {
                    connection_manager
                        .set_state(ConnectionState::Reconnecting)
                        .await;
                    reconnect_attempt += 1;

                    if reconnect_attempt > 5 {
                        error!("Max reconnection attempts exceeded");
                        connection_manager
                            .set_state(ConnectionState::Failed("Max reconnection attempts exceeded".to_string()))
                            .await;
                        break;
                    }

                    let delay = Duration::from_secs(2u64.pow(reconnect_attempt.min(5)));
                    warn!("Attempting to reconnect (attempt {reconnect_attempt}), waiting {delay:?}...");
                    tokio::time::sleep(delay).await;
                } else if !first_start {
                    warn!("Streaming disconnected and reconnection is disabled");
                    break;
                }

                connection_manager
                    .set_state(ConnectionState::Connecting)
                    .await;

                // Create the streamer
                let mut streamer = BetfairStreamer::new(api_key.clone(), session_token.clone());

                info!("Streaming client initialized");

                // Set up orderbook callback
                let orderbooks_ref = orderbooks.clone();
                let update_times_ref = last_update_times.clone();
                let callback_clone = custom_orderbook_callback.clone();
                streamer.set_orderbook_callback(move |market_id, runner_orderbooks, market_definition| {
                info!("Orderbook callback triggered for market {market_id} with {} runners", runner_orderbooks.len());

                if let Some(ref market_def) = market_definition {
                    debug!("Market {market_id} status: {:?}, inPlay: {}", market_def.status, market_def.in_play);
                }

                if let Ok(mut obs) = orderbooks_ref.write() {
                    obs.insert(market_id.clone(), runner_orderbooks.clone());
                    info!("Successfully updated shared orderbooks for market {market_id}. Total markets in shared state: {}", obs.len());
                } else {
                    error!("Failed to acquire write lock on shared orderbooks for market {market_id}");
                }

                if let Ok(mut times) = update_times_ref.write() {
                    times.insert(market_id.clone(), Instant::now());
                    debug!("Updated last update time for market {market_id}");
                } else {
                    error!("Failed to acquire write lock on update times for market {market_id}");
                }

                if let Some(ref callback) = callback_clone {
                    debug!("Calling custom orderbook callback for market {market_id}");
                    callback(market_id, runner_orderbooks, market_definition);
                }
            });

                let orders_ref = orders.clone();
                let order_callback_clone = custom_order_callback.clone();
                streamer.set_orderupdate_callback(move |order_change_message| {
                if let Ok(mut order_cache_map) = orders_ref.write() {
                    for order_change in &order_change_message.order_changes {
                        let market_id = &order_change.id;
                        let cache = order_cache_map
                            .entry(market_id.clone())
                            .or_insert_with(|| OrderCache::new(market_id.clone()));

                        cache.update_timestamp(order_change_message.pt);

                        if order_change.full_image {
                            cache.clear();
                        }

                        if let Some(ref runner_changes) = order_change.order_runner_change {
                            for runner_change in runner_changes {
                                let runner = cache.get_runner_mut(runner_change.id);
                                runner.set_handicap(runner_change.handicap);

                                if runner_change.full_image {
                                    if let Some(ref orders) = runner_change.unmatched_orders {
                                        runner.apply_full_image(orders.clone());
                                    } else {
                                        runner.orders.clear();
                                    }
                                } else if let Some(ref orders) = runner_change.unmatched_orders {
                                    for order in orders {
                                        runner.update_order(order.clone());
                                    }
                                }

                                if let Some(ref matched_backs) = runner_change.matched_backs {
                                    runner.update_matched_backs(matched_backs.clone());
                                }

                                if let Some(ref matched_lays) = runner_change.matched_lays {
                                    runner.update_matched_lays(matched_lays.clone());
                                }
                            }
                        }
                    }
                }

                if let Some(ref callback) = order_callback_clone {
                    callback(order_change_message);
                }
            });

                // Connect to streaming service
                if let Err(e) = streamer.connect_betfair_tls_stream().await {
                    error!("Failed to connect to streaming: {e}");
                    if first_start {
                        if let Ok(mut tx_opt) = ready_tx.write() {
                            if let Some(tx) = tx_opt.take() {
                                let _ = tx.send(Err(anyhow::anyhow!("Connection failed: {e}")));
                            }
                        }
                        return;
                    }
                    warn!("Connection failed, will retry if enabled: {e}");
                    continue;
                }

                info!("Connected to streaming service");

                // Mark as connected
                if let Ok(mut connected) = is_connected.write() {
                    *connected = true;
                }

                connection_manager
                    .set_state(ConnectionState::Connected)
                    .await;

                if !first_start {
                    info!("Successfully reconnected, resetting reconnect counter");
                    reconnect_attempt = 0;
                }

                // Get the message sender and store in shared state
                let message_sender = match streamer.get_message_sender() {
                    Some(sender) => sender,
                    None => {
                        error!("Message sender not available");
                        if first_start {
                            if let Ok(mut tx_opt) = ready_tx.write() {
                                if let Some(tx) = tx_opt.take() {
                                    let _ = tx.send(Err(anyhow::anyhow!("Message sender not available")));
                                }
                            }
                            return;
                        }
                        warn!("Message sender not available, will retry");
                        continue;
                    }
                };

                // Update the shared message sender for the command handler
                if let Ok(mut sender_guard) = active_message_sender_clone.write() {
                    *sender_guard = Some(message_sender.clone());
                }

                // Resubscribe to markets after reconnection
                if !first_start {
                    info!("Resubscribing to previously subscribed markets");

                    // Get market subscription data without holding the lock
                    let (market_list, levels) = {
                        if let Ok(markets) = subscribed_markets.read() {
                            if !markets.is_empty() {
                                let list: Vec<String> = markets.keys().cloned().collect();
                                let lvl = markets.values().next().copied().unwrap_or(10);
                                (list, lvl)
                            } else {
                                (vec![], 10)
                            }
                        } else {
                            (vec![], 10)
                        }
                    };

                    if !market_list.is_empty() {
                        let sub_msg = Self::create_batch_market_subscription_message(&market_list, levels);
                        if let Err(e) = message_sender.send(sub_msg).await {
                            error!("Failed to resubscribe to markets: {e}");
                        } else {
                            info!("Resubscribed to {} markets", market_list.len());
                        }
                    }

                    // Resubscribe to orders
                    let (should_subscribe, filter_json) = {
                        if let Ok(subscribed) = subscribed_to_orders.read() {
                            if *subscribed {
                                if let Ok(filter) = order_filter.read() {
                                    let json = if let Some(ref f) = *filter {
                                        serde_json::to_string(f).unwrap_or_else(|_| "{}".to_string())
                                    } else {
                                        "{}".to_string()
                                    };
                                    (true, json)
                                } else {
                                    (false, String::new())
                                }
                            } else {
                                (false, String::new())
                            }
                        } else {
                            (false, String::new())
                        }
                    };

                    if should_subscribe {
                        info!("Resubscribing to order updates");
                        let sub_msg = format!(
                            "{{\"op\":\"orderSubscription\",\"orderFilter\":{filter_json},\"segmentationEnabled\":true}}\r\n"
                        );
                        if let Err(e) = message_sender.send(sub_msg).await {
                            error!("Failed to resubscribe to orders: {e}");
                        } else {
                            info!("Resubscribed to order updates");
                        }
                    }
                }

                // Signal that we're ready (only on first start)
                if first_start {
                    if let Ok(mut tx_opt) = ready_tx.write() {
                        if let Some(tx) = tx_opt.take() {
                            let _ = tx.send(Ok(()));
                        }
                    }
                    first_start = false;
                }

                // Start listening (this blocks until disconnection)
                let listen_result = streamer.start().await;

                // Mark as disconnected
                if let Ok(mut connected) = is_connected.write() {
                    *connected = false;
                }

                connection_manager
                    .set_state(ConnectionState::Disconnected)
                    .await;

                // Clear the active message sender
                if let Ok(mut sender_guard) = active_message_sender_clone.write() {
                    *sender_guard = None;
                }

                match listen_result {
                    Ok(_) => {
                        info!("Streaming listener ended normally");
                    }
                    Err(e) => {
                        error!("Streaming listener error: {e}");
                    }
                }

                // Loop will continue for reconnection if enabled
                if !enable_reconnection {
                    info!("Reconnection disabled, exiting streaming task");
                    break;
                }

                warn!("Stream disconnected, will attempt reconnection...");
            }
        });

        self.streaming_task = Some(handle);

        // Wait for the ready signal
        match ready_rx.await {
            Ok(Ok(())) => {
                info!("Streaming client started successfully");
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Failed to start streaming client: {}", e);
                Err(e)
            }
            Err(_) => Err(anyhow::anyhow!("Failed to receive ready signal")),
        }
    }

    /// Subscribe to a market
    pub async fn subscribe_to_market(&self, market_id: String, levels: usize) -> Result<()> {
        info!("Subscribing to market {market_id} with {levels} levels");

        if let Some(sender) = &self.command_sender {
            match sender
                .send(StreamingCommand::Subscribe(market_id.clone(), levels))
                .await
            {
                Ok(_) => {
                    info!("Successfully queued subscription command for market {market_id}");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to send subscription command for market {market_id}: {e}");
                    Err(anyhow::anyhow!("Failed to send subscription command: {e}"))
                }
            }
        } else {
            error!("Cannot subscribe to market {market_id}: streaming client not started");
            Err(anyhow::anyhow!("Streaming client not started"))
        }
    }

    /// Subscribe to multiple markets in a single subscription (recommended approach)
    /// This sends all markets in one message to Betfair, avoiding subscription replacement
    pub async fn subscribe_to_markets(&self, market_ids: Vec<String>, levels: usize) -> Result<()> {
        if market_ids.is_empty() {
            return Err(anyhow::anyhow!("Cannot subscribe to empty market list"));
        }

        if let Some(sender) = &self.command_sender {
            sender
                .send(StreamingCommand::SubscribeBatch(market_ids, levels))
                .await?;
        } else {
            return Err(anyhow::anyhow!("Streaming client not started"));
        }
        Ok(())
    }

    /// Unsubscribe from a market
    pub async fn unsubscribe_from_market(&self, market_id: String) -> Result<()> {
        if let Some(sender) = &self.command_sender {
            sender
                .send(StreamingCommand::Unsubscribe(market_id))
                .await?;
        } else {
            return Err(anyhow::anyhow!("Streaming client not started"));
        }
        Ok(())
    }

    /// Subscribe to order updates
    pub async fn subscribe_to_orders(&self, filter: Option<OrderFilter>) -> Result<()> {
        if let Some(sender) = &self.command_sender {
            sender
                .send(StreamingCommand::SubscribeOrders(filter))
                .await?;
        } else {
            return Err(anyhow::anyhow!("Streaming client not started"));
        }
        Ok(())
    }

    /// Stop the streaming client
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(sender) = &self.command_sender {
            sender.send(StreamingCommand::Stop).await?;
        }

        if let Some(handle) = self.streaming_task.take() {
            handle.await?;
        }

        Ok(())
    }

    /// Check if the streaming client is connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
            .read()
            .map(|connected| *connected)
            .unwrap_or(false)
    }

    /// Create a market subscription message for a single market
    fn create_market_subscription_message(market_id: &str, levels: usize) -> String {
        // Use a timestamp-based ID to avoid conflicts
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            % 10000; // Keep it small but unique

        format!(
            "{{\"op\": \"marketSubscription\", \"id\": {id}, \"marketFilter\": {{ \"marketIds\":[\"{market_id}\"]}}, \"marketDataFilter\": {{ \"fields\": [\"EX_BEST_OFFERS\"], \"ladderLevels\": {levels}}}}}\r\n"
        )
    }

    /// Create a market subscription message for multiple markets
    fn create_batch_market_subscription_message(market_ids: &[String], levels: usize) -> String {
        // Use a timestamp-based ID to avoid conflicts
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            % 10000; // Keep it small but unique

        let market_ids_json = market_ids
            .iter()
            .map(|id| format!("\"{id}\""))
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{{\"op\": \"marketSubscription\", \"id\": {id}, \"marketFilter\": {{ \"marketIds\":[{market_ids_json}]}}, \"marketDataFilter\": {{ \"fields\": [\"EX_BEST_OFFERS\", \"EX_MARKET_DEF\"], \"ladderLevels\": {levels}}}}}\r\n"
        )
    }
}

impl Drop for StreamingClient {
    fn drop(&mut self) {
        if let Some(handle) = self.streaming_task.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BetfairConfig;
    use std::time::Duration;
    use tokio::time::sleep;

    fn create_test_config() -> Config {
        Config {
            betfair: BetfairConfig {
                username: "test_user".to_string(),
                password: "test_pass".to_string(),
                api_key: "test_api_key".to_string(),
                pem_path: "/tmp/test.pem".to_string(),
            },
        }
    }

    #[test]
    fn test_new_streaming_client() {
        let client = StreamingClient::new("test_api_key".to_string());
        assert_eq!(client.api_key, "test_api_key");
        assert!(client.session_token.is_none());
        assert!(client.streaming_task.is_none());
        assert!(client.command_sender.is_none());
        assert!(!client.is_connected());
    }

    #[test]
    fn test_with_session_token() {
        let client = StreamingClient::with_session_token(
            "test_api_key".to_string(),
            "test_token".to_string(),
        );
        assert_eq!(client.api_key, "test_api_key");
        assert_eq!(client.session_token, Some("test_token".to_string()));
        assert!(!client.is_connected());
    }

    #[test]
    fn test_from_config() {
        let config = create_test_config();
        let client = StreamingClient::from_config(config);
        assert_eq!(client.api_key, "test_api_key");
        assert!(client.session_token.is_none());
        assert!(!client.is_connected());
    }

    #[test]
    fn test_set_session_token() {
        let mut client = StreamingClient::new("test_api_key".to_string());
        assert!(client.session_token.is_none());

        client.set_session_token("new_token".to_string());
        assert_eq!(client.session_token, Some("new_token".to_string()));
    }

    #[test]
    fn test_get_orderbooks_empty() {
        let client = StreamingClient::new("test_api_key".to_string());
        let orderbooks = client.get_orderbooks();
        let books = orderbooks.read().unwrap();
        assert!(books.is_empty());
    }

    #[test]
    fn test_get_orderbooks_returns_same_reference() {
        let client = StreamingClient::new("test_api_key".to_string());
        let ob1 = client.get_orderbooks();
        let ob2 = client.get_orderbooks();
        assert!(Arc::ptr_eq(&ob1, &ob2));
    }

    #[test]
    fn test_get_last_update_time_none() {
        let client = StreamingClient::new("test_api_key".to_string());
        let time = client.get_last_update_time("1.123456");
        assert!(time.is_none());
    }

    #[test]
    fn test_get_last_update_time_with_data() {
        let client = StreamingClient::new("test_api_key".to_string());
        let now = Instant::now();
        {
            let mut times = client.last_update_times.write().unwrap();
            times.insert("1.123456".to_string(), now);
        }
        let time = client.get_last_update_time("1.123456");
        assert!(time.is_some());
        assert_eq!(time.unwrap(), now);
    }

    #[tokio::test]
    async fn test_subscribe_without_start() {
        let client = StreamingClient::new("test_api_key".to_string());
        let result = client.subscribe_to_market("1.123456".to_string(), 5).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Streaming client not started"));
    }

    #[tokio::test]
    async fn test_unsubscribe_without_start() {
        let client = StreamingClient::new("test_api_key".to_string());
        let result = client.unsubscribe_from_market("1.123456".to_string()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Streaming client not started"));
    }

    #[tokio::test]
    async fn test_stop_without_start() {
        let mut client = StreamingClient::new("test_api_key".to_string());
        let result = client.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_drop_client() {
        let mut client = StreamingClient::new("test_api_key".to_string());
        client.streaming_task = Some(tokio::spawn(async {
            loop {
                sleep(Duration::from_secs(1)).await;
            }
        }));
        drop(client);
    }

    #[test]
    fn test_is_connected_false() {
        let client = StreamingClient::new("test_api_key".to_string());
        assert!(!client.is_connected());
    }

    #[test]
    fn test_is_connected_true() {
        let client = StreamingClient::new("test_api_key".to_string());
        {
            let mut connected = client.is_connected.write().unwrap();
            *connected = true;
        }
        assert!(client.is_connected());
    }

    #[test]
    fn test_concurrent_orderbook_access() {
        use std::thread;

        let client = Arc::new(StreamingClient::new("test_api_key".to_string()));
        let orderbooks = client.get_orderbooks();

        let mut handles = vec![];
        for i in 0..10 {
            let ob_clone = Arc::clone(&orderbooks);
            let handle = thread::spawn(move || {
                let mut books = ob_clone.write().unwrap();
                books.insert(format!("market_{i}"), HashMap::new());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let books = orderbooks.read().unwrap();
        assert_eq!(books.len(), 10);
    }
}
