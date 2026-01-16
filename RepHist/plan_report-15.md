# Project Status Report: code-rag (Report 15)

## 1. Feature Implementation Status

This section tracks the status of features outlined in the original plan.

| Feature | State | Optimization Status | Notes |
| :--- | :--- | :--- | :--- |
| **1. `--no-rerank` Flag** | **Done** | [Optimized] | Skips heavy re-ranking step completely. |
| **2. Progress Indicators** | **Done** | N/A | UX improvement, no perf impact. |
| **3. Path Normalization** | **Done** | [Optimized] | Pre-processing step, negligible cost. |
| **4. Model Pre-caching** | **Done** | [Optimized] | Lazy-loading implemented for `Reranker`. |
| **5. Embed Batch Size** | **Done** | [Optimized] | `embed` accepts batch size, improved throughput. |
| **7. Server Mode** | **Done** | [Optimized] | Uses `Arc<Mutex>` for shared state (zero-copy cloning). |
| **8. Hybrid Search** | **Done** | [Optimized] | Uses `k=60` RRF and parallel query execution strategy. |
| **6. DB Index on `filename`** | **Done** | [Optimized] | Filename index created for fast filtering. |
| **9. Configuration File** | **Done** | N/A | `code-ragcnf.toml` with hierarchical loading. |
| **10. Structured Logging** | **Done** | [Optimized] | `tracing` crate replacing `println!` for ops. |

## 2. Error Handling Analysis

**Current State:**
- **Local Handling:** The `server.rs` uses local `match` statements inside handlers (e.g., `search_handler`) to catch errors and return `500 Internal Server Error`.
- **Propagated Errors:** `main.rs` uses `Result<(), Box<dyn Error>>` to propagate fatal errors to the CLI.

**Missing Logic (Global Error Handler):**
- **No Global Middleware:** There is no `tower_http::catch_panic::CatchPanicLayer` or similar middleware to catch panics or unhandled errors strictly at the layer level.
- **No Unified Error Type:** Errors are typically cast to `Box<dyn Error>` or stringified, rather than using a strongly typed `AppError` enum that implements `IntoResponse`.

**Recommendation:**
Implement a custom `AppError` type and use `map_response` or `layer` middleware to standardize error JSON formats (`{ "error": "message" }`) across the entire API.

## 3. Remaining Backlog (Prioritized)

Sorted by **Effort** (Low → High) and **Complexity**.

### Low Hanging Fruit [done]
1.  **LanceDB Index on `filename`** (Item 6)
    *   *Effort:* Low (~40 LOC)
    *   *Complexity:* Low
    *   *Value:* High for filtered searches.

### High Value / High Effort
2.  **File System Watcher** (Item 9)
    *   *Effort:* Very High (~450 LOC)
    *   *Complexity:* High (Concurrency, Debouncing)
    *   *Value:* Critical for "set and forget" developer experience.
    
3.  **LSP Integration** (Item 10)
    *   *Effort:* Maximum (~800 LOC)
    *   *Complexity:* Very High (Protocol details, State management)
    *   *Value:* Transformational (Embeds tool into workflow).

### Research / Experimental
4.  **Query Expansion** (Item 11)
    *   *Effort:* High (~600 LOC)
    *   *Complexity:* High
    *   *Value:* Medium (Accuracy boost, but improves latency).

5.  **GPU Acceleration** (Item 12)
    *   *Effort:* High (Config hell)
    *   *Complexity:* High
    *   *Value:* High (Batch indexing speed).

## 4. Agent Observations & Recommendations

**App Assessment:**
`code-rag` is developing into a robust, high-performance local tool. The architecture (LanceDB + Tantivy) provides a best-in-class foundation for offline semantic search. The implementation of "Hybrid Search" significantly closes the recall gap for technical queries (variable names, error codes).

**Recommended New Features:**
1.  **Structured Error Middleware**: As verified above, adding a global error handler layer in `server.rs` is critical before any heavy production use or LSP integration.
2.  **Configuration File**: Replace strict CLI args with a `config.toml` loader to manage model paths, exclusions, and port settings persistently.
3.  **Telemetry/Tracing**: Replace `println!` with the `tracing` crate. This is essential for debugging the "File Watcher" or "LSP" features when they run in the background.

