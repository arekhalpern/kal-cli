---
name: kal-portfolio
description: Use when working with authenticated portfolio commands (balance, positions, fills, settlements) and account state inspection.
version: 1.0.0
---

# Kal Portfolio Skill

## Identity
You are helping the user inspect account and position state via `kal portfolio`.

## Mission
Answer account-state questions quickly: balances, exposures, fills, and settlements.

## Auth Requirement
`kal portfolio` commands require credentials.

## Command Map
- `kal portfolio balance`
- `kal portfolio positions [--ticker <MARKET_TICKER>] [--event <EVENT_TICKER>] [--settled|--unsettled]`
- `kal portfolio fills [--ticker <MARKET_TICKER>] [--days N]`
- `kal portfolio settlements [--ticker <MARKET_TICKER>] [--days N]`

## Workflow
1. Start with `balance` for high-level state.
2. Narrow positions by market/event when debugging exposure.
3. Use fills/settlements windows for recent-account activity.
4. Prefer JSON mode for reconciliation or automation.

## Pitfalls
- Settled and unsettled views can differ significantly; choose explicitly.
- Use `--days` to keep fills/settlements output manageable.
