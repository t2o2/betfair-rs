# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust library for interacting with the Betfair Exchange API, providing trading capabilities, real-time market data streaming, and order management. The library uses async/await patterns with Tokio and includes rate limiting, retry logic, and comprehensive error handling.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run tests
cargo test                    # Run all tests
cargo test --lib             # Run library tests only
cargo test test_name         # Run specific test

# Code quality
cargo clippy                 # Run linter
cargo fmt                    # Format code
cargo fmt --check           # Check formatting without changes

# Run examples (requires config.toml with Betfair credentials)

## Dashboard (Interactive Terminal UI)
cargo run -- dashboard                                        # Launch interactive terminal dashboard
# Or after building:
./target/debug/betfair dashboard                              # Run the built binary directly
```

## Dashboard (Terminal UI)

The library includes an interactive terminal dashboard for real-time trading:

- **Interactive terminal dashboard** with real-time market data monitoring
- Split-screen layout with market browser, order book, active orders, and order entry panels
- Keyboard navigation with vim-style keybindings (hjkl, Tab to switch panels)
- Real-time order book display with bid/ask ladder
- Quick order placement and management
- Account balance and P&L tracking
- Connection status indicators
- Keyboard shortcuts:
  - `Tab`/`Shift+Tab` - Navigate between panels
  - `j/k` or `↑/↓` - Move in lists
  - `Enter` - Select/Confirm
  - `o` - Order mode
  - `c` - Cancel order
  - `r` - Refresh data
  - `?` - Help
  - `q` - Quit

## Architecture Overview

### Core Components

**Two Client Architectures:**
1. **BetfairApiClient** (`src/api_client.rs`) - Modern unified REST API client
   - Uses JSON-RPC for all API calls
   - Built-in rate limiting per endpoint type (navigation/data/transaction)
   - Retry policy with exponential backoff
   - Session token management
   - Optional MarketFilter parameters for flexible querying

2. **BetfairClient** (`src/betfair.rs`) - Legacy client with streaming support
   - WebSocket streaming for real-time market data
   - Order placement and management
   - Orderbook maintenance with callbacks
   - Heartbeat monitoring and auto-reconnection

### Key Architectural Patterns

**Rate Limiting Strategy:**
- Different limits for endpoint categories:
  - Navigation: 10 requests/second
  - Data: 20 requests/second  
  - Transaction: 5 requests/second
- Token bucket implementation with automatic replenishment

**Authentication Flow:**
- Certificate-based login using PKCS12 (.pfx) files
- Session token obtained via `/certlogin` endpoint
- Token passed in `X-Authentication` header for subsequent requests

**DTO Organization** (`src/dto/`):
- Modular structure: `market.rs`, `order.rs`, `account.rs`, `streaming.rs`
- Uses `#[serde(rename_all = "camelCase")]` for API compatibility
- `MarketFilter` with `#[derive(Default)]` for optional filtering

**Streaming Architecture:**
- Persistent WebSocket connection to `stream-api.betfair.com`
- Orderbook state management with configurable depth
- Callback-based event handling for market changes and order updates
- Automatic reconnection on connection loss

## Configuration Setup

Create `config.toml` in project root:
```toml
[betfair]
username = "your_username"
password = "your_password"  
api_key = "your_api_key"
pfx_path = "/absolute/path/to/client.pfx"
pfx_password = "certificate_password"
```

Certificate conversion (from Betfair-provided files):
```bash
openssl pkcs12 -export -out client.pfx -inkey client.key -in client.crt
```

## API Design Principles

1. **Optional Filtering Pattern**: Methods accept `Option<MarketFilter>` for flexibility
   - `None` returns unfiltered results
   - `Some(filter)` applies specific constraints

2. **Error Handling**: Uses `anyhow::Result` throughout for consistent error propagation

3. **Async-First**: All API calls are async using Tokio runtime

4. **Rate Limit Awareness**: Automatic throttling based on Betfair's limits

## Testing Approach

- Unit tests for core logic (rate limiter, retry policy)
- Integration tests require valid Betfair credentials
- Examples serve as integration tests and usage documentation
- Use `cargo test --lib` to run tests without credentials

## Dependencies Note

Uses reqwest 0.9 (older version) for HTTP client - be aware of API differences from modern reqwest when making changes.