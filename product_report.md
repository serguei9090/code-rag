# Product Report: code-rag

## 📊 Executive Summary
`code-rag` is a high-performance, local-first Code RAG (Retrieval-Augmented Generation) engine built in Rust. It enables developers and AI agents to semantically search, index, and analyze large codebases without relying on cloud-based services. By combining traditional keyword search (BM25) with modern vector embeddings and cross-encoder re-ranking, it provides state-of-the-art retrieval accuracy with 100% data privacy.

---

## 🛠️ Feature Matrix & Capabilities

### 🔎 Search Performance
- **Hybrid Retrieval**: Combines **LanceDB** (Vector Search) and **Tantivy** (BM25) for high-precision results covering both semantic meaning and exact keywords.
- **Cross-Encoder Re-ranking**: Uses `BGE-Reranker` to refine top results, significantly reducing "hallucination" precursors by providing the most relevant code context.
- **Dynamic Context Optimization**: Automatically merges adjacent code chunks to fit within a specific token budget (e.g., 8000 tokens), ideal for feeding directly into LLMs.

### 🧠 AI & Embedding Engine
- **Local ONNX Execution**: Powered by `ort` and `fastembed`, ensuring models run efficiently on CPU, CUDA (Nvidia), or Metal (Apple Silicon).
- **Default Models**:
    - **Embeddings**: `NomicEmbedTextV15` (high-performance retrieval).
    - **Re-ranking**: `BGE-Reranker-Base`.
- **Query Expansion**: Integrates with **Ollama** to expand user queries into multiple search variants, capturing better semantic coverage.

### 📂 Language Support
Comprehensive parsing using **Tree-sitter** for:
- Rust, Python, Go, JS/TS, C/C++, Java, C#, Ruby, PHP, HTML, CSS, Bash, PowerShell, YAML, JSON, Zig, Elixir, Haskell, Solidity.

---

## 🏗️ Architecture & Construal for Use Cases

### 🤖 Agentic Use (AI Integration)
`code-rag` is intentionally designed as a "building block" for AI Agents:
- **CLI JSON Mode**: Tools can call `code-rag search "..." --json` to get machine-readable context instantly.
- **Restful API**: The Axum-based server provides high-concurrency access for multi-agent workflows.
- **Call Hierarchy Extraction**: Automatically identifies function calls and usages, allowing agents to "walk" the codebase logic.

### 👤 Manual Developer Use
- **Interactive CLI**: Rich terminal output with highlighted code snippets and scores.
- **HTML Reporting**: Generate visual search reports (`--html`) for sharing or auditing.
- **Live Watcher**: Background re-indexing via `code-rag watch` ensures the search results stay in sync with your edits.

---

## 🌐 API & Server Integration

The integrated Axum server provides a robust programmatic interface:

### Endpoints
| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/search` | `POST` | Semantic search with filtering (ext, dir) and query expansion. |
| `/health` | `GET` | Service status check. |
| `/metrics` | `GET` | Prometheus-compatible metrics for monitoring. |

### Technical Stack
- **Framework**: `Axum` (Tokio-native, high performance).
- **Observability**: Built-in `OpenTelemetry` support with OTLP exporting for traces and metrics (Jaeger/Prometheus).

---

## 🔌 MCP Server Status
**Model Context Protocol (MCP)** support is currently in a "Pre-Native" state:
- **Compatibility**: Highly compatible via a simple shim or wrapper that converts MCP stdio/HTTP requests into `code-rag` CLI or REST API calls.
- **Roadmap**: Native integration of the MCP protocol is a primary gap/opportunity (see below).

---

## 📉 Gap Analysis & Improvements

### 1. Native MCP Protocol Support
- **Issue**: Requires a wrapper for direct integration with LLM clients like Claude Desktop.
- **Improvement**: Implement native JSON-RPC MCP over stdio directly in a new `code-rag mcp` command.

### 2. Multi-Workspace Isolation in Server
- **Issue**: The current server implementation is optimized for a single "global" or "default" workspace.
- **Improvement**: Extend the API to support dynamic workspace switching via request headers or URL parameters.

### 3. Server-Side Authentication
- **Issue**: No built-in authentication for the REST API.
- **Improvement**: Implement API Key or Bearer Token support for shared environment deployments.

### 4. Git Integration
- **Issue**: Search results don't currently include Git metadata (Commit hash, author, date).
- **Improvement**: Use `libgit2` to annotate search results with `git blame` data.

### 5. Memory Management (Indexing)
- **Issue**: High memory usage during very large file processing (path-based).
- **Improvement**: Implement streaming chunking to handle multi-GB files without loading full content into RAM.

---

## 📈 Code Quality Assessment
- **Safety**: 100% Safe Rust implementation (No `unsafe` blocks used in core logic).
- **Correctness**: Comprehensive integration suite spanning CLI, Resilience, and Logic.
- **Maintainability**: Modular design with clear separation between storage (`LanceDB`), indexing (`Tree-sitter`), and search.
- **Idioms**: Strictly follows Rust best practices (documented in `.agent/rules/rust_bp.md`).