**Immediate Next Step Suggestion:**
Implement the **LanceDB Index** (Item 6) to close out Phase 2 completely, then tackle **Global Error Handling** as a hardening step before starting the complex **File Watcher**.

## 5. Implementation Roadmap Recommendation

This roadmap balances quick wins with long-term strategic value.

### Phase 1: Hardening & Clean-up (Immediate) [started]
*Goal: Ensure the current codebase is production-ready before adding complexity.*

1.  **LanceDB Index on `filename`** [done]
    *   **Priority:** High
    *   **Complexity:** Low
    *   **LOC:** ~40
    *   **Reason:** Quick performance win for filtered searches.
2.  **Global Error Handling Middleware** [done]
    *   **Priority:** Critical
    *   **Complexity:** Medium
    *   **LOC:** ~150
    *   **Reason:** Standardize API errors before LSP integration.
3.  **Structured Ops (Tracing/Config)** [done]
    *   **Priority:** Medium
    *   **Complexity:** Medium
    *   **LOC:** ~200
    *   **Reason:** Essential for debugging future background tasks.

### Phase 2: Core Experience (Next 2 Weeks)
*Goal: Make the tool seamless to use ("Magic").*

4.  **File System Watcher**
    *   **Priority:** High
    *   **Complexity:** High
    *   **LOC:** ~450
    *   **Reason:** Removes the manual "re-index" step, crucial for efficient workflows.

### Phase 3: Platform Expansion (Next Month)
*Goal: Embed the tool where developers work.*

5.  **LSP Integration**
    *   **Priority:** Very High (Long-term)
    *   **Complexity:** Very High
    *   **LOC:** ~800
    *   **Reason:** The ultimate form factor for this tool.

### Phase 4: Future Tech
*Goal: Push accurate & speed boundaries.*

6.  **Query Expansion (Local LLM)**
    *   **Priority:** Low
    *   **Complexity:** High
    *   **LOC:** ~600
    *   **Reason:** Improve recall for fuzzy concepts.
7.  **GPU Acceleration**
    *   **Priority:** Low
    *   **Complexity:** High
    *   **LOC:** ~200
    *   **Reason:** Only needed for massive codebases (>1M LOC).

# =============================================================================
# RECOMMENDATIONS: WHAT TO INCLUDE NEXT
# =============================================================================

The following features are missing and recommended for the next iteration, sorted from **Low Effort (Simple)** to **High Effort (Complex)**.

### 1. CI/CD Pipeline
*   **Effort:** Low
*   **Description:** implementation of GitHub Actions for automated testing, linting (`clippy`), and release binary generation.
*   **Value:** Ensures code quality and simplifies distribution.

### 2. Model Selection (Configurable)
*   **Effort:** Medium
*   **Description:** Currently hardcoded in `src/embedding.rs`. Making this configurable allows users to switch between speed (smaller models) and accuracy (larger models).
*   **Config:** `embedding_model = "nomic-embed-text-v1.5"`

### 3. Chunking Strategy (Configurable)
*   **Effort:** Medium
*   **Description:** Currently hardcoded. Configurable chunk sizes allow tuning for different codebases (e.g., larger chunks for Java, smaller for Python).
*   **Config:** `chunk_size_tokens = 512`, `chunk_overlap_tokens = 64`

### 4. Hybrid Search Tuning
*   **Effort:** Medium
*   **Description:** Weights for the Reciprocal Rank Fusion (RRF) between Vector and BM25 results.
*   **Config:** `vector_weight = 1.0`, `bm25_weight = 1.0`

### 5. File System Watcher
*   **Effort:** High
*   **Description:** Implement `notify` crate to watch for file changes and auto-update the index in the background. Requires generic implementation of a background worker.
*   **Value:** Developers never have to run `index --update` manually.

### 6. Interactive TUI (Terminal UI)
*   **Effort:** High
*   **Description:** A rich terminal interface using `ratatui` for browsing search results, viewing snippets, and navigating code without leaving the CLI.
*   **Value:** vastly improved UX for heavy CLI users.

### 7. LSP Integration (Language Server Protocol)
*   **Effort:** Very High
*   **Description:** Wrap `code-rag` in an LSP interface so it can provide "Semantic Search" results directly in VS Code / Neovim via code actions or hover providers.
*   **Value:** The ultimate developer experience.
