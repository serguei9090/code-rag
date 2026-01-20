# Code RAG – Local Code Search with Embeddings

![Project Status](https://img.shields.io/badge/Status-Active_Development-green)
![License](https://img.shields.io/badge/License-MIT-blue)
![Rust Version](https://img.shields.io/badge/rustc-1.75+-orange)
![Build](https://img.shields.io/badge/Build-Passing-brightgreen)

`code-rag` is a local-first **Semantic Code Search Engine** powered by LanceDB and local embeddings. It turns your codebase into a queryable knowledge base without sending a single line of code to the cloud.

---

## ⚡ Key Features

- **100% Local**: No cloud dependencies. Your code never leaves your machine.
- **Semantic Search**: Understands *concepts*, not just keywords (e.g., search for "auth flow" finds login functions).
- **Hybrid Reranking**: Combines BM25 and Vector search for high accuracy using `BGE-Reranker`.
- **Multi-Language**: Supports Rust, Python, TS/JS, Go, C++, Java, and more via Tree-sitter.
- **Server Mode**: Runs as a REST API for IDE plugins or other tools.
- **MCP Support**: Native implementation of the **Model Context Protocol** for AI Assistant integration.
- **Production Hardened**: OOM protection, stale file cleanup, and observability endpoints.

---

## 🚀 Quick Start

### 1. Installation

**Prerequisites:**
- [Rust](https://rustup.rs/) (1.75+)
- Git

```bash
git clone https://github.com/yourusername/code-rag.git
cd code-rag
cargo build --release
```

### 2. Basic Usage

**Index a repository:**
```bash
./target/release/code-rag index /path/to/project
```

**Search:**
```bash
./target/release/code-rag search "how is configuration loaded?"
```

---

## ⚙️ Core Commands

| Command | Description | Example |
| :--- | :--- | :--- |
| `index` | Scans and embeds code files. | `code-rag index .` |
| `search` | Semantic search query. | `code-rag search "db connection"` |
| `serve` | Starts REST API server. | `code-rag serve --port 3000` |
| `start` | Unified mode (Server + MCP + Watcher). | `code-rag start` |
| `grep` | Fast regex-based text search. | `code-rag grep "TODO:"` |

See [docs/commands](docs/commands/) for detailed CLI reference.

---

## 🏗️ Architecture

`code-rag` is built on a modular pipeline designed for performance and extensibility:

```mermaid
graph LR
    A[Source Files] --> B[Crawler]
    B --> C[Tree-sitter Chunker]
    C --> D[Embedding Model (ONNX)]
    D --> E[LanceDB (Vector Store)]
    E --> F[Search Engine]
    F --> G[Reranker]
    G --> H[Results]
```

### Key Components

- **Indexer**: Uses `ignore` crate for fast file walking, respecting `.gitignore`.
- **Chunker**: Language-aware splitting using `tree-sitter` to preserve context.
- **Embedder**: Runs `nomic-embed-text-v1.5` locally via `ort` (ONNX Runtime).
- **Database**: `LanceDB` for high-performance vector storage on disk.

---

## 🛠️ Developer Guide

### Development Environment

1.  **Dependencies**: Ensure you have the `onnxruntime` libraries or let `ort` download them automatically.
2.  **Config**: Copy the template to customize your dev environment.
    ```bash
    cp code-rag.toml.example code-rag.toml
    ```
3.  **Build Scripts**:
    - **Windows**: `.\build.ps1`
    - **Linux/Mac**: `./build.sh`

### Testing

We aim for comprehensive coverage including Unit, Integration, and E2E tests.

```bash
# Run all unit tests
cargo test

# Run specific integration test
cargo test --test integration -- verify_hardening
```

### Project Structure

- `src/commands/`: CLI command implementations.
- `src/server/`: Axum-based HTTP server and WorkspaceManager.
- `src/embedding/`: ONNX runtime integration.
- `tests/integration/`: Unified integration test suite.

---

## 📚 Documentation

Detailed documentation is available in the `docs/` directory:

- [Configuration Guide](docs/configuration/configuration.md)
- [Server Mode & API](docs/features/server_mode.md)
- [Model Context Protocol (MCP)](docs/features/mcp.md)
- [Supported Languages](docs/features/supported_languages.md)

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on code style, testing, and PR limits.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

