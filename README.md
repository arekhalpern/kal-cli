# kalshi-cli (kal)

Rust CLI for Kalshi's REST and WebSocket APIs with RSA-PSS auth.

## Features

- Human-friendly table output (`-o table`) and machine-friendly JSON (`-o json`)
- Config resolution: CLI flags > env vars > config file
- Public commands without auth (markets/events/trades/exchange)
- Auth-gated portfolio/order/watch commands with clear errors
- WebSocket streaming with NDJSON in JSON mode
- Interactive shell (`kal shell`)

## Agent Skills

Reusable agent skills live in `skills/` (one skill per command group). These are designed to be copied into any coding-agent skills directory.

- `skills/kal-markets/`
- `skills/kal-events/`
- `skills/kal-order/`
- `skills/kal-portfolio/`
- `skills/kal-trades/`
- `skills/kal-exchange/`
- `skills/kal-watch/`
- `skills/kal-config/`
- `skills/kal-shell/`

Install in your agent of choice:

```bash
# Codex
cp -R skills/* ~/.codex/skills/

# Claude Code
cp -R skills/* ~/.claude/skills/

# OpenClaw (example path)
cp -R skills/* ~/.openclaw/skills/
```

## Install

```bash
cargo build --release
./target/release/kal --help
```

## Global flags

- `-o, --output <table|json>`
- `-e, --env <prod|demo>`
- `--api-key <key>`
- `--api-secret <path_or_inline_pem>`

## Commands

### `kal markets`

- `kal markets list` - list markets with optional status/event filters (`--status`, `--active`, `--event`, `--limit`)
- `kal markets get <TICKER>` - get a single market
- `kal markets search <QUERY>` - search markets by ticker/title (`--days`, `--limit`, `--compact`)
- `kal markets top` - top upcoming markets by open interest and total volume (`--days`, `--min-open-interest`, `--min-total-volume`, `--active`, `--universe`)
- `kal markets orderbook <TICKER>` - fetch orderbook snapshot (`--depth`)

### `kal events`

- `kal events list` - list events (`--status`, `--series`, `--with-markets`)
- `kal events get <TICKER>` - get one event (`--with-markets`)
- `kal events top` - top upcoming events by aggregated open interest and total volume (`--days`, `--min-open-interest`, `--min-total-volume`, `--active`, `--universe`)

### `kal order` (auth required)

- `kal order create <TICKER>` - place order (`--side`, `--action`, `--count`, `--price`, `--type`, `--tif`)
- `kal order cancel <ORDER_ID>` - cancel one order
- `kal order cancel-all` - cancel all resting orders (`--ticker` optional scope)
- `kal order amend <ORDER_ID>` - amend order (`--price`, `--count`)
- `kal order list` - list account orders (`--ticker`, `--status`)
- `kal order get <ORDER_ID>` - fetch one order

### `kal portfolio` (auth required)

- `kal portfolio balance` - account balance summary
- `kal portfolio positions` - list positions (`--ticker`, `--event`, `--settled`, `--unsettled`)
- `kal portfolio fills` - recent fills (`--ticker`, `--days`)
- `kal portfolio settlements` - recent settlements (`--ticker`, `--days`)

### `kal trades`

- `kal trades list` - public market trades (`--ticker`, `--limit`)

### `kal exchange`

- `kal exchange status` - current exchange status
- `kal exchange schedule` - exchange trading schedule
- `kal exchange announcements` - exchange announcements

### `kal watch` (auth required, WebSocket)

- `kal watch ticker <TICKER>` - stream ticker updates (`--tickers` comma-separated for multi-market)
- `kal watch orderbook <TICKER>` - stream orderbook deltas
- `kal watch trades <TICKER>` - stream real-time trades

### `kal config`

- `kal config setup` - interactive config wizard
- `kal config show` - show current config (masked)
- `kal config path` - print config file path
- `kal config reset` - delete config with confirmation

### `kal shell`

- `kal shell` - interactive REPL for running CLI commands

## Configuration

Config path:

- Linux/macOS: `~/.config/kalshi-cli/config.json`

Stored format:

```json
{
  "api_key": "abc123",
  "api_secret_path": "/path/to/key.pem",
  "environment": "prod"
}
```

Config file permissions are set to owner-only on Unix (`0600`).

## Examples

```bash
kal exchange status
kal markets list --status open --limit 5
kal markets list --limit 5 -o json
kal config setup
kal portfolio balance
kal watch ticker KXBTC-24DEC31-B100K -o json
kal shell
```
