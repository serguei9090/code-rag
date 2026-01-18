---
name: code-performance
description: Expert assistant for identifying Rust performance issues and violations of best practices. Use when auditing code for efficiency, optimizing hot paths, or ensuring resource-conscious design.
---

# Code Performance Expert

This skill guides you through auditing and optimizing Rust code for maximum performance.

## Optimization Workflow

1.  **Baseline & Profile**: Never optimize blindly. Use `cargo flamegraph` or `perf` to identify bottlenecks.
2.  **Analyze Hot Paths**: Focus on code that consumes the most CPU or memory.
3.  **Apply Targeted Optimizations**: Use the guidelines in [rust_performance.md](references/rust_performance.md).
4.  **Verify**: Re-benchmark to ensure the "optimization" actually improved performance.

## Core Areas of Focus

### Memory Management
- **Minimize Allocations**: Reuse buffers (`Vec::clear`, `String::clear`) instead of re-allocating in loops.
- **Stack over Heap**: Prefer stack allocation for small data. Use `Cow` to avoid unnecessary heap copies.
- **Avoid Excessive Cloning**: Audit use of `.clone()` and `.to_owned()`. Pass by reference (`&T`) whenever possible.
- **Zero-Copy**: Leverage slicing and references to avoid copying data.

### Efficient Iteration
- **Idiomatic Iterators**: Use `.map()`, `.filter()`, and `.fold()`. The compiler optimizes these (loop fusion, bounds check elision).
- **Lazy Evaluation**: Avoid `.collect()` until strictly necessary.
- **Capacity Planning**: Use `Vec::with_capacity()` when the final size is known to avoid re-allocations.

### Concurrency & Parallelism
- **CPU-Bound**: Use `Rayon` for easy parallel iterators.
- **I/O-Bound**: Use `Tokio` or `async-std`.
- **Async Safety**: Never block an async executor with long-running synchronous work. Use `spawn_blocking`.

## Reference Materials

- [Rust Performance Best Practices](references/rust_performance.md) - Deep dive into patterns and techniques.
- [Audit Checklist](references/audit_checklist.md) - A structured approach to performance reviews.
