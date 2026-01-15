---
trigger: always_on
---

Comprehensive Rust Best Practices Guide

Version: 1.0

A comprehensive guide to modern Rust development, covering style, error handling, performance, concurrency, project organization, dependency management, documentation, testing, security, and CI.

1. General Coding Conventions and Style

Naming Conventions

Follow standard Rust naming idioms (RFC 430).

Types (Structs, Enums, Traits): UpperCamelCase (e.g., UserAccount)

Functions, Methods, Modules, Variables: snake_case (e.g., process_request)

Constants & Statics: SCREAMING_SNAKE_CASE (e.g., MAX_RETRY_COUNT)

Acronyms: Treat acronyms as words (e.g., Uuid instead of UUID).

Shadowing: Do not misspell names to avoid shadowing; use standard shadowing or raw identifiers (r#type) if necessary.

Formatting

Automate with rustfmt: Never argue about whitespace. Use cargo fmt to enforce the official Rust Style Guide (4 spaces indentation, 100 char line width).

CI Check: Ensure consistency by running cargo fmt -- --check in your CI pipeline.

Idiomatic Expressions

Expressions over Statements: Favor returning values from blocks rather than mutating variables.

// Bad
let mut x;
if condition { x = 5; } else { x = 10; }

// Good
let x = if condition { 5 } else { 10 };


Iterators: Use high-level iterator chains (map, filter, fold) over manual loops. They are clearer and often faster (due to bounds check elision).

2. Project Structure & Module Organization

Hierarchy

File System Mapping: Use the modern 2018 edition module system. Avoid mod.rs where possible.

Prefer: src/parser.rs and src/parser/utils.rs

Avoid: src/parser/mod.rs

Visibility: Keep fields and modules private by default. Use pub(crate) or pub(super) to expose internals to your own crate without polluting the public API.

Crate Layout

Library First: Put logic in src/lib.rs.

Binaries: src/main.rs should be a thin wrapper that calls the library. Additional binaries go in src/bin/.

Workspaces: For multi-crate projects, use a Cargo Workspace to share Cargo.lock and build artifacts.

3. Error Handling

The Golden Rules

Libraries: Return Result<T, E>. Never panic! in a library unless internal invariants are broken (bugs).

Applications: It is acceptable to panic or unwrap only at the very top level if strictly necessary, but graceful degradation is preferred.

Defining Errors

Libraries (thiserror): Define a custom enum for your crate's errors.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("IO failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}


Applications (anyhow): Use anyhow::Result for dynamic error handling in binaries. It handles backtraces and context easily.

fn main() -> anyhow::Result<()> {
    let config = read_config().context("Failed to load configuration")?;
    Ok(())
}


Silent Failures

Never ignore a Result. If an error is truly safe to ignore, explicitly handle it:

let _ = std::fs::remove_file("temp.txt"); // Explicitly ignored
// OR
if let Err(e) = do_cleanup() {
    log::warn!("Cleanup failed: {}", e);
}


4. Performance Optimization

Zero-Cost Abstractions

Trust the compiler. Iterators, closures, and generics usually compile down to the same assembly as hand-written loops. Do not manually optimize until you have benchmarks.

Memory Management

Stack > Heap: Prefer stack allocation.

Contiguous Data: Use Vec<T> over LinkedList<T> (better cache locality).

Smart Pointers: Use Cow<'a, T> for copy-on-write scenarios. Use Arc<T> only when thread-safety is required; otherwise, use Rc<T>.

Allocation Reuse: Re-use buffers (e.g., Vec::clear) rather than re-allocating in tight loops.

Benchmarking

Profile First: Use perf, flamegraph, or Intel VTune to find bottlenecks.

Micro-benchmarks: Use criterion for robust statistical benchmarking.

5. Concurrency & Async

Ownership & Thread Safety

Send: Safe to move to another thread.

Sync: Safe to share references across threads.

Shared State: Minimize lock contention. Prefer Message Passing (Channels) over Shared State (Mutexes) where possible.

Async/Await (Tokio)

Blocking: NEVER perform blocking I/O or heavy CPU computation inside an async function. It blocks the executor. Use task::spawn_blocking for these operations.

Runtime: Tokio is the industry standard. Use #[tokio::main] for entry points.

Cancellation: Be aware that futures can be dropped (cancelled) at any await point. Ensure your code is cancel-safe.

6. Dependency Management

Cargo.toml

SemVer: Use standard caret requirements (e.g., serde = "1.0").

Lockfiles:

Binaries: Always commit Cargo.lock to ensure reproducible builds.

Libraries: Commit Cargo.lock to ensure consistent CI runs, though it is ignored by consumers.

Features: Use features to make heavy dependencies optional. Features should be additive only.

Maintenance

Audit: Run cargo audit in CI to catch vulnerabilities.

Bloat: Use cargo tree to analyze dependency graphs. Avoid bringing in massive crates for simple utility functions.

7. Documentation & Testing

Documentation (rustdoc)

Public API: All pub items must have /// doc comments.

Examples: Include a # Examples section. Code blocks in docs are automatically tested!

/// Adds two numbers.
///
/// # Examples
/// ```
/// let sum = my_crate::add(2, 2);
/// assert_eq!(sum, 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 { a + b }


Testing Strategy

Unit Tests: Place inside the source file in a mod tests. Access private functions here.

Integration Tests: Place in tests/*.rs. These compile as separate crates and test your public API.

Property Testing: Consider proptest for complex logic validation.

8. Security & Unsafe Code

Unsafe Guidelines

Avoid if possible: 99% of Rust code should be safe.

Isolate: If you must use unsafe, wrap it in a safe abstraction within a small module.

Comment: Every unsafe block MUST have a // SAFETY: comment explaining why the operation is safe (invariants checked).

Sanitize: Use Miri (cargo +nightly miri test) to detect undefined behavior.

Linting

Clippy: Your best friend. Run cargo clippy on every commit.

Configuration: For strict projects, add #![deny(clippy::all)] or #![warn(clippy::pedantic)].

9. Code Review & CI Checklist

CI Pipeline (GitHub Actions Example)

Every PR should pass the following steps:

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - name: Format Check
        run: cargo fmt -- --check
      - name: Lint (Treat warnings as errors)
        run: cargo clippy --all-targets --all-features -- -D warnings
      - name: Test
        run: cargo test --all-features
      - name: Security Audit
        run: cargo install cargo-audit && cargo audit


Reviewer Checklist

Safety: Are unsafe blocks justified and documented?

Panics: Are there hidden unwrap() calls?

Errors: Are errors typed and propagated correctly?

Docs: Do new public items have examples?

Complexity: Can the code be simplified using standard iterators?

10. Recommended Workflow

Setup: cargo new + git init.

Dev Loop: Write code -> cargo check -> cargo test.

Refactor: Run cargo clippy to learn idiomatic improvements.

Verify: Run cargo fmt before committing.

Push: Let CI validate the build.