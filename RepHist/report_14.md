# Report 14: Phase 3 Roadmap & Status

This report describes the upcoming Phase 3 for the `code-rag` project, focusing on making the tool more autonomous and intelligent.

## Phase 3: Real-time Sync & Intelligent Querying

### 1. Real-time Indexing
- **Background Watcher**: Implementation of a file system watcher (using the `notify` crate) to detect file saves.
- **Auto-Sync**: Incremental re-indexing of modified files without user intervention.

### 2. Context-Aware Search
- **Metadata Filtering**: Adding flags to the search command to filter by extension or directory (e.g., `--ext rs` or `--dir src/utils`).
- **Query Expansion**: Heuristic-based expansion of short queries to improve hit rates for semantic search.

### 3. Agentic Integration
- **Persistent Mode**: A REPL-like session for faster interactive searches.
- **LSP Backend**: Exploring the possibility of exposing the search engine via LSP for IDE integration.

## Current Project Status
- **Phase 1 (Fixes)**: Completed.
- **Phase 2 (JSON & New Languages)**: Completed and verified with 49 tests.
- **Phase 3**: Planned and ready for implementation.
