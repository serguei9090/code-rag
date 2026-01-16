# Task: Optimizing Indexing Performance

- [x] Search for Tantivy usage (Direct use in bm25.rs)
- [x] Investigate LanceDB integration (Not used for full-text search)
- [x] Locate index creation logic (`BM25Index::new`)
- [x] Determine if Merge Policy can be configured (Yes, using `set_merge_policy`)
- [x] Update Implementation Plan for Configurable Policy <!-- id: 7 -->
- [x] Add `merge_policy` to `AppConfig` <!-- id: 8 -->
- [x] Apply Code Changes (`src/config.rs`, `src/bm25.rs`) <!-- id: 9 -->
- [x] Verify Fix with Tests <!-- id: 10 -->
- [x] Update `code-rag.toml.example` with `merge_policy` <!-- id: 11 -->
- [x] Update `docs/configuration/configuration.md` with `merge_policy` details <!-- id: 12 -->

