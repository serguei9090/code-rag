# search

## Syntax
`code-rag search <QUERY> [OPTIONS]`

## Overview
Performs semantic search using embeddings and re-ranking. Converts the query to a vector, retrieves top candidates from LanceDB, re-ranks them using a Cross-Encoder, and returns the most relevant results.

## Arguments
- `<QUERY>`: Natural language search query (required)

## Options
- `--limit <N>`: Number of results to return (default: 5)
- `--db-path <PATH>`: Override database location
- `--html`: Generate an HTML report (`results.html`)
- `--json`: Output results as JSON (for automation/CI/CD)
- `--ext <EXTENSION>`: Filter results by file extension (e.g., `rs`, `py`)
- `--dir <DIRECTORY>`: Filter results to files within a specific directory
- `--no-rerank`: Skip the re-ranking step for faster (but potentially less accurate) results

## Output
Ranked list of code chunks with file paths, line numbers, and relevance scores.

## Examples

**Basic search:**
```bash
code-rag search "how is authentication handled?"
```

**Skip re-ranking:**
```bash
code-rag search "quick lookup" --no-rerank
```

**JSON output:**
```bash
code-rag search "database setup" --json
```
