# AI Models in code-rag

This document explains where the AI models come from, how they are managed, and how to find their names.

## Frequently Asked Questions

### Where do the models come from?
The models are sourced from **Hugging Face**, a leading platform for machine learning models. We use the `fastembed-rs` library, which provides optimized ONNX versions of these models for local CPU execution.

### Does the application load them automatically?
**Yes.** When you run `code-rag index` or `code-rag search` for the first time with a specific model, the application will automatically:
1.  Check if the model is already in your local cache.
2.  If not, download the model files from Hugging Face.
3.  Load the model into memory.

### How does it know where to download from?
The `fastembed-rs` library contains an internal **Registry** (a mapping) that links human-readable names (like `nomic-embed-text-v1.5`) to specific **Hugging Face Repository IDs**.

When we call `TextEmbedding::try_new(options)`, the library looks up the specified model name in its registry, finds the corresponding Hugging Face URL, and uses the `hf-hub` (Hugging Face Hub) client to fetch the required ONNX and tokenizer files.

**Technical Implementation:**
In `src/embedding.rs`, the following lines trigger the automatic lookup and download:
- **Embedding Models:** [Line 41](file:///i:/01-Master_Code/Test-Labs/code-rag/src/embedding.rs#L41) (`TextEmbedding::try_new(options)?`)
- **Reranker Models:** [Line 74](file:///i:/01-Master_Code/Test-Labs/code-rag/src/embedding.rs#L74) (`TextRerank::try_new(options)?`)

Downloaded models are cached locally (typically in `C:\Users\<User>\.fastembed` on Windows or `~/.fastembed` on Linux/macOS) so they don't need to be downloaded again.

### Where can I find the names of supported models?
The currently supported model names for your `code-ragcnf.toml` are:

#### Embedding Models (for `embedding_model`)
These models convert your code into vectors.
*   **nomic-embed-text-v1.5** (Default) - High performance, 8192 token context.
*   **all-minilm-l6-v2** - Very fast and lightweight.
*   **bge-small-en-v1.5** - Good balance of speed and accuracy.
*   **bge-base-en-v1.5** - Higher accuracy than the small version.

#### Reranker Models (for `reranker_model`)
These models re-score search results for better precision.
*   **bge-reranker-base** (Default) - Highly effective for re-ranking code snippets.

> [!TIP]
> You can find these names and their descriptions in the [code-ragcnf.toml.template](file:///i:/01-Master_Code/Test-Labs/code-rag/code-ragcnf.toml.template) file. For a full list of models supported by the underlying library, visit the [FastEmbed Documentation](https://qdrant.github.io/fastembed/examples/Supported_Models/).

## Loading Models from Local Paths
If you have custom models or want to operate entirely offline (air-gapped), you can specify local directory paths in your configuration.

#### Using `code-ragcnf.toml`:
```toml
# Local paths override the model names above
embedding_model_path = "C:/models/nomic-embed-text-v1.5"
reranker_model_path = "C:/models/bge-reranker-base"
```

#### Requirements for Local Directories:
The specified directory must contain the following files:
- `model.onnx`: The actual model binary.
- `tokenizer.json`: Tokenizer configuration.
- `config.json`: Model configuration.
- `tokenizer_config.json`: Tokenizer-specific configuration.
- `special_tokens_map.json`: Mapping for special tokens (CLS, SEP, etc.).

When a path is provided, `code-rag` will bypass all Hugging Face checks and load the model directly from that directory.
