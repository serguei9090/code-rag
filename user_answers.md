# Answers to Your Questions

Here are the detailed answers to your questions based on a review of the codebase.

## Case 1: Indexing Update Behavior

**Question:** "When I do update it index all again? check the code give me result"

**Answer:** **No, it does NOT index everything again.**

However, the "Processing [filename]" log message is currently misleading.

*   **Code Analysis:** In `src/main.rs`, the code prints `Processing [filename]` **before** it checks the file's modification time.
*   **Actual Logic:**
    1.  The loop starts and prints "Processing...".
    2.  Use check: `if let Some(stored_mtime) = existing_files.get(&fname_str)`.
    3.  Time check: `if *stored_mtime == mtime { continue; }`.
    4.  If the file hasn't changed, it strictly **skips** the heavy operations (reading, chunking, embedding, and indexing).

So, while you *see* the log message scrolling by, the underlying expensive work is correctly skipped for unchanged files.

## Case 2: Ranking (Indexing vs. Search)

**Question:** "When we index the also do ranking? yes or no if yes we also do ranking when we use search?"

**Answer:**
*   **Indexing:** **NO.** We only generate "Embeddings" (dense vectors) during indexing. This converts code into numbers but does not rank them against any specific query.
*   **Search:** **YES.** Reranking happens during search.
    1.  First, we find the top N candidates using the fast Vector/BM25 search.
    2.  Then, we use the "Reranker" model (Cross-Encoder) to compare the `query` against those top chunks to calculate a precise relevance score.
    3.  This behavior is active by default. You can disable it with `--no-rerank` for faster (but less accurate) results.

## Case 3: Memory Consumption

**Question:** "How well we are managing memory consumption? how much memory we need for index for search?"

**Answer:** **Memory usage is well-managed but biased towards performance (speed).**

1.  **Model Overhead (Static):**
    *   The application loads two Machine Learning models (Embedding + Reranker) into memory.
    *   **Estimated Base Memory:** ~300MB - 600MB depending on the specific quantized models loaded by the `fastembed` crate. This is a one-time cost.

2.  **Indexing (Dynamic):**
    *   **Batching:** The code processes files in batches of **256 chunks** (`chunks_buffer.len() >= 256` in `main.rs`). This prevents memory from spiking even if you index a massive repository. It flushes to disk frequently.
    *   **BM25:** The Tantivy index writer is capped at **50MB** buffer (`index.writer(50_000_000)` in `bm25.rs`).

3.  **Search (Dynamic):**
    *   Search is relatively lightweight, primarily using memory to store the loaded results and perform the reranking on a small subset (e.g., top 50-100 items).

**Summary:** You should expect the app to use around **500MB - 1GB** of RAM during operation, primarily due to the AI models. Code execution itself is efficient Rust.

## Case 4: Log Configuration

**Question:** "ok question so I have log level depend of what i put on the config I will get the log level?"

**Answer:** **Yes, for your application code, but with "Noise Cancellation" for dependencies.**

*   **Config Level (`log_level`):** The value you set in `code-rag.toml` (e.g., `"debug"`) **WILL** apply to all of the `code-rag` logic. If you set it to `trace`, you will see every detail of the app's internal workings.
*   **Overrides (Noise Filters):** To prevent the progress bar from glitching and your console from flooding, I have **hardcoded filters** for specific noisy libraries:
    *   `lance` (Database) -> Locked to `WARN`
    *   `tantivy` (Search Index) -> Locked to `WARN`
    *   `opendal` (Storage) -> Locked to `WARN`

So effectively: **Your Code = Your Config**, but **Internal Database Noise = Muted**.
