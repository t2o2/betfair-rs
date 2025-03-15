use native_tls::TlsConnector;
use native_tls::TlsStream;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpStream;
use tracing::info;
use anyhow::Result;
use tokio::sync::mpsc;
use tokio::io::split;

const STREAM_API_ENDPOINT: &str = "stream-api.betfair.com:443";
const STREAM_API_HOST: &str = "stream-api.betfair.com";

type MessageCallback = Box<dyn Fn(String) + Send + 'static>;

pub struct BetfairStreamer {
    app_key: String,
    ssoid: String,
    stream: Option<TlsStream<TcpStream>>,
    message_sender: Option<mpsc::UnboundedSender<String>>,
    callback: Option<MessageCallback>,
}

impl BetfairStreamer {
    pub fn new(app_key: String, ssoid: String) -> Self {
        Self { 
            app_key, 
            ssoid, 
            stream: None,
            message_sender: None,
            callback: None,
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

        let connector = TlsConnector::new().unwrap();

        let tcp_stream = TcpStream::connect(STREAM_API_ENDPOINT)?;
        let mut tls_stream = connector.connect(STREAM_API_HOST, tcp_stream)?;
        tls_stream.write_all(auth_msg.as_bytes())?;
        self.stream = Some(tls_stream);
        
        Ok(())
    }

    pub async fn on_message(&mut self, message: String) {
        println!("Received message: {}", message);
    }

    pub async fn subscribe(&mut self, market_id: String) -> Result<()> {
        let sub_msg = format!(
            "{{\"op\":\"marketSubscription\",\"id\":1,\"marketFilter\":{{\"marketIds\":[\"{}\"]}}}}\r\n",
            &market_id
        );
        info!("Sending subscription: {}", sub_msg);
        
        if let Some(sender) = &self.message_sender {
            sender.send(sub_msg)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Stream writer not initialized"))
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let stream = self.stream.take().ok_or_else(|| anyhow::anyhow!("Stream not connected"))?;
        let (tx_messages, mut rx_messages) = mpsc::unbounded_channel::<String>();
        let (tx_outgoing, mut rx_outgoing) = mpsc::unbounded_channel::<String>();
        
        self.message_sender = Some(tx_outgoing);
        let callback = self.callback.take();

        // Spawn the main stream handling task
        tokio::spawn(async move {
            // Spawn task for handling outgoing messages
            let write_task = tokio::spawn(async move {
                while let Some(message) = rx_outgoing.recv().await {
                    if stream.write_all(message.as_bytes()).is_err() {
                        break;
                    }
                }
            });

            // Handle incoming messages
            let read_task = tokio::spawn(async move {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) => {
                            let message = line.strip_suffix("\r\n").unwrap_or(&line).to_string();
                            if tx_messages.send(message).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            info!("Error reading from stream: {}", e);
                            break;
                        }
                    }
                }
            });

            // Wait for either task to complete
            tokio::select! {
                _ = write_task => info!("Write task completed"),
                _ = read_task => info!("Read task completed"),
            }
        });

        // Handle received messages with callback
        if let Some(callback) = callback {
            tokio::spawn(async move {
                while let Some(message) = rx_messages.recv().await {
                    callback(message);
                }
            });
        }

        Ok(())
    }
}