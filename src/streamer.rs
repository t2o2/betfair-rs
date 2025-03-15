use native_tls::TlsConnector;
use native_tls::TlsStream;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpStream;
use tracing::info;
use anyhow::Result;
use tokio::sync::mpsc;

const STREAM_API_ENDPOINT: &str = "stream-api.betfair.com:443";
const STREAM_API_HOST: &str = "stream-api.betfair.com";

pub struct BetfairStreamer {
    app_key: String,
    ssoid: String,
    stream: Option<TlsStream<TcpStream>>,
}

impl BetfairStreamer {
    pub fn new(app_key: String, ssoid: String) -> Self {
        Self { 
            app_key, 
            ssoid, 
            stream: None,
        }
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
        
        if let Some(stream) = &mut self.stream {
            stream.write_all(sub_msg.as_bytes())?;
        } else {
            return Err(anyhow::anyhow!("Stream writer not initialized"));
        }
        
        Ok(())
    }

    pub async fn start_listening(&mut self) -> Result<()> {
        let stream = self.stream.take().ok_or_else(|| anyhow::anyhow!("Stream not connected"))?;
        let (sender, mut receiver) = mpsc::unbounded_channel::<String>();

        // Spawn a task to handle the stream reading
        tokio::spawn(async move {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        let message = line.strip_suffix("\r\n").unwrap_or(&line).to_string();
                        if sender.send(message).is_err() {
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

        // Handle received messages
        while let Some(message) = receiver.recv().await {
            self.on_message(message).await;
        }

        Ok(())
    }
}