# Implementation Plan: Core Engine Enhancements

## Phase 1: Streaming Chunking (Memory Optimization)
- [x] Refactor `CodeChunker` logic in `src/indexer.rs` to support `AsyncRead` (or `BufRead` for streaming).
    - [x] Change `chunk_file` signature to accept `impl Read + Seek`.
    - [x] Update `tree-sitter` parsing to use stream-based parsing.
- [x] Update `src/commands/index.rs` to pass file stream to chunker instead of loading full string.
- [x] Verify: Index a 100MB dummy source file and assert RSS (Integration Test).
    - [x] Create proper tests for the implemented features.
- [x] Update `README.md` and documentation (architecture, features).

## **Phase 2: Native Model Context Protocol (MCP) Support** [x]
- [x] Add `serde_json-rpc` (or manual JSON-RPC implementation) and `tokio-util` dependencies.
- [x] Create `src/commands/mcp.rs` implementing the MCP server loop over stdio.
- [x] Implement MCP handlers: `initialize`, `resources/list`, `tools/list`, `tools/call` (wrapping `search` command).
- [x] Update `src/main.rs` to expose `code-rag mcp` subcommand.
- [x] Verify with an MCP Inspector (via automated integration test).
- [x] Update Documentation
- [x] Update README.md

## Phase 3: Multi-Workspace Isolation (Server)
- [ ] Implement `WorkspaceManager` in `src/server.rs`
- [ ] Update Axum Router for workspace isolation
- [ ] Modify `AppState` to include `WorkspaceManager`
- [ ] Implement Dynamic Initialization
- [ ] Verify isolation with multiple repos
- [ ] Update `code-rag.toml.example` (if config changes needed)
- [ ] Update Documentation
- [ ] Update README.md

