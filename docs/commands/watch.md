# Watch Command

The `watch` command starts a background process that monitors the target directory for file changes (creation, modification, deletion) and updates the index in real-time.

## Usage

```bash
code-rag watch [OPTIONS] [PATH]
```

### Arguments

- `[PATH]`: The directory to watch. Defaults to the current directory or the `default_index_path` configured in `code-ragcnf.toml`.

### Options

- `--db-path <DB_PATH>`: Custom path to the LanceDB database.

## Behavior

1.  **Startup**: Initializes the models and opens the database.
2.  **Monitoring**: Uses file system events (debounced by 2 seconds) to detect changes.
3.  **Updates**:
    -   **New/Modified File**: Re-chunks, embeds, and indexes the file, replacing any old chunks.
    -   **Deleted File**: Removes all chunks and BM25 entries associated with the file.
4.  **Exclusions**: Respects `.gitignore` and global exclusions defined in configuration.

## Example

```bash
# Watch the current directory
code-rag watch

# Watch a specific project
code-rag watch ./my-project

# Use a custom database
code-rag watch --db-path ./custom.lancedb
```
