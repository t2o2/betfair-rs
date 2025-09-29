# betfair-rs

A Rust library for interacting with the Betfair Exchange API, providing trading capabilities, real-time market data streaming, and order management.

## Features

- **REST API Client** with built-in rate limiting and retry logic
- **WebSocket Streaming** for real-time market data and order updates
- **Interactive Terminal Dashboard** for live trading
- **Async/await** patterns with Tokio runtime
- **Comprehensive error handling** with anyhow
- **Modular architecture** with unified client design

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
betfair-rs = { git = "https://github.com/t2o2/betfair-rs" }
```

## Quick Start

### Configuration

Create a `config.toml` file:
```toml
[betfair]
username = "your_username"
password = "your_password"
api_key = "your_api_key"
pem_path = "/path/to/client.pem"  # Combined cert + private key
```

### Basic Usage

```rust
use betfair_rs::BetfairClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize client
    let client = BetfairClient::from_config_file("config.toml").await?;

    // Get markets
    let markets = client.list_market_catalogue(None).await?;

    // Stream market data
    client.subscribe_markets(vec!["1.123456".to_string()]).await?;

    Ok(())
}
```

## Dashboard

Run the interactive terminal UI for real-time trading:

```bash
cargo run -- dashboard
```

![Dashboard Screenshot](docs/images/dashboard.png)

Features:
- Real-time market data streaming
- Live orderbook with bid/ask ladder
- Order placement and management
- Account balance tracking
- Vim-style keyboard navigation

## CLI Commands

```bash
# Stream market data
cargo run -- stream 1.123456 1.789012 --depth 10

# Run examples
cargo run --example streaming_orderbook
cargo run --example interactive_login_test
```

## Architecture

- **BetfairClient**: Unified client combining REST and streaming
- **RestClient**: JSON-RPC REST API with rate limiting
- **StreamingClient**: WebSocket real-time data streaming
- **Rate Limiting**: Automatic throttling per endpoint type
- **Authentication**: Certificate-based or interactive login

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

## License

MIT License - see [LICENSE](LICENSE) file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.