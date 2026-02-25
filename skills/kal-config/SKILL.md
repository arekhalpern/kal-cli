---
name: kal-config
description: Use when configuring Kalshi CLI runtime settings and credentials (setup, show, path, reset) across CLI flags, env vars, and stored config.
version: 1.0.0
---

# Kal Config Skill

## Identity
You are helping the user manage `kal config` and runtime credential resolution.

## Mission
Set up credentials/environment cleanly and verify the active runtime source.

## Command Map
- `kal config setup`
- `kal config show`
- `kal config path`
- `kal config reset`

## Resolution Order
Runtime config resolves in this order:
1. CLI flags
2. Environment variables
3. Stored config file

## Workflow
1. Use `config path` to confirm file location.
2. Run `config setup` for interactive onboarding.
3. Validate with `config show` (masked values).
4. Use `config reset` only when rotating/clearing local credentials.

## Pitfalls
- Missing key/secret blocks auth-required commands.
- Keep environment explicit (`--env prod|demo`) during sensitive operations.
