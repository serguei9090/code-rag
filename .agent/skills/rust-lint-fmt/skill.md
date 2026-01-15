---
name: Rust Format + Clippy Orchestrator
description: >
  Agent-run Rust quality gate for formatting and clippy. No watcher scripts.
  After edits, the agent runs cargo fmt and cargo clippy with the correct flags,
  fixes issues, and re-runs until clean.
version: 1.0
dependencies: []
---

# Rust Format + Clippy Orchestrator (Skill)

## Mission
After any Rust code change, keep the codebase:
- **formatted** (rustfmt)
- **lint-clean** (clippy with warnings as errors)

This skill does **not** run a file watcher script.
It instructs the agent exactly **which commands to run**, in which order, and how to react to failures.

---

## Trigger
Use this skill whenever:
- any `*.rs` file changes
- `Cargo.toml` or `Cargo.lock` changes
- feature flags / workspace config changes

---

## Non-Negotiables
- Always run **fmt before clippy**
- Clippy warnings must fail the run (`-D warnings`)
- If fmt modifies files, re-run clippy
- Do not ignore errors; fix them or explicitly justify rare suppressions

---

## Operating Modes

### FAST (default during development)
Use when iterating quickly.
Commands:
```sh
cargo fmt
cargo clippy -- -D warnings
