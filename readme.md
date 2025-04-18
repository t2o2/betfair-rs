# Betfair in Rust

[![Unit Tests](https://github.com/t2o2/betfair-rs/actions/workflows/unit_tests.yml/badge.svg)](https://github.com/t2o2/betfair-rs/actions/workflows/unit_tests.yml)

The goal of the project is to allow user to trade with betfair with the benefit of the speed of rust and a stable simple interface to trade.

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

## Features

### Streaming Capabilities

The library provides real-time market data streaming through Betfair's streaming API. Here's how to use it:

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
- Real-time orderbook updates with price levels
- Automatic heartbeat monitoring and reconnection
- Support for multiple market subscriptions
- Configurable orderbook depth (1-10 levels)
- Thread-safe orderbook state management

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

For complete examples of all features, see the `examples/` directory:
- `streaming.rs` - Market data streaming
- `streaming_order.rs` - Order updates streaming
- `ordering.rs` - Order placement and management
- `reconcile.rs` - Order status reconciliation
- `account.rs` - Account information and funds
