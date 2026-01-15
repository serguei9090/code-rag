use fastembed::{InitOptions, TextEmbedding, EmbeddingModel, TextRerank, RerankerModel, RerankInitOptions};
use std::error::Error;

pub struct Embedder {
    model: TextEmbedding,
    reranker: Option<TextRerank>,
}

impl Embedder {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut options = InitOptions::new(EmbeddingModel::NomicEmbedTextV15);
        options.show_download_progress = true;
        
        // Indicate loading status
        println!("Loading embedding model (NomicEmbedTextV15)...");
        let model = TextEmbedding::try_new(options)?;
        Ok(Self { model, reranker: None })
    }

    pub fn embed(&mut self, texts: Vec<String>, batch_size: Option<usize>) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        let embeddings = self.model.embed(texts, batch_size)?;
        Ok(embeddings)
    }

    pub fn init_reranker(&mut self) -> Result<(), Box<dyn Error>> {
        if self.reranker.is_none() {
            let mut options = RerankInitOptions::new(RerankerModel::BGERerankerBase);
            options.show_download_progress = true;
            
            // Indicate loading status (this can be slow)
            println!("Initializing re-ranker (BGERerankerBase) - this may take a moment...");
            let reranker = TextRerank::try_new(options)?;
            self.reranker = Some(reranker);
        }
        Ok(())
    }

    pub fn rerank(&mut self, query: &str, documents: Vec<String>) -> Result<Vec<(usize, f32)>, Box<dyn Error>> {
        self.init_reranker()?;
        if let Some(reranker) = &mut self.reranker {
            // Pass reference to documents to satisfy AsRef<[S]> ?
            // Signature: rerank<S: AsRef<str>...>(query, documents: impl AsRef<[S]>, return_documents: bool, batch_size: Option<usize>)
            let refs: Vec<&str> = documents.iter().map(|s| s.as_str()).collect();
            let results = reranker.rerank(query, &refs, true, None)?; 
            Ok(results.iter().map(|r| (r.index, r.score)).collect())
        } else {
            Err("Reranker not initialized".into())
        }
    }
}
