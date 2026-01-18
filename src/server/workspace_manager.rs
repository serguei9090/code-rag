use crate::bm25::BM25Index;
use crate::embedding::Embedder;
use crate::llm::expander::QueryExpander;
use crate::search::CodeSearcher;
use crate::server::ServerStartConfig;
use crate::storage::Storage;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub struct WorkspaceManager {
    workspaces: DashMap<String, Arc<Mutex<CodeSearcher>>>,
    config: Arc<ServerStartConfig>,
    embedder: Arc<Embedder>,
    expander: Option<Arc<QueryExpander>>,
}

impl WorkspaceManager {
    pub fn new(
        config: ServerStartConfig,
        embedder: Arc<Embedder>,
        expander: Option<Arc<QueryExpander>>,
    ) -> Self {
        Self {
            workspaces: DashMap::new(),
            config: Arc::new(config),
            embedder,
            expander,
        }
    }

    /// Retrieves a searcher for the given workspace ID.
    ///
    /// If the searcher is not in the cache, it attempts to load logic from:
    /// `config.db_path / workspace_id`.
    ///
    /// The "default" workspace works on `config.db_path` directly to maintain backward compatibility.
    pub async fn get_searcher(&self, workspace_id: &str) -> Result<Arc<Mutex<CodeSearcher>>> {
        if let Some(entry) = self.workspaces.get(workspace_id) {
            return Ok(entry.clone());
        }

        // Cache miss - try to load
        let searcher = self.load_searcher(workspace_id).await?;
        let searcher_arc = Arc::new(Mutex::new(searcher));

        self.workspaces
            .insert(workspace_id.to_string(), searcher_arc.clone());
        Ok(searcher_arc)
    }

    async fn load_searcher(&self, workspace_id: &str) -> Result<CodeSearcher> {
        let db_path = if workspace_id == "default" {
            PathBuf::from(&self.config.db_path)
        } else {
            Path::new(&self.config.db_path).join(workspace_id)
        };

        if !db_path.exists() {
            return Err(anyhow!(
                "Workspace '{}' not found at {:?}",
                workspace_id,
                db_path
            ));
        }

        info!("Loading workspace '{}' from {:?}", workspace_id, db_path);

        let storage_path = db_path.to_string_lossy().to_string();
        let storage = Storage::new(&storage_path).await?;

        // Ensure valid index
        if storage.get_indexed_metadata().await.is_err() {
            return Err(anyhow!(
                "Workspace '{}' does not appear to be a valid index.",
                workspace_id
            ));
        }

        let bm25_index = BM25Index::new(&storage_path, true, "log").ok();
        if bm25_index.is_none() {
            warn!("BM25 index not found for workspace '{}'", workspace_id);
        }

        Ok(CodeSearcher::new(
            Some(Arc::new(storage)),
            Some(self.embedder.clone()), // Share the heavy embedder
            bm25_index.map(Arc::new),
            self.expander.clone(), // Share the LLM client
            1.0,
            1.0,
            60.0,
        ))
    }
}
