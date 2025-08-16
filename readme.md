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

![Betfair Dashboard](docs/images/dashboard.png)

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

The library provides a unified client architecture:

### Core Components

- **BetfairApiClient**: REST API client for all trading operations
  - JSON-RPC based API calls
  - Built-in rate limiting per endpoint type (navigation/data/transaction)
  - Automatic retry with exponential backoff
  - Session token management
  - Flexible market filtering with optional parameters

- **StreamingClient**: Real-time WebSocket streaming client
  - Non-blocking architecture for market data updates
  - Can accept external session token from BetfairApiClient
  - Shared orderbook state management
  - Direct subscription management

## Features

### Streaming Capabilities

The library provides real-time market data streaming through Betfair's streaming API with non-blocking architecture:

```rust
use betfair_rs::{BetfairApiClient, StreamingClient, Config};
use std::time::Duration;

// Initialize configuration and API client
let config = Config::new()?;
let mut api_client = BetfairApiClient::new(config.clone());

// Login and get session token
api_client.login().await?;
let session_token = api_client.get_session_token()
    .ok_or_else(|| anyhow::anyhow!("Failed to get session token"))?;

// Create streaming client with session token
let mut streaming_client = StreamingClient::with_session_token(
    config.betfair.api_key.clone(),
    session_token,
);

// Start streaming connection
streaming_client.start().await?;

// Subscribe to market with orderbook depth
let market_id = "1.244922596";
streaming_client.subscribe_to_market(market_id.to_string(), 10).await?;

// Monitor orderbook updates in a loop
let orderbooks = streaming_client.get_orderbooks();
loop {
    if let Ok(books) = orderbooks.read() {
        if let Some(market_books) = books.get(market_id) {
            for (selection_id, orderbook) in market_books {
                // Access bid/ask price levels
                if let (Some(best_bid), Some(best_ask)) = 
                    (orderbook.get_best_bid(), orderbook.get_best_ask()) {
                    println!("Selection {}: Bid {:.2} @ {:.2}, Ask {:.2} @ {:.2}", 
                        selection_id, 
                        best_bid.size, best_bid.price,
                        best_ask.size, best_ask.price);
                }
            }
        }
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

The streaming implementation includes:
- Non-blocking architecture for real-time updates
- Real-time orderbook updates with bid/ask price levels
- Automatic heartbeat monitoring and reconnection
- Support for multiple market subscriptions
- Configurable orderbook depth (1-10 levels)
- Thread-safe orderbook state management via Arc<RwLock>
- Direct subscription management for improved performance

### Order Management

The library provides comprehensive order management capabilities:

#### Order Placement and Cancellation

```rust
use betfair_rs::{Config, BetfairApiClient};
use betfair_rs::dto::order::*;
use betfair_rs::dto::common::Side;

// Initialize the API client
let config = Config::new()?;
let mut client = BetfairApiClient::new(config);
client.login().await?;

// Place a back order
let market_id = "1.240634817";
let instruction = PlaceInstruction {
    order_type: OrderType::Limit,
    selection_id: 39674645,
    side: Side::Back,
    limit_order: Some(LimitOrder {
        size: 10.0,
        price: 2.5,
        persistence_type: PersistenceType::Lapse,
        ..Default::default()
    }),
    ..Default::default()
};

let request = PlaceOrdersRequest {
    market_id: market_id.to_string(),
    instructions: vec![instruction],
    ..Default::default()
};

let order_response = client.place_orders(request).await?;

// Cancel the order if it was placed successfully
if let Some(report) = order_response.instruction_reports.and_then(|r| r.first()) {
    if let Some(bet_id) = &report.bet_id {
        let cancel_request = CancelOrdersRequest {
            market_id: Some(market_id.to_string()),
            instructions: Some(vec![CancelInstruction {
                bet_id: bet_id.clone(),
                size_reduction: None,
            }]),
            ..Default::default()
        };
        let cancel_response = client.cancel_orders(cancel_request).await?;
        println!("Order canceled: {:?}", cancel_response);
    }
}
```

#### List Current Orders

```rust
// Get current orders
let request = ListCurrentOrdersRequest {
    bet_ids: None,
    market_ids: None,
    order_projection: Some(OrderProjection::All),
    ..Default::default()
};

let current_orders = client.list_current_orders(request).await?;

// Process order information including:
// - Execution status
// - Matched/remaining amounts
// - Price information
```

### Account Management

```rust
use betfair_rs::dto::account::GetAccountFundsRequest;

// Get account funds information
let request = GetAccountFundsRequest {
    wallet: None,
};
let account_funds = client.get_account_funds(request).await?;

// Access account details including:
// - Available balance
// - Exposure
// - Exposure limits
// - Discount rates
// - Points balance
// - Wallet information

// Get account details
let account_details = client.get_account_details().await?;
```

## Examples

- `examples/dashboard.rs` - Full-featured terminal dashboard with real-time trading, market browsing, order management, and account monitoring
- `examples/streaming_orderbook.rs` - Simple streaming orderbook viewer that displays real-time bid/ask data for a specific market

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

MIT License - see [LICENSE](LICENSE) file for details
