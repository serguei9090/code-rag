# Model Context Protocol (MCP) Support
`code-rag` provides native support for the [Model Context Protocol (MCP)](https://github.com/model-context-protocol/mcp), allowing it to serve as a context provider for AI assistants like Claude, Cursor, and others.

## Features
- **Semantic Search Tool**: Exposes `search` tool via MCP, enabling AI agents to search your indexed codebase.
- **Project Context**: (Planned) Exposing file reading capabilities directly if needed, though agents usually have their own.

## Usage
To start the MCP server, simply run:
```bash
code-rag mcp
```
This will start the server on `stdio`, which is the standard transport for MCP.

## Integration
### Claude Desktop
Add the following to your `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "code-rag": {
      "command": "absolute/path/to/code-rag",
      "args": ["mcp"]
    }
  }
}
```

### Supported Tools
- **search**:
  - `query` (string): The search query.
  - `limit` (number, optional): Max results (default 10).
  - `workspace` (string, optional): Workspace to search in.

## Protocol Implementation
- Implementation follows MCP 2024-11-05 draft.
- Supports `initialize`, `notifications/initialized`, `tools/list`, `tools/call`.
