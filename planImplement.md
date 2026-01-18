# Implementation Plan: Core Engine Enhancements

This document outlines the detailed implementation steps for the three high-priority enhancements approved for `code-rag`.

---

## 1. Streaming Chunking (Memory Optimization)

**Objective**: Eliminate OOM risks and scale indexing to multi-gigabyte files by processing code in buffered streams rather than loading full file contents into memory.

### Technical Steps
1.  **Refactor `CodeChunker`**:
    *   Modify `CodeChunker::chunk_file` (in `src/indexer.rs`) to accept a `impl AsyncRead + AsyncSeek` instead of a `String` or `Path`.
    *   Implementation of the `tree_sitter::Parser::parse_with` callback to read byte chunks from a `tokio::io::BufReader`.
2.  **Incremental Analysis**:
    *   Update the metadata extraction logic (function calls, imports) to run on individual nodes as they are discovered, rather than a secondary pass over a full string.
3.  **Buffer Management**:
    *   Implement a sliding window buffer (approx. 64KB) to ensure `Tree-sitter` has enough context for local parsing while keeping RAM usage constant.

**Files Affected**:
- `src/indexer.rs`
- `src/commands/index.rs`

**Verification**:
- Index a 100MB dummy source file and assert RSS (Resident Set Size) stays below 200MB.

---

## 2. Native Model Context Protocol (MCP) Support

**Objective**: Enable seamless integration with AI Agents (like Claude/ChatGPT) by implementing the MCP standard directly in the binary.

### Technical Steps
1.  **Add Dependencies**:
    *   Add `serde_json-rpc` and `tokio-util` (for Framed Read/Write over stdio).
2.  **MCP Command & Logic**:
    *   Create `src/commands/mcp.rs`.
    *   Implement JSON-RPC handlers for:
        *   `initialize`: Protocol versioning and capabilities.
        *   `resources/list`: List indexed workspaces.
        *   `tools/list`: Expose `search` and `index`.
        *   `tools/call`: Map `search` tool calls to `CodeSearcher::semantic_search`.
3.  **CLI Integration**:
    *   Update `src/main.rs` to include a `mcp` subcommand.

**Files Affected**:
- `Cargo.toml`
- `src/main.rs`
- `src/commands/mcp.rs` (New)
- `src/commands/mod.rs`

**Verification**:
- Use an MCP Inspector tool to successfully connect to `code-rag mcp` and execute a "search" tool call.

---

## 3. Multi-Workspace Isolation (Server)

**Objective**: Allow a single server instance to securely manage and search across multiple independent codebases.

### Technical Steps
1.  **Workspace Manager**:
    *   Implement a `WorkspaceManager` struct in `src/server.rs` that maintains a cache of `CodeSearcher` instances.
    *   Use `DashMap<String, Arc<Mutex<CodeSearcher>>>` for thread-safe, concurrent access to different project indices.
2.  **Middleware / Routing Update**:
    *   Update the Axum `Router` to extract a workspace identifier (e.g., `POST /v1/:workspace/search`).
    *   Modify `AppState` to hold the `WorkspaceManager`.
3.  **Dynamic Initialization**:
    *   Configure the server to automatically "discover" and load an index when first queried if it exists in the configured `.lancedb/` root.

**Files Affected**:
- `src/server.rs`
- `src/config.rs`
- `src/commands/serve.rs`

**Verification**:
- Index two different folders (`repo-a`, `repo-b`). Start server. Query `repo-a` and `repo-b` endpoints and verify results are strictly isolated to their respective repositories.

---

## Summary of Impact
| Feature | Primary Benefit | Target Audience |
| :--- | :--- | :--- |
| **Streaming** | Stability & Scalability | Enterprise/Monorepo Users |
| **MCP** | "Plug-and-Play" AI | LLM Agents & Power Users |
| **Isolation** | Resource Efficiency | Teams & Shared Infrastructure |
