# 🚀 Next Steps: Feature Roadmap

This document outlines the planned feature progression for `code-rag`, organized from lowest to highest implementation complexity.

## Phase 0: Foundation & Professionalization
* **Complexity:** Medium
* **Goal:** Pay down technical debt, improve reliability, and establish "Pro" quality standards before major feature work.

### Phase 0.1: Code Organization & Refactoring
- [x] **Library Split:** Move core logic (indexing, search, server) into `src/lib.rs`. `src/main.rs` should only handle CLI argument parsing and call library functions.
- [x] **Module Restructuring:**
    -   `src/commands/`: Separate files for `index.rs`, `search.rs`, `serve.rs`.
    -   `src/core/`: Proper domain logic separation.
- [x] **Error Handling Upgrade:**
    -   Replace generic `Box<dyn Error>` with specific types using `thiserror` in the library.
    -   Use `anyhow` + `Context` in the binary for user-friendly error messages (e.g., "Failed to open config file at X" instead of "File not found").

### Phase 0.2: Comprehensive Testing
- [x] **Unit Tests:** High coverage for `chunker.rs` and `config.rs`.
- [x] **Integration Tests:** Expand the `tests/` folder. Test the **Server API** properly using `reqwest` to hit the running server endpoints.
- [x] **Property-Based Testing:** Use `proptest` to generate random file contents and fuzz the indexer to ensure it never crashes on bad input.

### Phase 0.3: Release Engineering
- [x] **Semantic Versioning:** Set up `cargo-release` or GitHub Actions to automate version bumping (Major.Minor.Patch) and strict changelog generation.

### Phase 0.4: Benchmarking
- [x] **Criterion Setup:** Add a `benches/` directory with `criterion`.
- [x] **Critical Paths:** Create benchmarks for:
    -   Indexing throughput (files/sec).
    -   Search latency (P95 and P99).
    -   Embedding generation speed (CPU).

### Phase 0.5: Telemetry (Observability) [done]
**Goal:** Implement "Dynamic Layering" telemetry that adapts to the runtime mode (CLI vs. Server).

#### 1. Dependencies
- **Common:** `tracing`, `tracing-subscriber`, `sysinfo`.
- **CLI Mode:** `tracing-chrome`.
- **Server Mode:** `opentelemetry`, `opentelemetry-otlp`, `opentelemetry_sdk`, `tracing-opentelemetry`, `opentelemetry-prometheus`, `axum` (for metrics endpoint).

#### 2. Logic: `telemetry.rs`
- Implement `init_telemetry(mode: AppMode)`.
- **Mode A: CLI (Ask command)**
    - Initialize `tracing-chrome` layer.
    - Write trace to local file (e.g., `trace-{timestamp}.json`).
    - **Constraint:** Zero network/docker usage.
- **Mode B: Server (Serve command)**
    - Initialize full OpenTelemetry pipeline.
    - **Tracing:** Push to Jaeger (OTLP via gRPC `localhost:4317`).
    - **Metrics:** Expose Prometheus endpoint (scrape config).
    - **Critical Monitor:** Register an Asynchronous Gauge using `sysinfo` to track `app_memory_usage_bytes` (Process RAM) to prevent OOM.

#### 3. Infrastructure
- Generate `docker-compose.yaml` with:
    - **Jaeger:** Ports 16686 (UI), 4317 (OTLP).
    - **Prometheus:** Port 9090 (configured to scrape host).
    - **Grafana:** Port 3001 (provisioned for visualization).

#### 4. Implementation Steps
- [X] Update `Cargo.toml`.
- [X] Create `telemetry.rs` with `AppMode` enum and logic.
- [X] In `main.rs`, switch telemetry based on subcommand (`Ask` vs `Serve`).
- [X] Ensure Server web-layer exposes `/metrics`.
---

## Phase 1: Multi-Workspace Support [done]
* **Complexity:** Low (Architecture/Config)
* **Goal:** Enable a single `code-rag` database to efficiently manage multiple distinct projects.

### Q: Is this optional?
**Yes.** If you do not specify a workspace, the system defaults to a "default" workspace. This ensures backward compatibility—users who don't care about workspaces can ignore this feature completely.

### Implementation Plan
- [x] **Schema Update:** Add a `workspace` column (defaulting to "default") to LanceDB.
- [x] **CLI Update:** Add `--workspace <NAME>` argument.
- [x] **Filter Logic:** Apply `workspace == current_workspace` filter during search.

---

## Phase 2: Context Window Optimization (Smart Context) [done]
* **Complexity:** Medium (Algorithmic)
* **Goal:** Optimize search results for LLM consumption.

### Explanation & Example
LLMs have a limited context window (e.g., 8k, 32k tokens). Searching often returns fragmented chunks:
*   **Result 1:** `FileA.rs` lines 10-20
*   **Result 2:** `FileA.rs` lines 21-40 (Adjacent!)

**Without Optimization (Bad):** You send two separate blocks with metadata headers, wasting tokens and breaking flow.
**With Optimization (Good):** The tool detects these are adjacent and merges them into **one block** (`FileA.rs` lines 10-40) before sending to the LLM. It also prioritizes "definition" chunks (structs/functions) over "implementation" details if space is tight.

### Implementation Plan
- [x] **Token Counter:** Integrate `tiktoken-rs` or similar.
- [x] **Smart Merge:** Algorithm to coalesce adjacent/overlapping chunks.
- [x] **Budget Selector:** "Knapsack" algorithm to fit best content into `N` tokens.
- [x] **Maintenance:** Resolved technical debt (Clippy warnings, config precedence, test isolation).

---

## Phase 3.5: Resource Management & Stability [partial]
* **Complexity:** Medium
* **Goal:** Ensure the application respects system resources (CPU/RAM) to prevent freezing the host machine during heavy indexing.

### Implementation Plan
- [x] **Concurrency Control:**
    -   Implement a global semaphore or `rayon` thread pool configuration to limit active indexing threads (e.g., `num_cpus - 1`).
    -   [x] Add `--threads` CLI argument.
- [x] **Memory Management:**
    -   [x] Implement batch processing for embedding generation (e.g., process 10 files at a time instead of 1000).
    -   Stream file reading instead of loading full content into RAM where possible.
- [X] **Throttling:**
    -   Add process priority adjustments (Lower priority on Windows/Linux) for background tasks.
    -   Implement "Nice" mode for `index` command.

---

## Phase 4: GPU Acceleration & Build Infrastructure [done]
* **Complexity:** High (Build Engineering)
* **Goal:** Speed up embedding generation for massive codebases (>1M LOC).

### Implementation Plan
- [x] **Feature Flags:** Add `cuda` and `metal` features.
- [x] **Build Pipeline:** Dockerfiles/Scripts for GPU builds.
- [x] **Runtime Detection:** Auto-select fastest provider.

---

## Phase 5: Query Expansion (Local LLM)
* **Complexity:** High (AI Integration)
* **Goal:** Solve "vocabulary mismatch" (User says "auth", Code says "identity").

### Implementation Plan
- [X] **Inference:** Integrate `ollama-rs`.
- [X] **Prompting:** System prompts to generate synonyms.
- [X] **RRF:** Merge results from original query + expanded synonyms.

---



