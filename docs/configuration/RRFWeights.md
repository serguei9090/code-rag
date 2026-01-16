# Hybrid Search Tuning (RRF)

`code-rag` uses **Reciprocal Rank Fusion (RRF)** to combine results from two different search algorithms:

1.  **Semantic Search (Vectors)**: Finds code based on meaning and concepts (e.g., "authentication logic").
2.  **Keyword Search (BM25)**: Finds code based on exact token matches (e.g., "function login_user").

## How RRF Works

RRF assigns a score to each document based on its rank in the individual search lists:

$$ Score(d) = \sum_{r \in R} \frac{1}{k + r(d)} $$

Where:
*   $r(d)$ is the rank of document $d$ in a results list (1st, 2nd, etc.).
*   $k$ is a constant that smooths the importance of high ranks.

## Configuration

You can tune the hybrid search behavior in `code-ragcnf.toml`:

```toml
# Weight multipliers applied to the RRF score for each method
vector_weight = 1.0
bm25_weight = 1.0

# The smoothing constant 'k'
rrf_k = 60.0
```

### Tuning Scenarios

#### 1. Prioritize Exact Matches (Code Grep Style)
If you want the tool to behave more like a "smart grep" where exact keyword matches almost always win:
*   **Increase `bm25_weight`**: Set to `2.0` or higher.
*   **Decrease `vector_weight`**: Set to `0.5`.

#### 2. Prioritize Concepts (Exploration Style)
If you are exploring a codebase and don't know the exact function names:
*   **Increase `vector_weight`**: Set to `2.0`.
*   **Decrease `bm25_weight`**: Set to `0.5`.

#### 3. Adjusting the "Top Result" Bias (`rrf_k`)
*   **Lower `rrf_k` (e.g., 10)**: results that appear at the very top (Rank 1 or 2) of *either* list get a massive score boost. This is "winner-takes-all".
*   **Higher `rrf_k` (e.g., 100)**: The difference between Rank 1 and Rank 5 is smaller. This is more "democratic" and blends the lists more evenly. The default of `60.0` is a standard industry value.
