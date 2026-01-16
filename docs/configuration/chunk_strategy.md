# Configurable Chunking Strategy

`code-rag` allows you to fine-tune how code is split into chunks for embedding. This is critical for optimizing retrieval recall and context window usage.

## Configuration

You can configure these settings in your `code-ragcnf.toml`:

```toml
# Maximum size of a single chunk in characters (default: 1024)
chunk_size = 1024

# Overlap between split chunks in characters (default: 128)
chunk_overlap = 128
```

## How it works

1.  **Semantic Chunking First**: The tool first attempts to split code by semantic boundaries (AST nodes) like functions, classes, and methods.
2.  **Size Check**: If a semantic chunk (e.g., a very long function) exceeds `chunk_size`, it is further split using a text splitter.
3.  **Overlap**: When splitting large chunks, `chunk_overlap` ensures that context is preserved at the boundaries of splits.

## Recommended Strategies

| Language | Recommended Size | Reasoning |
| :--- | :--- | :--- |
| **Python** | 1024 | Python functions are often concise. 1024 chars covers most methods. |
| **Rust** | 1500 | Rust syntax can be verbose (types, traits). Larger chunks help capture full context. |
| **Java** | 2000 | Java is very verbose. Larger chunks prevent splitting functions too aggressively. |
| **JavaScript** | 1024 | Standard split. |

## When to change these?

-   **Increase `chunk_size`** if:
    -   You find that search results are missing context (e.g., finding the middle of a function but not the start).
    -   Your embedding model supports large contexts (nomic-embed-v1.5 supports 8192 tokens).

-   **Decrease `chunk_size`** if:
    -   Retrieval results are too noisy (containing unrelated code).
    -   You want very specific, granular search results.
