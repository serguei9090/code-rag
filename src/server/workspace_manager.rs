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
use tracing::{info, warn};

/// Thread-safe search context for a single workspace.
///
/// All components are wrapped in Arc for concurrent access without locks.
pub struct WorkspaceSearchContext {
    pub storage: Arc<Storage>,
    pub embedder: Arc<Embedder>,
    pub bm25: Option<Arc<BM25Index>>,
    pub expander: Option<Arc<QueryExpander>>,
    pub vector_weight: f32,
    pub bm25_weight: f32,
    pub rrf_k: f64,
}

pub struct WorkspaceManager {
    workspaces: DashMap<String, Arc<WorkspaceSearchContext>>,
    loading_locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
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
            loading_locks: DashMap::new(),
            config: Arc::new(config),
            embedder,
            expander,
        }
    }

    /// Retrieves search context for the given workspace ID.
    ///
    /// Returns Arc<WorkspaceSearchContext> which can be shared across
    /// multiple concurrent requests without blocking.
    ///
    /// If the context is not in the cache, it attempts to load from:
    /// `config.db_path / workspace_id`.
    ///
    /// The "default" workspace works on `config.db_path` directly to maintain backward compatibility.
    pub async fn get_search_context(
        &self,
        workspace_id: &str,
    ) -> Result<Arc<WorkspaceSearchContext>> {
        if let Some(entry) = self.workspaces.get(workspace_id) {
            return Ok(entry.clone());
        }

        // Cache miss - synchronize loading to prevent race conditions
        // 1. Get or create a lock for this specific workspace ID
        let lock = self
            .loading_locks
            .entry(workspace_id.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone();

        // 2. Acquire the lock (await)
        let _guard = lock.lock().await;

        // 3. Double-check: verify if another thread loaded it while we were waiting
        if let Some(entry) = self.workspaces.get(workspace_id) {
            return Ok(entry.clone());
        }

        // 4. Actually load (still under lock)
        let context = self.load_search_context(workspace_id).await?;
        let context_arc = Arc::new(context);

        self.workspaces
            .insert(workspace_id.to_string(), context_arc.clone());

        // cleanup lock map to avoid memory leak?
        // Ideally we remove the lock, but doing so safely without re-introducing a race is tricky.
        // Given logical workspaces are few, keeping empty mutexes is acceptable overhead.

        Ok(context_arc)
    }

    /// Legacy compatibility method - returns CodeSearcher wrapped in Mutex.
    ///
    /// **Deprecated**: Use `get_search_context()` for better concurrency.
    pub async fn get_searcher(
        &self,
        workspace_id: &str,
    ) -> Result<Arc<tokio::sync::Mutex<CodeSearcher>>> {
        // For backward compatibility with existing code
        let context = self.get_search_context(workspace_id).await?;

        let searcher = CodeSearcher::new(
            Some(context.storage.clone()),
            Some(context.embedder.clone()),
            context.bm25.clone(),
            context.expander.clone(),
            context.vector_weight,
            context.bm25_weight,
            context.rrf_k,
        );

        Ok(Arc::new(tokio::sync::Mutex::new(searcher)))
    }

    async fn load_search_context(&self, workspace_id: &str) -> Result<WorkspaceSearchContext> {
        // Logical Isolation: All workspaces share the same physical DB path.
        // Isolation is handled by "workspace" column in LanceDB and field in BM25.
        let db_path = PathBuf::from(&self.config.db_path);

        if !db_path.exists() {
            return Err(anyhow!("Database root not found at {:?}", db_path));
        }

        info!(
            "Loading workspace '{}' context from {:?}",
            workspace_id, db_path
        );

        let storage_path = db_path.to_string_lossy().to_string();
        let storage = Storage::new(&storage_path).await?;

        // Ensure valid index (and check if we have data for this workspace?)
        if storage.get_indexed_metadata(workspace_id).await.is_err() {
            warn!(
                "Could not fetch metadata for workspace '{}'. It might be empty.",
                workspace_id
            );
        }

        let bm25_index = BM25Index::new(&storage_path, true, "log").ok();
        if bm25_index.is_none() {
            warn!("BM25 index not found for workspace '{}'", workspace_id);
        }

        Ok(WorkspaceSearchContext {
            storage: Arc::new(storage),
            embedder: self.embedder.clone(),
            bm25: bm25_index.map(Arc::new),
            expander: self.expander.clone(),
            vector_weight: 1.0,
            bm25_weight: 1.0,
            rrf_k: 60.0,
        })
    }
}
