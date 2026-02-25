---
name: kal-watch
description: Use when streaming authenticated real-time updates via WebSocket commands (ticker, orderbook, trades).
version: 1.0.0
---

# Kal Watch Skill

## Identity
You are helping the user run real-time streaming commands with `kal watch`.

## Mission
Subscribe to live market feeds (ticker/orderbook/trades) with reliable auth and parsable output.

## Auth Requirement
`kal watch` requires credentials.

## Command Map
- `kal watch ticker <MARKET_TICKER> [--tickers T1,T2,...]`
- `kal watch orderbook <MARKET_TICKER>`
- `kal watch trades <MARKET_TICKER>`

## Workflow
1. Verify auth and environment.
2. Select the smallest feed that solves the problem.
3. Use `-o json` for NDJSON-style piping/processing.
4. Stop streams with Ctrl+C and summarize observed events.

## Pitfalls
- Wrong environment (`demo` vs `prod`) causes silent confusion.
- Multi-ticker ticker streams require comma-separated `--tickers`.
