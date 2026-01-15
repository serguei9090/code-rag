---
name: Rust Best Practices Guide
description: >
  Practical, implementation-focused Rust best practices for agents and developers:
  style, error handling, performance, concurrency, project structure, dependencies,
  documentation, testing, security, and CI.
version: 1.0
dependencies: []
---

# Rust Best Practices Guide (Skill)

## Mission
Write **idiomatic, safe, maintainable Rust** with strong defaults:
- consistent style & structure
- robust error handling
- measured performance work
- safe concurrency/async
- reliable docs/tests
- security + CI enforcement

## Scope
Use this skill when:
- writing or reviewing Rust code
- designing crate/project layout
- adding dependencies/features
- defining CI checks and quality gates
- improving safety/performance

Out of scope:
- domain/business logic decisions
- performance claims without measurement
- unsafe code unless explicitly required

---

## Non-Negotiables (Invariants)

### Formatting & Lints
- Always run `cargo fmt` (or enforce `cargo fmt -- --check` in CI).
- Always run Clippy in CI: `cargo clippy -- -D warnings`.
- Avoid introducing new warnings; treat warnings as failures.

### Error Handling
- **Recoverable errors:** return `Result<T, E>`.
- **panic!/unwrap/expect:** only for invariants, tests, prototypes, or `main` with clear messages.
- Add context to errors in apps (e.g., `anyhow::Context`).
- Never silently ignore errors.

### Safety & Security
- Prefer safe Rust. Minimize and encapsulate `unsafe`.
- Any `unsafe` must include:
  - a short justification comment
  - documented invariants (see `# Safety` docs section)
- Run dependency vulnerability checks in CI (`cargo audit` or `cargo deny`).

---

## Style & Conventions

### Naming
- Types/traits/enums/variants: `UpperCamelCase`
- Functions/modules/vars: `snake_case`
- Const/statics: `SCREAMING_SNAKE_CASE`
- Avoid all-caps acronyms in CamelCase: prefer `Uuid` over `UUID`
- If keyword: use `r#type` or append `_`

### Idiomatic Rust
- Prefer expressions over mutable temp variables where clear.
- Prefer iterators over manual loops when it improves readability.
- Comments:
  - `//` for line comments, complete sentences
  - `///` for public rustdoc
- Keep lines ~100 chars (let rustfmt handle it).

---

## Project Structure (Cargo-idiomatic)

### Default Layout
- Put most logic in `src/lib.rs` (library crate).
- Keep `src/main.rs` thin (wiring + CLI + runtime).
- Additional binaries in `src/bin/*`.

### Modules
- Follow Cargo conventions (avoid `#[path]` unless necessary).
- Split large modules into submodules.
- Prefer consistent module style:
  - either `foo.rs` + `foo/bar.rs`
  - or `foo/mod.rs` + `foo/bar.rs`
  Pick one and stick to it.

### Visibility
- Keep APIs small and intentional:
  - use `pub(crate)` for internal APIs
  - use `pub(super)` for parent-only helpers
- Don’t make items `pub` “just because”.

### Workspaces
Use a workspace when:
- multiple crates share deps/lockfile
- separate responsibilities (core, cli, server, utils)
- consistent builds across crates

---

## Error Handling Playbook

### When to use what
- **Library crate:** structured error type (enum) + `thiserror`
- **Binary/app:** `anyhow::Result<T>` + `.context(...)`

### Patterns
**Library**
- define `enum Error { ... }`
- implement via `thiserror`
- return `Result<T, Error>`

**App**
- `fn main() -> anyhow::Result<()>`
- add context at boundaries:
  - file I/O
  - network calls
  - parsing config
  - external commands

### Anti-patterns
- returning `String` as an error in libraries (prefer typed errors)
- `unwrap()` in library code
- swallowing errors (`let _ = ...`) without comment/log

---

## Performance (Do the right kind of fast)

### Defaults
- Write clear code first; trust zero-cost abstractions.
- Optimize only after measuring.

### Do
- prefer cache-friendly structures (`Vec`, slices) over `LinkedList`
- avoid unnecessary clones; pass by reference when possible
- pre-allocate when you know sizes (`Vec::with_capacity`)
- benchmark with `criterion` (micro) and profile for hotspots

### Don’t
- add `#[inline(always)]` everywhere
- micro-optimize without evidence
- cross FFI boundary in hot loops without validating impact

---

## Concurrency & Async

### Threads (sync)
- Use `std::thread::spawn` for parallel CPU-ish work.
- Use channels to reduce shared state (message passing).
- If shared state is required: `Arc<Mutex<T>>` / `Arc<RwLock<T>>`.

### Async (I/O bound)
- Use Tokio as default runtime for production async.
- Never block inside async:
  - offload with `tokio::task::spawn_blocking`
- Use async-aware primitives:
  - `tokio::sync::Mutex`, `tokio::sync::mpsc`
- Ensure spawned tasks are `Send` unless using local runtime/`LocalSet`.

### Send/Sync rules
- Don’t manually implement `Send`/`Sync` unless you truly must (unsafe).
- Prefer compiler auto-derivation via composing safe types.

---

## Dependencies & Features

### Cargo.toml rules
- Use semver constraints normally (`dep = "1.2"`).
- Commit `Cargo.lock` for binaries/apps.
- For libraries: optional (but okay for CI reproducibility).
- Set MSRV via `rust-version` if you care about it.

### Features
- Use features to make functionality optional and dependencies lightweight:
  - `serde = { version="1", optional=true }`
  - `[features] json = ["serde"]`
- Features must be additive; avoid “negative” features.
- Document features (README and/or crate docs).

### Avoid bloat
- justify new dependencies
- check tree: `cargo tree -e features`
- disable heavy default features where possible

---

## Documentation & Testing

### Rustdoc requirements (public API)
Every public item should have:
- description
- `# Examples` (doctest when possible)
- `# Errors` (if returns Result)
- `# Panics` (if it can panic)
- `# Safety` (if unsafe API)

### Testing strategy
- Unit tests: `#[cfg(test)] mod tests` near code
- Integration tests: `tests/*.rs` use public API as an external user
- Doc tests: run with `cargo test` (keep examples compiling)

Recommended tools (optional):
- CLI integration: `assert_cmd`
- property-based: `proptest`
- coverage: `cargo llvm-cov` (or tarpaulin where applicable)

---

## Security Checklist
- minimize `unsafe` (or `#![forbid(unsafe_code)]` when possible)
- run:
  - `cargo audit` (RustSec)
  - optionally `cargo deny` (licenses + bans + advisories)
- use well-reviewed crypto crates; don’t roll your own
- consider `zeroize` for secret material

---

## CI Minimum Pipeline (Copy/Paste)

### Local commands
```sh
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo audit
