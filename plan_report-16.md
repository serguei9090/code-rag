# Project Status Report: code-rag (Report 16)

## Executive Summary
This report outlines the **remaining roadmap** for `code-rag`. Completed features (Configuration, Logging, Filename Index) are excluded. The focus is on moving from a rigid prototype to a flexible, developer-friendly tool.

The tasks are ordered from **Lowest Effort** (Configuration tweaks) to **Highest Effort** (Complex integrations), with CI/CD placed at the end as a final distribution step.

---

## Prioritized Implementation Roadmap

### 1. Configurable Model Selection [done]
*   **Effort:** Medium
*   **Current State:** Hardcoded to `nomic-embed-text-v1.5` and `bge-reranker-base` in `src/embedding.rs`.
*   **What it is for:** Allows the user to specify which ONNX models to use via `code-ragcnf.toml`.
*   **What it improves:** **Flexibility**. Users can trade off between speed (smaller models) and accuracy (larger models) or use specialized models for different languages.

### 2. Configurable Chunking Strategy [done]
*   **Effort:** Medium
*   **Current State:** Hardcoded chunk sizes in `src/indexer.rs` (defaulting often to function boundaries or arbitrary limits).
*   **What it is for:** Exposes `chunk_size` and `overlap` parameters in the configuration file.
*   **What it improves:** **Recall & Context**. Different languages and coding styles require different chunking strategies (e.g., verbose Java vs. concise Python). Tuning this improves search relevance.
*   **Documentation Required:** Create a documentation with recommended chunk size by different languages, `chunk_strategy.md` inside `docs/configurations`.

### 3. Hybrid Search Tuning (RRF Weights)
*   **Effort:** Medium
*   **Current State:** Reciprocal Rank Fusion (RRF) constants are hardcoded (`k=60`, weights=1.0).
*   **What it is for:** Adds configuration options to weight Vector Search vs. Keyword Search (BM25).
*   **What it improves:** **Precision**. Allows fine-tuning the search algorithm. For example, a user strictly looking for exact error codes can increase the BM25 weight, while a user doing conceptual search can prioritize vectors.
*   **Documentation Required:** Write doc `RRFWeights.md` inside `docs/configurations`, and update the conf template `code-ragcnf.toml.template`

### 4. File System Watcher (`notify`)
*   **Effort:** High
*   **Current State:** Users must manually run `code-rag index` or `code-rag index --update` to refresh the DB.
*   **What it is for:** A background process (daemon) that watches the project directory for file changes and updates the index in real-time.
*   **What it improves:** **Developer Experience (DX)**. Removes the manual friction of re-indexing. The tool becomes "set and forget," ensuring search results are always fresh.

### 5. Interactive Terminal UI (TUI)
*   **Effort:** High
*   **Current State:** CLI outputs raw text or JSON to stdout.
*   **What it is for:** A rich, interactive terminal interface (using `ratatui`) to browse results, preview snippets with syntax highlighting, and navigate code without leaving the terminal window.
*   **What it improves:** **Usability**. Transforms `code-rag` from a pipeable utility into a standalone developer tool, similar to `lazygit`.

### 6. LSP Integration (Language Server Protocol)
*   **Effort:** Very High
*   **Current State:** No IDE integration.
*   **What it is for:** Wraps `code-rag` in a standard LSP interface. Editors (VS Code, Neovim, Zed) can query it to provide "Semantic Search" code actions or definitions.
*   **What it improves:** **Workflow Integration**. Puts the power of the tool directly inside the cursor's context, making it a seamless part of writing code.

### 7. Query Expansion (Local LLM)
*   **Effort:** High (Research/Experimental)
*   **Current State:** Queries are embedded directly.
*   **What it is for:** Uses a small, local SLM (e.g., Phi-2, Qwen) to generate synonyms or rephrase the query before embedding.
*   **What it improves:** **Recall**. Helps with the "vocabulary mismatch" problem where the user queries "auth" but the code calls it "identity".

### 8. GPU Acceleration (CUDA/Metal)
*   **Effort:** High (Configuration/Build Complexity)
*   **Current State:** CPU-only via ONNX Runtime.
*   **What it is for:** Enables `fastembed-rs` to utilize GPU backends for higher throughput during batch indexing.
*   **What it improves:** **Performance**. Critical for indexing massive codebases (>1M LOC) quickly, though less impactful for search latency.

---

### 9. CI/CD Pipeline (GitHub Actions)
*   **Effort:** Low (but infrastructure dependent)
*   **Current State:** Manual local builds.
*   **What it is for:** Automated workflow to run `cargo test`, `cargo clippy`, and build release binaries for Windows/Linux/macOS on every push.
*   **What it improves:** **Reliability & Distribution**. Ensures no bad code is merged and automates the creation of binaries for other users/machines.
