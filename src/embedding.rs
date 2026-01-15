use fastembed::{
    EmbeddingModel, InitOptions, RerankInitOptions, RerankerModel, TextEmbedding, TextRerank,
};
use std::error::Error;

pub struct Embedder {
    model: TextEmbedding,
    reranker: Option<TextRerank>,
    reranker_model_name: String,
}

impl Embedder {
    pub fn new(embedding_model: String, reranker_model: String) -> Result<Self, Box<dyn Error>> {
        Self::new_with_quiet(false, embedding_model, reranker_model)
    }

    pub fn new_with_quiet(
        quiet: bool,
        embedding_model: String,
        reranker_model: String,
    ) -> Result<Self, Box<dyn Error>> {
        let model_enum = match embedding_model.to_lowercase().as_str() {
            "nomic-embed-text-v1.5" => EmbeddingModel::NomicEmbedTextV15,
            "all-minilm-l6-v2" => EmbeddingModel::AllMiniLML6V2,
            "bge-small-en-v1.5" => EmbeddingModel::BGESmallENV15,
            "bge-base-en-v1.5" => EmbeddingModel::BGEBaseENV15,
            _ => {
                tracing::warn!(
                    "Unknown embedding model '{}', falling back to NomicEmbedTextV15",
                    embedding_model
                );
                EmbeddingModel::NomicEmbedTextV15
            }
        };

        let mut options = InitOptions::new(model_enum);
        options.show_download_progress = !quiet;

        // Indicate loading status
        // Indicate loading status (handled by caller)
        let model = TextEmbedding::try_new(options)?;
        Ok(Self {
            model,
            reranker: None,
            reranker_model_name: reranker_model,
        })
    }

    pub fn embed(
        &mut self,
        texts: Vec<String>,
        batch_size: Option<usize>,
    ) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        let embeddings = self.model.embed(texts, batch_size)?;
        Ok(embeddings)
    }

    pub fn init_reranker(&mut self) -> Result<(), Box<dyn Error>> {
        if self.reranker.is_none() {
            let reranker_enum = match self.reranker_model_name.to_lowercase().as_str() {
                "bge-reranker-base" => RerankerModel::BGERerankerBase,
                _ => {
                    tracing::warn!(
                        "Unknown reranker model '{}', falling back to BGERerankerBase",
                        self.reranker_model_name
                    );
                    RerankerModel::BGERerankerBase
                }
            };

            let mut options = RerankInitOptions::new(reranker_enum);
            options.show_download_progress = true;

            let reranker = TextRerank::try_new(options)?;
            self.reranker = Some(reranker);
        }
        Ok(())
    }

    pub fn rerank(
        &mut self,
        query: &str,
        documents: Vec<String>,
    ) -> Result<Vec<(usize, f32)>, Box<dyn Error>> {
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
