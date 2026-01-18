use anyhow::Result;
use fastembed::{
    EmbeddingModel, InitOptions, InitOptionsUserDefined, RerankInitOptions, RerankerModel,
    TextEmbedding, TextRerank, TokenizerFiles, UserDefinedEmbeddingModel,
};
use ort::execution_providers::CPUExecutionProvider;
#[cfg(feature = "cuda")]
use ort::execution_providers::CUDAExecutionProvider;
#[cfg(feature = "metal")]
use ort::execution_providers::CoreMLExecutionProvider;

use std::fs;
use std::path::{Path, PathBuf};

pub struct Embedder {
    model: TextEmbedding,
    reranker: Option<TextRerank>,
    reranker_model_name: String,
    reranker_model_path: Option<String>,
    dim: usize,
}

fn load_tokenizer_files(path: &Path) -> std::io::Result<TokenizerFiles> {
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
        device: String,
    ) -> Result<Self> {
        Self::new_with_quiet(
            false,
            embedding_model,
            reranker_model,
            embedding_model_path,
            reranker_model_path,
            device,
        )
    }

    pub fn new_with_quiet(
        quiet: bool,
        embedding_model: String,
        reranker_model: String,
        embedding_model_path: Option<String>,
        reranker_model_path: Option<String>,
        device: String,
    ) -> Result<Self> {
        let providers = match device.to_lowercase().as_str() {
            "cuda" => {
                #[cfg(feature = "cuda")]
                {
                    vec![
                        CUDAExecutionProvider::default().build(),
                        CPUExecutionProvider::default().build(),
                    ]
                }
                #[cfg(not(feature = "cuda"))]
                {
                    tracing::warn!("CUDA feature not enabled, falling back to CPU");
                    vec![CPUExecutionProvider::default().build()]
                }
            }
            "metal" => {
                #[cfg(feature = "metal")]
                {
                    vec![
                        CoreMLExecutionProvider::default().build(),
                        CPUExecutionProvider::default().build(),
                    ]
                }
                #[cfg(not(feature = "metal"))]
                {
                    tracing::warn!("Metal feature not enabled, falling back to CPU");
                    vec![CPUExecutionProvider::default().build()]
                }
            }
            "cpu" => vec![CPUExecutionProvider::default().build()],
            "auto" | _ => {
                let mut p = Vec::new();
                #[cfg(feature = "cuda")]
                p.push(CUDAExecutionProvider::default().build());
                #[cfg(feature = "metal")]
                p.push(CoreMLExecutionProvider::default().build());
                p.push(CPUExecutionProvider::default().build());
                p
            }
        };
        tracing::info!("Requested Execution Providers: {:?}", providers);

        let mut model = if let Some(path_str) = embedding_model_path {
            let path = Path::new(&path_str);
            tracing::info!("Loading user-defined embedding model from: {}", path_str);

            let tokenizer_files = load_tokenizer_files(path)?;
            let onnx_file = fs::read(path.join("model.onnx"))?;

            let model_def = UserDefinedEmbeddingModel::new(onnx_file, tokenizer_files);

            // Reconstruct user options with providers
            let mut user_options = InitOptionsUserDefined::new();
            user_options.execution_providers = providers;

            TextEmbedding::try_new_from_user_defined(model_def, user_options)?
        } else {
            let model_enum = match embedding_model.to_lowercase().as_str() {
                "nomic-embed-text-v1.5" => EmbeddingModel::NomicEmbedTextV15,
                "all-minilm-l6-v2" => EmbeddingModel::AllMiniLML6V2,
                "bge-base-en-v1.5" => EmbeddingModel::BGEBaseENV15,
                "bge-small-en-v1.5" => EmbeddingModel::BGESmallENV15,
                "multilingual-e5-large" => EmbeddingModel::MultilingualE5Large,
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
            options.execution_providers = providers;

            TextEmbedding::try_new(options)?
        };

        // Determine embedding dimension dynamically
        let dim = match model.embed(vec!["warmup".to_string()], Some(1)) {
            Ok(vec) => vec.first().map(|v| v.len()).unwrap_or(768),
            Err(e) => {
                tracing::warn!(
                    "Failed to determine model dimension, defaulting to 768: {}",
                    e
                );
                768
            }
        };

        let rerank_init_options = if !quiet {
            let model_enum = match reranker_model.to_lowercase().as_str() {
                "bge-reranker-base" => RerankerModel::BGERerankerBase,
                // "bge-reranker-v2-m3" => RerankerModel::BGERerankerV2M3, // Not verified in list
                _ => {
                    tracing::warn!(
                        "Unknown reranker model '{}', defaulting to BGERerankerBase",
                        reranker_model
                    );
                    RerankerModel::BGERerankerBase
                }
            };

            let mut rerank_init_options = RerankInitOptions::default();
            rerank_init_options.model_name = model_enum;
            if let Some(ref path) = reranker_model_path {
                rerank_init_options.cache_dir = PathBuf::from(path);
            }

            Some(rerank_init_options)
        } else {
            None
        };

        let reranker = if let Some(options) = rerank_init_options {
            Some(TextRerank::try_new(options)?)
        } else {
            None
        };

        Ok(Self {
            model,
            reranker,
            reranker_model_name: reranker_model,
            reranker_model_path,
            dim,
        })
    }

    pub fn embed(
        &mut self,
        texts: Vec<String>,
        batch_size: Option<usize>,
    ) -> Result<Vec<Vec<f32>>> {
        let embeddings = self.model.embed(texts, batch_size)?;
        Ok(embeddings)
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn init_reranker(&mut self) -> Result<()> {
        if self.reranker.is_none() {
            let model_enum = match self.reranker_model_name.to_lowercase().as_str() {
                "bge-reranker-base" => RerankerModel::BGERerankerBase,
                // "bge-reranker-v2-m3" => RerankerModel::BGERerankerV2M3, // Not verified
                _ => {
                    tracing::warn!(
                        "Unknown reranker model '{}', defaulting to BGERerankerBase",
                        self.reranker_model_name
                    );
                    RerankerModel::BGERerankerBase
                }
            };

            let mut rerank_init_options = RerankInitOptions::default();
            rerank_init_options.model_name = model_enum;
            if let Some(path) = self.reranker_model_path.as_ref() {
                rerank_init_options.cache_dir = PathBuf::from(path);
            }

            self.reranker = Some(TextRerank::try_new(rerank_init_options)?);
        }
        Ok(())
    }

    pub fn rerank(
        &mut self,
        query: &str,
        documents: Vec<String>,
        top_k: usize,
    ) -> Result<Vec<(usize, f32)>> {
        if let Some(reranker) = &mut self.reranker {
            let doc_refs: Vec<&str> = documents.iter().map(|s| s.as_str()).collect();
            let results = reranker.rerank(query, doc_refs, true, Some(top_k))?;
            Ok(results.into_iter().map(|r| (r.index, r.score)).collect())
        } else {
            // If no reranker is allowed (e.g. quiet mode or explicit configuration),
            // we technically can't rerank.
            // However, the caller should handle this or we return error.
            anyhow::bail!("Reranker not initialized")
        }
    }
}
