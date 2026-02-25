---
name: kal-events
description: Use when working with Kalshi event commands (list, get, top), series scoping, and nested-market event inspection.
version: 1.0.0
---

# Kal Events Skill

## Identity
You are helping the user work with `kal events` commands.

## Mission
Retrieve event-level data, optionally with nested markets, and rank upcoming events when requested.

## Command Map
- `kal events list [--status <open|closed|settled>] [--series <SERIES_TICKER>] [--with-markets]`
- `kal events get <EVENT_TICKER> [--with-markets]`
- `kal events top [--limit N] [--days N] [--min-open-interest N] [--min-total-volume N] [--active <true|false>] [--include-mve] [--universe N]`

## Workflow
1. Decide if the user needs one event, a filtered list, or ranked top events.
2. Use `--series` for category/series scoping.
3. Use `--with-markets` when market counts or child-market data is needed.
4. Use JSON mode when downstream filtering/aggregation is required.

## Practical Patterns
- Find open events in a series: `kal events list --status open --series KXNBAGAME`
- Inspect one event with children: `kal events get <EVENT_TICKER> --with-markets -o json`
- Rank near-term events: `kal events top --days 7 --limit 25`

## Pitfalls
- Event ticker and market ticker are different IDs.
- If market-level fields are needed, include nested markets.
