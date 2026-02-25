---
name: kal-markets
description: Use when working with Kalshi market discovery commands (list, get, search, top, orderbook), including compact/table defaults, JSON output, and volume-first ranking.
version: 1.0.0
---

# Kal Markets Skill

## Identity
You are helping the user operate `kal markets` commands quickly and correctly.

## Mission
Find the right market data fast, prefer compact table output for humans and JSON output for automation.

## Command Map
- `kal markets list [--status <open|closed|settled|unopened>] [--active <true|false>] [--event <EVENT_TICKER>] [--limit N] [--compact]`
- `kal markets get <MARKET_TICKER>`
- `kal markets search <QUERY> [--days N] [--limit N] [--compact]`
- `kal markets top [--limit N] [--days N] [--min-open-interest N] [--min-total-volume N] [--active <true|false>] [--include-mve] [--universe N]`
- `kal markets orderbook <MARKET_TICKER> [--depth N]`

## Workflow
1. Confirm if the user has a market ticker, event ticker, or free-text query.
2. Pick the narrowest command that satisfies the request.
3. Use table output for quick inspection and `-o json` for filtering/scripting.
4. For search requests, rely on default volume-first ordering and adjust `--days`/`--limit` when needed.

## Practical Patterns
- Single market details: `kal markets get KX...`
- Event-scoped listing: `kal markets list --event <EVENT_TICKER> --limit 50`
- Fast text search: `kal markets search 'new york' --limit 25`
- Broader search horizon: `kal markets search 'query' --days 30 --limit 100 -o json`
- Top liquid upcoming markets: `kal markets top --days 7 --limit 25`

## Pitfalls
- `markets get` requires a market ticker, not a series ticker.
- Search can be broad; use `--days` and `--limit` to control latency/result size.
- For exact processing, use `-o json` and parse by `ticker`, `event_ticker`, `volume`, `open_interest`.
