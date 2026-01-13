# Implementation Plan: Code-RAG V2

## Overview
This plan addresses the gaps identified in [gap.md](gap.md), moving `code-rag` from a prototype to a production tool.

## Phase 1: Usability & Configuration (Immediate)
*Goal: Enable flexible deployment and improve developer experience.*

### 1.1 Database Location Control ([High Priority](gap.md#2-database-location-control-db-path))
- [ ] Add `--db-path` argument to `Args` struct.
- [ ] Update `Storage::new` to accept custom paths.
- [ ] **Verification:** Index a repo to a separate temp folder.

### 1.2 Configuration Management ([High Priority](gap.md#1-configuration-management-code-ragtoml))
- [ ] Create `Config` struct (using `config` crate).
- [ ] Load from `~/.code-rag/config.toml` -> `./code-rag.toml` -> Env Vars -> CLI Args.
- [ ] **Verification:** Verify `default_index_path` is respected when running without args.

### 1.3 Visual Feedback ([UX](gap.md#4-visual-feedback-progress-bar))
- [ ] Add `indicatif` dependency.
- [ ] Wrap file walking and embedding loops with progress bars.
- [ ] **Verification:** Run index on a large folder (e.g., node_modules) and watch the bar.

### 1.4 Result Pretty Printing ([UX](gap.md#5-result-pretty-printing))
- [ ] Create `Display` impl for search results.
- [ ] Use `colored` crate for syntax highlighting.
- [ ] Format: Rank | Score | File:Line.
- [ ] **Verification:** Run search and check legibility.

---

## Phase 2: Efficiency (Next Steps)
*Goal: Scale to large repositories.*

### 2.1 Incremental Indexing ([Performance](gap.md#3-incremental-indexing-update-mode))
- [ ] Add schema column: `last_modified` (int).
- [ ] Implement `should_index(path, current_mtime)` check against DB.
- [ ] Add `--update` flag to `Index` command.
- [ ] **Verification:** Index, modify 1 file, run `--update`, verify only 1 file processed.

---

## Phase 3: Advanced Intelligence (Future)
*Goal: Improve retrieval accuracy and analysis.*

### 3.1 HTML Reporting ([UX](gap.md#6-html-report-viewer))
- [ ] Generate static HTML with `minijinja` or similar.
- [ ] Add `serve` command.

### 3.2 Call Hierarchy ([Intelligence](gap.md#7-call-hierarchy-awareness))
- [ ] Update AST parsing to extract function calls.
- [ ] Add `calls` list column to LanceDB.

### 3.3 Re-ranking ([Intelligence](gap.md#8-semantic-re-ranking))
- [ ] Integrate cross-encoder model (e.g., via `ort` or `fastembed` if supported).
- [ ] Implement two-stage pipelined search.
