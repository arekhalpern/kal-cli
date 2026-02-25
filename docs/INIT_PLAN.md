# Plan: Build `kalshi-cli` (`kal`) in Rust

## Context

Build a robust, publishable Rust CLI for the Kalshi prediction market API — designed for both humans and coding agents. Modeled after [Polymarket CLI](https://github.com/Polymarket/polymarket-cli) (also Rust/Clap). Reimplements the Kalshi REST + WebSocket client from scratch in Rust, using `~/projects/kalshi-service/` TypeScript implementation as the definitive reference for endpoints, auth signing, and types.

Binary name: **`kal`**

---

## Project Structure

```
~/projects/kalshi-cli/
├── Cargo.toml
├── src/
│   ├── main.rs                     # Clap CLI definition, Commands enum, dispatch
│   ├── client.rs                   # Kalshi REST API client (reqwest + RSA-PSS auth)
│   ├── websocket.rs                # WebSocket client (tokio-tungstenite)
│   ├── auth.rs                     # RSA-PSS SHA256 request signing
│   ├── config.rs                   # Config resolution: CLI > env > file (~/.config/kalshi-cli/)
│   ├── types.rs                    # Kalshi API types (serde structs)
│   ├── error.rs                    # Error types (thiserror)
│   ├── shell.rs                    # Interactive REPL (rustyline)
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── markets.rs              # markets list|get|search|orderbook
│   │   ├── events.rs               # events list|get
│   │   ├── order.rs                # order create|cancel|amend|list|cancel-all
│   │   ├── portfolio.rs            # portfolio balance|positions|fills|settlements
│   │   ├── trades.rs               # trades list
│   │   ├── exchange.rs             # exchange status|schedule|announcements
│   │   ├── watch.rs                # watch ticker|orderbook|trades (WebSocket)
│   │   └── config_cmd.rs           # config setup|show|path|reset
│   └── output/
│       ├── mod.rs                  # OutputFormat enum, print dispatcher
│       ├── markets.rs              # Market table formatters
│       ├── events.rs               # Event formatters
│       ├── orders.rs               # Order formatters
│       ├── portfolio.rs            # Balance/positions/fills formatters
│       ├── trades.rs               # Trade formatters
│       └── exchange.rs             # Exchange status formatters
├── .env.example
└── README.md
```

---

## Dependencies (Cargo.toml)

```toml
[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }
# HTTP
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
# WebSocket
tokio-tungstenite = { version = "0.24", features = ["rustls-tls-native-roots"] }
# Async
tokio = { version = "1", features = ["full"] }
futures-util = "0.3"
# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# Crypto (RSA-PSS signing)
rsa = "0.9"
sha2 = "0.10"
base64 = "0.22"
pkcs8 = { version = "0.10", features = ["pem"] }
# Output
tabled = "0.17"
# Config
dirs = "6"
# Shell
rustyline = "15"
# Error handling
thiserror = "2"
anyhow = "1"
# Time
chrono = { version = "0.4", features = ["serde"] }
```

---

## Command Reference

### Global Flags
```
-o, --output <format>    Output format: table (default) | json
-e, --env <environment>  Environment: prod (default) | demo
--api-key <key>          Override API key
--api-secret <path>      Override API secret (PEM file path or raw)
-h, --help               Show help
-V, --version            Show version
```

### `kal markets`
```
kal markets list [--status open|closed|settled] [--event <ticker>] [--limit N]
kal markets get <TICKER>
kal markets search <query>
kal markets orderbook <TICKER> [--depth N]
```

### `kal events`
```
kal events list [--status open|closed|settled] [--series <ticker>] [--with-markets]
kal events get <TICKER> [--with-markets]
```

### `kal order` (requires auth)
```
kal order create <TICKER> --side yes|no --action buy|sell --count N --price N [--type limit|market] [--tif gtc|fok|ioc]
kal order cancel <ORDER_ID>
kal order cancel-all [--ticker <TICKER>]
kal order amend <ORDER_ID> [--price N] [--count N]
kal order list [--ticker <TICKER>] [--status resting|executed|canceled]
kal order get <ORDER_ID>
```

### `kal portfolio` (requires auth)
```
kal portfolio balance
kal portfolio positions [--ticker <T>] [--event <T>] [--settled|--unsettled]
kal portfolio fills [--ticker <T>] [--limit N]
kal portfolio settlements [--ticker <T>] [--limit N]
```

### `kal trades`
```
kal trades list [--ticker <TICKER>] [--limit N]
```

### `kal exchange`
```
kal exchange status
kal exchange schedule
kal exchange announcements
```

### `kal watch` (WebSocket streaming)
```
kal watch ticker <TICKER> [--tickers T1,T2,T3]
kal watch orderbook <TICKER>
kal watch trades <TICKER>
```
In JSON mode, streams NDJSON (one JSON object per line) — ideal for piping to agents.

### `kal config`
```
kal config setup          # Interactive wizard
kal config show           # Show current config (secrets masked)
kal config path           # Print config file path
kal config reset          # Remove config (with confirmation)
```

### `kal shell`
```
kal shell
kal> markets list --status open --limit 5
kal> portfolio balance
kal> exit
```

---

## Key Design Decisions

### 1. Dual Output (Agent-Friendly)
Following Polymarket CLI pattern:
- **`table`** (default): Human-readable tables via `tabled` crate. Prices as `65¢`, volumes abbreviated.
- **`json`**: Machine-readable JSON to stdout. Errors to stderr. WebSocket streams NDJSON.

### 2. Config Resolution (Priority Order)
1. CLI flags (`--api-key`, `--api-secret`)
2. Environment variables (`KALSHI_API_KEY`, `KALSHI_API_SECRET`, `KALSHI_ENV`)
3. Config file (`~/.config/kalshi-cli/config.json`)
4. None (allowed for public endpoints, error for authenticated)

Config file (0o600 permissions):
```json
{
  "api_key": "abc123",
  "api_secret_path": "/path/to/key.pem",
  "environment": "prod"
}
```

### 3. Auth: RSA-PSS SHA256
Port the signing from `kalshi-service/src/utils/auth.ts`:
```
message = timestamp_ms + METHOD + /trade-api/v2/path (no query params)
signature = RSA-PSS.sign(SHA256, message, private_key, salt_len=digest)
headers: KALSHI-ACCESS-KEY, KALSHI-ACCESS-TIMESTAMP, KALSHI-ACCESS-SIGNATURE
```

### 4. REST Client
`reqwest` with:
- RSA-PSS auth headers on authenticated endpoints
- Retry on 429/503 with exponential backoff + Retry-After
- Query parameter builder (skip None values)
- Pagination helpers (auto-cursor iteration)

### 5. WebSocket Client
`tokio-tungstenite` with:
- Auth headers on connect
- Subscribe/unsubscribe commands
- Typed message dispatch (ticker, orderbook, trade, fill, etc.)
- Auto-reconnect with exponential backoff
- Ping/heartbeat keepalive

---

## Implementation Reference (from kalshi-service TypeScript)

These TypeScript files are the definitive reference for porting to Rust:

| Reference File | What to Port |
|---|---|
| `kalshi-service/src/types/index.ts` | All API types → Rust serde structs |
| `kalshi-service/src/utils/auth.ts` | RSA-PSS signing → `rsa` + `sha2` crates |
| `kalshi-service/src/client/KalshiClient.ts` | REST endpoints, retry logic, query builder |
| `kalshi-service/src/client/KalshiWebSocket.ts` | WS connect, subscribe, message dispatch |
| `kalshi-service/src/utils/formatting.ts` | Price/currency/date formatting helpers |

### Key API Details (from TypeScript source)
- **Base URLs**: prod `https://api.elections.kalshi.com/trade-api/v2`, demo `https://demo-api.kalshi.co/trade-api/v2`
- **WS URLs**: prod `wss://api.elections.kalshi.com/trade-api/ws/v2`, demo `wss://demo-api.kalshi.co/trade-api/ws/v2`
- **Prices**: Integer cents (1-99), yes_price + no_price = 100
- **Signing path**: Must include `/trade-api/v2` prefix, must NOT include query params
- **Public endpoints** (no auth): markets, events, trades, exchange, candlesticks
- **Private endpoints** (auth required): portfolio/*, orders/*

---

## Implementation Order

### Phase 0: Setup
1. Install Rust via rustup
2. `cargo new kalshi-cli` at `~/projects/kalshi-cli/`
3. Add all dependencies to `Cargo.toml`
4. Verify `cargo check` passes

### Phase 1: Core Infrastructure
5. `src/error.rs` — Define `KalshiError` enum (API errors, auth errors, config errors)
6. `src/types.rs` — Port all types from `kalshi-service/src/types/index.ts` as serde structs
7. `src/auth.rs` — RSA-PSS signing (port from `kalshi-service/src/utils/auth.ts`)
8. `src/config.rs` — Config file management + multi-source resolution
9. `src/client.rs` — REST client with auth, retry, pagination

### Phase 2: Output Layer
10. `src/output/mod.rs` — `OutputFormat` enum, `print_table`/`print_json` dispatcher
11. `src/output/markets.rs` — Market/orderbook table formatters
12. `src/output/exchange.rs` — Exchange status formatters

### Phase 3: Read-Only Commands
13. `src/main.rs` — Clap CLI skeleton with global flags + Commands enum
14. `src/commands/exchange.rs` — `kal exchange status|schedule|announcements`
15. `src/commands/markets.rs` — `kal markets list|get|search|orderbook`
16. `src/commands/events.rs` — `kal events list|get`
17. `src/commands/trades.rs` — `kal trades list`
18. `src/commands/config_cmd.rs` — `kal config setup|show|path|reset`

### Phase 4: Authenticated Commands
19. `src/output/portfolio.rs` — Balance/positions/fills formatters
20. `src/output/orders.rs` — Order formatters
21. `src/commands/portfolio.rs` — `kal portfolio balance|positions|fills|settlements`
22. `src/commands/order.rs` — `kal order create|cancel|amend|list|get|cancel-all`

### Phase 5: WebSocket + Shell
23. `src/websocket.rs` — WebSocket client with auth, subscribe, reconnect
24. `src/commands/watch.rs` — `kal watch ticker|orderbook|trades`
25. `src/shell.rs` — Interactive REPL with rustyline

### Phase 6: Polish
26. README.md
27. `.env.example`
28. `cargo build --release` and test the binary

---

## Verification

1. `cargo check` — Compiles without errors
2. `cargo run -- --help` — Shows all commands and global flags
3. `cargo run -- exchange status` — Works without auth
4. `cargo run -- markets list --limit 5` — Table output
5. `cargo run -- markets list --limit 5 -o json` — JSON output
6. `cargo run -- config setup` — Interactive API key setup
7. `cargo run -- portfolio balance` — After config, shows balance
8. `cargo run -- markets orderbook <TICKER>` — Shows bid/ask levels
9. `cargo run -- watch ticker <TICKER> -o json` — Streams NDJSON
10. `cargo run -- shell` — Opens REPL, accepts commands
11. `cargo build --release` — Optimized binary at `target/release/kal`
