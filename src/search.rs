use crate::bm25::BM25Index;
use crate::embedding::Embedder;
use crate::llm::QueryExpander;
use crate::storage::Storage;
use anyhow::{anyhow, Context, Result};
use arrow_array::{Array, Int32Array, Int64Array, ListArray, StringArray};
use grep_regex::RegexMatcher;
use grep_searcher::sinks::UTF8;
use grep_searcher::Searcher;
use ignore::WalkBuilder;
use serde::Serialize;
use std::error::Error;
use std::sync::Arc;

/// A single search result from code search.
///
/// Contains the matched code chunk with metadata and relevance score.
#[derive(Serialize, Clone, Debug)]
pub struct SearchResult {
    pub rank: usize,
    pub score: f32,
    pub filename: String,
    pub code: String,
    pub line_start: i32,
    pub line_end: i32,
    pub last_modified: i64,
    pub calls: Vec<String>,
}

impl SearchResult {
    pub fn merge(_chunks: Vec<SearchResult>) -> Self {
        // Implementation detail if needed, but we use ContextOptimizer
        unimplemented!()
    }
}

/// Hybrid code search engine combining BM25 and vector search.
///
/// Uses RRF (Reciprocal Rank Fusion) to combine keyword and semantic results.
pub struct CodeSearcher {
    storage: Option<Storage>,
    embedder: Option<Embedder>,
    bm25: Option<BM25Index>,
    expander: Option<Arc<QueryExpander>>,
    vector_weight: f32,
    bm25_weight: f32,
    rrf_k: f64,
}

