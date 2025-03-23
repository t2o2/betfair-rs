use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
use tokio::net::TcpStream;
use tokio_native_tls::native_tls::TlsConnector;
use tracing::{info, error, debug};
use anyhow::Result;
use tokio::sync::mpsc;
use crate::msg_model::MarketChangeMessage;
use crate::msg_model::HeartbeatMessage;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use serde_json::Value;
use std::sync::{Arc, Mutex};

const STREAM_API_ENDPOINT: &str = "stream-api.betfair.com:443";
const STREAM_API_HOST: &str = "stream-api.betfair.com";

type MessageCallback = Box<dyn Fn(String) + Send + 'static>;

pub struct BetfairStreamer {
    app_key: String,
    ssoid: String,
    callback: Option<MessageCallback>,
    message_sender: Option<mpsc::Sender<String>>,
    message_receiver: Option<mpsc::Receiver<String>>,
    subscribed_markets: HashSet<String>,
    last_message_ts: Arc<Mutex<Instant>>,
    heartbeat_threshold: Duration,
    is_resubscribing: Arc<Mutex<bool>>,
}

impl BetfairStreamer {
    pub fn new(app_key: String, ssoid: String) -> Self {
        Self { 
            app_key, 
            ssoid, 
            callback: None,
            message_sender: None,
            message_receiver: None,
            subscribed_markets: HashSet::new(),
            last_message_ts: Arc::new(Mutex::new(Instant::now() + Duration::from_secs(10))),
            heartbeat_threshold: Duration::from_secs(10),
            is_resubscribing: Arc::new(Mutex::new(false)),
        }
    }

    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: Fn(String) + Send + 'static,
    {
        self.callback = Some(Box::new(callback));
    }

    pub async fn connect_betfair_tls_stream(&mut self) -> Result<()> {
        info!("TLS connect starting");

        let auth_msg = format!(
            "{{\"op\": \"authentication\",\"id\":1, \"appKey\": \"{}\", \"session\": \"{}\"}}\r\n",
            self.app_key, self.ssoid
        );
        info!("{}", auth_msg);
        let tcp_stream = TcpStream::connect(STREAM_API_ENDPOINT).await?;
        
        let connector = TlsConnector::builder()
            .build()
            .unwrap();
        let connector = tokio_native_tls::TlsConnector::from(connector);
        
        let tls_stream = connector.connect(STREAM_API_HOST, tcp_stream).await.unwrap();
        
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
                    eprintln!("Error writing to stream: {}", e);
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
                    Ok(n) if n == 0 => break, // EOF
                    Ok(_) => {
                        line = line.strip_suffix("\r\n").unwrap_or(&line).to_string();
                        debug!("Received message: {}", line);
                        
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

        Ok(())
    }

    async fn send_message(&self, message: String) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            sender.send(message).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Message sender not initialized"))
        }
    }

    fn create_subscription_message(market_id: &str) -> String {
        format!(
            "{{\"op\": \"marketSubscription\", \"id\": 1, \"marketFilter\": {{ \"marketIds\":[\"{}\"]}}, \"marketDataFilter\": {{ \"fields\": [\"EX_BEST_OFFERS\"], \"ladderLevels\": 3 }}}}\r\n",
            market_id
        )
    }

    pub async fn subscribe(&mut self, market_id: String) -> Result<()> {
        let sub_msg = Self::create_subscription_message(&market_id);
        info!("Sending subscription: {}", sub_msg);
        
        self.send_message(sub_msg).await?;
        self.subscribed_markets.insert(market_id);
        Ok(())
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
                    last_heartbeat.lock().unwrap().elapsed()
                };
                
                if elapsed > heartbeat_threshold {
                    let should_resubscribe = {
                        let mut guard = is_resubscribing.lock().unwrap();
                        if !*guard {
                            *guard = true;
                            true
                        } else {
                            false
                        }
                    };

                    if should_resubscribe {
                        if let Some(sender) = &message_sender {
                            // Resubscribe to all markets
                            info!("Resubscribing to {} markets", subscribed_markets.len());
                            for market_id in &subscribed_markets {
                                let subscription_message = BetfairStreamer::create_subscription_message(market_id);
                                info!("Sending subscription: {}", subscription_message);
                                if let Err(e) = sender.send(subscription_message).await {
                                    error!("Failed to send resubscription message: {}", e);
                                }
                            }
                        }
                        
                        let mut guard = is_resubscribing.lock().unwrap();
                        *guard = false;
                    }
                }
            }
        });

        while let Some(message) = receiver.recv().await {
            self.handle_message(message).await?;
        }

        heartbeat_handle.abort();
        
        Ok(())
    }

    async fn handle_message(&mut self, message: String) -> Result<()> {
        let message: Value = serde_json::from_str(&message)?;
        
        if let Some(op) = message.get("op").and_then(Value::as_str) {
            match op {
                "mcm" => {
                    if let Ok(market_change_message) = serde_json::from_str::<MarketChangeMessage>(&message.to_string()) {
                        info!("Parsed MarketChangeMessage: {:?}", market_change_message);
                    }
                    else if let Ok(heartbeat_message) = serde_json::from_str::<HeartbeatMessage>(&message.to_string()) {
                        info!("Parsed HeartbeatMessage: {:?}", heartbeat_message);
                    }
                    else {
                        info!("Received unknown message: {}", message);
                    }
                    *self.last_message_ts.lock().unwrap() = Instant::now();
                }
                _ => {}
            }
        }

        Ok(())
    }
}