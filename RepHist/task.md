# Task: Implement Code-RAG V2 Phase 3 (Completed) & Phase 4

## Phase 3: Indexing Completeness (Completed)
- [x] **3.1 HTML & CSS Support**
    - [x] Create `test.html` and `test.css` for verification.
    - [x] Add `element`, `script_element`, `style_element` to `indexer.rs` (HTML).
    - [x] Add `rule_set`, `media_statement`, `keyframes_statement` to `indexer.rs` (CSS).
    - [x] Verification: Index `test.html` and confirm chunks are found.
- [x] **3.2 Top-Level Logic & Scripts**
    - [x] Create `script.py` and `app.js` (test.py) with top-level code.
    - [x] Add `lexical_declaration`, `variable_declaration` to `indexer.rs`.
    - [x] Add `if_statement` (for `__main__` blocks) to `indexer.rs`.
    - [x] Verification: Confirm global constants and scripts are indexed (8 chunks total).

## Phase 4: Advanced Intelligence (Active)
- [x] **4.1 HTML Reporting**
    - [x] Create `Report` command in CLI.
    - [x] Implement HTML template (using `minijinja` embedding).
    - [x] Generate `results.html` from search results.
    - [x] Verification: Run `search --html` and open file.
- [x] **4.2 Call Hierarchy**
    - [x] Capture function calls in `indexer.rs`.
    - [x] Store `calls` in LanceDB schema.
    - [x] Verification: Query for callers of a function (verified via report).
- [x] **4.3 Re-ranking**
    - [x] Research: Confirm `fastembed` support for Cross-Encoders.
    - [x] Implementation:
        - [x] Add `rerank` method to `Embedder`.
        - [x] Update `CodeSearcher` to use two-stage retrieval (top 50 -> rerank -> top N).
    - [x] Verification: Compare search scores before and after (Verified model download and execution).

## [x] **Phase 5: Extended Language Support**
    - [x] Add path-based indexing logic
    - [x] Integrate Tree-sitter parsers for Bash, PowerShell, YAML, JSON
    - [x] Add test assets for new languages
    - [x] Verify semantic chunking rules for scripts and configs
    - [x] Update documentation and enterprise report
- [x] **5.2 Documentation**
    - [x] Update `README.md` with new features (HTML report, Call Hierarchy, Reranking).
    - [x] Create `docs/` folder with deep-dive guides.
- [x] **5.3 Enterprise Report**
    - [x] Create `report.md` evaluating speed, accuracy, and standards.

## Future Work (Added to Plan)
- **Extended Language Support**: Bash, PowerShell, Dockerfile, YAML, TOML.
