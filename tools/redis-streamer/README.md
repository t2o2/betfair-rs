# Redis Market Streamer

A standalone CLI tool that integrates Redis with betfair-rs library for streaming market data from Redis-stored venue games.

## Overview

This tool provides Redis integration without adding Redis as a dependency to the main betfair-rs library. It's designed as a separate binary that uses betfair-rs as a path dependency while maintaining its own Redis-specific functionality.

## Features

- **Stream venue markets**: Stream specific venue game markets from Redis
- **Stream all markets**: Stream all available markets found in Redis
- **List venues**: Discover available venue games in Redis
- **Real-time orderbook**: Live market data with configurable depth
- **Statistics tracking**: Monitor update frequencies and connection health

## Installation

```bash
# From the tools/redis-streamer directory
cargo build --release

# Or run directly
cargo run -- --help
```

## Usage

### Stream a specific venue game
```bash
# Stream markets from venue game ID 34750237
cargo run -- stream-venue 34750237 --depth 10 --interval 2

# With custom Redis URL
cargo run -- stream-venue 34750237 --redis-url "redis://localhost:6380"
```

### Stream all available markets
```bash
# Stream all markets found in Redis
cargo run -- stream-all --depth 5 --interval 3

# Limit to first 5 markets
cargo run -- stream-all --limit 5 --depth 10
```

### List available venue games
```bash
# Discover what venue games are available
cargo run -- list-venues

# With custom Redis URL
cargo run -- list-venues --redis-url "redis://your-redis-server:6379"
```

## Command Line Options

### Global Options
- `--redis-url`: Redis connection URL (default: `redis://127.0.0.1:6379`)

### Stream Options
- `--depth, -d`: Orderbook depth (default: 10)
- `--interval, -i`: Update interval in seconds (default: 2)
- `--limit, -l`: Limit number of markets to stream (stream-all only)

## Redis Data Format

The tool expects Redis keys in the format:
```
venue_game:betfair:{venue_game_id}:match_odds:{market_key}
```

Each key should contain JSON data with a `market_id` field accessible via:
```
JSON.GET key $.market_id
```

## Configuration

The tool uses the same betfair configuration as the main library. Ensure you have a `config.toml` file in the project root:

```toml
[betfair]
username = "your_username"
password = "your_password"
api_key = "your_api_key"
pem_path = "/path/to/client.pem"
```

## Output Format

The tool provides real-time streaming output with:

- **Connection status**: Live/Slow/Stale indicators
- **Update statistics**: Message counts and timing
- **Market summaries**: Selection count and status
- **Orderbook data**: Bid/ask ladder with prices and sizes
- **Spread information**: Best bid/ask spread and mid-price

## Architecture

- **Separate binary**: Independent from main library
- **Path dependency**: Uses local betfair-rs library
- **Redis isolation**: Redis dependency only in this tool
- **Async streaming**: Non-blocking real-time market data
- **Statistics tracking**: Performance monitoring built-in

## Dependencies

- `betfair-rs`: Path dependency to main library
- `redis`: Redis client functionality
- `tokio`: Async runtime
- `clap`: Command-line argument parsing
- `tracing`: Structured logging
- `serde_json`: JSON parsing for Redis data

## Development

To modify or extend the tool:

1. Edit source in `src/main.rs`
2. Add dependencies to `Cargo.toml`
3. Build with `cargo build`
4. Test with `cargo run -- <command>`

The tool maintains complete separation from the main library while providing full Redis integration capabilities.