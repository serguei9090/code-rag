use crate::storage::Storage;
use crate::embedding::Embedder;
use grep_regex::RegexMatcher;
use grep_searcher::Searcher;
use grep_searcher::sinks::UTF8;
use ignore::WalkBuilder;
use arrow_array::{StringArray, Float32Array, Int32Array, ListArray, Array};
use std::error::Error;
use serde::Serialize;

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
}

impl CodeSearcher {
    pub fn new(storage: Option<Storage>, embedder: Option<Embedder>) -> Self {
        Self { storage, embedder }
    }

    pub async fn semantic_search(&mut self, query: &str, limit: usize, extension: Option<String>, directory: Option<String>, no_rerank: bool) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        if let (Some(storage), Some(embedder)) = (&self.storage, &mut self.embedder) {
            let vectors = embedder.embed(vec![query.to_string()])?;
            
            if let Some(vector) = vectors.first() {
                 let mut filters = Vec::new();
                 if let Some(ext) = extension {
                     let clean_ext = if ext.starts_with('.') { &ext[1..] } else { &ext };
                     filters.push(format!("filename LIKE '%.{}'", clean_ext));
                 }
                 if let Some(dir) = directory {
                     // Windows paths use backslashes which need escaping in SQL LIKE
                     // Convert input to backslashes (Windows standard) and escape for SQL
                     let normalized = dir.replace("/", "\\\\");
                     // Double escape: one for Rust string, one for SQL
                     let escaped = normalized.replace("\\", "\\\\");
                     filters.push(format!("filename LIKE '%{}%'", escaped));
                 }
                 let filter_str = if filters.is_empty() { None } else { Some(filters.join(" AND ")) };

                 // Fetch candidates
                 // If reranking is enabled, fetch more candidates. If disabled, fetch exact limit (or slightly more for robustness)
                 let fetch_limit = if no_rerank { limit } else { std::cmp::max(50, limit * 5) };
                 let batches = storage.search(vector.clone(), fetch_limit, filter_str).await?;
                 
                 let mut candidates = Vec::new();
                 
                 // Convert RecordBatches to SearchResults
                 for batch in batches {
                     let filenames: &StringArray = batch.column_by_name("filename")
                         .ok_or("filename missing")?.as_any().downcast_ref().ok_or("filename wrong type")?;
                     let codes: &StringArray = batch.column_by_name("code")
                         .ok_or("code missing")?.as_any().downcast_ref().ok_or("code wrong type")?;
                     let line_starts: &Int32Array = batch.column_by_name("line_start")
                         .ok_or("line_start missing")?.as_any().downcast_ref().ok_or("line_start wrong type")?;
                     let line_ends: &Int32Array = batch.column_by_name("line_end")
                         .ok_or("line_end missing")?.as_any().downcast_ref().ok_or("line_end wrong type")?;
                     let calls_col: Option<&ListArray> = batch.column_by_name("calls")
                         .and_then(|c| c.as_any().downcast_ref());
                     let scores: Option<&Float32Array> = batch.column_by_name("_score")
                         .and_then(|c| c.as_any().downcast_ref());

                     for i in 0..batch.num_rows() {
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
                             rank: 0, // Assigned later
                             score: scores.map(|s| s.value(i)).unwrap_or(0.0),
                             filename: filenames.value(i).to_string(),
                             code: codes.value(i).to_string(),
                             line_start: line_starts.value(i),
                             line_end: line_ends.value(i),
                             calls: calls_vec,
                         });
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
                             candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                         },
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
                Err("No embedding generated".into())
            }
        } else {
            Err("Storage or Embedder not initialized".into())
        }
    }

    pub fn grep_search(&self, pattern: &str, base_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let matcher = RegexMatcher::new(pattern)?;
        let mut matches = Vec::new();
        let walker = WalkBuilder::new(base_path).build(); // Respects .gitignore by default

        for result in walker {
            match result {
                Ok(entry) => {
                    if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                        continue;
                    }

                    let path = entry.path().to_path_buf();
                    // We need to clone path or print string inside closure.
                    // The sink closure needs to satisfy 'static or be scoped?
                    // grep_searcher::search_path takes a sink.
                    
                    // Simple collection for now.
                    let mut file_matches = Vec::new(); // Local to file
                     let _ = Searcher::new().search_path(&matcher, &path, UTF8(|ln, line| {
                         file_matches.push(format!("{}:{}: {}", path.display(), ln, line));
                         Ok(true)
                     }));
                     
                     matches.extend(file_matches);
                }
                Err(err) => {
                    // Log error but continue? For CLI we might want to warn.
                    eprintln!("Error walking: {}", err);
                }
            }
        }
        Ok(matches)
    }
}
