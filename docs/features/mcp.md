# Model Context Protocol (MCP) Support

`code-rag` provides native support for the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/), allowing it to serve as a context provider for AI assistants like Claude Desktop, Cursor, and other IDEs.

## Overview

The MCP integration allows AI agents to "use" `code-rag` as a tool to find relevant code snippets in your project. It uses a **Hybrid Search** strategy (Semantic + Keyword) to provide high-quality context to the LLM.

## Usage

To start the MCP server, use:

```bash
code-rag mcp
```

This starts the server using the **stdio** transport (Standard Input/Output). The server reads the configuration and index from your local machine and communicates with the host application (like Claude Desktop) via JSON-RPC.

## Configuration

To use `code-rag` with an MCP-compatible client, you need to register it in the client's configuration.

### Claude Desktop Integration

#### 1. Standard Global Installation
Add `code-rag` to the `mcpServers` section using absolute paths:

```json
{
  "mcpServers": {
    "code-rag": {
      "command": "C:\\Program Files\\code-rag\\code-rag.exe",
      "args": ["mcp"]
    }
  }
}
```

#### 2. Project-Local Setup (Inside a Repository)
If you have the `code-rag.exe` and the `.toml` config file directly in your project root, use absolute paths pointing to that specific folder:

```json
{
  "mcpServers": {
    "code-rag": {
      "command": "C:\\Users\\Name\\Projects\\my-app\\code-rag.exe",
      "args": ["mcp"],
      "env": {
        "CODE_RAG_CONFIG": "C:\\Users\\Name\\Projects\\my-app\\code-rag.toml"
      }
    }
  }
}
```

> **Note**: While you might be tempted to use relative paths like `.\\code-rag.exe`, the MCP host (like Claude Desktop) usually needs **absolute paths** to reliably locate the executable and configuration, as its working directory may vary.

### Antigravity & Agentic AI Integration

When using **Antigravity** or other advanced agentic AI assistants, you can configure the MCP server within your workspace settings. This allows the agent to automatically discover and use the `search` tool to explore your codebase.

**Key Instructions for Agents:**
- **Context Discovery**: If the agent needs to understand a new codebase, its first step should be to list available tools and call `search`.
- **Iterative Search**: Agents should use the `search` tool to follow function calls or find implementations across different modules.
- **Project Structure**: Agents can ask `code-rag` about high-level concepts (e.g., "how is telemetry initialized?") and get specific file/line references immediately.

### Environment Variables
- `CODE_RAG_CONFIG`: Path to your `code-rag.toml` configuration file. In a project-local setup, this should point to the config file located in your project root.

## Available Tools

The MCP server currently exposes the following tool:

### `search`
Performs a hybrid semantic and keyword search over the indexed codebase.

**Arguments:**
- `query` (string, required): The natural language search query (e.g., "how is error handling implemented in the parser?").
- `limit` (integer, optional): Maximum number of results to return (default: 10).
- `workspace` (string, optional): The name of the workspace to search in (default: "default").

**Example Tool Call (Internal):**
```json
{
  "name": "search",
  "arguments": {
    "query": "authentication logic",
    "limit": 5
  }
}
```

## Protocol Implementation Details

- **Version**: Implementation follows the `2024-11-05` protocol version.
- **Capabilities**:
    - `tools`: Supports tool discovery and execution.
- **Methods**: Supports `initialize`, `notifications/initialized`, `tools/list`, and `tools/call`.

## Troubleshooting

- **Absolute Paths**: Always use absolute paths for the `command` and `CODE_RAG_CONFIG` in your JSON configuration.
- **Index Dependencies**: The MCP server can only search workspaces that have been previously indexed using the `code-rag index` command.
- **Logs**: Errors and initialization messages are written to `stderr`, which most MCP hosts (like Claude Desktop) will capture in their log files. `stdout` is reserved strictly for the MCP protocol.
