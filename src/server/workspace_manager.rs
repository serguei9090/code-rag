use crate::bm25::BM25Index;
use crate::embedding::Embedder;
use crate::llm::expander::QueryExpander;
use crate::search::CodeSearcher;
use crate::server::ServerStartConfig;
use crate::storage::Storage;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use std::path::PathBuf;
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
        // Logical Isolation: All workspaces share the same physical DB path.
        // Isolation is handled by "workspace" column in LanceDB and field in BM25.
        // Previously this logic attempted to create subdirectories, which conflicted with Indexer logic.
        let db_path = PathBuf::from(&self.config.db_path);

        if !db_path.exists() {
            // Note: If the root DB doesn't exist, no workspace exists.
            // If the DB exists but has no data for this workspace, that's handled by search returning empty.
            return Err(anyhow!("Database root not found at {:?}", db_path));
        }

        info!(
            "Loading workspace '{}' context from {:?}",
            workspace_id, db_path
        );

        let storage_path = db_path.to_string_lossy().to_string();
        let storage = Storage::new(&storage_path).await?;

        // Ensure valid index (and check if we have data for this workspace?)
        // Providing workspace_id allows checking specifically for that workspace's metadata.
        if storage.get_indexed_metadata(workspace_id).await.is_err() {
            // It's acceptable if it's empty, but if the TABLE doesn't exist or error occurs:
            // Warn but proceed? Or error out?
            // For now, let's just log.
            warn!(
                "Could not fetch metadata for workspace '{}'. It might be empty.",
                workspace_id
            );
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
