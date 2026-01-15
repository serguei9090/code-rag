# serve

## Syntax
`code-rag serve [options]`

## Overview
Starts a persistent HTTP server that exposes the search and indexing functionality via a REST API. This is useful for building IDE plugins or other tools that need to query the codebase programmatically.

## Options
- `--port <PORT>`: Port to listen on (default: 3000)
- `--host <HOST>`: Host to bind to (default: 127.0.0.1)
- `--db-path <PATH>`: Custom database path

## Output
Server logs indicating the listening address and incoming requests.

## Examples

**Start on default port:**
```bash
code-rag serve
```

**Custom port and host:**
```bash
code-rag serve --port 8080 --host 0.0.0.0
```
