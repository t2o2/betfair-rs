use crate::config::Config;
use crate::dto::streaming::{OrderChangeMessage, OrderFilter};
use crate::order_cache::OrderCache;
use crate::orderbook::Orderbook;
use crate::streamer::BetfairStreamer;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

/// Type alias for orderbook callback function
type OrderbookCallback = Arc<dyn Fn(String, HashMap<String, Orderbook>) + Send + Sync + 'static>;
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
        F: Fn(String, HashMap<String, Orderbook>) + Send + Sync + 'static,
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

    /// Initialize and start the streaming client in a background task
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

        // Create a oneshot channel to signal when ready
        let (ready_tx, ready_rx) = oneshot::channel();

        // Spawn the streaming task
        let handle = tokio::spawn(async move {
            // Create the streamer
            let mut streamer = BetfairStreamer::new(api_key, session_token);

            info!("Streaming client initialized");

            // Set up orderbook callback
            let orderbooks_ref = orderbooks.clone();
            let update_times_ref = last_update_times.clone();
            streamer.set_orderbook_callback(move |market_id, runner_orderbooks| {
                info!("Orderbook callback triggered for market {market_id} with {} runners", runner_orderbooks.len());

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

                if let Some(ref callback) = custom_orderbook_callback {
                    debug!("Calling custom orderbook callback for market {market_id}");
                    callback(market_id, runner_orderbooks);
                }
            });

            let orders_ref = orders.clone();
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

                if let Some(ref callback) = custom_order_callback {
                    callback(order_change_message);
                }
            });

            // Connect to streaming service
            if let Err(e) = streamer.connect_betfair_tls_stream().await {
                error!("Failed to connect to streaming: {}", e);
                let _ = ready_tx.send(Err(anyhow::anyhow!("Connection failed: {}", e)));
                return;
            }

            info!("Connected to streaming service");

            // Mark as connected
            if let Ok(mut connected) = is_connected.write() {
                *connected = true;
            }

            // Get the message sender for direct message sending
            let message_sender = streamer
                .get_message_sender()
                .ok_or_else(|| anyhow::anyhow!("Message sender not available"));

            let message_sender = match message_sender {
                Ok(sender) => sender,
                Err(e) => {
                    error!("Failed to get message sender: {}", e);
                    let _ = ready_tx.send(Err(e));
                    return;
                }
            };

            // Signal that we're ready
            let _ = ready_tx.send(Ok(()));

            // Spawn command handler task
            let cmd_handle = tokio::spawn(async move {
                while let Some(cmd) = cmd_rx.recv().await {
                    match cmd {
                        StreamingCommand::Subscribe(market_id, levels) => {
                            info!(
                                "Processing subscription for market {} with {} levels",
                                market_id, levels
                            );
                            // Only clear data for this specific market, not all markets
                            if let Ok(mut obs) = orderbooks.write() {
                                obs.remove(&market_id);
                            }
                            if let Ok(mut times) = last_update_times.write() {
                                times.remove(&market_id);
                            }

                            // Send subscription message directly through the message channel
                            let sub_msg =
                                Self::create_market_subscription_message(&market_id, levels);
                            info!("Sending subscription: {}", sub_msg);
                            if let Err(e) = message_sender.send(sub_msg).await {
                                error!("Failed to send subscription: {}", e);
                            } else {
                                info!("Successfully sent subscription for market {}", market_id);
                            }
                        }
                        StreamingCommand::SubscribeBatch(market_ids, levels) => {
                            info!(
                                "Processing batch subscription for {} markets with {} levels",
                                market_ids.len(),
                                levels
                            );

                            // Clear data for all markets in the batch
                            if let Ok(mut obs) = orderbooks.write() {
                                for market_id in &market_ids {
                                    obs.remove(market_id);
                                }
                            }
                            if let Ok(mut times) = last_update_times.write() {
                                for market_id in &market_ids {
                                    times.remove(market_id);
                                }
                            }

                            // Create subscription message with multiple market IDs
                            let sub_msg =
                                Self::create_batch_market_subscription_message(&market_ids, levels);

                            info!("Sending batch subscription: {}", sub_msg);
                            if let Err(e) = message_sender.send(sub_msg).await {
                                error!("Failed to send batch subscription: {}", e);
                            } else {
                                info!(
                                    "Successfully sent batch subscription for {} markets",
                                    market_ids.len()
                                );
                            }
                        }
                        StreamingCommand::Unsubscribe(market_id) => {
                            info!("Unsubscription for market {} - skipping (Betfair handles replacement automatically)", market_id);
                            // Don't send unsubscribe - Betfair automatically replaces subscriptions
                            // Just clear the local data for this market
                            if let Ok(mut obs) = orderbooks.write() {
                                obs.remove(&market_id);
                            }
                            if let Ok(mut times) = last_update_times.write() {
                                times.remove(&market_id);
                            }
                        }
                        StreamingCommand::SubscribeOrders(filter) => {
                            info!("Processing order subscription");

                            let filter_json = if let Some(f) = filter {
                                serde_json::to_string(&f).unwrap_or_else(|_| "{}".to_string())
                            } else {
                                "{}".to_string()
                            };

                            let sub_msg = format!(
                                "{{\"op\":\"orderSubscription\",\"orderFilter\":{filter_json},\"segmentationEnabled\":true}}\r\n"
                            );

                            info!("Sending order subscription: {}", sub_msg);
                            if let Err(e) = message_sender.send(sub_msg).await {
                                error!("Failed to send order subscription: {}", e);
                            } else {
                                info!("Successfully sent order subscription");
                            }
                        }
                        StreamingCommand::Stop => {
                            info!("Stopping streaming client");
                            break;
                        }
                    }
                }
            });

            // Start listening (this blocks)
            let listen_result = streamer.start().await;

            if let Err(e) = listen_result {
                error!("Streaming listener error: {}", e);
            }

            // Stop the command handler if still running
            cmd_handle.abort();

            // Mark as disconnected
            if let Ok(mut connected) = is_connected.write() {
                *connected = false;
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
            "{{\"op\": \"marketSubscription\", \"id\": {id}, \"marketFilter\": {{ \"marketIds\":[{market_ids_json}]}}, \"marketDataFilter\": {{ \"fields\": [\"EX_BEST_OFFERS\"], \"ladderLevels\": {levels}}}}}\r\n"
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
                pfx_path: "/tmp/test.pfx".to_string(),
                pfx_password: "test_pfx_pass".to_string(),
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
                books.insert(format!("market_{}", i), HashMap::new());
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
