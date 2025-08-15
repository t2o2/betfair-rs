use crate::betfair::BetfairClient;
use crate::config::Config;
use crate::orderbook::Orderbook;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tracing::{error, info};

/// A non-blocking wrapper around BetfairClient for streaming
pub struct StreamingClient {
    config: Config,
    #[allow(dead_code)]
    session_token: Option<String>,
    streaming_task: Option<JoinHandle<()>>,
    command_sender: Option<mpsc::Sender<StreamingCommand>>,
    orderbooks: Arc<RwLock<HashMap<String, HashMap<String, Orderbook>>>>,
    is_connected: Arc<RwLock<bool>>,
    last_update_times: Arc<RwLock<HashMap<String, Instant>>>, // Track update times per market
}

#[derive(Debug)]
enum StreamingCommand {
    Subscribe(String, usize), // market_id, levels
    Unsubscribe(String),
    Stop,
}

impl StreamingClient {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            session_token: None,
            streaming_task: None,
            command_sender: None,
            orderbooks: Arc::new(RwLock::new(HashMap::new())),
            is_connected: Arc::new(RwLock::new(false)),
            last_update_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a reference to the shared orderbooks
    pub fn get_orderbooks(&self) -> Arc<RwLock<HashMap<String, HashMap<String, Orderbook>>>> {
        self.orderbooks.clone()
    }

    /// Get the last update time for a market
    pub fn get_last_update_time(&self, market_id: &str) -> Option<Instant> {
        self.last_update_times.read().ok()?.get(market_id).copied()
    }

    /// Initialize and start the streaming client in a background task
    pub async fn start(&mut self) -> Result<()> {
        // Create command channel
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<StreamingCommand>(100);
        self.command_sender = Some(cmd_tx.clone());

        // Clone necessary data for the task
        let config = self.config.clone();
        let orderbooks = self.orderbooks.clone();
        let is_connected = self.is_connected.clone();
        let last_update_times = self.last_update_times.clone();

        // Create a oneshot channel to signal when ready
        let (ready_tx, ready_rx) = oneshot::channel();

        // Spawn the streaming task
        let handle = tokio::spawn(async move {
            // Create and initialize the client
            let mut client = BetfairClient::new(config);

            // Login
            if let Err(e) = client.login().await {
                error!("Failed to login to streaming: {}", e);
                let _ = ready_tx.send(Err(anyhow::anyhow!("Login failed: {}", e)));
                return;
            }

            info!("Streaming client logged in successfully");

            // Set up orderbook callback
            let orderbooks_ref = orderbooks.clone();
            let update_times_ref = last_update_times.clone();
            if let Err(e) = client.set_orderbook_callback(move |market_id, runner_orderbooks| {
                if let Ok(mut obs) = orderbooks_ref.write() {
                    obs.insert(market_id.clone(), runner_orderbooks);
                }
                // Track update time
                if let Ok(mut times) = update_times_ref.write() {
                    times.insert(market_id, Instant::now());
                }
            }) {
                error!("Failed to set orderbook callback: {}", e);
                let _ = ready_tx.send(Err(anyhow::anyhow!("Callback setup failed: {}", e)));
                return;
            }

            // Connect to streaming service
            if let Err(e) = client.connect().await {
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
            let message_sender = match client.get_message_sender() {
                Ok(Some(sender)) => sender,
                Ok(None) => {
                    error!("Message sender not available from client");
                    let _ = ready_tx.send(Err(anyhow::anyhow!("Message sender not available")));
                    return;
                }
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
                            info!("Processing subscription for market {} with {} levels", market_id, levels);
                            // Send subscription message directly through the message channel
                            let sub_msg = format!(
                                "{{\"op\": \"marketSubscription\", \"id\": 1, \"marketFilter\": {{ \"marketIds\":[\"{market_id}\"]}}, \"marketDataFilter\": {{ \"fields\": [\"EX_BEST_OFFERS\"], \"ladderLevels\": {levels}}}}}\r\n"
                            );
                            info!("Sending subscription: {}", sub_msg);
                            if let Err(e) = message_sender.send(sub_msg).await {
                                error!("Failed to send subscription: {}", e);
                            } else {
                                info!("Successfully sent subscription for market {}", market_id);
                            }
                        }
                        StreamingCommand::Unsubscribe(_market_id) => {
                            info!("Unsubscribe not yet implemented");
                        }
                        StreamingCommand::Stop => {
                            info!("Stopping streaming client");
                            break;
                        }
                    }
                }
            });

            // Start listening (this blocks)
            let listen_result = client.start_listening().await;
            
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
        if let Some(sender) = &self.command_sender {
            sender
                .send(StreamingCommand::Subscribe(market_id, levels))
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
}

impl Drop for StreamingClient {
    fn drop(&mut self) {
        if let Some(handle) = self.streaming_task.take() {
            handle.abort();
        }
    }
}