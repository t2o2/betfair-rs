//! # betfair-rs
//!
//! A high-performance Rust library for interacting with the Betfair Exchange API,
//! featuring real-time market data streaming, order management, and an interactive
//! terminal dashboard for trading.
//!
//! ## Quick Start
//!
//! ```no_run
//! use betfair_rs::{BetfairClient, Config};
//! use betfair_rs::dto::market::{MarketFilter, ListMarketCatalogueRequest};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Load configuration from config.toml
//! let config = Config::new()?;
//!
//! // Create API client and login
//! let mut client = BetfairClient::new(config);
//! client.login().await?;
//!
//! // List available sports (event types)
//! let sports = client.list_sports(None).await?;
//!
//! // Get markets for a specific event type
//! let filter = MarketFilter {
//!     event_type_ids: Some(vec!["1".to_string()]), // Soccer
//!     ..Default::default()
//! };
//! let request = ListMarketCatalogueRequest {
//!     filter,
//!     market_projection: None,
//!     sort: None,
//!     max_results: Some(10),
//!     locale: None,
//! };
//! let markets = client.list_market_catalogue(request).await?;
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
//! Create a `config.toml` file in your project root:
//!
//! ```toml
//! [betfair]
//! username = "your_username"
//! password = "your_password"
//! api_key = "your_api_key"
//! pem_path = "/path/to/certificate.pem"
//! ```
//!
//! ## Example: Streaming Market Data
//!
//! ```no_run
//! use betfair_rs::{BetfairClient, StreamingClient, Config};
//! use std::time::Duration;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = Config::new()?;
//! let mut api_client = BetfairClient::new(config.clone());
//! api_client.login().await?;
//!
//! let session_token = api_client.get_session_token()
//!     .ok_or_else(|| anyhow::anyhow!("No session token"))?;
//!
//! let mut streaming_client = StreamingClient::with_session_token(
//!     config.betfair.api_key.clone(),
//!     session_token
//! );
//!
//! streaming_client.start().await?;
//! streaming_client.subscribe_to_market("1.234567".to_string(), 10).await?;
//!
//! let orderbooks = streaming_client.get_orderbooks();
//! # Ok(())
//! # }
//! ```

use std::sync::Once;

pub mod account;
pub mod api_client;
pub mod config;
pub mod connection_state;
pub mod dto;
pub mod msg_model;
pub mod order;
pub mod order_cache;
pub mod orderbook;
mod public_data;
mod rate_limiter;
mod retry;
mod streamer;
pub mod streaming_client;
pub mod unified_client;

static CRYPTO_PROVIDER_INIT: Once = Once::new();

pub(crate) fn ensure_crypto_provider() {
    CRYPTO_PROVIDER_INIT.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

pub use api_client::RestClient;
pub use config::Config;
pub use streaming_client::StreamingClient;
pub use unified_client::BetfairClient;

// Type aliases for backward compatibility and ease of use
pub type BetfairApiClient = RestClient;
pub type UnifiedBetfairClient = BetfairClient;
