# Implementation Plan - Optimize Indexing Merge Policy

## Goal
Reduce the frequency and aggressiveness of Tantivy's index segment merging to prevent "writer lock" timeouts during high-concurrency tests and large bulk imports.

## Proposed Changes

### 1. `src/config.rs`
*   Add `merge_policy` field to `AppConfig` struct (String, default: "log").
*   Supported values: "log" (relaxed, default), "fast-write" (very relaxed), "fast-search" (aggressive/default behavior).

### 2. `src/bm25.rs`
*   Update `BM25Index::new` signature to accept `AppConfig` or just the `merge_policy` string.
*   Implement parsing logic:
    *   `"log"` or `"fast-write"` -> `tantivy::merge_policy::LogMergePolicy` with `min_merge_size` = 8.
    *   `"fast-search"` -> Default Tantivy policy (aggressive merging).
*   Apply the policy: `index.set_merge_policy(...)`.

## Pros & Cons

### Pros
*   **Performance Stability:** Prevents "stop-the-world" pauses where the application hangs while merging large index segments.
*   **Faster Batch Indexing:** Reduces the overhead of constant merging during initial bulk indexing of large codebases.
*   **Test Reliability:** Eliminates timeouts in E2E tests caused by writer locks held during aggressive merges.

### Cons
*   **Search Latency:** Potentially slight increase in search latency if there are many unmerged segments (though `LogMergePolicy` manages this reasonably well).
*   **File Handle Usage:** May use more open file handles if many small segments accumulate.
*   **Disk Usage:** Temporary disk usage may be higher as deleted documents in old segments occupy space until a merge occurs.

## Verification Plan

### Automated Tests
1.  **Unit Tests:** Run existing BM25 tests to ensure no regression.
    *   `cargo test --lib -- bm25::tests`
2.  **Concurrency Test:** Run the `test_concurrent_ops.ps1` script (which previously passed, but we want to ensure it still passes and maybe runs faster).
    *   `powershell -ExecutionPolicy Bypass -File tests/e2e/test_concurrent_ops.ps1`
3.  **Full Suite:** Run the unified runner that previously timed out.
    *   `powershell -ExecutionPolicy Bypass -File tests/e2e/run_all_tests.ps1`

### Manual Verification
*   Check logs during `run_all_tests.ps1` to see if `"Deleted ..."` messages (indicating merges) are less frequent or blocking less.
