use fastembed::{InitOptions, TextEmbedding, EmbeddingModel};
use std::error::Error;

pub struct Embedder {
    model: TextEmbedding,
}

impl Embedder {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut options = InitOptions::new(EmbeddingModel::NomicEmbedTextV15);
        options.show_download_progress = true;
        
        let model = TextEmbedding::try_new(options)?;
        Ok(Self { model })
    }

    pub fn embed(&mut self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        let embeddings = self.model.embed(texts, None)?;
        Ok(embeddings)
    }
}
