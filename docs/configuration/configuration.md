# Configuration Guide

`code-rag` uses a cascading configuration system that allows you to set defaults at multiple levels.

## Configuration Priority

Settings are loaded in this order (highest priority first):

1. **CLI Arguments** (e.g., `--db-path`)
2. **Local Config** (`./code-rag.toml` in current directory)
3. **Global Config** (`~/.code-rag/config.toml`)
4. **Environment Variables** (prefix: `CODE_RAG_`)
5. **Built-in Defaults**

## Configuration File Format

### Example `code-rag.toml`

```toml
# Database location (relative or absolute)
# IMPORTANT: Use single quotes for Windows paths!
db_path = './.lancedb'

# Default path to index when no argument is provided
default_index_path = '.'

# Optional: List of additional file extensions to index
# (Currently not implemented, reserved for future use)
# extensions = ['rs', 'py', 'js', 'ts']
```

### Windows Path Handling

**Correct:**
```toml
db_path = 'C:\Users\MyUser\projects\db'
default_index_path = 'I:\01-Master_Code\Test-Labs\code-rag\test_assets'
```

**Incorrect:**
```toml
db_path = "C:\Users\MyUser\projects\db"  # ❌ Double quotes cause escape issues
```

Use **single quotes** to avoid TOML escape sequence errors.

## Environment Variables

You can override any config value using environment variables:

```bash
export CODE_RAG_DB_PATH="./.custom-db"
export CODE_RAG_DEFAULT_INDEX_PATH="./src"
```

On Windows (PowerShell):
```powershell
$env:CODE_RAG_DB_PATH = "./.custom-db"
$env:CODE_RAG_DEFAULT_INDEX_PATH = "./src"
```

## Global Configuration

Create a global config file at:
- **Linux/macOS**: `~/.code-rag/config.toml`
- **Windows**: `C:\Users\<YourName>\.code-rag\config.toml`

This is useful for setting defaults across all projects.

## Configuration Template

See `code-rag.toml.example` in the project root for a complete template.

## Merge Policy

The `merge_policy` setting controls how the underlying search engine (Tantivy) handles index segments. This affects indexing speed and search latency.

| Policy | Description | Recommended Use Case |
| :--- | :--- | :--- |
| `log` | Default balanced policy. Good trade-off between write speed and read speed. Uses logarithmic merging. | General usage, read/write mixed workloads. |
| `fast-write` | optimized for indexing speed. Sets a larger minimum segment size (10 docs) to reduce merge frequency during heavy writes. | Bulk indexing, initial index creation, CI/CD pipelines. |
| `fast-search` | Optimized for search performance. Uses standard segment sizing to keep the index compact, potentially at the cost of slower indexing. | Read-heavy workloads, production servers where indexing is infrequent. |

Example:
```toml
merge_policy = "fast-write"
```


## Common Scenarios

### Scenario 1: Per-Project Database
```toml
# ./code-rag.toml
db_path = './.lancedb'
default_index_path = './src'
```

### Scenario 2: Centralized Database
```toml
# ~/.code-rag/config.toml
db_path = '~/.cache/code-rag/db'
```

### Scenario 3: CI/CD Environment
```bash
export CODE_RAG_DB_PATH="/tmp/code-rag-db"
code-rag index ./repo --force
code-rag search "security vulnerabilities" --html
```

## Troubleshooting

### "TOML parse error"
- Check for unescaped backslashes in paths
- Use single quotes for all path values
- Ensure no trailing backslashes

### "Config file not found"
- This is normal! Config files are optional.
- The tool will use built-in defaults if no config is found.
