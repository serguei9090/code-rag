use tracing::{error, info};

use crate::bm25::BM25Index;
use crate::config::AppConfig;
use crate::core::CodeRagError;
use crate::embedding::Embedder;
use crate::indexer::CodeChunker;
use crate::storage::Storage;
use crate::watcher::start_watcher;

pub async fn watch_codebase(
    path: Option<String>,
    db_path: Option<String>,
    config: &AppConfig,
) -> Result<(), CodeRagError> {
    let actual_path = path.unwrap_or_else(|| config.default_index_path.clone());
    let actual_db = db_path.unwrap_or_else(|| config.db_path.clone());

    info!("Initializing watcher for path: {}", actual_path);

    // 1. Initialize Components
    let mut embedder = Embedder::new(
        config.embedding_model.clone(),
        config.reranker_model.clone(),
        config.embedding_model_path.clone(),
        config.reranker_model_path.clone(),
    )?;
    embedder
        .init_reranker()
        .map_err(|e| CodeRagError::Embedding(e.to_string()))?;

    let storage = Storage::new(&actual_db)
        .await
        .map_err(|e| CodeRagError::Database(e.to_string()))?;
    storage
        .init(embedder.dim())
        .await
        .map_err(|e| CodeRagError::Database(e.to_string()))?; // Ensure schema

    let bm25_index = match BM25Index::new(&actual_db, false, &config.merge_policy) {
        Ok(idx) => idx,
        Err(e) => {
            error!("Failed to initialize BM25 index: {}", e);
            return Err(CodeRagError::Tantivy(e.to_string()));
        }
    };

    let chunker = CodeChunker::new(config.chunk_size, config.chunk_overlap);

    // 2. Start Watcher
    start_watcher(&actual_path, storage, embedder, bm25_index, chunker)
        .await
        .map_err(|e| CodeRagError::Io(std::io::Error::other(e.to_string())))?;

    Ok(())
}
