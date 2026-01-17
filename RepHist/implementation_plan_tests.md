# Implementation Plan - Advanced E2E Tests

## Goal
Implement missing advanced test scenarios (Server Stress, Read/Write Contention) and create a unified automated test runner that executes all tests and reports timing.

## Proposed New Files

### 1. `tests/e2e/test_concurrent_ops.ps1`
**Purpose:** Test high-stress and conflicting operation scenarios.
*   **Server Stress Test:**
    *   Start `code-rag serve` on a random/fixed port.
    *   Use `Start-ThreadJob` (or `Start-Job`) to spawn 20+ parallel requests to `/search`.
    *   Verify all requests complete successfully (200 OK) and within reasonable time.
*   **Read/Write Contention Test:**
    *   Start a large `index` operation in the background.
    *   Immediately start a loop of `search` queries in the foreground.
    *   Verify that searching does not crash the indexer and vice versa.
    *   Verify that search returns results (either from old snapshot or new data) without erroring.

### 2. `tests/e2e/run_all_tests.ps1`
**Purpose:** "One automatic test with all tests that also include time taked".
*   This script will function as a test runner.
*   It will invoke:
    1.  `test_cli.ps1` (Existing CLI coverage)
    2.  `test_server.ps1` (Existing basic server coverage)
    3.  `test_concurrent_ops.ps1` (New advanced coverage)
*   **Timing:** It will measure `Measure-Command` (or `Stopwatch`) for each suite and report individual and total execution times.
*   **Reporting:** Output a summary table of Pass/Fail status and Duration for each suite.

## Execution Steps
1.  Create `tests/e2e/test_concurrent_ops.ps1`.
2.  Create `tests/e2e/run_all_tests.ps1`.
3.  Verify by running `run_all_tests.ps1`.
