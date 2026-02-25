---
name: kal-order
description: Use when working with authenticated order commands (create, amend, cancel, list, get, cancel-all) and order lifecycle operations.
version: 1.0.0
---

# Kal Order Skill

## Identity
You are helping the user execute `kal order` workflows.

## Mission
Place and manage orders safely with explicit parameters and verifiable outputs.

## Auth Requirement
`kal order` commands require credentials (`kal config setup`, env vars, or CLI key/secret flags).

## Command Map
- `kal order create <MARKET_TICKER> --side <yes|no> --action <buy|sell> --count N --type <market|limit> [--price N] [--tif <ioc|fok|gtc>]`
- `kal order amend <ORDER_ID> [--price N] [--count N]`
- `kal order cancel <ORDER_ID>`
- `kal order cancel-all [--ticker <MARKET_TICKER>]`
- `kal order list [--ticker <MARKET_TICKER>] [--status <resting|executed|canceled>]`
- `kal order get <ORDER_ID>`

## Workflow
1. Verify auth and environment (`prod` vs `demo`).
2. Validate ticker, side/action, size, and price units.
3. Execute create/amend/cancel.
4. Confirm with `order get` or `order list -o json`.

## Safety Patterns
- Use `demo` env first for new flows.
- Prefer explicit `--type` and `--tif`.
- After mutation commands, fetch the order state for confirmation.
