# Justfile for code-rag

# List available recipes
default:
    @just --list

# Format code
fmt:
    cargo fmt

# Check code style and common mistakes
clippy:
    cargo clippy -- -D warnings

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run security audit
audit:
    cargo audit

# Run full CI check locally
ci: fmt clippy test audit
    @echo "All CI checks passed!"

# Clean build artifacts
clean:
    cargo clean
