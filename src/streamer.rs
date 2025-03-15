use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_native_tls::native_tls::TlsConnector;
use tokio_native_tls::TlsStream;
use tracing::info;
use anyhow::Result;
use tokio::sync::mpsc;

const STREAM_API_ENDPOINT: &str = "stream-api.betfair.com:443";
const STREAM_API_HOST: &str = "stream-api.betfair.com";

type MessageCallback = Box<dyn Fn(String) + Send + 'static>;

pub struct BetfairStreamer {
    app_key: String,
    ssoid: String,
    stream: Option<TlsStream<TcpStream>>,
    callback: Option<MessageCallback>,
    message_writer: Option<tokio::io::WriteHalf<TlsStream<TcpStream>>>,
}

impl BetfairStreamer {
    pub fn new(app_key: String, ssoid: String) -> Self {
        Self { 
            app_key, 
            ssoid, 
            stream: None,
            callback: None,
            message_writer: None,
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
        
        let (reader, writer) = tokio::io::split(tls_stream);

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
        
        if let Some(writer) = &mut self.message_writer {
            writer.write_all(sub_msg.as_bytes()).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Stream writer not initialized"))
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        Ok(())
    }
}