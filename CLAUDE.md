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

## Stream Command (Real-time Market Data)
cargo run -- stream <MARKET_IDS>                             # Stream specific market IDs to console
cargo run -- stream 1.123456 1.789012 --depth 10 --interval 2  # Stream with custom depth and interval

## Redis Tools (Separate Binary)
# Build and run Redis market streamer (requires Redis server)
cd tools/redis-streamer
cargo run -- stream-venue 34750237                           # Stream specific venue game
cargo run -- stream-all --limit 5 --daemon --interval 10    # Stream all markets with daemon mode
cargo run -- list-venues                                    # List available venue games
```

## Dashboard (Terminal UI)

The library includes an enhanced interactive terminal dashboard for real-time trading:

- **Real-time streaming integration** with live market data and order status updates
- **Interactive terminal dashboard** with comprehensive market data monitoring
- Split-screen layout with market browser, order book, active orders, and order entry panels
- Keyboard navigation with vim-style keybindings (hjkl, Tab to switch panels)
- **Live order book display** with bid/ask ladder and real-time price updates
- **Streaming order management** with live order status monitoring
- Quick order placement and management with immediate feedback
- Account balance and P&L tracking with real-time updates
- Connection status indicators for both REST and streaming connections
- **Enhanced streaming capabilities**:
  - Real-time market data streaming
  - Live order book depth updates
  - Order execution status streaming
  - Market subscription management
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

**Unified Client Architecture:**
1. **BetfairClient** (`src/unified_client.rs`) - Main client combining REST and streaming
   - Combines RestClient for REST operations
   - Integrates StreamingClient for real-time market data
   - Single login for both REST and streaming
   - Shared session token management

2. **RestClient** (`src/api_client.rs`) - REST API client
   - Uses JSON-RPC for all API calls
   - Built-in rate limiting per endpoint type (navigation/data/transaction)
   - Retry policy with exponential backoff
   - Session token management
   - Optional MarketFilter parameters for flexible querying

3. **StreamingClient** (`src/streaming_client.rs`) - Real-time streaming client
   - All streaming markets need to be subscribed in one request
   - Old subscription will be replaced by new request
   - WebSocket streaming for real-time market data
   - Non-blocking architecture
   - Can accept external session token
   - Orderbook maintenance with shared state
   - Automatic reconnection handling

### Key Architectural Patterns

**Rate Limiting Strategy:**
- Different limits for endpoint categories:
  - Navigation: 10 requests/second
  - Data: 20 requests/second  
  - Transaction: 5 requests/second
- Token bucket implementation with automatic replenishment

**Authentication Flow:**
- **Certificate-based login** (primary method): Uses PEM format (combined cert + private key)
  - Session token obtained via `/certlogin` endpoint
  - Requires valid Betfair application certificate
- **Interactive login** (alternative): Username/password authentication without certificates
  - Session token obtained via standard login endpoint
  - Useful for development and testing
  - Use `login_interactive(username, password)` method
- Token passed in `X-Authentication` header for subsequent requests

**DTO Organization** (`src/dto/`):
- Modular structure: `market.rs`, `order.rs`, `account.rs`, `streaming.rs`
- Uses `#[serde(rename_all = "camelCase")]` for API compatibility
- `MarketFilter` with `#[derive(Default)]` for optional filtering

**Streaming Architecture:**
- Persistent WebSocket connection to `stream-api.betfair.com`
- Non-blocking architecture with background task
- Orderbook state management with configurable depth
- Shared orderbook state accessible via Arc<RwLock>
- Automatic reconnection on connection loss
- Direct subscription management via command channels

## Configuration Setup

### Certificate-based Authentication (Primary)

Create `config.toml` in project root:
```toml
[betfair]
username = "your_username"
password = "your_password"
api_key = "your_api_key"
pem_path = "/absolute/path/to/client.pem"
```

Certificate conversion (from Betfair-provided files):
```bash
# Combine certificate and private key into PEM format
cat client.crt client.key > client.pem
```

### Interactive Login (Alternative)

For development and testing without certificates, use environment variables:
```bash
export BETFAIR_USERNAME="your_username"
export BETFAIR_PASSWORD="your_password"
export BETFAIR_API_KEY="your_api_key"
```