impl CodeSearcher {
    pub fn new(
        storage: Option<Storage>,
        embedder: Option<Embedder>,
        bm25: Option<BM25Index>,
        expander: Option<Arc<QueryExpander>>,
        vector_weight: f32,
        bm25_weight: f32,
        rrf_k: f64,
    ) -> Self {
        Self {
            storage,
            embedder,
            bm25,
            expander,
            vector_weight,
            bm25_weight,
            rrf_k,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
        ext: Option<String>,
        dir: Option<String>,
        no_rerank: bool,
        workspace: Option<String>,
        max_tokens: Option<usize>,
        enable_expansion: bool,
    ) -> Result<Vec<SearchResult>> {
        let storage = self.storage.as_ref().context("Storage not initialized")?;
        let embedder = self.embedder.as_ref().context("Embedder not initialized")?;

        // 1. Expand Query if enabled
        let mut search_queries = vec![query.to_string()];
        if enable_expansion {
            if let Some(expander) = &self.expander {
                match expander.expand(query).await {
                    Ok(expanded) => {
                        // expander returns original query too, so we can just use that
                        search_queries = expanded;
                        tracing::info!("Expanded query '{}' to: {:?}", query, search_queries);
                    }
                    Err(e) => {
                        tracing::warn!("Query expansion failed: {}. Using original query.", e);
                    }
                }
            }
        }

        // 2. Vector Search for all queries (Standard + Expanded)
        // We accumulate RRF scores from all vector searches
        let mut vector_rrf_scores: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        // Also map ID to SearchResult to reconstruct later.
        let mut all_vector_results: std::collections::HashMap<String, SearchResult> =
            std::collections::HashMap::new();

        // Batched Embedding Generation
        let all_query_vectors = embedder
            .embed(search_queries.clone(), None)
            .map_err(|e: fastembed::Error| anyhow!(e.to_string()))?;

        for vector in all_query_vectors {
            // Construct Filters
            let mut filters = Vec::new();
            if let Some(ext_val) = &ext {
                let clean_ext = if let Some(stripped) = ext_val.strip_prefix('.') {
                    stripped
                } else {
                    ext_val
                };
                filters.push(format!("filename LIKE '%.{}'", clean_ext));
            }
            if let Some(dir_val) = &dir {
                let clean_dir = dir_val.replace("\\", "/");
                filters.push(format!("filename LIKE '%{}%'", clean_dir));
            }
            let filter_str = if filters.is_empty() {
                None
            } else {
                Some(filters.join(" AND "))
            };

            let fetch_limit = if no_rerank {
                limit
            } else {
                std::cmp::max(50, limit * 5)
            };

            let results = storage
                .search(vector, fetch_limit, filter_str, workspace.as_deref())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;

            // Process batch
            for batch in results {
                let ids: &StringArray = batch
                    .column_by_name("id")
                    .ok_or_else(|| anyhow!("id missing"))?
                    .as_any()
                    .downcast_ref()
                    .ok_or_else(|| anyhow!("id wrong type"))?;
                let filenames: &StringArray = batch
                    .column_by_name("filename")
                    .ok_or_else(|| anyhow!("filename missing"))?
                    .as_any()
                    .downcast_ref()
                    .ok_or_else(|| anyhow!("filename wrong type"))?;
                let codes: &StringArray = batch
                    .column_by_name("code")
                    .ok_or_else(|| anyhow!("code missing"))?
                    .as_any()
                    .downcast_ref()
                    .ok_or_else(|| anyhow!("code wrong type"))?;
                let line_starts: &Int32Array = batch
                    .column_by_name("line_start")
                    .ok_or_else(|| anyhow!("line_start missing"))?
                    .as_any()
                    .downcast_ref()
                    .ok_or_else(|| anyhow!("line_start wrong type"))?;
                let line_ends: &Int32Array = batch
                    .column_by_name("line_end")
                    .ok_or_else(|| anyhow!("line_end missing"))?
                    .as_any()
                    .downcast_ref()
                    .ok_or_else(|| anyhow!("line_end wrong type"))?;
                let last_modifieds: &Int64Array = batch
                    .column_by_name("last_modified")
                    .ok_or_else(|| anyhow!("last_modified missing"))?
                    .as_any()
                    .downcast_ref()
                    .ok_or_else(|| anyhow!("last_modified wrong type"))?;
                let calls_col: Option<&ListArray> = batch
                    .column_by_name("calls")
                    .and_then(|c| c.as_any().downcast_ref());

                for i in 0..batch.num_rows() {
                    let id = ids.value(i).to_string();
                    let rank = i + 1; // Rank in this specific query result list

                    // Accumulate RRF score
                    *vector_rrf_scores.entry(id.clone()).or_insert(0.0) +=
                        Self::compute_rrf_component(rank, self.rrf_k);

                    // Store Result Data if not present
                    all_vector_results.entry(id.clone()).or_insert_with(|| {
                        let mut calls_vec = Vec::new();
                        if let Some(calls_arr) = calls_col {
                            if !calls_arr.is_null(i) {
                                if let Some(str_arr) =
                                    calls_arr.value(i).as_any().downcast_ref::<StringArray>()
                                {
                                    for s in str_arr.iter().flatten() {
                                        calls_vec.push(s.to_string());
                                    }
                                }
                            }
                        }
                        SearchResult {
                            rank: 0,
                            score: 0.0,
                            filename: filenames.value(i).to_string(),
                            code: codes.value(i).to_string(),
                            line_start: line_starts.value(i),
                            line_end: line_ends.value(i),
                            last_modified: last_modifieds.value(i),
                            calls: calls_vec,
                        }
                    });
                }
            }
        } // End of vector search loop

        // Convert Map back to List
        let mut candidates: Vec<SearchResult> = all_vector_results.into_values().collect();

        // --- 2. Process BM25 Results ---
        if let Some(bm25) = &self.bm25 {
            let fetch_limit = if no_rerank {
                limit
            } else {
                std::cmp::max(50, limit * 5)
            };
            match bm25.search(query, fetch_limit, workspace.as_deref()) {
                Ok(bm25_results) => {
                    let bm25_ranks: std::collections::HashMap<String, usize> = bm25_results
                        .iter()
                        .enumerate()
                        .map(|(rank, res)| (res.id.clone(), rank + 1))
                        .collect();

                    let mut existing_ids: std::collections::HashSet<String> = candidates
                        .iter()
                        .map(|c| format!("{}-{}-{}", c.filename, c.line_start, c.line_end))
                        .collect();

                    // Add unique BM25 hits
                    for res in &bm25_results {
                        if existing_ids.contains(&res.id) {
                            continue;
                        }

                        // Manual Filter
                        if let Some(ext_val) = &ext {
                            let clean_ext = if let Some(stripped) = ext_val.strip_prefix('.') {
                                stripped
                            } else {
                                ext_val
                            };
                            let suffix = format!(".{}", clean_ext);
                            if !res.filename.ends_with(&suffix) {
                                continue;
                            }
                        }

                        if let Some(dir_val) = &dir {
                            let clean_dir = dir_val.replace("\\", "/");
                            if !res.filename.replace("\\", "/").contains(&clean_dir) {
                                continue;
                            }
                        }

                        candidates.push(SearchResult {
                            rank: 0,
                            score: 0.0,
                            filename: res.filename.clone(),
                            code: res.code.clone(),
                            line_start: res.line_start as i32,
                            line_end: res.line_end as i32,
                            last_modified: 0, // BM25 doesn't track this currently, might need update
                            calls: Vec::new(),
                        });
                        existing_ids.insert(res.id.clone());
                    }

                    for candidate in candidates.iter_mut() {
                        let id = format!(
                            "{}-{}-{}",
                            candidate.filename, candidate.line_start, candidate.line_end
                        );

                        // Get accumulated vector score
                        let vec_rrf_sum = vector_rrf_scores.get(&id).copied().unwrap_or(0.0);

                        let bm25_rank = bm25_ranks.get(&id).copied();

                        let vec_score = vec_rrf_sum as f32 * self.vector_weight;

                        let bm25_score = bm25_rank
                            .map(|r| Self::compute_rrf_component(r, self.rrf_k))
                            .unwrap_or(0.0) as f32
                            * self.bm25_weight;

                        candidate.score = vec_score + bm25_score;
                    }
                }
                Err(e) => eprintln!("BM25 search failed: {}", e),
            }
        } else {
            // No BM25, just set score from vectors
            for candidate in candidates.iter_mut() {
                let id = format!(
                    "{}-{}-{}",
                    candidate.filename, candidate.line_start, candidate.line_end
                );
                let vec_rrf_sum = vector_rrf_scores.get(&id).copied().unwrap_or(0.0);
                candidate.score = vec_rrf_sum as f32 * self.vector_weight;
            }
        }

        if !no_rerank && !candidates.is_empty() {
            // Re-rank
            let texts: Vec<String> = candidates.iter().map(|c| c.code.clone()).collect();

            match embedder.rerank(query, texts.clone(), texts.len()) {
                Ok(rerank_results) => {
                    // Update scores
                    for (original_idx, new_score) in rerank_results {
                        if let Some(candidate) = candidates.get_mut(original_idx) {
                            candidate.score = new_score;
                        }
                    }
                    // Sort by new score (descending)
                    candidates.sort_by(|a, b| {
                        b.score
                            .partial_cmp(&a.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
                Err(e) => {
                    eprintln!("Reranking failed/skipped: {}. Using vector scores.", e);
                }
            }
        }

        // Truncate and assign ranks
        let mut final_results = candidates.into_iter().take(limit).collect::<Vec<_>>();
        for (i, res) in final_results.iter_mut().enumerate() {
            res.rank = i + 1;
        }

        if let Some(tokens) = max_tokens {
            use crate::context::ContextOptimizer;
            let optimizer = ContextOptimizer::new(tokens);
            let merged_chunks = optimizer.optimize(final_results)?;

            // Map back to SearchResult
            let mut mapped_results = Vec::new();
            for (i, chunk) in merged_chunks.into_iter().enumerate() {
                mapped_results.push(SearchResult {
                    rank: i + 1,
                    score: chunk.max_score, // Use max score of the group
                    filename: chunk.filename,
                    code: chunk.code,
                    line_start: chunk.start_line,
                    line_end: chunk.end_line,
                    last_modified: chunk.last_modified,
                    calls: chunk.calls,
                });
            }
            Ok(mapped_results)
        } else {
            Ok(final_results)
        }
    }

    pub fn grep_search(
        &self,
        pattern: &str,
        base_path: &str,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let matcher = RegexMatcher::new(pattern)?;
        let mut matches = Vec::new();
        let walker = WalkBuilder::new(base_path).build(); // Respects .gitignore by default

        for result in walker {
            match result {
                Ok(entry) => {
                    if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                        continue;
                    }

                    let path = entry.path().to_path_buf();
                    let mut file_matches = Vec::new(); // Local to file
                    let _ = Searcher::new().search_path(
                        &matcher,
                        &path,
                        UTF8(|ln, line| {
                            file_matches.push(format!("{}:{}: {}", path.display(), ln, line));
                            Ok(true)
                        }),
                    );

                    matches.extend(file_matches);
                }
                Err(err) => {
                    // Log error but continue
                    eprintln!("Error walking: {}", err);
                }
            }
        }
        Ok(matches)
    }

    /// Helper to compute RRF score component: 1.0 / (k + rank)
    fn compute_rrf_component(rank: usize, k: f64) -> f64 {
        1.0 / (k + rank as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_scoring_formula() {
        let k = 60.0;
        // Rank 1
        let score_1 = CodeSearcher::compute_rrf_component(1, k);
        assert!((score_1 - (1.0 / 61.0)).abs() < f64::EPSILON);

        // Rank 10
        let score_10 = CodeSearcher::compute_rrf_component(10, k);
        assert!((score_10 - (1.0 / 70.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sorting_logic() {
        let mut results = [
            SearchResult {
                rank: 0,
                score: 0.1,
                filename: "A".into(),
                code: "".into(),
                line_start: 0,
                line_end: 0,
                last_modified: 0,
                calls: Vec::new(),
            },
            SearchResult {
                rank: 0,
                score: 0.9,
                filename: "B".into(),
                code: "".into(),
                line_start: 0,
                line_end: 0,
                last_modified: 0,
                calls: Vec::new(),
            },
            SearchResult {
                rank: 0,
                score: 0.5,
                filename: "C".into(),
                code: "".into(),
                line_start: 0,
                line_end: 0,
                last_modified: 0,
                calls: Vec::new(),
            },
        ];

        results.sort_by(|a, b| b.score.total_cmp(&a.score));

        assert_eq!(results[0].filename, "B"); // 0.9
        assert_eq!(results[1].filename, "C"); // 0.5
        assert_eq!(results[2].filename, "A"); // 0.1
    }
}
