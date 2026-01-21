# Server Mode

## Overview
Server Mode allows `code-rag` to run as a persistent HTTP service. This enables external tools, IDE plugins, and other applications to perform code search and indexing operations programmatically via a REST API.

The server now supports **Multi-Workspace Isolation**, allowing multiple projects or tenants to manage their own isolated vector databases within a single server instance.

## Usage

To start the server, use the `serve` command:

```bash
code-rag serve --port 3000 --db-path /path/to/data-root
```

The server will start listening on `http://127.0.0.1:3000` by default.

## API Endpoints

### 1. Default Workspace Search
Legacy endpoint for backward compatibility or single-workspace setups.

- **URL**: `POST /search`
- **Description**: Performs a semantic search in the `default` workspace.

**Request Body:**
```json
{
  "query": "authentication flow",
  "limit": 5,
  "no_rerank": false,
  "ext": "rs"
}
```

**curl Example:**
```bash
curl -X POST http://localhost:3000/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "authentication flow",
    "limit": 5,
    "no_rerank": false,
    "ext": "rs"
  }'
```

**Response:**
Returns a JSON object containing search results.
```json
{
  "results": [
    {
      "filename": "auth.rs",
      "score": 0.95,
      "code": "fn authenticate() { ... }"
    }
  ]
}
```

### 2. Isolated Workspace Search (Recommended)
New endpoint for targeting specific workspaces.

- **URL**: `POST /v1/{workspace}/search`
- **Description**: Performs a semantic search within the isolated storage of the specified `{workspace}`.
- **Path Parameters**:
  - `workspace`: Unique identifier for the workspace (e.g., `whitsler2`, `project-a`, `tenant-123`).

**Request Body:**
```json
{
  "query": "llm configuration",
  "limit": 10,
  "no_rerank": false,
  "ext": "py",
  "dir": "ai"
}
```

**curl Example:**
```bash
# Search in 'whitsler2' workspace
curl -X POST http://localhost:3000/v1/whitsler2/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "llm configuration",
    "limit": 10,
    "ext": "py"
  }'
```

**PowerShell Example:**
```powershell
$body = @{
    query = "llm configuration"
    limit = 10
    ext = "py"
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://localhost:3000/v1/whitsler2/search" `
  -Method Post `
  -ContentType "application/json" `
  -Body $body
```

**Bruno/Postman Collection:**
```
Method: POST
URL: http://localhost:3000/v1/{{workspace}}/search
Headers:
  Content-Type: application/json
Body (JSON):
{
  "query": "{{searchQuery}}",
  "limit": 10,
  "no_rerank": false
}

Variables:
  workspace: whitsler2
  searchQuery: llm configuration
```

**Request Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | The search query text |
| `limit` | integer | No | Maximum results (default: 10) |
| `no_rerank` | boolean | No | Skip reranking for faster search |
| `ext` | string | No | Filter by file extension (e.g., "py", "rs") |
| `dir` | string | No | Filter by directory path |

**Behavior:**
- If the workspace database does not exist, returns an error listing available workspaces
- Each workspace maintains its own independent LanceDB index structure

### 3. Health Check
- **URL**: `GET /health`
- **Response**: `200 OK`

**curl Example:**
```bash
curl http://localhost:3000/health
```

### 4. Server Status
- **URL**: `GET /status`
- **Description**: Returns statistics about loaded workspaces and active locks.

**curl Example:**
```bash
curl http://localhost:3000/status
```

**Response:**
```json
{
  "loaded_workspaces": 1,
  "active_ids": ["whitsler2"],
  "active_locks": 0
}
```


## Architecture & Isolation

The server uses a `WorkspaceManager` to handle isolation:
- **Root DB Path**: The `--db-path` argument points to a root directory.
- **Workspace Layout**: Each workspace gets a subdirectory: `$ROOT_DB_PATH/{workspace_id}/`.
- **Resource Sharing**: 
  - Heavy resources like the Embedding Model (ONNX) and Re-ranker are **shared** in memory across all workspaces.
  - Storage connections (LanceDB) are created per-workspace and cached.

## Observability and Metrics

The server exposes telemetry endpoints for monitoring and debugging when telemetry is enabled in the configuration.

### Available Endpoints

#### Metrics Endpoint
- **URL**: `GET /metrics`
- **Description**: Prometheus-compatible metrics endpoint
- **Status**: ✅ **Implemented** - Available in latest builds

**Current Metrics**:
- `app_memory_usage_bytes` - Current process memory consumption

**Example**:
```bash
curl http://localhost:3000/metrics
```

**Output**:
```
# HELP app_memory_usage_bytes Current RAM usage of the application process
# TYPE app_memory_usage_bytes gauge
app_memory_usage_bytes 45678912
```

**Setup**: See [Telemetry Guide](telemetry.md) for enabling observability stack (Jaeger + Prometheus + Grafana).

### Distributed Tracing

When `telemetry_enabled = true`, the server exports traces to Jaeger for request tracing.

**Access Jaeger UI**: http://localhost:16686

**Common Use Cases**:
- Debug slow search requests
- Trace workspace loading behavior
- Monitor concurrent request handling

See [Telemetry Configuration](../configuration/telemetry_config.md) for setup details.

## Limitations
- **No Authentication**: The server currently does not support authentication. Ensure it is only exposed to trusted networks (localhost).
