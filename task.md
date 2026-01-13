# Task: Implement Code-RAG V2 Phase 2

## Phase 2: Efficiency (Incremental Indexing)
- [ ] **2.1 Database Schema Update**
    - [x] Add `last_modified` (Int64) column to `code_chunks` table in `storage.rs`.
    - [x] Handle migration (or forced re-index warning) since schema changed. (Implicit via failure on old schema?)
- [ ] **2.2 Logic Implementation**
    - [x] Implement `get_indexed_files()` method in `Storage`.
    - [x] Implement `should_index(path, current_mtime)` logic in `main.rs` (via update flag).
- [x] **2.3 CLI Update**
    - [x] Add `--update` flag to `Index` command in `main.rs`.
    - [x] Integrate logic: If `--update` is set, skip unchanged files.
    - [x] Add `--force` flag to `Index` command to wipe DB and re-index.
- [ ] **2.4 Verification**
    - [x] Test fresh index (via --force).
    - [/] Test `--update` with no changes (0 chunks processed).
    - [ ] Test `--update` with modified file (re-chunked).
