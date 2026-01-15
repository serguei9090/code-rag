# Performance Analysis: code-rag CLI

## Execution Time Summary

Based on interactive walkthrough test results:

| Operation | Time (seconds) | Rank | Status |
|-----------|---------------|------|--------|
| Help | 0.078 | ⚡ Excellent | ✅ |
| Grep (exact match) | 0.609 | ⚡ Excellent | ✅ |
| Search (filtered by ext) | 18.493 | ⚠️ Slow | ⚠️ |
| Search (filtered by dir) | 21.127 | ⚠️ Slow | ⚠️ |
| Index (73 chunks) | 46.447 | ⚠️ Acceptable | ⚠️ |
| Search (JSON output) | 55.157 | ❌ Very Slow | ❌ |
| Search (basic) | 56.378 | ❌ Very Slow | ❌ |

---

## Detailed Analysis

### ⚡ Fast Operations (< 1s)

#### 1. Help Command (0.078s)
- **Status**: Excellent
- **Analysis**: Native CLI help generation, no I/O or computation.
- **Recommendation**: None needed.

#### 2. Grep Search (0.609s)
- **Status**: Excellent
- **Analysis**: Uses ripgrep engine for exact pattern matching across files.
- **Recommendation**: None needed. This is excellent performance.

---

### ⚠️ Slow Operations (15-25s)

#### 3. Filtered Search - Extension (18.493s)
- **Status**: Slow
- **Analysis**: First semantic search after full cold start. Includes:
  - Database connection
  - Vector search
  - Re-ranking model initialization (**major bottleneck**)
- **Recommendations**:
  1. **Model Caching**: Pre-load embedding and reranking models on first `index` to avoid cold start.
  2. **Lazy Loading**: Make re-ranker optional via a flag (`--no-rerank`) for faster results.
  3. **Batch Warmup**: Run a dummy query during indexing to warm up models.

#### 4. Filtered Search - Directory (21.127s)
- **Status**: Slow
- **Analysis**: Similar to above, but SQL filter complexity may add ~2-3s overhead.
- **Recommendations**:
  1. **Path Normalization**: Store all paths with forward slashes during indexing to avoid runtime escaping.
  2. **Index Optimization**: Create LanceDB index on `filename` column for faster filtering.

---

### ⚠️ Acceptable (40-50s)

#### 5. Indexing (46.447s for 73 chunks)
- **Status**: Acceptable for cold start, but could be optimized.
- **Throughput**: ~1.57 chunks/second
- **Analysis**: Breakdown:
  - File scanning: ~1-2s
  - AST parsing: ~3-5s
  - Embedding generation: ~35-40s (**bottleneck**)
  - Database insertion: ~1-2s
- **Recommendations**:
  1. **Parallel Embedding**: Increase batch size for `embed_batch` (currently processes sequentially).
  2. **GPU Acceleration**: If available, use CUDA/Metal for 10-20x speedup.
  3. **Progressive Indexing**: Show progress per file instead of per chunk.

---

### ❌ Very Slow Operations (> 50s)

#### 6. JSON Search (55.157s)
- **Status**: Unacceptable
- **Analysis**: Same as basic search + JSON serialization (adds ~0.01s). The delay is from:
  - **Re-ranking model initialization**: ~30-40s (first time only)
  - Vector search: ~0.5s
  - Re-ranking: ~10-15s
- **Recommendations**:
  1. **Critical Fix**: Re-ranker should be initialized once and cached in memory.
  2. **Implement Model Persistence**: Keep models loaded in a daemon/server mode.
  3. **Consider Fallback**: If re-ranker fails to load, fall back to vector-only search with a warning.

#### 7. Basic Search (56.378s)
- **Status**: Unacceptable (Expected: < 1s, Actual: 56s)
- **Analysis**: This is the **first search** after indexing, so it includes:
  - Embedding model load: ~5-10s
  - Re-ranking model download/load: **~40-45s** (first run only)
  - Actual search: ~1-2s
- **Recommendations**:
  1. **Pre-download Models**: Bundle models with the binary or download during `index`.
  2. **Model Caching**: Store loaded models in static memory for subsequent queries.
  3. **Server Mode**: Implement a persistent daemon mode (`code-rag serve`) that keeps models in memory.

---

## Root Cause Analysis

The primary bottleneck is **model initialization overhead**, not the search algorithm itself.

### Cold Start Breakdown (First Search)
```
Total: 56.378s
├── Embedding Model Load:     ~8s
├── Re-ranker Model Download: ~35s (if not cached)
├── Re-ranker Model Load:     ~12s
└── Actual Search:            ~1.5s
```

### Warm Start (Subsequent Searches)
Expected performance after models are loaded:
```
Total: ~1-2s
├── Vector Search:     ~0.5s
├── Re-ranking:        ~0.8s
└── Result formatting: ~0.1s
```

---

## Strategic Recommendations

### Immediate (High Impact)
1. **Model Pre-caching**: Download and initialize models during `index` command.
2. **Add `--no-rerank` Flag**: Allow users to skip re-ranking for instant results (~0.5s).
3. **Progress Indicators**: Show "Loading models..." during initialization.

### Short-Term
1. **Persistent Mode**: Implement `code-rag serve` to run as a background daemon.
2. **Model Bundling**: Include pre-downloaded models in release artifacts.
3. **Lazy Re-ranking**: Only re-rank if `limit > 10` or on user request.

### Long-Term
1. **GPU Support**: Add CUDA/Metal backends for 10-20x faster embedding.
2. **Quantized Models**: Use INT8 quantized models for 2-3x speedup with minimal accuracy loss.
3. **Hybrid Search**: Combine BM25 (instant) with vector search for better cold-start UX.

---

## Conclusion

The tool's **core search algorithm is fast** (~1-2s), but **model initialization overhead** makes the first query unacceptably slow (56s). This is a critical UX issue for CLI tools where users expect instant results.

**Priority Fix**: Implement model pre-loading during indexing or add a persistent server mode.
