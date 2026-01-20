# Configuration Guide

`code-rag` uses a cascading configuration system that allows you to set defaults at multiple levels.

## Configuration Priority

Settings are loaded in this order (highest priority first):

1. **CLI Arguments** (e.g., `--db-path`)
2. **Local Config** (`./config_rag.toml` in current directory)
3. **Global Config** (`~/.config/code-rag/config_rag.toml`)
4. **Environment Variables** (prefix: `CODE_RAG_`)
5. **Built-in Defaults**

## Quick Start

Copy the example template to creating your own config:

```bash
cp code-rag.toml.example config_rag.toml
```

## Configuration Reference

### Core Paths

| Setting | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `db_path` | string | Location of the LanceDB database. | `./.lancedb` |
| `default_index_path` | string | Default directory to index. | `.` |

### Server Settings

| Setting | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `server_host` | string | Host address to bind the server to. | `127.0.0.1` |
| `server_port` | integer | Port to listen on. | `3000` |

### Indexing & Search

| Setting | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `default_limit` | size | Default number of search results. | `5` |
| `exclusions` | list | List of patterns to exclude (e.g., `["target", "node_modules"]`). | `[]` |
| `embedding_model` | string | Model for generating embeddings. | `nomic-embed-text-v1.5` |
| `reranker_model` | string | Model used for reranking results. | `bge-reranker-base` |
| `device` | string | Inference device: `auto`, `cpu`, `cuda`, `metal`. | `auto` |
| `chunk_size` | size | Size of text chunks for embedding. | `1024` |
| `chunk_overlap` | size | Overlap between chunks. | `128` |
| `max_file_size_bytes` | size | Skip files larger than this (default 10MB) to prevent OOM. | `10485760` |
| `merge_policy` | string | Index merge policy: `log`, `fast-write`, `fast-search`. | `log` |

### Resource Management

| Setting | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `batch_size` | size | Files to process per batch. Lower to reduce RAM. | `256` |
| `threads` | integer | Max threads for processing (null = auto). | `null` |
| `priority` | string | Process priority: `low`, `normal`, `high`. | `normal` |

### Logging

| Setting | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `log_level` | string | Log verbosity: `error`, `warn`, `info`, `debug`, `trace`. | `info` |
| `log_format` | string | Output format: `text` or `json`. | `text` |
| `log_to_file` | bool | Write logs to `logs/` directory. | `false` |
| `log_dir` | string | Directory for log files. | `logs` |

### Telemetry & LLM (Experimental)

| Setting | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `telemetry_enabled` | bool | Enable OpenTelemetry tracing. | `false` |
| `telemetry_endpoint` | string | OTLP endpoint URL. | `http://localhost:4317` |
| `llm_enabled` | bool | Enable LLM features (e.g., query expansion). | `false` |
| `llm_host` | string | LLM provider URL (e.g., Ollama). | `http://localhost:11434` |
| `llm_model` | string | LLM model name. | `mistral` |

## Example `config_rag.toml`

```toml
db_path = './.lancedb'
default_limit = 10
priority = 'low'
log_level = 'debug'
log_to_file = true
```

### Windows Path Handling

**Use single quotes** to avoid escape issues:

```toml
# ✅ Correct
db_path = 'C:\Users\MyUser\projects\db'

# ❌ Incorrect (Double quotes interpret backslashes as escapes)
db_path = "C:\Users\MyUser\projects\db"
```
