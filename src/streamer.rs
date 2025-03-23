use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
use tokio::net::TcpStream;
use tokio_native_tls::native_tls::TlsConnector;
use tracing::info;
use anyhow::Result;
use tokio::sync::mpsc;
use crate::model::MarketChangeMessage;
use crate::model::HeartbeatMessage;
const STREAM_API_ENDPOINT: &str = "stream-api.betfair.com:443";
const STREAM_API_HOST: &str = "stream-api.betfair.com";

type MessageCallback = Box<dyn Fn(String) + Send + 'static>;

pub struct BetfairStreamer {
    app_key: String,
    ssoid: String,
    callback: Option<MessageCallback>,
    message_sender: Option<mpsc::Sender<String>>,
}

impl BetfairStreamer {
    pub fn new(app_key: String, ssoid: String) -> Self {
        Self { 
            app_key, 
            ssoid, 
            callback: None,
            message_sender: None,
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
                        info!("Received message: {}", line);
                        if let Ok(market_change_message) = serde_json::from_str::<MarketChangeMessage>(&line) {
                            info!("Parsed MarketChangeMessage: {:?}", market_change_message);
                        }

                        if let Ok(heartbeat_message) = serde_json::from_str::<HeartbeatMessage>(&line) {
                            info!("Parsed HeartbeatMessage: {:?}", heartbeat_message);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from stream: {}", e);
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
        
        self.send_message(sub_msg).await
    }

    pub async fn start(&mut self) -> Result<()> {
        Ok(())
    }
}