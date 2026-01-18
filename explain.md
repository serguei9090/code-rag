# Improvement Roadmap: Detailed Explanation

This document provides a deep dive into the **Gap Analysis & Improvements** proposed in the `product_report.md`. It outlines the specific technical changes, the functional benefits, and the resulting feature improvements for `code-rag`.

---

## 1. Native MCP Protocol Support

### 🛠️ The Proposed Change
Add a native JSON-RPC layer to the `code-rag` binary specifically for the **Model Context Protocol (MCP)**. This involves implementing a new command `code-rag mcp` that communicates via standard input/output (stdio) using the MCP schema.

### 🍱 What It Brings
- **Frictionless Integration**: LLM clients (like Claude Desktop or IDE extensions) can interact with `code-rag` directly as a "Tool" without needing intermediary scripts or shims.
- **Protocol Discovery**: Automatically exposes search and indexing capabilities to any MCP-compliant agent.

### 🚀 Feature Improvement
- **"Agent-Native" Engine**: Transforms `code-rag` from a standalone tool into a standardized plugin for the entire AI ecosystem, allowing any MCP host to "see" and "query" your local code without extra configuration.

---

## 2. Multi-Workspace Isolation (Server)

### 🛠️ The Proposed Change
Refactor the `AppState` in the Axum server to use a `HashMap` or a directory-backed lookup mechanism. Instead of loading one static index, the server will dynamically open or switch to different LanceDB/Tantivy indices based on a `X-Workspace-Name` header or a URL parameter.

### 🍱 What It Brings
- **SaaS-Ready Architecture**: Enables a single server instance to manage multiple separate projects (e.g., `workspace/project-a` and `workspace/project-b`) in isolation.
- **Resource Efficiency**: No need to start multiple server processes for different codebases.

### 🚀 Feature Improvement
- **Multi-Tenant Search**: Allows enterprise users or developers with many repositories to host a single centralized `code-rag` service that serves context for all their work independently.

---

## 3. Server-Side Authentication

### 🛠️ The Proposed Change
Implement a middleware layer in the `axum` server to validate `Authorization` headers. This could support a simple static `API_KEY` configured in `config_rag.toml` or more complex JWT (Json Web Token) validation.

### 🍱 What It Brings
- **Security for Shared Hosts**: Protects the search API from unauthorized access when the server is exposed on a local network or a development VM.
- **Data Privacy Enforcement**: Ensures that only authorized users/agents can retrieve potentially sensitive source code chunks.

### 🚀 Feature Improvement
- **Security-Hardened API**: Moves `code-rag` from a "local-only" hobbyist tool to a professional-grade microservice that can be safely integrated into team-wide infrastructure.

---

## 4. Git Metadata Integration

### 🛠️ The Proposed Change
Integrate the `git2-rs` crate (Rust bindings for `libgit2`) into the indexing pipeline. During chunking, the indexer will perform a `git blame` on the file and store the author, commit hash, and last modified date as metadata fields in LanceDB.

### 🍱 What It Brings
- **Ownership Context**: Search results will not just show *what* code was found, but *who* wrote it and *when* it was last changed.
- **Logic Tracing**: Helps agents understand the "recency" of code, allowing them to prioritize newer patterns or avoid deprecated logic.

### 🚀 Feature Improvement
- **"Living" Code Search**: Enables advanced queries like "Search for auth logic changed in the last 30 days" or "Show me code related to database-init written by developer-x".

---

## 5. Streaming Chunking (Memory Optimization)

### 🛠️ The Proposed Change
Replace the current "load-then-process" file indexing logic with a streaming approach. Using `Tokio`'s asynchronous I/O and buffered readers, the indexer will read files in small blocks and pass them through the Tree-sitter parser incrementally, rather than loading the entire file into a `String`.

### 🍱 What It Brings
- **Extreme Scalability**: Dramatically reduces the RAM footprint when indexing very large files (e.g., generated code, multi-MB vendor files).
- **No More OOM (Out Of Memory)**: Prevents the application from crashing when encountering massive text files on memory-constrained systems.

### 🚀 Feature Improvement
- **Industrial-Scale Indexing**: Allows `code-rag` to handle repositories containing high-volume data files or monoliths with million-line files that would typically choke a standard RAG pipeline.
