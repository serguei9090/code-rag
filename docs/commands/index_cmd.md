# index

## Syntax
`code-rag index [PATH] [OPTIONS]`

## Overview
Scans a directory recursively, parses source files using Tree-sitter (streaming), extracts semantic chunks (functions, classes, modules), generates embeddings, and stores them in **LanceDB** (Vector) and **Tantivy** (BM25).

## Arguments
- `[PATH]`: Optional path to index. Defaults to `default_index_path` from config.

## Options
- `--db-path <PATH>`: Override database location (default: `./.lancedb`)
- `--update`: Incremental indexing mode. Only processes new or modified files based on `mtime`.
- `--force`: Deletes existing database and performs a fresh index.

## Output
Progress bars for scanning and embedding generation, followed by a completion summary.

## Examples

**Basic indexing:**
```bash
code-rag index ./my-project
```

**Incremental update:**
```bash
code-rag index --update
```

**Force re-index:**
```bash
code-rag index --force
```
