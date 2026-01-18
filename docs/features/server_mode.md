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
- **Description**: Performs a sematic search in the `default` workspace.

**Request Body:**
```json
{
  "query": "authentication flow",
  "limit": 5,
  "no_rerank": false,
  "ext": "rs"
}
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

### 2. Isolated Workspace Search
New endpoint for targeting specific workspaces.

- **URL**: `POST /v1/{workspace}/search`
- **Description**: Performs a semantic search within the isolated storage of the specified `{workspace}`.
- **Path Parameters**:
  - `workspace`: Unique identifier for the workspace (e.g., `project-a`, `tenant-123`).

**Request Body:**
Same as `/search`.

**Behavior:**
- If the workspace database does not exist, the server attempts to open it (dynamic loading).
- Each workspace maintains its own independent LanceDB index structure.

### 3. Health Check
- **URL**: `GET /health`
- **Response**: `200 OK`

## Architecture & Isolation

The server uses a `WorkspaceManager` to handle isolation:
- **Root DB Path**: The `--db-path` argument points to a root directory.
- **Workspace Layout**: Each workspace gets a subdirectory: `$ROOT_DB_PATH/{workspace_id}/`.
- **Resource Sharing**: 
  - Heavy resources like the Embedding Model (ONNX) and Re-ranker are **shared** in memory across all workspaces.
  - Storage connections (LanceDB) are created per-workspace and cached.

## Limitations
- **No Authentication**: The server currently does not support authentication. Ensure it is only exposed to trusted networks (localhost).
