#!/bin/bash
set -e

# Suppress debug/trace logs from dependencies (tantivy, tokenizers, etc.)
export RUST_LOG=error

# ==========================================
#    CODE-RAG UNIFIED TEST RUNNER (Linux)
# ==========================================

echo "=========================================="
echo "   CODE-RAG UNIFIED TEST RUNNER"
echo "=========================================="

# 1. Build Release Binary
echo ""
echo "[1/4] Building Release Binary..."
cargo build --release

# 2. Run Standard Tests (Unit, Integration, Doc)
echo ""
echo "[2/4] Running Standard Tests..."
# Excludes expensive/long-running tests by default (unless --ignored is passed)
cargo test --workspace --exclude code-rag-benchmarks

# 3. Run Performance/Advanced Tests
echo ""
echo "[3/4] Running Advanced Verification..."

echo " -> Performance Test (Indexing/Search Benchmarks)..."
cargo test --test performance

echo " -> JSON Contract Test (CLI Output Cleanliness)..."
cargo test --test cli_json_test

# Scale and Stress tests are excluded from CI (run manually)
# Scale test: cargo test --test scale_test -- --ignored (10k files, ~8 minutes)
# Stress test: cargo test --test stress_test -- --ignored (concurrent load)

# 4. Smoke Test (Manual Verification Simulation)
echo ""
echo "[4/4] Running Smoke Test (Multi-Workspace Startup)..."

# Create a temporary config for smoke testing
CONFIG_FILE="smoke_test_config.toml"
cat <<EOF > $CONFIG_FILE
db_path = ".lancedb_smoke_test"
default_index_path = "."
enable_server = true
enable_mcp = true
enable_watch = true

[workspaces.ProjectA]
path = "./test_assets/ProjectA"

[workspaces.ProjectB]
path = "./test_assets/ProjectB"
EOF

# Launch standard bin in background
./target/release/code-rag --config $CONFIG_FILE start &
PID=$!

echo "   Process launched with PID $PID. Waiting 5s to verify stability..."
sleep 5

if ps -p $PID > /dev/null; then
    echo "   ✅ SUCCESS: Process is still running."
    kill $PID
    rm $CONFIG_FILE
    rm -rf .lancedb_smoke_test
    echo ""
    echo "=========================================="
    echo "   ALL TESTS PASSED SUCCESSFULLY"
    echo "=========================================="
    exit 0
else
    echo "   ❌ FAILURE: Process died prematurely."
    rm $CONFIG_FILE
    rm -rf .lancedb_smoke_test
    exit 1
fi
