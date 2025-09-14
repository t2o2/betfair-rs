//! # betfair-rs
//!
//! A high-performance Rust library for interacting with the Betfair Exchange API,
//! featuring real-time market data streaming, order management, and an interactive
//! terminal dashboard for trading.
//!
//! ## Quick Start
//!
//! ```no_run
//! use betfair_rs::{BetfairApiClient, Config};
//! use betfair_rs::dto::market::MarketFilter;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Load configuration from config.toml
//! let config = Config::new()?;
//!
//! // Create API client and login
//! let mut client = BetfairApiClient::new(config);
//! client.login().await?;
//!
//! // List available event types (e.g., Soccer, Tennis, Horse Racing)
//! let event_types = client.list_event_types(None).await?;
//!
//! // Get markets for a specific event type
//! let filter = MarketFilter {
//!     event_type_ids: Some(vec!["1".to_string()]), // Soccer
//!     ..Default::default()
//! };
//! let markets = client.list_market_catalogue(Some(filter), None, 10).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **REST API Client**: Complete implementation of Betfair's JSON-RPC API
//! - **Real-time Streaming**: WebSocket streaming for live market data and orderbook updates
//! - **Order Management**: Place, cancel, and monitor orders programmatically
//! - **Rate Limiting**: Built-in rate limiting to respect API limits
//! - **Retry Logic**: Automatic retry with exponential backoff for transient failures
//! - **Terminal Dashboard**: Interactive TUI for real-time trading (binary included)
//!
//! ## Configuration
//!
//! Create a `config.toml` file with your Betfair credentials:
//!
//! ```toml
//! [betfair]
//! username = "your_username"
//! password = "your_password"
//! api_key = "your_api_key"
//! pfx_path = "/path/to/certificate.pfx"
//! pfx_password = "certificate_password"
//! ```
//!
//! ## Certificate Setup
//!
//! Convert your Betfair certificate files to PKCS#12 format:
//!
//! ```bash
//! openssl pkcs12 -export -out client.pfx -inkey client.key -in client.crt
//! ```
//!
//! ## Streaming Example
//!
//! ```no_run
//! use betfair_rs::{BetfairApiClient, StreamingClient, Config};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = Config::new()?;
//! let mut api_client = BetfairApiClient::new(config.clone());
//!
//! // Login and get session token
//! api_client.login().await?;
//! let session_token = api_client.get_session_token()
//!     .ok_or_else(|| anyhow::anyhow!("No session token"))?;
//!
//! // Create streaming client
//! let mut streaming_client = StreamingClient::with_session_token(
//!     config.betfair.api_key.clone(),
//!     session_token,
//! );
//!
//! // Start streaming and subscribe to market
//! streaming_client.start().await?;
//! streaming_client.subscribe_to_market("1.240634817".to_string(), 5).await?;
//!
//! // Access orderbook data
//! let orderbooks = streaming_client.get_orderbooks();
//! # Ok(())
//! # }
//! ```

pub mod account;
pub mod api_client;
pub mod config;
pub mod connection_state;
pub mod dto;
pub mod msg_model;
pub mod order;
pub mod orderbook;
pub mod public_data;
pub mod rate_limiter;
pub mod retry;
pub mod streamer;
pub mod streaming_client;
pub mod unified_client;

// Re-export commonly used types at the crate root
pub use api_client::BetfairApiClient;
pub use config::Config;
pub use dto::*;
pub use streaming_client::StreamingClient;
pub use unified_client::UnifiedBetfairClient;
