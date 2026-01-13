# Implementation Plan: Code-RAG V2

## Overview
This plan addresses the gaps identified in [gap.md](gap.md), moving `code-rag` from a prototype to a production tool.

## Phase 1: Usability & Configuration (Immediate)
*Goal: Enable flexible deployment and improve developer experience.*

### 1.1 Database Location Control ([High Priority](gap.md#2-database-location-control-db-path))
- [x] Add `--db-path` argument to `Args` struct.
- [x] Update `Storage::new` to accept custom paths.
- [x] **Verification:** Index a repo to a separate temp folder.

### 1.2 Configuration Management ([High Priority](gap.md#1-configuration-management-code-ragtoml))
- [x] Create `Config` struct (using `config` crate).
- [x] Load from `~/.code-rag/config.toml` -> `./code-rag.toml` -> Env Vars -> CLI Args.
- [x] **Verification:** Verify `default_index_path` is respected when running without args.

### 1.3 Visual Feedback ([UX](gap.md#4-visual-feedback-progress-bar))
- [x] Add `indicatif` dependency.
- [x] Wrap file walking and embedding loops with progress bars.
- [x] **Verification:** Run index on a large folder (e.g., node_modules) and watch the bar.

### 1.4 Result Pretty Printing ([UX](gap.md#5-result-pretty-printing))
- [x] Create `Display` impl for search results.
- [x] Use `colored` crate for syntax highlighting.
- [x] Format: Rank | Score | File:Line.
- [x] **Verification:** Run search and check legibility.

---

## Phase 2: Efficiency (Completed)
*Goal: Scale to large repositories.*

### 2.1 Incremental Indexing ([Performance](gap.md#3-incremental-indexing-update-mode))
- [x] Add schema column: `last_modified` (int).
- [x] Implement `should_index(path, current_mtime)` check against DB.
- [x] Add `--update` flag to `Index` command.
- [x] **Verification:** Index, modify 1 file, run `--update`, verify only 1 file processed.

### 2.2 Force Re-indexing ([Recovery](gap.md#2-database-location-control-db-path))
- [x] Add `--force` flag to `Index` command.
- [x] Implement database directory removal logic.
- [x] **Verification:** Run `index --force` and confirm fresh scan.

---

## Phase 3: Indexing Completeness (Completed)
*Goal: Ensure no code is left behind.*

### 3.1 HTML & CSS Support ([Coverage](gap.md#6-indexing-completeness))
- [x] Add `tree-sitter-html` node matching (elements, attributes).
- [x] Add `tree-sitter-css` node matching (rules, selectors).
- [x] **Verification:** Create `test.html` and confirm chunks are found.

### 3.2 Top-Level Logic & Scripts ([Coverage](gap.md#6-indexing-completeness))
- [x] Add support for Python global code (`if __main__`).
- [x] Index global constants and variables in Rust/JS.
- [x] **Verification:** Create `script.py` with only top-level code and verify indexing.

---

## Phase 4: Advanced Intelligence (Future)
*Goal: Improve retrieval accuracy and analysis.*

### 4.1 HTML Reporting ([UX](gap.md#6-html-report-viewer))
- [x] Generate static HTML with `minijinja` or similar.
- [x] Add `serve` command.

### 4.2 Call Hierarchy ([Intelligence](gap.md#7-call-hierarchy-awareness))
- [x] Update AST parsing to extract function calls.
- [x] Add `calls` list column to LanceDB.

### 4.3 Re-ranking ([Intelligence](gap.md#8-semantic-re-ranking))
- [x] Integrate cross-encoder model (e.g., via `ort` or `fastembed` if supported).
- [x] Implement two-stage pipelined search.

---

## Phase 5: Extended Language Support (Future)
*Goal: Support shell scripts and infrastructure-as-code.*

### 5.1 Shell Scripting
- [ ] Add `tree-sitter-bash` and `tree-sitter-powershell` (community).
- [ ] Implement AST traversal for shell commands and functions.

### 5.2 Configuration & Infrastructure
- [ ] Add `tree-sitter-dockerfile`, `tree-sitter-yaml`, `tree-sitter-json`.
- [ ] Implement chunking for declarative formats.
