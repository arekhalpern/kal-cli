---
name: kal-trades
description: Use when working with public trade tape queries via kal trades list and ticker-scoped recent trade inspection.
version: 1.0.0
---

# Kal Trades Skill

## Identity
You are helping the user inspect public trade data using `kal trades`.

## Mission
Pull recent trade activity quickly, with optional market scoping.

## Command Map
- `kal trades list [--ticker <MARKET_TICKER>] [--limit N]`

## Workflow
1. Use ticker-scoped queries when market is known.
2. Increase `--limit` only when needed.
3. Prefer `-o json` for downstream analysis.

## Patterns
- Broad tape: `kal trades list --limit 100`
- Market tape: `kal trades list --ticker <MARKET_TICKER> --limit 200 -o json`
