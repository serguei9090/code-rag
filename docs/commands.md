# Code-RAG Command Reference

This document provides detailed information about all available commands in `code-rag`.

---

## `index` - Index Source Code

### Synopsis
```bash
code-rag index [PATH] [OPTIONS]
```

### Description
Scans a directory recursively, parses source files using Tree-sitter, extracts semantic chunks (functions, classes, modules), generates embeddings, and stores them in LanceDB.

### Arguments
- `[PATH]` - Optional path to index. Defaults to `default_index_path` from config.

### Options
- `--db-path <PATH>` - Override database location (default: `./.lancedb`)
- `--update` - Incremental indexing mode. Only processes new or modified files based on `mtime`.
- `--force` - Deletes existing database and performs a fresh index.

### Examples

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

**Custom database path:**
```bash
code-rag index ./src --db-path ./custom-db
```

### Expected Output
```
Indexing path: ./my-project
Scanning files...
⠁ Scanning: ./my-project/src/main.rs (42 chunks found) [00:00:03]
Scan complete. Found 42 chunks across 8 files.
Generating embeddings...
[████████████████████████████████████████] 42/42 (5s) Indexing complete.
```

### Performance
- **Speed**: ~100-500 files/second (scanning), ~50-100 chunks/second (embedding)
- **Memory**: ~200MB baseline + ~50MB per 1000 chunks
- **Disk**: ~1KB per chunk (vector + metadata)

---

## `search` - Semantic Code Search

### Synopsis
```bash
code-rag search <QUERY> [OPTIONS]
```

### Description
Performs semantic search using embeddings and re-ranking. Converts the query to a vector, retrieves top candidates from LanceDB, re-ranks them using a Cross-Encoder, and returns the most relevant results.

### Arguments
- `<QUERY>` - Natural language search query (required)

### Options
- `--limit <N>` - Number of results to return (default: 5)
- `--db-path <PATH>` - Override database location
- `--html` - Generate an HTML report (`results.html`)

### Examples

**Basic search:**
```bash
code-rag search "how is authentication handled?"
```

**Limit results:**
```bash
code-rag search "database connection" --limit 10
```

**Generate HTML report:**
```bash
code-rag search "error handling" --html
```

### Expected Output (Console)
```
Searching for: 'how is authentication handled?'

Rank 1 (Score: -2.3456)
File: src/auth.rs:45-67
---
pub fn authenticate_user(token: &str) -> Result<User, AuthError> {
    let decoded = decode_jwt(token)?;
    validate_claims(&decoded)
}
---

Rank 2 (Score: -3.1234)
File: src/middleware.rs:12-28
---
async fn auth_middleware(req: Request) -> Result<Response> {
    let token = extract_token(&req)?;
    authenticate_user(token).await
}
---
```

### Expected Output (HTML)
When using `--html`, a file `results.html` is generated with:
- Styled result cards
- Syntax-highlighted code snippets
- Call hierarchy tags
- Relevance scores

### Performance
- **Speed**: ~50-200ms for vector search, ~500-1000ms for re-ranking (first run includes model download)
- **Accuracy**: Re-ranking improves precision by ~15-30% over pure vector search

---

## `grep` - Text Pattern Search

### Synopsis
```bash
code-rag grep <PATTERN>
```

### Description
Performs exact text pattern matching using `ripgrep`. Respects `.gitignore` and `.ignore` files.

### Arguments
- `<PATTERN>` - Regex pattern to search for

### Examples

**Find function calls:**
```bash
code-rag grep "tokio::main"
```

**Find imports:**
```bash
code-rag grep "use std::"
```

### Expected Output
```
src/main.rs:5: use std::error::Error;
src/lib.rs:12: use std::path::Path;
src/config.rs:3: use std::fs;
```

### Performance
- **Speed**: ~1-5ms per file (depends on file size)
- Uses native `ripgrep` engine for maximum performance

---

## Configuration

See [configuration.md](configuration.md) for details on `code-rag.toml` and environment variables.

---

## Supported Languages

| Language | Extensions | AST Nodes Extracted |
|----------|-----------|---------------------|
| Rust | `.rs` | functions, impl blocks, structs, enums, modules |
| Python | `.py` | functions, classes, top-level scripts |
| Go | `.go` | functions, methods, type declarations |
| C/C++ | `.c`, `.cpp`, `.h`, `.hpp` | functions, classes, structs |
| JavaScript | `.js`, `.jsx` | functions, classes, arrow functions |
| TypeScript | `.ts`, `.tsx` | functions, classes, interfaces |
| Java | `.java` | methods, classes, interfaces |
| C# | `.cs` | methods, classes, interfaces |
| Ruby | `.rb` | methods, classes, modules |
| PHP | `.php` | functions, classes |
| HTML | `.html` | script elements, style elements |
| CSS | `.css` | rule sets, media queries, keyframes |

---

## Advanced Features

### Call Hierarchy
The indexer automatically extracts function calls from code chunks. These are displayed in search results and HTML reports.

**Example:**
```rust
pub fn process_data() {
    validate_input();  // ← Extracted
    transform();       // ← Extracted
}
```

Search results will show: `Calls: validate_input, transform`

### Re-ranking
Search uses a two-stage pipeline:
1. **Vector Search**: Fetch top 50 candidates using cosine similarity
2. **Cross-Encoder Re-ranking**: Use `BGE-Reranker-Base` to refine results
3. **Return**: Top N results with updated scores

This significantly improves relevance, especially for complex queries.

---

## Troubleshooting

### "No results found"
- Ensure you've run `code-rag index` first
- Check that `./.lancedb` exists
- Try `--force` to rebuild the index

### "Model download failed"
- Check internet connection (models download on first use)
- Models are cached in `~/.cache/fastembed/`

### "Out of memory"
- Reduce batch size by indexing smaller directories
- Use `--update` instead of full re-indexing

---

## See Also
- [Configuration Guide](configuration.md)
- [Architecture Overview](architecture.md)
