#!/bin/bash
# Black-Box CLI Test Suite for code-rag (Linux/macOS)
# Tests the compiled binary through all commands and validates outputs

set -e

BINARY_PATH="${1:-./target/release/code-rag}"
TEST_DB_PATH="./.lancedb-blackbox-test"
TEST_ASSETS="./test_assets"
TESTS_PASSED=0
TESTS_FAILED=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Helper functions
write_success() { echo -e "${GREEN}✓ $1${NC}"; }
write_failure() { echo -e "${RED}✗ $1${NC}"; }
write_info() { echo -e "${CYAN}→ $1${NC}"; }
write_section() { echo -e "\n${YELLOW}=== $1 ===${NC}"; }

cleanup_test_db() {
    rm -rf "$TEST_DB_PATH" 2>/dev/null || true
}

assert_success() {
    local test_name="$1"
    local condition="$2"
    local error_msg="${3:-Test failed}"
    
    if [ "$condition" = "true" ]; then
        write_success "$test_name"
        ((TESTS_PASSED++))
    else
        write_failure "$test_name - $error_msg"
        ((TESTS_FAILED++))
    fi
}

# Pre-flight checks
write_section "Pre-flight Checks"
if [ ! -f "$BINARY_PATH" ]; then
    write_failure "Binary not found at $BINARY_PATH"
    write_info "Please run: cargo build --release"
    exit 1
fi
write_success "Binary found at $BINARY_PATH"

if [ ! -d "$TEST_ASSETS" ]; then
    write_failure "Test assets not found at $TEST_ASSETS"
    exit 1
fi
write_success "Test assets found at $TEST_ASSETS"

cleanup_test_db
write_success "Test database cleaned"

# Test 1: Version Command
write_section "Test 1: Version Command"
if version_output=$("$BINARY_PATH" --version 2>&1); then
    assert_success "Version command executes" "true"
    if echo "$version_output" | grep -q "code-rag"; then
        assert_success "Version output contains 'code-rag'" "true"
    else
        assert_success "Version output contains 'code-rag'" "false" "Output: $version_output"
    fi
else
    assert_success "Version command executes" "false" "Exit code: $?"
fi

# Test 2: Help Command
write_section "Test 2: Help Command"
if help_output=$("$BINARY_PATH" --help 2>&1); then
    assert_success "Help command executes" "true"
    echo "$help_output" | grep -q "index" && assert_success "Help contains 'index' subcommand" "true" || assert_success "Help contains 'index' subcommand" "false"
    echo "$help_output" | grep -q "search" && assert_success "Help contains 'search' subcommand" "true" || assert_success "Help contains 'search' subcommand" "false"
    echo "$help_output" | grep -q "grep" && assert_success "Help contains 'grep' subcommand" "true" || assert_success "Help contains 'grep' subcommand" "false"
else
    assert_success "Help command executes" "false"
fi

# Test 3: Index Command
write_section "Test 3: Index Command (Basic)"
write_info "Indexing test assets..."
if index_output=$("$BINARY_PATH" index "$TEST_ASSETS" --db-path "$TEST_DB_PATH" 2>&1); then
    assert_success "Index command executes" "true"
    [ -d "$TEST_DB_PATH" ] && assert_success "Database directory created" "true" || assert_success "Database directory created" "false"
    echo "$index_output" | grep -q "chunks" && assert_success "Index output mentions chunks" "true" || assert_success "Index output mentions chunks" "false"
else
    assert_success "Index command executes" "false" "Exit code: $?"
fi

# Test 4: Search Command - Rust
write_section "Test 4: Search Command (Rust)"
write_info "Searching for Rust function..."
if search_output=$("$BINARY_PATH" search "rust function" --db-path "$TEST_DB_PATH" 2>&1); then
    assert_success "Search command executes" "true"
    echo "$search_output" | grep -qE "Rank|File|Score" && assert_success "Search returns results" "true" || assert_success "Search returns results" "false"
    echo "$search_output" | grep -q "test\.rs" && assert_success "Search finds Rust file" "true" || assert_success "Search finds Rust file" "false"
else
    assert_success "Search command executes" "false"
fi

# Test 5: Grep Command
write_section "Test 5: Grep Command"
write_info "Testing grep for exact pattern..."
if grep_output=$("$BINARY_PATH" grep "function" 2>&1); then
    assert_success "Grep command executes" "true"
    [ -n "$grep_output" ] && assert_success "Grep finds matches" "true" || assert_success "Grep finds matches" "false"
else
    assert_success "Grep command executes" "false"
fi

# Cleanup
write_section "Cleanup"
cleanup_test_db
write_success "Test database cleaned up"

# Summary
write_section "Test Summary"
total=$((TESTS_PASSED + TESTS_FAILED))
echo -e "\nTotal Tests: $total"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "\n${YELLOW}⚠️  Some tests failed${NC}"
    exit 1
fi
