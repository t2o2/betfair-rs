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

## CLI Tools (Recommended)
# Unified CLI with hierarchical filtering
cargo run --example betfair -- list_sports                             # List all sports
cargo run --example betfair -- list_competitions -s 1                  # List competitions for Soccer
cargo run --example betfair -- list_events -s 1 -c 10932509          # List events for Premier League
cargo run --example betfair -- list_markets -s 1 -e 34433119         # List markets for specific event
cargo run --example betfair -- list_runners -m 1.241472080        # List runners/selections for specific market
cargo run --example betfair -- get_odds -m 1.241472080            # Get odds/prices for specific market
cargo run --example betfair -- get_account                        # Get account funds and details

# Ordering commands
cargo run --example betfair -- place_order -m 1.241472080 -r 123456 -s BACK -p 2.5 -a 10  # Place a back bet
cargo run --example betfair -- place_order -m 1.241472080 -r 123456 -s LAY -p 3.0 -a 5 --persistence PERSIST  # Place a persistent lay bet
cargo run --example betfair -- list_orders                        # List all current orders
cargo run --example betfair -- list_orders -m 1.241472080         # List orders for specific market
cargo run --example betfair -- cancel_order -m 1.241472080 -b 12345678  # Cancel specific order

# Alternative CLI with standard formatting
cargo run --example cli -- list-sports
cargo run --example cli -- list-competitions -s 1
cargo run --example cli -- list-events -s 1 -c 10932509
cargo run --example cli -- list-markets -s 1 -c 10932509 -e 34433119
cargo run --example cli -- list-runners -m 1.241472080
cargo run --example cli -- get-odds -m 1.241472080
cargo run --example cli -- get-account

# Ordering commands (standard formatting)
cargo run --example cli -- place-order -m 1.241472080 -r 123456 -s BACK -p 2.5 -a 10
cargo run --example cli -- place-order -m 1.241472080 -r 123456 -s LAY -p 3.0 -a 5 --persistence PERSIST
cargo run --example cli -- list-orders
cargo run --example cli -- list-orders -m 1.241472080
cargo run --example cli -- cancel-order -m 1.241472080 -b 12345678

## Other Examples
cargo run --example streaming
cargo run --example ordering
cargo run --example unified_api_example
cargo run --example account
```

## CLI Tools

The library includes two comprehensive CLI tools for browsing Betfair data hierarchically:

### betfair CLI (Recommended)
- Enhanced UX with emojis and helpful hints
- Hierarchical data browsing from sports → competitions → events → markets → odds/runners
- Parameter-based filtering at each level
- Commands: `list_sports`, `list_competitions`, `list_events`, `list_markets`, `get_odds`, `list_runners`, `get_account`
- Order management: `place_order`, `cancel_order`, `list_orders`

### cli Tool
- Standard formatting without emojis
- Same hierarchical browsing capabilities
- Commands use kebab-case: `list-sports`, `list-competitions`, `list-events`, `list-markets`, `get-odds`, `list-runners`, `get-account`
- Order management: `place-order`, `cancel-order`, `list-orders`

Both CLIs support:
- Required sport ID for all filtered queries
- Optional competition ID for events and markets
- Optional event ID for markets
- Market ID for retrieving live odds/prices and runner details
- Display of back/lay prices with available liquidity
- Runner names, handicaps, and metadata for each selection
- Account information including funds, balances, and user details
- **Order Management Features:**
  - Place BACK and LAY orders with customizable price and stake
  - Set order persistence (LAPSE or PERSIST)
  - List all current orders or filter by market
  - Cancel orders by bet ID
  - View order status, matched amounts, and timestamps
- Automatic sorting and pagination of results
- Context-aware hints for next steps

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
- commands in list commands
- remember get_account