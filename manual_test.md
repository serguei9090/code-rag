# Manual Test Checklist: `code-rag`

This document outlines the manual verification steps to ensure the `code-rag` application is functioning correctly.

## 1. Setup & Build

- [X] **Build Debug Version**
  ```powershell
  cargo build
  ```
- [X] **Build Release Version** (Recommended for performance testing)
  ```powershell
  cargo build --release
  ```
- [X] **Verify Binary Help Message**
  ```powershell
  ./target/debug/code-rag --help
  ```

## 2. Indexing Workflow

- [X] **Initial Indexing**
  Index the current directory using default settings.
  ```powershell
  ./target/debug/code-rag index
  ```
- [X] **Custom Path Indexing**
  Index a specific subdirectory (e.g., `src`).
  ```powershell
  ./target/debug/code-rag index src
  ```
- [X] **Force Re-indexing**
  Force a full rebuild of the index, clearing existing data.
  ```powershell
  ./target/debug/code-rag index --force
  ```
- [X] **Incremental Update**
  Run an update and ensure only changed files are processed (check verbose logs or speed).
  ```powershell
  ./target/debug/code-rag index --update
  ```
- [X] **Custom DB Path**
  Index into a custom database location.
  ```powershell
  ./target/debug/code-rag index --db-path ./custom_db
  ```

## 3. Search Capabilities

- [X] **Basic Semantic Search**
  Search for a known concept in the codebase.
  ```powershell
  ./target/debug/code-rag search "indexing logic"
  ```
- [X] **JSON Output**
  Verify valid JSON output for integration with other tools.
  ```powershell
  ./target/debug/code-rag search "config" --json
  ```
- [X] **HTML Report**
  Generate `results.html` and verify it opens and renders correctly in a browser.
  ```powershell
  ./target/debug/code-rag search "struct definitions" --html
  ```
- [X] **Filter by Extension**
  Search only within Rust files.
  ```powershell
  ./target/debug/code-rag search "impl" --ext rs
  ```
- [X] **Filter by Directory**
  Search only within the `src` directory.
  ```powershell
  ./target/debug/code-rag search "main" --dir src
  ```
- [X] **No Re-rank**
  Perfom a faster search without the re-ranking step.
  ```powershell
  ./target/debug/code-rag search "error handling" --no-rerank
  ```

## 4. Grep Functionality

- [X] **Regex Search**
  Search using a regex pattern.
  ```powershell
  ./target/debug/code-rag grep "struct.*Config"
  ```
- [X] **Grep JSON Output**
  Verify grep results in JSON format.
  ```powershell
  ./target/debug/code-rag grep "TODO" --json
  ```

## 5. Server & API

- [ ] **Start Server**
  Start the HTTP server on the default port (3000).
  ```powershell
  ./target/debug/code-rag serve
  ```
- [ ] **Start Server on Custom Port**
  ```powershell
  ./target/debug/code-rag serve --port 8080
  ```
- [ ] **Test Health/Search Endpoint** (using curl or browser)
  *Requires server to be running.*
  ```powershell
  curl "http://localhost:3000/search?q=indexer"
  ```

## 6. Watcher Mode

- [ ] **Start Watcher**
  ```powershell
  ./target/debug/code-rag watch
  ```
- [ ] **Verify Auto-Index**
  With the watcher running:
  1. Modify a file (e.g., add a comment).
  2. Save the file.
  3. Verify the watcher logs an update event.
  4. Perform a search in another terminal to confirm the change is indexed.

## 7. Logging & Debugging

- [ ] **Verify File Logging (Client)**
  Run a command with logging enabled and check `logs/client.log`.
  ```powershell
  $env:CODE_RAG__LOG_TO_FILE="true"; ./target/debug/code-rag grep "test"
  Get-Content logs/client.log.* | Select-Object -Last 5
  ```
- [ ] **Verify File Logging (Server)**
  Start server with logging and check `logs/server.log`.
  ```powershell
  $env:CODE_RAG__LOG_TO_FILE="true"; ./target/debug/code-rag serve --port 9999
  # (Wait for startup, then CTRL+C)
  Get-Content logs/server.log.* | Select-Object -Last 5
  ```
- [ ] **Optimize Memory**
  Verify RAM usage remains low during indexing (check Task Manager during `index`).

## 8. Cleanup

- [ ] **Remove Database**
  Manually remove the `.lancedb` directory to reset the state.
  ```powershell
  Remove-Item -Recurse -Force .lancedb
  ```
