# Server Mode

## Overview
Server Mode allows `code-rag` to run as a persistent HTTP service. This enables external tools, IDE plugins, and other applications to perform code search and indexing operations programmatically via a REST API, without shelling out to the CLI for every request.

## Usage

To start the server, use the `serve` command:

```bash
code-rag serve --port 3000
```

The server will start listening on `http://127.0.0.1:3000` by default.

## API Endpoints

### `POST /search`
Perform a semantic search.

**Request Body:**
```json
{
  "query": "how is authentication handled?",
  "limit": 5,
  "db_path": optional string
}
```

**Response:**
Returns a JSON array of search results.

### `POST /index`
Trigger an indexing job.

**Request Body:**
```json
{
  "path": "/absolute/path/to/project",
  "force": false
}
```

### `GET /health`
Returns status 200 OK if the server is running.

## Key Components
- **Actix Web**: The web framework used for the HTTP server.
- **Shared State**: The `Storage` and `Embedder` instances are shared across requests for efficiency.

## Limitations
- **No Authentication**: The server currently does not support authentication. Ensure it is only exposed to trusted networks (localhost).
- **Single Threaded Indexing**: While search is concurrent, indexing operations may block heavily depending on file lock contention.
