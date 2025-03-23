use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
use tokio::net::TcpStream;
use tokio_native_tls::native_tls::TlsConnector;
use tracing::{info, error, debug};
use anyhow::Result;
use tokio::sync::mpsc;
use crate::model::MarketChangeMessage;
use crate::model::HeartbeatMessage;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use serde_json::{Value, json};

const STREAM_API_ENDPOINT: &str = "stream-api.betfair.com:443";
const STREAM_API_HOST: &str = "stream-api.betfair.com";

type MessageCallback = Box<dyn Fn(String) + Send + 'static>;

pub struct BetfairStreamer {
    app_key: String,
    ssoid: String,
    callback: Option<MessageCallback>,
    message_sender: Option<mpsc::Sender<String>>,
    subscribed_markets: HashSet<String>,
    last_heartbeat: Instant,
    heartbeat_threshold: Duration,
}

impl BetfairStreamer {
    pub fn new(app_key: String, ssoid: String) -> Self {
        Self { 
            app_key, 
            ssoid, 
            callback: None,
            message_sender: None,
            subscribed_markets: HashSet::new(),
            last_heartbeat: Instant::now(),
            heartbeat_threshold: Duration::from_secs(10),
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
        self.message_sender = Some(tx_write);

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
            let mut reader = tokio::io::BufReader::new(reader); // Use tokio's BufReader
            let mut line = String::new();
            
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(n) if n == 0 => break, // EOF
                    Ok(_) => {
                        line = line.strip_suffix("\r\n").unwrap_or(&line).to_string();
                        debug!("Received message: {}", line);
                        if let Ok(market_change_message) = serde_json::from_str::<MarketChangeMessage>(&line.to_string()) {
                            debug!("Parsed MarketChangeMessage: {:?}", market_change_message);
                        }

                        if let Ok(heartbeat_message) = serde_json::from_str::<HeartbeatMessage>(&line.to_string()) {
                            debug!("Parsed HeartbeatMessage: {:?}", heartbeat_message);
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

    pub async fn subscribe(&mut self, market_id: String) -> Result<()> {
        let sub_msg = format!(
            "{{\"op\": \"marketSubscription\", \"id\": 1, \"marketFilter\": {{ \"marketIds\":[\"{}\"]}}, \"marketDataFilter\": {{ \"fields\": [\"EX_BEST_OFFERS\"], \"ladderLevels\": 3 }}}}\r\n",
            &market_id
        );
        info!("Sending subscription: {}", sub_msg);
        
        self.send_message(sub_msg).await?;
        self.subscribed_markets.insert(market_id);
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        Ok(())
    }

    async fn handle_message(&mut self, message: String) -> Result<()> {
        let message: Value = serde_json::from_str(&message)?;
        
        if let Some(op) = message.get("op").and_then(Value::as_str) {
            match op {
                "mcm" => {
                    info!("Received message: {}", message);
                    if let Ok(market_change_message) = serde_json::from_str::<MarketChangeMessage>(&message.to_string()) {
                        info!("Parsed MarketChangeMessage: {:?}", market_change_message);
                    }

                    if let Ok(heartbeat_message) = serde_json::from_str::<HeartbeatMessage>(&message.to_string()) {
                        info!("Parsed HeartbeatMessage: {:?}", heartbeat_message);
                        self.last_heartbeat = Instant::now();
                    }
                }
                _ => {}
            }
        }

        // Check if we need to resubscribe
        if self.last_heartbeat.elapsed() > self.heartbeat_threshold {
            self.resubscribe_all_markets().await?;
        }

        Ok(())
    }

    async fn resubscribe_all_markets(&mut self) -> Result<()> {
        if self.subscribed_markets.is_empty() {
            return Ok(());
        }

        info!("Resubscribing to {} markets due to heartbeat timeout", self.subscribed_markets.len());
        
        // Create a new connection if needed
        if !self.is_connected() {
            self.connect_betfair_tls_stream().await?;
        }

        // Resubscribe to all markets
        for market_id in self.subscribed_markets.clone() {
            let subscription_message = format!(
                "{{\"op\": \"marketSubscription\", \"id\": 1, \"marketFilter\": {{ \"marketIds\":[\"{}\"]}}, \"marketDataFilter\": {{ \"fields\": [\"EX_BEST_OFFERS\"], \"ladderLevels\": 3 }}}}\r\n",
                market_id
            );

            self.send_message(subscription_message).await?;
            info!("Resubscribed to market: {}", market_id);
        }

        Ok(())
    }

    fn is_connected(&self) -> bool {
        // For now, we'll assume we're not connected if we don't have a message sender
        self.message_sender.is_some()
    }
}