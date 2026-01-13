# Code-RAG Usage & Test Guide

This document defines the standard usage patterns, expected outputs, and assertion criteria for testing the `code-rag` CLI.

## 1. Setup & Build

**Pre-requisite:**
Ensure the binary is built in release mode for performance.

```powershell
cargo build --release --bin code-rag
$BIN = ".\target\release\code-rag.exe"
```

## 2. Assertion Test Cases

### Test Case 1: Indexing a Repository
**Command:**
```powershell
& $BIN index .
```

**Assertion Criteria:**
1.  **Standard Output:**
    -   Must start with `Indexing path: .`
    -   Must end with `Indexing complete.`
    -   **CRITICAL:** Must contain `Found X chunks.` where `X` matches the approximate number of functions/structs in the repo (for this repo: ~30-50).
    -   If `Found 0 chunks`, the test **FAILS**.

2.  **File System:**
    -   Folder `./.lancedb` must exist.
    -   Folder `./.lancedb/code_chunks.lance` (or similar table dir) must exist.

---

### Test Case 2: Semantic Search (Concept Retrieval)
**Query:** "how is the database schema defined?"
**Command:**
```powershell
& $BIN search "how is the database schema defined?" --limit 1
```

**Assertion Criteria:**
1.  **Output Format:**
    -   Must print `Searching for: 'how is the database schema defined?'`
    -   Must print `Match 1: ...`
2.  **Content Verification:**
    -   **Filename:** Must be `.\src\storage.rs` (or `src/storage.rs`).
    -   **Code Snippet:** Must contain `pub struct CodeChunk`.

---

### Test Case 3: Semantic Search (Specific Component)
**Query:** "embedding model"
**Command:**
```powershell
& $BIN search "embedding model" --limit 1
```

**Assertion Criteria:**
1.  **Content Verification:**
    -   **Filename:** Must be `.\src\embedding.rs`.
    -   **Code Snippet:** Must contain `NomicEmbedTextV15` or `Embedder::new`.

---

### Test Case 4: Exact Match Search (Grep)
**Pattern:** "CodeChunker::get_language"
**Command:**
```powershell
& $BIN grep "CodeChunker::get_language"
```

**Assertion Criteria:**
1.  **Output Format:**
    -   Must print `Grepping for: 'CodeChunker::get_language'`
2.  **Matches:**
    -   Must find usage in `.\src\main.rs`.
    -   Must find definition in `.\src\indexer.rs`.
    -   Output format: `path:line: content`.

---

### Test Case 5: Error Handling (Missing Index)
**Setup:** `Remove-Item -Recurse -Force .lancedb`
**Command:**
```powershell
& $BIN search "anything"
```

**Assertion Criteria:**
1.  **Exit Code:** Non-zero (or explicit error message).
2.  **Error Message:** Should likely fail with "LanceDB error" or "Table not found".

## 3. Automated Verification Script

Use this PowerShell snippet to run a quick health check:

```powershell
$ErrorActionPreference = "Stop"
$BIN = ".\target\release\code-rag.exe"

Write-Host "1. Testing Index..." -ForegroundColor Cyan
& $BIN index .
if (!(Test-Path .\.lancedb)) { throw "Index unavailable" }

Write-Host "2. Testing Search..." -ForegroundColor Cyan
$result = & $BIN search "database schema" --limit 1
if ($result -notmatch "storage.rs") { throw "Search failed to find storage.rs" }

Write-Host "3. Testing Grep..." -ForegroundColor Cyan
$grep = & $BIN grep "CodeChunker"
if ($grep -notmatch "indexer.rs") { throw "Grep failed to find indexer.rs" }

Write-Host "ALL TESTS PASSED" -ForegroundColor Green
```
