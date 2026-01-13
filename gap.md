# Gaps & Feature Roadmap

This document analyzes current limitations of `code-rag` and proposes specific enhancements to mature the tool from a prototype to a production-grade developer utility.

## A. Operational Flexibility & Configuration (High Priority)

### 1. Configuration Management (`code-rag.toml`)
*   **Gap:** Currently, behavior is hardcoded. Users cannot define persistent defaults.
*   **Solution:** Implement a Cascading Configuration system.
    *   **Priority:** CLI Args (`--db-path`) > Local Config (`./code-rag.toml`) > Global Config (`~/.code-rag/config.toml`).
    *   **Fields:**
        ```toml
        [general]
        default_index_path = "."
        db_path = "C:/Users/Dev/AppData/Local/code-rag/db" # Centralized DB
        
        [indexing]
        exclude_patterns = ["*.test.rs", "target/"]
        ```

### 2. Database Location Control (`--db-path`)
*   **Gap:** Database is locked to `./.lancedb`. This forces a 1:1 relationship between the repo and the DB, preventing shared caches or central storage.
*   **Solution:** Add `--db-path <PATH>` argument.
*   **Benefit:** Enables separating the index from the codebase (preventing accidental commits) and allows switching contexts without moving files.

### 3. Incremental Indexing (Update Mode)
*   **Gap:** "Re-indexing" requires deleting the DB and starting over. Slow for large repos.
*   **Solution:** Implement intelligent updates.
    *   **Strategy:** Store file modification times (mtime) or hashes in LanceDB.
    *   **Logic:**
        *   `--update`: Scan files; if `mtime > last_indexed`, re-chunk and upsert. If file missing, delete chunks.
        *   `--force`: Wipe and rebuild.
*   **Benefit:** Reduces indexing time from minutes to seconds for daily workflows.

## B. User Experience (UX)

### 4. Visual Feedback (Progress Bar)
*   **Gap:** Long operations (indexing 10k files) look frozen/hung.
*   **Solution:** Integrate `indicatif` crate.
*   **Display:** `[Scanning Files] 1200/5000 | [Embedding] 50% [===>...]`

### 5. Result "Pretty Printing"
*   **Gap:** Output is raw text/JSON-like. Hard to visually scan.
*   **Solution:** Format search results for readability.
    *   **Header:** Rank #1 (Score: 0.89) - `src/main.rs:50-80` (Clickable in VSCode terminals).
    *   **Body:** Syntax-highlighted code snippet.
    *   **Action:** `code --goto src/main.rs:50` hint.

### 6. HTML Report Viewer
*   **Gap:** Terminal buffers are limited for reviewing long code blocks.
*   **Solution:** `code-rag serve` or `code-rag report --html results.html`.
*   **Benefit:** Provides a rich, filterable view (by file type, score) and easier copy-pasting.

## C. Advanced RAG Intelligence

### 7. Call Hierarchy Awareness
*   **Gap:** `grep` and vector search miss indirect relationships (e.g., Interface -> Implementation or Function -> Import Alias).
*   **Solution:** Lightweight Call Graph.
    *   During AST parsing, record `calls: [function_names]` metadata.
    *   Allow "Graph Walk" queries: "Find callers of `process_data`".

### 8. Semantic Re-ranking
*   **Gap:** Vector similarity is fuzzy. It may rank a "comment about auth" higher than the "auth implementation".
*   **Solution:** Two-stage retrieval.
    1.  Vector Search (Top 50 candidates).
    2.  Cross-Encoder Re-ranker (Top 5 highly relevant matches).
*   **Benefit:** Drastically improves precision for ambiguous queries.
