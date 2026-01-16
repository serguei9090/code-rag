# Task: Optimizing Indexing Performance

- [x] Search for Tantivy usage (Direct use in bm25.rs)
- [x] Investigate LanceDB integration (Not used for full-text search)
- [x] Locate index creation logic (`BM25Index::new`)
- [x] Determine if Merge Policy can be configured (Yes, using `set_merge_policy`)
- [ ] Update Implementation Plan for Configurable Policy <!-- id: 7 -->
- [ ] Add `merge_policy` to `AppConfig` <!-- id: 8 -->
- [ ] Apply Code Changes (`src/config.rs`, `src/bm25.rs`) <!-- id: 5 -->
- [ ] Verify Fix with Tests <!-- id: 6 -->
