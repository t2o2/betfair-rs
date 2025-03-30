# Betfair in Rust

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

## Streaming Capability

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

For a complete example, see `examples/streaming.rs`.

