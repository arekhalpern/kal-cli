# kalshi-cli (kal)

Rust CLI for Kalshi's REST and WebSocket APIs with RSA-PSS auth.

## Features

- Human-friendly table output (`-o table`) and machine-friendly JSON (`-o json`)
- Config resolution: CLI flags > env vars > config file
- Public commands without auth (markets/events/trades/exchange)
- Auth-gated portfolio/order/watch commands with clear errors
- WebSocket streaming with NDJSON in JSON mode
- Interactive shell (`kal shell`)

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

- `kal markets list|get|search|orderbook`
- `kal events list|get`
- `kal order create|cancel|cancel-all|amend|list|get`
- `kal portfolio balance|positions|fills|settlements`
- `kal trades list`
- `kal exchange status|schedule|announcements`
- `kal watch ticker|orderbook|trades`
- `kal config setup|show|path|reset`
- `kal shell`

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
