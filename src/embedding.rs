use fastembed::{
    EmbeddingModel, InitOptions, InitOptionsUserDefined, OnnxSource, RerankInitOptions,
    RerankInitOptionsUserDefined, RerankerModel, TextEmbedding, TextRerank, TokenizerFiles,
    UserDefinedEmbeddingModel, UserDefinedRerankingModel,
};
use std::error::Error;
use std::fs;
use std::path::Path;

pub struct Embedder {
    model: TextEmbedding,
    reranker: Option<TextRerank>,
    reranker_model_name: String,
    reranker_model_path: Option<String>,
    dim: usize,
}

fn load_tokenizer_files(path: &Path) -> Result<TokenizerFiles, Box<dyn Error>> {
    Ok(TokenizerFiles {
        tokenizer_file: fs::read(path.join("tokenizer.json"))?,
        config_file: fs::read(path.join("config.json"))?,
        special_tokens_map_file: fs::read(path.join("special_tokens_map.json"))?,
        tokenizer_config_file: fs::read(path.join("tokenizer_config.json"))?,
    })
}

impl Embedder {
    pub fn new(
        embedding_model: String,
        reranker_model: String,
        embedding_model_path: Option<String>,
        reranker_model_path: Option<String>,
    ) -> Result<Self, Box<dyn Error>> {
        Self::new_with_quiet(
            false,
            embedding_model,
            reranker_model,
            embedding_model_path,
            reranker_model_path,
        )
    }

    pub fn new_with_quiet(
        quiet: bool,
        embedding_model: String,
        reranker_model: String,
        embedding_model_path: Option<String>,
        reranker_model_path: Option<String>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut options = InitOptions::new(EmbeddingModel::NomicEmbedTextV15);
        options.show_download_progress = !quiet;

        let mut model = if let Some(path_str) = embedding_model_path {
            let path = Path::new(&path_str);
            tracing::info!("Loading user-defined embedding model from: {}", path_str);

            let tokenizer_files = load_tokenizer_files(path)?;
            let onnx_file = fs::read(path.join("model.onnx"))?;

            let model_def = UserDefinedEmbeddingModel::new(onnx_file, tokenizer_files);
            let user_options = InitOptionsUserDefined::default();

            TextEmbedding::try_new_from_user_defined(model_def, user_options)?
        } else {
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
            options.model_name = model_enum;
            TextEmbedding::try_new(options)?
        };

        // Determine dimension
        let warmup_text = vec!["warmup".to_string()];
        let warmup_vecs = model.embed(warmup_text, None)?;
        let dim = warmup_vecs.first().map(|v| v.len()).unwrap_or(768);

        Ok(Self {
            model,
            reranker: None,
            reranker_model_name: reranker_model,
            reranker_model_path,
            dim,
        })
    }

    pub fn dim(&self) -> usize {
        self.dim
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
            let mut options = RerankInitOptions::new(RerankerModel::BGERerankerBase);
            options.show_download_progress = true;

            let reranker = if let Some(path_str) = &self.reranker_model_path {
                let path = Path::new(path_str);
                tracing::info!("Loading user-defined re-ranking model from: {}", path_str);

                let tokenizer_files = load_tokenizer_files(path)?;
                let onnx_path = path.join("model.onnx");

                let model_def =
                    UserDefinedRerankingModel::new(OnnxSource::File(onnx_path), tokenizer_files);
                let user_options = RerankInitOptionsUserDefined::default();

                TextRerank::try_new_from_user_defined(model_def, user_options)?
            } else {
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
                options.model_name = reranker_enum;
                TextRerank::try_new(options)?
            };
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
