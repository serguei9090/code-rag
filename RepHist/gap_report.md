# Test Gap Report & Implementation Plan

## Overview
This report analyzes the current test coverage of `code-rag` and outlines a plan to address identified gaps.

**Current State:**
- **E2E/CLI (`tests/e2e/test_cli.ps1`):** High coverage. Validates all commands, flags, and major workflows.
- **Integration (`tests/integration/`):** High coverage. Core flows + Resilience (Error handling, Corrupt DBs) covered.
- **Unit (`src/`):** Good coverage. `indexer.rs` (Chunking/Overlap) and `search.rs` (Scoring/Sorting) now covered.

## Missing Feature Tests & Implementation Plan

### 1. Error Handling & Resilience
**Gaps:**
- **Invalid Database State:** No tests for corrupt files or locked databases.
- **Input Edge Cases:** Syntax errors, empty files, large files (>10MB).
- **Invalid Regex:** Graceful failure for bad regex patterns.

**Implementation Plan:**
- [ ] **Create `tests/integration/resilience.rs`:**
  - **Test: `test_corrupt_database`:** Manually write garbage data to a `.lancedb` table file, attempt to `Search`, and assert `Err` (not panic).
  - **Test: `test_empty_file_indexing`:** Index an empty file and assert 0 chunks created without error.
  - **Test: `test_large_file_chunking`:** Generate a 5MB dummy file, index it, and verify valid chunk count and performance (<1s).
  - **Test: `test_invalid_syntax`:** Index a `.rs` file containing Python code (or random garbage). Assert parser does not panic and still attempts to extract text/comments if possible, or returns graceful empty result.
  - **Test: `test_invalid_regex`:** Verify correct error handling (not panic) when an invalid regex pattern is supplied to grep search.

### 2. Logic & Algorithms (Unit Tests)
**Gaps:**
- **Ranking Logic (RRF):** Complex scoring logic in `search.rs` is untested.
- **Chunking Boundaries:** Overlap and max size enforcement needs verification.

**Implementation Plan:**
- [ ] **Update `src/search.rs`:**
  - Add `#[cfg(test)] mod tests`.
  - **Test: `test_rrf_scoring`:** Instantiate `CodeSearcher` with specific weights. Manually feed mocked vector and BM25 results. Verify `score = (vector_rank_score * weight) + (bm25_rank_score * weight)`.
  - **Test: `test_sorting`:** Verify that results are correctly sorted by the final RRF score in descending order.
- [ ] **Update `src/indexer.rs`:**
  - **Test: `test_chunk_overlap`:** Create a string slightly larger than `max_chunk_size`. Verify the second chunk starts at `max_chunk_size - overlap`.
  - **Test: `test_exact_size_limit`:** Feed a huge single line. Verify it is split exactly at `max_chunk_size`.

### 3. Configuration & Environment
**Gaps:**
- **Config Loading:** Priority of CLI args vs. Config file vs. Env vars is not tested.

**Implementation Plan:**
- [ ] **Update `src/config.rs` (or `tests/integration/config.rs`):**
  - **Test: `test_default_config`:** Assert default values.
  - **Test: `test_env_override`:** Set `CODE_RAG_DB_PATH` env var, initialize config, verify it takes precedence over default.

### 4. Concurrency & Server
**Gaps:**
- **Concurrent Search:** No stress test for multiple simultaneous requests.

**Implementation Plan:**
- [ ] **Update `tests/integration/server.rs`:**
  - **Test: `test_concurrent_searches`:** Spawn 20 async tasks hitting the search endpoint simultaneously. Assert all return 200 OK and valid JSON. This verifies `Arc<Mutex<CodeSearcher>>` doesn't deadlock.

## Summary Checklist
- [x] Add `tests/integration/resilience.rs` (Edge cases, Error handling)
- [x] Add unit tests to `src/search.rs` (RRF Logic)
- [x] Add unit tests to `src/indexer.rs` (Overlap/Size logic)
- [x] Add unit tests to `src/config.rs` (Env vars)
- [x] Add concurrency test to `tests/integration/server.rs`
