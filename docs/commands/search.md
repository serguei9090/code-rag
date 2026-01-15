# search

## Syntax
`code-rag search <QUERY> [OPTIONS]`

## Overview
Performs **Hybrid Search** (Vector + BM25) using embeddings and re-ranking.
1.  **Vector Search**: Finds semantic matches (meaning).
2.  **BM25 Search**: Finds exact keyword matches (usage).
3.  **Fusion**: Combines results using Reciprocal Rank Fusion (RRF).
4.  **Re-ranking**: Re-ranks top candidates using a Cross-Encoder for high precision.

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
