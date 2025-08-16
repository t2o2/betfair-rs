# Betfair in Rust

[![Unit Tests](https://github.com/t2o2/betfair-rs/actions/workflows/unit_tests.yml/badge.svg)](https://github.com/t2o2/betfair-rs/actions/workflows/unit_tests.yml)

A high-performance Rust library for interacting with the Betfair Exchange API, featuring real-time market data streaming, order management, and an interactive terminal dashboard for trading.

## Setup

Need to convert the cert and key files that were uploaded to betfair to pkcs12 format

```bash
openssl pkcs12 -export -out client.pfx -inkey client.key -in client.crt
```

Configuration are expected to be set in a `config.toml` file containing the following.

```toml
[betfair]
username = ""
password = ""
api_key = ""
pfx_path = "[absolute path]"
pfx_password = ""
```

## Quick Start

### Running the Interactive Dashboard

The library includes a powerful interactive terminal dashboard for real-time trading:

```bash
# Using cargo run
cargo run -- dashboard

# Or after building
./target/debug/betfair dashboard
```

The dashboard provides:
- **Real-time market browser** with live price updates
- **Interactive orderbook** with bid/ask ladder visualization
- **Active order management** with one-click cancellation
- **Quick order placement** with keyboard shortcuts
- **Account balance tracking** and P&L monitoring
- **Vim-style navigation** (hjkl, Tab to switch panels)

### Key Bindings

- `Tab`/`Shift+Tab` - Navigate between panels
- `j/k` or `↑/↓` - Move up/down in lists
- `h/l` or `←/→` - Navigate horizontally (in order entry)
- `Enter` - Select market/confirm order
- `o` - Enter order placement mode
- `c` - Cancel selected order
- `r` - Refresh market data
- `?` - Show help
- `q` - Quit application

## Architecture

The library provides two client architectures:

### Modern Unified API Client (`BetfairApiClient`)
- JSON-RPC based REST API client
- Built-in rate limiting per endpoint type
- Automatic retry with exponential backoff
- Session token management
- Flexible market filtering with optional parameters

### Legacy Client with Streaming (`BetfairClient`)
- WebSocket streaming for real-time market data
- Order placement and management
- Orderbook maintenance with callbacks
- Heartbeat monitoring and auto-reconnection

## Features

### Streaming Capabilities

The library provides real-time market data streaming through Betfair's streaming API with non-blocking architecture:

```rust
use betfair_rs::{config, betfair, model};
use std::collections::HashMap;

// Initialize the client
let config = config::Config::new()?;
let mut betfair_client = betfair::BetfairClient::new(config);
betfair_client.login().await?;

// Set up orderbook callback
betfair_client.set_orderbook_callback(|market_id, orderbooks| {
    println!("Market ID: {}", market_id);
    for (runner_id, orderbook) in orderbooks {
        println!("Runner ID: {}", runner_id);
        println!("{}", orderbook.pretty_print());
    }
});

// Connect and subscribe to markets
betfair_client.connect().await?;
betfair_client.subscribe_to_markets(vec!["1.241529489".to_string()], 3).await?;
betfair_client.start_listening().await?;
```

The streaming implementation includes:
- Non-blocking architecture for real-time updates
- Real-time orderbook updates with price levels
- Automatic heartbeat monitoring and reconnection
- Support for multiple market subscriptions
- Configurable orderbook depth (1-10 levels)
- Thread-safe orderbook state management
- Direct subscription management for improved performance

### Order Management

The library provides comprehensive order management capabilities:

#### Order Placement and Cancellation

```rust
use betfair_rs::{config, betfair, order::OrderSide};

// Initialize the client
let config = config::Config::new()?;
let mut betfair_client = betfair::BetfairClient::new(config);
betfair_client.login().await?;

// Place a back order
let market_id = "1.240634817".to_string();
let runner_id = 39674645;
let side = OrderSide::Back;
let price = 10.0;
let size = 1.0;

let order = betfair_rs::order::Order::new(market_id.clone(), runner_id, side, price, size);
let order_response = betfair_client.place_order(order).await?;

// Cancel the order if it was placed successfully
if let Some(bet_id) = order_response.instruction_reports[0].bet_id.clone() {
    let cancel_response = betfair_client.cancel_order(market_id, bet_id).await?;
    println!("Order canceled: {:?}", cancel_response);
}
```

#### Order Streaming and Updates

```rust
use betfair_rs::{config, betfair, msg_model::OrderChangeMessage};

// Set up order update callback
betfair_client.set_orderupdate_callback(|order_change: OrderChangeMessage| {
    // Handle order updates including:
    // - Initial order state
    // - Order status changes
    // - Matched/unmatched amounts
    // - Order cancellations
});
```

#### Order Reconciliation

```rust
// Get current status of orders
let bet_ids = vec!["384087398733".to_string(), "384086212322".to_string()];
let order_statuses = betfair_client.get_order_status(bet_ids).await?;

// Process order statuses including:
// - Execution status
// - Matched/remaining amounts
// - Price information
```

### Account Management

```rust
// Get account funds information
let account_funds = betfair_client.get_account_funds().await?;

// Access account details including:
// - Available balance
// - Exposure
// - Exposure limits
// - Discount rates
// - Points balance
// - Wallet information
```

## Example

The main example application is the interactive dashboard:
- `examples/dashboard.rs` - Full-featured terminal dashboard with real-time trading, market browsing, order management, and account monitoring

## Performance

The library is built for high-performance trading with:
- Asynchronous I/O using Tokio
- Rate limiting to respect API limits
- Efficient JSON parsing with Serde
- Non-blocking streaming architecture
- Minimal memory allocations in hot paths

## Testing

```bash
# Run all tests
cargo test

# Run library tests only (no credentials required)
cargo test --lib

# Run specific test
cargo test test_name
```

## License

[Add your license information here]
