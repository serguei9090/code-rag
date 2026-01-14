# Code-RAG Test Suite

This directory contains both **integration tests** (white-box) and **CLI tests** (black-box) for the `code-rag` project.

## Test Types

### 1. Integration Tests (White-Box)
**File**: `integration_tests.rs`

These tests directly call the Rust code to verify internal logic:
- AST parsing correctness
- Vector embedding accuracy
- Database storage integrity
- Language detection
- Semantic chunking rules

**Run with:**
```bash
cargo test --test integration_tests
cargo test --test integration_tests -- --nocapture  # With output
```

### 2. CLI Tests (Black-Box)
**Files**: `test_cli.ps1` (Windows), `test_cli.sh` (Linux/macOS)

These tests run the compiled binary as an end-user would:
- Command-line interface validation
- Output format verification
- Cross-language search accuracy
- Error handling
- Real-world usage scenarios

**Run with:**
```powershell
# Windows
.\tests\test_cli.ps1

# Linux/macOS
chmod +x ./tests/test_cli.sh
./tests/test_cli.sh
```

## Test Coverage

### Integration Tests (12 tests)
1. ✓ Index all test assets
2. ✓ Search Rust functions
3. ✓ Search Python classes
4. ✓ Search Bash scripts
5. ✓ Search PowerShell functions
6. ✓ Search JSON configurations
7. ✓ Multi-language search
8. ✓ Language detection
9. ✓ Rust file chunking
10. ✓ Python file chunking

### CLI Tests (15 tests)
1. ✓ Version command
2. ✓ Help command
3. ✓ Basic indexing
4. ✓ Force re-indexing
5. ✓ Search Rust code
6. ✓ Search Python code
7. ✓ Search Bash scripts
8. ✓ Search PowerShell scripts
9. ✓ Search JSON files
10. ✓ Search YAML files
11. ✓ Limit parameter
12. ✓ HTML report generation
13. ✓ Grep exact match
14. ✓ Grep case sensitivity
15. ✓ Multi-language verification

## Running All Tests

```bash
# Run both integration and CLI tests
cargo test && .\tests\test_cli.ps1

# Or in one command (PowerShell)
cargo test --test integration_tests; if ($?) { .\tests\test_cli.ps1 }
```

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run integration tests
        run: cargo test --test integration_tests
      - name: Build release binary
        run: cargo build --release
      - name: Run CLI tests
        run: ./tests/test_cli.sh
```

## Test Database

Both test suites use temporary databases:
- Integration tests: `./.lancedb-test`
- CLI tests: `./.lancedb-blackbox-test`

These are automatically cleaned up after each test run.

## Requirements

- Rust toolchain (for integration tests)
- All test assets in `test_assets/`
- Internet connection (first run only, downloads embedding model)
- ~500MB disk space for model cache

## Troubleshooting

**Integration tests fail to compile:**
```bash
cargo clean
cargo test --test integration_tests
```

**CLI tests can't find binary:**
```bash
cargo build --release
.\tests\test_cli.ps1 -BinaryPath ".\target\release\code-rag.exe"
```

**Model download fails:**
- Check internet connection
- Ensure `~/.cache/fastembed` has write permissions
- Try running once manually: `cargo run -- index ./test_assets`
