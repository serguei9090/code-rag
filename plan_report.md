# Implementation Priority Plan: code-rag

This plan prioritizes improvements based on:
- **Performance Impact** (from timespend.md)
- **Implementation Complexity** (LOC estimate)
- **User Value** (from product_an.md)

Items are ordered from **simple → complex** and **few lines → many lines**.

---

## ⚡ Tier 1: Quick Wins (< 50 LOC)

### 1. Add `--no-rerank` Flag [Done]
- **Estimated LOC**: ~15 lines
- **Files**: `main.rs`, `search.rs`
- **Impact**: Reduces search time from 56s → ~1s (skips re-ranker initialization)
- **Complexity**: Very Low
- **Description**: Add optional flag to skip cross-encoder re-ranking for instant vector-only results.

### 2. Progress Indicators for Model Loading [Done]
- **Estimated LOC**: ~25 lines
- **Files**: `embedding.rs`, `search.rs`
- **Impact**: UX improvement (user knows what's happening during 40s wait)
- **Complexity**: Very Low
- **Description**: Display "Loading embedding model..." and "Initializing re-ranker..." messages.

### 3. Path Normalization During Indexing [Done]
- **Estimated LOC**: ~10 lines
- **Files**: `indexer.rs`
- **Impact**: Eliminates Windows path escaping complexity
- **Complexity**: Very Low
- **Description**: Convert all indexed paths to forward slashes (`/`) instead of backslashes.

---

## 🔧 Tier 2: Performance Fixes (50-200 LOC)

### 4. Model Pre-caching During Index [Done]
- **Estimated LOC**: ~80 lines
- **Files**: `main.rs`, `embedding.rs`
- **Impact**: First search drops from 56s → ~2s
- **Complexity**: Medium
- **Description**: Download and initialize models during `index` command, then run a warmup query.

### 5. Increase Embedding Batch Size [Done]
- **Estimated LOC**: ~30 lines
- **Files**: `embedding.rs`
- **Impact**: Indexing speed improves from 1.57 → ~3-5 chunks/s
- **Complexity**: Low
- **Description**: Process embeddings in larger batches (currently too conservative).

### 6. LanceDB Index on `filename` Column [deffered]
- **Estimated LOC**: ~40 lines
- **Files**: `storage.rs`
- **Impact**: Filtered searches drop from 21s → ~1-2s
- **Complexity**: Low
- **Description**: Add database index to speed up `WHERE filename LIKE` queries.

---

## 🚀 Tier 3: Architectural Improvements (200-500 LOC)

### 7. Persistent Server Mode (`code-rag serve`) [Done]
- **Estimated LOC**: ~350 lines
- **Files**: New `server.rs`, `main.rs`
- **Impact**: All searches become instant (~0.5-1s) after initial startup
- **Complexity**: High
- **Description**: HTTP/gRPC server that keeps models in memory. Clients query via API, this must be tailored to be used by mcp server, so we will need mcp server integration.
- **Dependencies**: `axum` or `tonic`

### 8. Hybrid BM25 + Vector Search
- **Estimated LOC**: ~280 lines
- **Files**: New `bm25.rs`, `search.rs`, `storage.rs`
- **Impact**: Better recall for exact keyword matches (e.g., error codes)
- **Complexity**: High
- **Description**: Combine full-text search (BM25) with vector similarity for hybrid retrieval.
- **Dependencies**: `tantivy` or custom BM25 implementation

---

## 🔮 Tier 4: Advanced Features (500+ LOC)

### 9. File System Watcher (Real-time Indexing)
- **Estimated LOC**: ~450 lines
- **Files**: New `watcher.rs`, `main.rs`, `indexer.rs`
- **Impact**: Auto-updates index when files change (no manual re-index)
- **Complexity**: Very High
- **Description**: Background process that watches file changes and incrementally updates DB.
- **Dependencies**: `notify` crate
- **Challenges**: Debouncing, error handling, cross-platform compatibility

### 10. LSP Integration
- **Estimated LOC**: ~800 lines
- **Files**: New `lsp/` module
- **Impact**: Semantic search directly in IDEs (VS Code, IntelliJ, etc.)
- **Complexity**: Very High
- **Description**: Implement Language Server Protocol to expose search as IDE feature.
- **Dependencies**: `tower-lsp`
- **Challenges**: Protocol compliance, state management, IDE-specific quirks

### 11. Query Expansion with Local LLM [For later this one is last]
- **Estimated LOC**: ~600 lines
- **Files**: New `query_expansion.rs`, `search.rs`
- **Impact**: Better fuzzy matching (e.g., "auth" → "authenticate, login, credential")
- **Complexity**: Very High
- **Description**: Use small local model (e.g., TinyLlama) to expand queries before embedding.
- **Dependencies**: `candle` or `llama-cpp-rs`
- **Challenges**: Model size, latency, prompt engineering

### 12. GPU Acceleration (CUDA/Metal)
- **Estimated LOC**: ~200 lines (config/setup)
- **Files**: `embedding.rs`, `Cargo.toml`
- **Impact**: 10-20x faster indexing and search
- **Complexity**: Very High
- **Description**: Enable GPU backends for FastEmbed and re-ranker.
- **Dependencies**: `ort` with CUDA/Metal features
- **Challenges**: Platform-specific builds, driver dependencies

---

## 📊 Summary Table

| # | Feature | LOC | Complexity | Performance Impact | User Value |
|---|---------|-----|------------|-------------------|------------|
| 1 | `--no-rerank` flag | 15 | ⭐ | ⚡⚡⚡ (56s → 1s) | High | ok
| 2 | Progress indicators | 25 | ⭐ | 📊 (UX only) | Medium | ok
| 3 | Path normalization | 10 | ⭐ | ⚡ (2-3s savings) | Low | ok
| 4 | Model pre-caching | 80 | ⭐⭐ | ⚡⚡⚡ (56s → 2s) | High | ok
| 5 | Batch size increase | 30 | ⭐ | ⚡⚡ (2-3x faster indexing) | Medium | ok
| 6 | DB index on filename | 40 | ⭐ | ⚡⚡ (21s → 2s) | High | 
| 7 | Server mode | 350 | ⭐⭐⭐ | ⚡⚡⚡ (all queries ~1s) | Very High |
| 8 | Hybrid BM25 search | 280 | ⭐⭐⭐ | ⚡ (better recall) | High |
| 9 | File watcher | 450 | ⭐⭐⭐⭐ | 📊 (convenience) | Medium |
| 10 | LSP integration | 800 | ⭐⭐⭐⭐⭐ | 📊 (new use case) | Very High |
| 11 | Query expansion | 600 | ⭐⭐⭐⭐ | ⚡ (better results) | Medium |
| 12 | GPU acceleration | 200 | ⭐⭐⭐⭐⭐ | ⚡⚡⚡ (10-20x faster) | High |

---

## Recommended Implementation Order

**Phase 1** (Immediate - 1-2 days):
1. `--no-rerank` flag
2. Progress indicators
3. Path normalization

**Phase 2** (Performance - 3-5 days):
4. Model pre-caching
5. Batch size increase
6. DB index on filename

**Phase 3** (Architecture - 1-2 weeks):
7. Server mode
8. Hybrid BM25 search

**Phase 4** (Advanced - 2-4 weeks):
9. File watcher
10. LSP integration

**Phase 5** (Experimental - open-ended):
11. Query expansion
12. GPU acceleration
