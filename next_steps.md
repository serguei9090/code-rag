# 🚀 Next Steps: Feature Roadmap

This document outlines the planned feature progression for `code-rag`, organized from lowest to highest implementation complexity.

## Phase 0: Foundation & Professionalization
* **Complexity:** Medium
* **Goal:** Pay down technical debt, improve reliability, and establish "Pro" quality standards before major feature work.

### Phase 0.1: Code Organization & Refactoring
- [ ] **Library Split:** Move core logic (indexing, search, server) into `src/lib.rs`. `src/main.rs` should only handle CLI argument parsing and call library functions.
- [ ] **Module Restructuring:**
    -   `src/commands/`: Separate files for `index.rs`, `search.rs`, `serve.rs`.
    -   `src/core/`: Proper domain logic separation.
- [ ] **Error Handling Upgrade:**
    -   Replace generic `Box<dyn Error>` with specific types using `thiserror` in the library.
    -   Use `anyhow` + `Context` in the binary for user-friendly error messages (e.g., "Failed to open config file at X" instead of "File not found").

### Phase 0.2: Comprehensive Testing
- [ ] **Unit Tests:** High coverage for `chunker.rs` and `config.rs`.
- [ ] **Integration Tests:** Expand the `tests/` folder. Test the **Server API** properly using `reqwest` to hit the running server endpoints.
- [ ] **Property-Based Testing:** Use `proptest` to generate random file contents and fuzz the indexer to ensure it never crashes on bad input.

### Phase 0.3: Release Engineering
- [ ] **Semantic Versioning:** Set up `cargo-release` or GitHub Actions to automate version bumping (Major.Minor.Patch) and strict changelog generation.

### Phase 0.4: Benchmarking
- [ ] **Criterion Setup:** Add a `benches/` directory with `criterion`.
- [ ] **Critical Paths:** Create benchmarks for:
    -   Indexing throughput (files/sec).
    -   Search latency (P95 and P99).
    -   Embedding generation speed (CPU).

### Phase 0.5: Telemetry (Observability)
**Goal:** Gather anonymous usage/performance stats locally to understand bottlenecks.

#### Options for Local Telemetry
1.  **Local Metrics File (Recommended Start)** 📄
    *   **How:** Use the `metrics` crate. Dump counters/histograms to `~/.code-rag/stats/`.
    *   **Pros:** Simple, local, no dependencies.
2.  **Prometheus Exporter** 📊
    *   **How:** Expose `/metrics` endpoint.
    *   **Pros:** Grafana dashboards.
3.  **Jaeger Tracing** 🕸️
    *   **How:** Full OpenTelemetry tracing.
    *   **Pros:** Visual debugging of latency waterfalls.
---

## Phase 1: Multi-Workspace Support
* **Complexity:** Low (Architecture/Config)
* **Goal:** Enable a single `code-rag` database to efficiently manage multiple distinct projects.

### Q: Is this optional?
**Yes.** If you do not specify a workspace, the system defaults to a "default" workspace. This ensures backward compatibility—users who don't care about workspaces can ignore this feature completely.

### Implementation Plan
- [ ] **Schema Update:** Add a `workspace` column (defaulting to "default") to LanceDB.
- [ ] **CLI Update:** Add `--workspace <NAME>` argument.
- [ ] **Filter Logic:** Apply `workspace == current_workspace` filter during search.

---

## Phase 2: Context Window Optimization (Smart Context)
* **Complexity:** Medium (Algorithmic)
* **Goal:** Optimize search results for LLM consumption.

### Explanation & Example
LLMs have a limited context window (e.g., 8k, 32k tokens). Searching often returns fragmented chunks:
*   **Result 1:** `FileA.rs` lines 10-20
*   **Result 2:** `FileA.rs` lines 21-40 (Adjacent!)

**Without Optimization (Bad):** You send two separate blocks with metadata headers, wasting tokens and breaking flow.
**With Optimization (Good):** The tool detects these are adjacent and merges them into **one block** (`FileA.rs` lines 10-40) before sending to the LLM. It also prioritizes "definition" chunks (structs/functions) over "implementation" details if space is tight.

### Implementation Plan
- [ ] **Token Counter:** Integrate `tiktoken-rs` or similar.
- [ ] **Smart Merge:** Algorithm to coalesce adjacent/overlapping chunks.
- [ ] **Budget Selector:** "Knapsack" algorithm to fit best content into `N` tokens.

---

## Phase 3: GPU Acceleration (CUDA/Metal)
* **Complexity:** High (Build Engineering)
* **Goal:** Speed up embedding generation for massive codebases (>1M LOC).

### Implementation Plan
- [ ] **Feature Flags:** Add `cuda` and `metal` features.
- [ ] **Build Pipeline:** Dockerfiles/Scripts for GPU builds.
- [ ] **Runtime Detection:** Auto-select fastest provider.

---

## Phase 4: Query Expansion (Local LLM)
* **Complexity:** High (AI Integration)
* **Goal:** Solve "vocabulary mismatch" (User says "auth", Code says "identity").

### Implementation Plan
- [ ] **Inference:** Integrate `ollama-rs`.
- [ ] **Prompting:** System prompts to generate synonyms.
- [ ] **RRF:** Merge results from original query + expanded synonyms.

---

## Phase 5: LSP Integration (Language Server Protocol)
* **Complexity:** Very High
* **Goal:** Embed `code-rag` directly into IDEs as a semantic engine.

### Implementation Plan
- [ ] **JSON-RPC:** Adopt `tower-lsp`.
- [ ] **Capabilities:** `textDocument/definition`, `textDocument/codeAction`.

---


