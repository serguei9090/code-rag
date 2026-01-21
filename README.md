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

> **New to code-rag?** Check out the [Fast Start Guide](docs/quickstart.md) for a step-by-step walkthrough of commands and configuration.

### 1. Installation

**Prerequisites:**
- [Rust](https://rustup.rs/) (1.75+)
- Git

```bash
git clone https://github.com/yourusername/code-rag.git
cd code-rag
cargo build --release
```

### 2. Hardware Acceleration

#### NVIDIA GPU (CUDA)

**Requirements**:
- [CUDA Toolkit 11.8+](https://developer.nvidia.com/cuda-downloads)
- [cuDNN 8.x](https://developer.nvidia.com/cudnn)

**Build Command**:
```bash
cargo build --release --features cuda
```

**Required DLLs (Windows)**:

The following DLLs must be in your system PATH or in the same directory as `code-rag.exe`:

- `cudart64_11.dll` - CUDA Runtime (from CUDA Toolkit)
- `cublas64_11.dll` - CUDA Basic Linear Algebra Subroutines
- `cublasLt64_11.dll` - CUDA Lightweight BLAS
- `cudnn64_8.dll` - cuDNN Deep Neural Network library

**Typical Installation Paths (Windows)**:
```
C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v11.8\bin
C:\Program Files\NVIDIA\CUDNN\v8.x\bin
```

**Add to PATH (Windows PowerShell)**:
```powershell
$env:PATH += ";C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v11.8\bin"
$env:PATH += ";C:\Program Files\NVIDIA\CUDNN\v8.x\bin"
```

**Required Libraries (Linux)**:

The following `.so` files must be in your `LD_LIBRARY_PATH` or `/usr/local/lib`:

- `libcudart.so.11` - CUDA Runtime
- `libcublas.so.11` - CUDA BLAS
- `libcublasLt.so.11` - CUDA Lightweight BLAS  
- `libcudnn.so.8` - cuDNN

**Typical Installation Paths (Linux)**:
```
/usr/local/cuda-11.8/lib64
/usr/local/cuda/lib64
```

**Add to LD_LIBRARY_PATH (Linux)**:
```bash
export LD_LIBRARY_PATH=/usr/local/cuda-11.8/lib64:$LD_LIBRARY_PATH
export LD_LIBRARY_PATH=/usr/local/cudnn/lib64:$LD_LIBRARY_PATH
```

**Troubleshooting**:
- **DLL Not Found Error**: Verify all DLLs are in PATH using `where cudart64_11.dll` (Windows) or `ldconfig -p | grep libcudart` (Linux)
- **Version Mismatch**: Ensure CUDA Toolkit and cuDNN versions match (both 11.x)
- **Installation Guide**: See [CUDA Installation Guide](https://docs.nvidia.com/cuda/cuda-installation-guide-microsoft-windows/) (Windows) or [Linux Guide](https://docs.nvidia.com/cuda/cuda-installation-guide-linux/)

---

#### macOS (Metal)

Enabled by default on Apple Silicon, but can be made explicit:
```bash
cargo build --release --features metal
```

**No additional dependencies required** - Metal is part of macOS.


### 3. Basic Usage

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

