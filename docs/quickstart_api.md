# Quick Start: API Usage with code-rag Server

This guide shows you how to use the `code-rag` HTTP API with curl, PowerShell, and API clients like Bruno or Postman.

## Starting the Server

```bash
# Start server with your workspace configuration
code-rag start --config code-rag.toml
```

The server will start on `http://localhost:3000` by default.

## Using the API

### Option 1: curl (Linux/macOS/Git Bash)

**Search in a specific workspace:**
```bash
curl -X POST http://localhost:3000/v1/whitsler2/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "llm configuration",
    "limit": 10,
    "ext": "py"
  }'
```

**Health check:**
```bash
curl http://localhost:3000/health
```

**Server status:**
```bash
curl http://localhost:3000/status
```

### Option 2: PowerShell (Windows)

**Search:**
```powershell
$body = @{
    query = "function bot"
    limit = 5
    ext = "py"
} | ConvertTo-Json

$response = Invoke-RestMethod `
  -Uri "http://localhost:3000/v1/whitsler2/search" `
  -Method Post `
  -ContentType "application/json" `
  -Body $body

$response.results | Format-Table filename, score
```

**Health check:**
```powershell
Invoke-RestMethod -Uri "http://localhost:3000/health"
```

### Option 3: Bruno/Postman

**Setup Collection:**
1. Create a new request
2. Set Method: `POST`
3. Set URL: `http://localhost:3000/v1/{{workspace}}/search`
4. Add Header: `Content-Type: application/json`
5. Set Body (JSON):
```json
{
  "query": "{{searchQuery}}",
  "limit": 10,
  "no_rerank": false,
  "ext": "{{fileExtension}}"
}
```

**Environment Variables:**
- `workspace`: `whitsler2`
- `searchQuery`: `llm configuration`
- `fileExtension`: `py`

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/v1/{workspace}/search` | POST | Search in specific workspace (recommended) |
| `/search` | POST | Search in default workspace (legacy) |
| `/health` | GET | Health check |
| `/status` | GET | Server statistics |
| `/metrics` | GET | Prometheus metrics (if telemetry enabled) |

## Request Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | Yes | - | Search query text |
| `limit` | integer | No | 10 | Maximum results to return |
| `no_rerank` | boolean | No | false | Skip reranking for faster search |
| `ext` | string | No | - | Filter by file extension ("py", "rs", "js") |
| `dir` | string | No | - | Filter by directory path |
| `max_tokens` | integer | No | - | Max tokens per result |
| `expand` | boolean | No | false | Enable query expansion |

## Response Format

```json
{
  "results": [
    {
      "filename": "./whitsler/ai/config.py",
      "score": 2.8451,
      "code": "class LLMConfig:\n    def __init__(self, ...):\n        ...",
      "start_line": 15,
      "end_line": 30
    }
  ]
}
```

## Error Handling

**Workspace not found:**
```json
{
  "error": "Workspace 'nonexistent' does not exist.\nAvailable workspaces: whitsler2, project-a\nRun 'code-rag index --path <path> --workspace nonexistent' to create this workspace."
}
```

## Examples

### Search for Python functions in workspace
```bash
curl -X POST http://localhost:3000/v1/whitsler2/search \
  -H "Content-Type: application/json" \
  -d '{"query": "async function", "ext": "py", "limit": 5}'
```

### Search in specific directory
```bash
curl -X POST http://localhost:3000/v1/whitsler2/search \
  -H "Content-Type: application/json" \
  -d '{"query": "bot configuration", "dir": "ai", "limit": 10}'
```

### Fast search (skip reranking)
```bash
curl -X POST http://localhost:3000/v1/whitsler2/search \
  -H "Content-Type: application/json" \
  -d '{"query": "llm", "no_rerank": true, "limit": 20}'
```

## Integration Examples

### JavaScript/Node.js
```javascript
const response = await fetch('http://localhost:3000/v1/whitsler2/search', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    query: 'authentication logic',
    limit: 10,
    ext: 'js'
  })
});

const data = await response.json();
console.log(data.results);
```

### Python
```python
import requests

response = requests.post(
    'http://localhost:3000/v1/whitsler2/search',
    json={
        'query': 'database connection',
        'limit': 10,
        'ext': 'py'
    }
)

results = response.json()['results']
for result in results:
    print(f"{result['filename']}: {result['score']}")
```

## Troubleshooting

**Server not responding:**
```bash
# Check if server is running
curl http://localhost:3000/health

# Check server status and loaded workspaces
curl http://localhost:3000/status
```

**Empty results:**
- Ensure the workspace exists: check `/status` endpoint
- Verify the workspace is indexed: `code-rag index --path <path> --workspace <name>`
- Try a broader query or remove filters (`ext`, `dir`)

**Slow searches:**
- Use `"no_rerank": true` to skip reranking
- Reduce `limit` value
- Consider indexing less data or splitting into multiple workspaces

## See Also

- [Server Mode Documentation](server_mode.md) - Full server documentation
- [Configuration Guide](../configuration/configuration.md) - Server configuration
- [Troubleshooting](../troubleshooting/empty_search_results.md) - Common issues
