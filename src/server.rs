use crate::bm25::BM25Index;
use crate::core::CodeRagError;
use crate::embedding::Embedder;
use crate::llm::client::OllamaClient;
use crate::llm::expander::QueryExpander;
use crate::search::{CodeSearcher, SearchResult};
pub mod workspace_manager;
use crate::server::workspace_manager::WorkspaceManager;
use crate::storage::Storage;
use anyhow::Result;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use prometheus::{Encoder, TextEncoder};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

// Shared state holding the workspace manager
#[derive(Clone)]
pub struct AppState {
    pub workspace_manager: Arc<WorkspaceManager>,
}

// Request payload
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub ext: Option<String>,
    pub dir: Option<String>,
    #[serde(default)]
    pub no_rerank: bool,

    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub expand: bool,
}

fn default_limit() -> usize {
    5
}

// Response payload
#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

pub struct ServerStartConfig {
    pub host: String,
    pub port: u16,
    pub db_path: String,
    pub embedding_model: String,
    pub reranker_model: String,
    pub embedding_model_path: Option<String>,
    pub reranker_model_path: Option<String>,
    pub device: String,
    pub llm_enabled: bool,
    pub llm_host: String,
    pub llm_model: String,
}

pub async fn start_server(config: ServerStartConfig) -> Result<()> {
    info!("Initializing server components...");

    // Extract connection info before moving config
    let host = config.host.clone();
    let port = config.port;

    // 1. Init Embedder (with re-ranker) - Shared across workspaces
    let embedder = Embedder::new(
        config.embedding_model.clone(),
        config.reranker_model.clone(),
        config.embedding_model_path.clone(),
        config.reranker_model_path.clone(),
        config.device.clone(),
    )?;
    embedder.init_reranker()?; // Pre-load re-ranker
    let embedder = Arc::new(embedder);

    // 2. Init LLM Client (Optional) - Shared
    let expander = if config.llm_enabled {
        let client = OllamaClient::new(&config.llm_host, &config.llm_model);
        Some(Arc::new(QueryExpander::new(
            Arc::new(client) as Arc<dyn crate::llm::client::LlmClient + Send + Sync>
        )))
    } else {
        None
    };

    // 3. Init WorkspaceManager
    let manager = WorkspaceManager::new(config, embedder, expander);

    // Pre-load default workspace if exists
    if let Err(e) = manager.get_searcher("default").await {
        info!("Note: Default workspace could not be pre-loaded: {}", e);
    } else {
        info!("Default workspace pre-loaded successfully.");
    }

    let state = AppState {
        workspace_manager: Arc::new(manager),
    };

    // 4. Build Router
    let router = create_router(state);

    // 5. Bind & Serve
    let addr = SocketAddr::new(host.parse()?, port);
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

/// Create router with routes and middleware
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/search", post(search_handler_default))
        .route("/v1/{workspace}/search", post(search_handler_workspace))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!("http_request", method = ?request.method(), uri = ?request.uri())
                })
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Health check handler
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

/// Handler for default workspace (POST /search)
async fn search_handler_default(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> impl IntoResponse {
    process_search(state, "default".to_string(), payload).await
}

/// Handler for specific workspace (POST /v1/:workspace/search)
async fn search_handler_workspace(
    State(state): State<AppState>,
    Path(workspace): Path<String>,
    Json(payload): Json<SearchRequest>,
) -> impl IntoResponse {
    process_search(state, workspace, payload).await
}

/// Core search logic shared by handlers
async fn process_search(
    state: AppState,
    workspace: String,
    payload: SearchRequest,
) -> impl IntoResponse {
    // 1. Get Searcher for Workspace
    let searcher_arc = match state.workspace_manager.get_searcher(&workspace).await {
        Ok(s) => s,
        Err(e) => {
            // Distinguish between errors if possible, but for now generic 404/400
            // If the workspace doesn't exist on disk, get_searcher fails.
            let error_msg = format!("Failed to access workspace '{}': {}", workspace, e);
            return (StatusCode::NOT_FOUND, error_msg).into_response();
        }
    };

    // 2. Lock Searcher
    let mut searcher = searcher_arc.lock().await;

    // 3. Execute Search
    let results = match searcher
        .semantic_search(
            &payload.query,
            payload.limit,
            payload.ext,
            payload.dir,
            payload.no_rerank,
            None,
            payload.max_tokens,
            payload.expand,
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Search error in workspace '{}': {}", workspace, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    // 4. Return Results
    (StatusCode::OK, Json(SearchResponse { results })).into_response()
}
