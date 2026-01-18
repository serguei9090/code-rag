use crate::search::SearchResult;
use anyhow::Result;
use tiktoken_rs::cl100k_base;

#[derive(Debug, Clone)]
pub struct MergedChunk {
    pub filename: String,
    pub start_line: i32,
    pub end_line: i32,
    pub code: String,
    pub scores: Vec<f32>,
    pub avg_score: f32,
    pub max_score: f32,
}

pub struct ContextOptimizer {
    token_limit: usize,
}

impl ContextOptimizer {
    pub fn new(token_limit: usize) -> Self {
        Self { token_limit }
    }

    /// Merges and selects chunks to fit within the token budget.
    pub fn optimize(&self, results: Vec<SearchResult>) -> Result<Vec<MergedChunk>> {
        if results.is_empty() {
            return Ok(Vec::new());
        }

        // 1. Group by filename and sort by line number
        let mut by_file: std::collections::HashMap<String, Vec<SearchResult>> =
            std::collections::HashMap::new();

        for res in results {
            by_file.entry(res.filename.clone()).or_default().push(res);
        }

        let mut all_merged = Vec::new();
        let bpe = cl100k_base()?; // GPT-4 tokenizer

        // 2. Coalesce adjacent chunks within each file
        for (_filename, mut file_results) in by_file {
            // Sort by start line
            file_results.sort_by_key(|r| r.line_start);

            let mut current_merged: Option<MergedChunk> = None;

            for res in file_results {
                match current_merged {
                    Some(mut curr) => {
                        // Check adjacency (e.g. within 5 lines)
                        if res.line_start <= curr.end_line + 5 {
                            // Merge
                            // We need to handle potential overlap or gap filling in a real implementation.
                            // For simplistic "line-based" chunks, we might just concat code if we had full file access,
                            // but here we only have snippets.
                            // If they are strictly adjacent snippets from the index, we can just join them.
                            // Ideally, we'd read the file content between them if minimal, but for now let's just join with a newline.

                            let had_gap = res.line_start > curr.end_line + 1;
                            curr.end_line = std::cmp::max(curr.end_line, res.line_end);
                            curr.code.push('\n');
                            if had_gap {
                                // Add gap marker if there is a gap but it's small enough to merge
                                curr.code.push_str("... (gap) ...\n");
                            }
                            curr.code.push_str(&res.code);
                            curr.scores.push(res.score);
                            curr.max_score = curr.max_score.max(res.score);

                            // Recompute average
                            let sum: f32 = curr.scores.iter().sum();
                            curr.avg_score = sum / curr.scores.len() as f32;

                            current_merged = Some(curr);
                        } else {
                            // Gap too large, push current and start new
                            all_merged.push(curr);
                            current_merged = Some(Self::from_single(&res));
                        }
                    }
                    None => {
                        current_merged = Some(Self::from_single(&res));
                    }
                }
            }

            if let Some(curr) = current_merged {
                all_merged.push(curr);
            }
        }

        // 3. Knapsack / Budgeting
        // Sort by max_score (prioritize keeping the most relevant bits)
        all_merged.sort_by(|a, b| b.max_score.total_cmp(&a.max_score));

        let mut final_selection = Vec::new();
        let mut current_tokens = 0;

        for chunk in all_merged {
            let tokens = bpe.encode_with_special_tokens(&chunk.code).len();
            if current_tokens + tokens <= self.token_limit {
                final_selection.push(chunk);
                current_tokens += tokens;
            } else {
                // If we implemented a "soft" break (trimming the chunk), we could fit partial here.
                // For now, strict exclusion.
                continue;
            }
        }

        // Sort back by score or perhaps by file/line for readability?
        // Usually LLMs prefer context grouped by file.
        final_selection.sort_by(|a, b| {
            let file_cmp = a.filename.cmp(&b.filename);
            if file_cmp == std::cmp::Ordering::Equal {
                a.start_line.cmp(&b.start_line)
            } else {
                file_cmp
            }
        });

        Ok(final_selection)
    }

    fn from_single(res: &SearchResult) -> MergedChunk {
        MergedChunk {
            filename: res.filename.clone(),
            start_line: res.line_start,
            end_line: res.line_end,
            code: res.code.clone(),
            scores: vec![res.score],
            avg_score: res.score,
            max_score: res.score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_adjacent() {
        let r1 = SearchResult {
            rank: 1,
            score: 0.9,
            filename: "A.rs".into(),
            code: "fn a() {}".into(),
            line_start: 10,
            line_end: 12,
            calls: vec![],
        };
        let r2 = SearchResult {
            rank: 2,
            score: 0.8,
            filename: "A.rs".into(),
            code: "fn b() {}".into(),
            line_start: 14,
            line_end: 16,
            calls: vec![],
        };

        let optimizer = ContextOptimizer::new(1000);
        let merged = optimizer.optimize(vec![r1, r2]).unwrap();

        // Should merge because 14 <= 12 + 5
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].start_line, 10);
        assert_eq!(merged[0].end_line, 16);
    }

    #[test]
    fn test_budget_limit() {
        let r1 = SearchResult {
            rank: 1,
            score: 0.9,
            filename: "A.rs".into(),
            code: "long code ".repeat(100),
            line_start: 1,
            line_end: 10,
            calls: vec![],
        };

        let optimizer = ContextOptimizer::new(10); // Very small budget
        let merged = optimizer.optimize(vec![r1]).unwrap();

        // Should be rejected
        assert_eq!(merged.len(), 0);
    }
}
