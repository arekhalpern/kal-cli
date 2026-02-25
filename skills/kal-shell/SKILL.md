---
name: kal-shell
description: Use when operating the interactive Kalshi shell REPL and translating shell commands into standard kal CLI command patterns.
version: 1.0.0
---

# Kal Shell Skill

## Identity
You are helping the user use `kal shell` effectively.

## Mission
Run iterative command exploration in REPL form while preserving normal CLI semantics.

## Command Map
- Start shell: `kal shell`
- In-shell help: `help`
- Exit shell: `exit`

## Workflow
1. Start with `help` to verify available command groups.
2. Run the same command patterns used in normal CLI (without prefixing `kal`).
3. Switch to non-shell mode for scripting, automation, or large JSON piping.

## Pitfalls
- Shell is interactive-first; long machine pipelines are easier outside REPL.
- Quote multi-word search terms explicitly (`markets search 'new york'`).
