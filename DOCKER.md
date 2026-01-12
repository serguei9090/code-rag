# Docker Build Guide

## Quick Start

### Build the Image
```bash
docker-compose build
```

### Run Commands

**Index a directory:**
```bash
docker-compose run --rm code-rag index /workspace
```

**Search:**
```bash
docker-compose run --rm code-rag search "authentication logic" --limit 5
```

**Grep:**
```bash
docker-compose run --rm code-rag grep "struct User"
```

## Manual Docker Commands

**Build:**
```bash
docker build -t code-rag:latest .
```

**Run:**
```bash
# Index current directory
docker run --rm -v "$(pwd):/workspace:ro" -v lancedb-data:/data/.lancedb code-rag:latest index /workspace

# Search
docker run --rm -v lancedb-data:/data/.lancedb code-rag:latest search "query" --limit 5
```

## Notes
- First build downloads ~500MB of model files (cached in image)
- LanceDB data persists in named volume `lancedb-data`
- Source code is mounted read-only for safety
