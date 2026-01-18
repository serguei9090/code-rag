
# Code Performance Report

### Overall Assessment

The codebase has undergone a significant performance optimization pass focusing on high-impact bottlenecks in the search hot path. The transition to a shareable `Arc`-based architecture and the use of `spawn_blocking` for ML tasks allows the system to utilize multi-core systems effectively without stalling the asynchronous runtime. Existing benchmarks or profiling data (e.g., flamegraphs) are recommended to validate these changes further.

### **Prioritized Recommendations**

1.  **Baseline Profiling**: Implement Criterion benchmarks or generate flamegraphs during search to identify remaining micro-bottlenecks.
2.  **Buffer Reuse**: In `storage.rs`, investigate if Arrow `RecordBatch` builders can be reused across indexing iterations to minimize allocation churn.
3.  **IO optimization**: Verify if Tantivy/LanceDB are currently benefiting from SSD-optimized I/O settings (e.g. `MmapDirectory`).

### **Detailed Feedback**

---

**[CONCURRENCY]** - Async Executor Blocking (FIXED)

**Previously identified issue:**
Heavy CPU tasks like `TextEmbedding::embed` and `TextRerank::rerank` were executed directly within `async` functions, blocking the entire thread pool.

**Solution implemented:**
Wrapped heavy calls in `tokio::task::spawn_blocking`:
```rust
let all_query_vectors = tokio::task::spawn_blocking(move || {
    embedder_handle
        .embed(query_batch, None)
        .map_err(|e| anyhow!(e.to_string()))
})
.await??;
```

**Rationale:**
Async executors (Tokio) are designed for I/O-bound tasks. CPU-intensive operations (ML inference) can cause "executor starvation," where I/O tasks are delayed. Offloading to a dedicated blocking pool is the standard Rust pattern for this.

---

**[MEMORY]** - Redundant Schema Lookups (FIXED)

**Previously identified issue:**
Tantivy field identifiers were looked up by string name on every search/index operation:
`schema.get_field("id")?`

**Solution implemented:**
Cached `Field` handles in the `BM25Index` struct during initialization. Use direct field access in hot paths:
```rust
let id_field = self.id_field;
// ...
doc.add_text(id_field, &chunk_id);
```

**Rationale:**
String-based schema lookups involve parsing and searching hash maps. While relatively fast, when performed inside tight loops (indexing thousands of chunks), the overhead accumulates. Caching handles brings this cost down to zero at runtime.

---

**[MEMORY]** - Collection Pre-allocation (FIXED)

**Previously identified issue:**
Intermediate `Vec` and `HashMap` structures in `semantic_search` were initialized without capacity.

**Solution implemented:**
Used `with_capacity` based on the requested `limit`:
```rust
let mut all_vector_results: std::collections::HashMap<String, SearchResult> =
    std::collections::HashMap::with_capacity(std::cmp::max(50, limit * 2));
```

**Rationale:**
Standard collections grow exponentially, causing multiple re-allocations and data copies as they fill. Pre-sizing prevents this churn, especially since the final size is often roughly proportional to the requested `limit`.

---
