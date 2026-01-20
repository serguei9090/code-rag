# Quick Start Guide

This guide will get you up and running with `code-rag` in minutes. It covers the essential commands, configuration setup, and recommended settings for a smooth start.

## 1. Fast Track (CLI Only)

If you just built the binary and want to try it out immediately without config files:

### Step 1: Index your Code
Navigate to your project root and run the indexer. This scans your files, splits them into chunks, and generates embeddings.

```bash
# Basic indexing (uses default ~/.cache/huggingface for models)
code-rag index /path/to/your/repo
```

### Step 2: Search
Once indexing is complete, you can search immediately using natural language.

```bash
# Semantic search
code-rag search "how does authentication work"
```

### Step 3: Start the Server (Optional)
If you want to use the API or integrate with an IDE plugin:

```bash
code-rag serve
```

---

## 2. Configuration Setup (Recommended)

For serious usage, you should configure the application to persist your settings and tune performance.

### Step 1: Create Config File
Copy the example configuration to your project root or config directory.

```bash
cp code-rag.toml.example code-rag.toml
```

### Step 2: Customizing `code-rag.toml`

The configuration file controls everything from database paths to model selection. Here are the key sections:

#### Core Paths
Define where your vector database lives.
```toml
# Use a hidden folder in your project or a global path
db_path = "./.lancedb" 
```

#### Performance Settings
Adjust these based on your machine's power.
```toml
# RAM usage vs Speed trade-off
# Larger batches = faster indexing but more RAM
batch_size = 256 

# Threading
# Leave commented out for auto-detection, or set explicitly
# threads = 8
```

---

## 3. Recommended "Golden" Settings

For a typical mid-sized project (e.g., 50k - 500k loc) on a modern laptop (16GB+ RAM), use these settings in your `code-rag.toml`:

```toml
[core]
# Keep the DB local to the project for portability
db_path = "./.lancedb"

[indexing]
# Efficient chunking for code context
chunk_size = 1024
chunk_overlap = 128
# Exclude build artifacts to keep unrelated noise out
exclusions = ["target", "node_modules", "dist", ".git", ".idea"]

[models]
# proven balance of speed and accuracy
embedding_model = "nomic-embed-text-v1.5"
reranker_model = "bge-reranker-base"
# Auto-detects CUDA/Metal if available
device = "auto"

[search]
# Hybrid search weights (Vector + BM25)
vector_weight = 0.7
bm25_weight = 0.3
limit = 10
```

---

## 4. Next Steps

- **REST API**: See [Server Mode Docs](features/server_mode.md) to integrate with other tools.
- **MCP**: See [MCP Docs](features/mcp.md) to connect with AI assistants like Claude Desktop or Cursor.
