# Code-RAG Feature Roadmap & Checklist

This document tracks the evolution of the `code-rag` project, detailing features from initial inception to the latest improvements.

## ✅ Completed Features

### 1. Foundation & Configuration
- [x] **Project Skeleton**: Rust CLI application structure using `clap` for argument parsing.
- [x] **Configuration System**: `config.rs` handles loading settings from `config.toml` (models, paths, timeouts).
- [x] **Logging**: structured logging with `tracing` and `tracing-subscriber`, capable of outputting to stderr to keep stdout clean for JSON piping.

### 2. Indexing Engine
- [x] **File Discovery**: `ignore` crate integration to walk directories while respecting `.gitignore`.
- [x] **AST Parsing**: `Tree-sitter` integration (`indexer.rs`) to parse code into Abstract Syntax Trees for robust chunking.
- [x] **Language Support**: Support for Rust, Python, JavaScript, TypeScript, Go, Java, C++, Shell, PowerShell, JSON, YAML, and more.
- [x] **Smart Chunking**: `CodeChunker` splits code based on semantics (functions, classes) and limits size with overlap.

### 3. Vector Database & Storage
- [x] **LanceDB Integration**: `storage.rs` manages the embedded Vector DB.
- [x] **Schema Definition**: Typed schema for `code_chunks` including filename, code, line numbers, and last_modified timestamps.
- [x] **Filename Indexing**: Scalar indexing on `filename` column to accelerate metadata filtering.
- [x] **Incremental Indexing**: Logic to verify file modification times (`mtime`) and only re-index changed files.

### 4. Embeddings & Reranking
- [x] **ONNX Runtime**: `embedding.rs` wraps `ort` to run distinct Embedding and Reranking models locally.
- [x] **Parallel Processing**: Batch processing of embeddings for performance.
- [x] **Reranker Integration**: Optional reranking step to refine vector search results using a cross-encoder model.

### 5. Search Capabilities
- [x] **Semantic Search**: Vector-based similarity search (ANN).
- [x] **BM25 Search**: `bm25.rs` implements traditional keyword-based probabilistic information retrieval.
- [x] **Hybrid Search**: Reciprocal Rank Fusion (RRF) algorithm combines Vector and BM25 scores for optimal relevance.
- [x] **Grep Command**: Regex-based text search as a fallback/utility (`search.rs`).
- [x] **Metadata Filtering**: CLI flags `--ext` and `--dir` to filter results by file extension or directory.

### 6. Advanced Features
- [x] **File Watcher**: `watcher.rs` monitors the file system for live changes and triggers re-indexing.
- [x] **HTTP Server**: `server.rs` allows the CLI to run as a backend API, serving search results over HTTP.
- [x] **HTML Reporting**: `reporting.rs` generates standalone HTML files visualizing search results.
- [x] **JSON Output**: Fully structured JSON output for integration with other tools/scripts.

### 7. Quality Assurance (Recent Improvements)
- [x] **E2E Test Suite**: `test_cli.ps1` covers 59 test cases across all commands.
- [x] **Debug Tooling**: `test_debug.ps1` allows isolated testing of specific components without running the full suite.
- [x] **Fix: Search Filtering**: Corrected BM25 result processing to strictly obey `--ext` and `--dir` filters (Resolved Pollution).
- [x] **Fix: Nested File Support**: Improved regex and discovery for deeply nested source files (e.g., Python packages).
- [x] **Fix: JSON Parsing**: Ensured CLI outputs valid JSON on stdout by properly redirecting logs to stderr.

## 🚀 Upcoming / Planned
- [ ] **Context Window Optimization**: Smarter context retrieval for LLM integration.
- [ ] **Multi-Workspace Support**: Managing multiple diverse codebases in a single DB.
- [ ] **LSP Integration**: Potential Language Server Protocol integration for IDEs.
- [ ] **GPU Acceleration (CUDA/Metal)**: Add support for GPU acceleration using CUDA (NVIDIA) and Metal (Apple) for faster embedding generation and search.
- [ ] **Query Expansion (Local LLM)**: Use a local LLM (e.g., via Ollama) to expand search queries with synonyms and related concepts before vector search.
