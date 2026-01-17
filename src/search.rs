use crate::bm25::BM25Index;
use crate::embedding::Embedder;
use crate::storage::Storage;
use anyhow::{anyhow, Context, Result};
use arrow_array::{Array, Int32Array, ListArray, StringArray};
use grep_regex::RegexMatcher;
use grep_searcher::sinks::UTF8;
use grep_searcher::Searcher;
use ignore::WalkBuilder;
use serde::Serialize;
use std::error::Error;

#[derive(Serialize, Clone, Debug)]
pub struct SearchResult {
    pub rank: usize,
    pub score: f32,
    pub filename: String,
    pub code: String,
    pub line_start: i32,
    pub line_end: i32,
    pub calls: Vec<String>,
}

pub struct CodeSearcher {
    storage: Option<Storage>,
    embedder: Option<Embedder>,
    bm25: Option<BM25Index>,
    vector_weight: f32,
    bm25_weight: f32,
    rrf_k: f64,
}

impl CodeSearcher {
    pub fn new(
        storage: Option<Storage>,
        embedder: Option<Embedder>,
        bm25: Option<BM25Index>,
        vector_weight: f32,
        bm25_weight: f32,
        rrf_k: f64,
    ) -> Self {
        Self {
            storage,
            embedder,
            bm25,
            vector_weight,
            bm25_weight,
            rrf_k,
        }
    }

    pub async fn semantic_search(
        &mut self,
        query: &str,
        limit: usize,
        ext: Option<String>,
        dir: Option<String>,
        no_rerank: bool,
    ) -> Result<Vec<SearchResult>> {
        let storage = self.storage.as_ref().context("Storage not initialized")?;
        let embedder = self.embedder.as_mut().context("Embedder not initialized")?;

        let vectors = embedder
            .embed(vec![query.to_string()], None)
            .map_err(|e| anyhow!(e.to_string()))?;

        if let Some(vector) = vectors.first() {
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
                // Normalize input to forward slashes since DB is normalized
                let clean_dir = dir_val.replace("\\", "/");
                filters.push(format!("filename LIKE '%{}%'", clean_dir));
            }
            let filter_str = if filters.is_empty() {
                None
            } else {
                Some(filters.join(" AND "))
            };

            // Fetch candidates
            // If reranking is enabled, fetch more candidates. If disabled, fetch exact limit (or slightly more for robustness)
            let fetch_limit = if no_rerank {
                limit
            } else {
                std::cmp::max(50, limit * 5)
            };
            let vector_results = storage
                .search(vector.clone(), fetch_limit, filter_str.clone())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;

            let mut candidates = Vec::new();
            let mut seen_ids = std::collections::HashSet::new();

            // --- 1. Process Vector Results ---
            for batch in vector_results {
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
                let calls_col: Option<&ListArray> = batch
                    .column_by_name("calls")
                    .and_then(|c| c.as_any().downcast_ref());

                for i in 0..batch.num_rows() {
                    let id = ids.value(i).to_string();
                    if seen_ids.contains(&id) {
                        continue;
                    }
                    seen_ids.insert(id.clone());

                    let mut calls_vec = Vec::new();
                    if let Some(calls_arr) = calls_col {
                        if !calls_arr.is_null(i) {
                            let list_val = calls_arr.value(i);
                            if let Some(str_arr) = list_val.as_any().downcast_ref::<StringArray>() {
                                for s in str_arr.iter().flatten() {
                                    calls_vec.push(s.to_string());
                                }
                            }
                        }
                    }

                    candidates.push(SearchResult {
                        rank: 0,    // Assigned later
                        score: 0.0, // RRF score
                        filename: filenames.value(i).to_string(),
                        code: codes.value(i).to_string(),
                        line_start: line_starts.value(i),
                        line_end: line_ends.value(i),
                        calls: calls_vec,
                    });
                }
            }

            // --- 2. Process BM25 Results ---
            if let Some(bm25) = &self.bm25 {
                match bm25.search(query, fetch_limit) {
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

                            // Manual Filter (Fix for pollution)
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
                                // Normalize filename too just in case
                                if !res.filename.replace("\\", "/").contains(&clean_dir) {
                                    continue;
                                }
                            }

                            // Construct SearchResult from BM25Result
                            // Note: 'calls' might be empty since we didn't index it in BM25 yet.
                            // If that's critical, we should add 'calls' to BM25 index too.
                            // For now, empty is acceptable for BM25-only hits.
                            candidates.push(SearchResult {
                                rank: 0,
                                score: 0.0,
                                filename: res.filename.clone(),
                                code: res.code.clone(),
                                line_start: res.line_start as i32,
                                line_end: res.line_end as i32,
                                calls: Vec::new(),
                            });
                            existing_ids.insert(res.id.clone());
                        }

                        for (i, candidate) in candidates.iter_mut().enumerate() {
                            // Vector rank is 'i + 1' IF it was in original vector list.
                            // But 'candidates' now has appended items.
                            // We need to know original vector rank.
                            // This simple loop assumes 'candidates' order matches vector order for the first N items.
                            // Items appended from BM25 are effectively rank > limit in vector search (or infinite).

                            // Correct logic:
                            // We can't rely on 'i' easily if we sort later, but we haven't sorted yet.
                            // So:

                            let id = format!(
                                "{}-{}-{}",
                                candidate.filename, candidate.line_start, candidate.line_end
                            );

                            // Determine Vector Rank
                            let vec_rank = if i < seen_ids.len() {
                                Some(i + 1)
                            } else {
                                None
                            };

                            let bm25_rank = bm25_ranks.get(&id).copied();

                            let vec_score = vec_rank
                                .map(|r| Self::compute_rrf_component(r, self.rrf_k))
                                .unwrap_or(0.0) as f32
                                * self.vector_weight;

                            let bm25_score = bm25_rank
                                .map(|r| Self::compute_rrf_component(r, self.rrf_k))
                                .unwrap_or(0.0) as f32
                                * self.bm25_weight;

                            candidate.score = vec_score + bm25_score;
                        }
                    }
                    Err(e) => eprintln!("BM25 search failed: {}", e),
                }
            }

            if !no_rerank {
                // Re-rank
                let texts: Vec<String> = candidates.iter().map(|c| c.code.clone()).collect();

                match embedder.rerank(query, texts) {
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

            Ok(final_results)
        } else {
            Err(anyhow!("No embedding generated"))
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
                calls: Vec::new(),
            },
            SearchResult {
                rank: 0,
                score: 0.9,
                filename: "B".into(),
                code: "".into(),
                line_start: 0,
                line_end: 0,
                calls: Vec::new(),
            },
            SearchResult {
                rank: 0,
                score: 0.5,
                filename: "C".into(),
                code: "".into(),
                line_start: 0,
                line_end: 0,
                calls: Vec::new(),
            },
        ];

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        assert_eq!(results[0].filename, "B"); // 0.9
        assert_eq!(results[1].filename, "C"); // 0.5
        assert_eq!(results[2].filename, "A"); // 0.1
    }
}
