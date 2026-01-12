use crate::storage::Storage;
use crate::embedding::Embedder;
use grep_regex::RegexMatcher;
use grep_searcher::Searcher;
use grep_searcher::sinks::UTF8;
use ignore::WalkBuilder;
use arrow_array::RecordBatch;
use std::error::Error;

pub struct CodeSearcher {
    storage: Option<Storage>,
    embedder: Option<Embedder>,
}

impl CodeSearcher {
    pub fn new(storage: Option<Storage>, embedder: Option<Embedder>) -> Self {
        Self { storage, embedder }
    }

    pub async fn semantic_search(&mut self, query: &str, limit: usize) -> Result<Vec<RecordBatch>, Box<dyn Error>> {
        if let (Some(storage), Some(embedder)) = (&self.storage, &mut self.embedder) {
            let vectors = embedder.embed(vec![query.to_string()])?;
            // Flatten if needed, but embed returns Vec<Vec<f32>>
            if let Some(vector) = vectors.first() {
                 let results = storage.search(vector.clone(), limit).await?;
                 Ok(results)
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