Or create a `.env` file:
```env
BETFAIR_USERNAME=your_username
BETFAIR_PASSWORD=your_password
BETFAIR_API_KEY=your_api_key
```

See `examples/interactive_login_test.rs` for usage example.

## API Operations

The library provides comprehensive access to Betfair's API operations:

### Account Operations
- **`get_account_funds()`** - Retrieve account balance and exposure
- **`get_account_details()`** - Get account profile information
- **`transfer_funds()`** - Transfer funds between wallets
- **`list_currency_rates()`** - Get current currency exchange rates (from GBP)

### Betting Operations
- **`list_event_types()`** - List available sports
- **`list_competitions()`** - List competitions for sports
- **`list_market_catalogue()`** - Get market information
- **`list_market_book()`** - Get live market prices and status
- **`place_orders()`** - Place new bets
- **`cancel_orders()`** - Cancel existing bets
- **`replace_orders()`** - Modify existing bets
- **`update_orders()`** - Update bet persistence
- **`list_current_orders()`** - Get active orders
- **`list_cleared_orders()`** - Get settled bets

### Streaming Operations
- **`subscribe_to_markets()`** - Stream real-time market data
- **`subscribe_to_orders()`** - Stream order updates
- **`get_orderbooks()`** - Access cached orderbook data

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

## Examples

The library includes several example applications demonstrating different features:

### `dashboard.rs` - Interactive Terminal Dashboard
Real-time trading interface with full market data and order management:
```bash
cargo run --example dashboard
```
- Split-screen terminal UI with market browser, orderbook, and order entry
- Vim-style keyboard navigation and shortcuts
- Real-time streaming market data integration
- Order placement and management capabilities

### `interactive_login_test.rs` - Certificate-free Authentication
Demonstrates username/password authentication without certificates:
```bash
# Set environment variables first
export BETFAIR_USERNAME="your_username"
export BETFAIR_PASSWORD="your_password"
export BETFAIR_API_KEY="your_api_key"

cargo run --example interactive_login_test
```
- Uses `login_interactive()` method
- Loads credentials from environment variables or `.env` file
- Tests basic API calls after authentication

### `currency_rates.rs` - Currency Exchange Rates
Fetches and displays current currency exchange rates from Betfair:
```bash
# Set environment variables first
export BETFAIR_USERNAME="your_username"
export BETFAIR_PASSWORD="your_password"
export BETFAIR_API_KEY="your_api_key"

cargo run --example currency_rates
```
- Demonstrates the `list_currency_rates()` API call
- Shows exchange rates from GBP to all supported currencies
- Uses interactive login (no certificate required)

### `streaming_orderbook.rs` - Real-time Market Data
Monitor live orderbook changes for specific markets:
```bash
cargo run --example streaming_orderbook
```
- WebSocket streaming of market data
- Real-time orderbook depth monitoring
- Configurable market subscription and depth

### `streaming_orders.rs` - Order Status Monitoring
Stream real-time order updates and status changes:
```bash
cargo run --example streaming_orders
```
- Real-time order execution monitoring
- Order status change notifications
- Integration with streaming client

## Tools and Extensions

### Redis Market Streamer (`tools/redis-streamer/`)

A separate binary tool that provides Redis integration without adding Redis as a library dependency:

- **Purpose**: Stream Betfair markets from Redis-stored venue game data
- **Architecture**: Independent binary with path dependency to betfair-rs
- **Features**:
  - Stream specific venue games from Redis keys
  - Stream all available markets with optional limits
  - List available venue games in Redis
  - Real-time orderbook monitoring with statistics
- **Usage**: See `tools/redis-streamer/README.md` for detailed instructions
- **Benefits**: Complete separation of Redis functionality from core library

### Design Philosophy

The tools directory allows extending functionality without bloating the core library:
- Tools have their own dependencies (e.g., Redis, additional CLI libraries)
- Library users aren't affected by tool-specific dependencies
- Tools can be distributed separately or included optionally
- Maintains clean separation of concerns

## Dependencies Note

Uses modern reqwest 0.12 for HTTP client with rustls for TLS. All async operations use tokio runtime. Structured logging implemented with the `tracing` library for better observability and debugging. File-based logging has been removed in favor of configurable structured logging output.