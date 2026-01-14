# Manual Test Results

## Issue Summary
**Symptom:** `code-rag index .` reported 0 chunks found.
**Root Cause:** Version mismatch between `tree-sitter` core library (v0.24.x) and language crates (`tree-sitter-rust` v0.24.0). The language crates were generating parsers with ABI version 15, while the core library (pinned or resolved incorrectly) was not supporting it fully or correctly in the mixed environment.
**Fix:** Downgraded language parsers to stable versions (Python 0.23, Rust 0.23) to ensure ABI compatibility.

## Verification Run

### 1. Indexing
Command: `.\target\release\code-rag.exe index .`
Result:
- **Chunks Found:** 29 (Correctly identified functions and structs in `src/*.rs`)
- **Status:** Success

### 2. Semantic Search
Command: `.\target\release\code-rag.exe search "how is the database schema defined?"`
Result:
- **Matches:** Found `storage.rs` containing `CodeChunk` struct definition.
- **Snippet:**
  ```rust
  pub struct CodeChunk {
      pub id: String,
      // ...
  }
  ```

Command: `.\target\release\code-rag.exe search "which embedding model is used?"`
Result:
- **Matches:** Found `embedding.rs` containing `NomicEmbedTextV15`.

### 3. Grep Search
Command: `.\target\release\code-rag.exe grep "CodeChunker"`
Result:
- **Matches:** `.\src\indexer.rs:11: pub struct CodeChunker {}`
- **Matches:** `.\src\main.rs:58: if CodeChunker::get_language(ext).is_some() {`

## Conclusion
The application is fully functional. The dependency hell with `tree-sitter` has been resolved by selecting compatible crate versions.
