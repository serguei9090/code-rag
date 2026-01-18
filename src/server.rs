use crate::bm25::BM25Index;
use crate::core::CodeRagError;
use crate::embedding::Embedder;
use crate::llm::client::OllamaClient;
use crate::llm::expander::QueryExpander;
use crate::search::{CodeSearcher, SearchResult};
use crate::storage::Storage;
use anyhow::Result;
use axum::{
    extract::{Json, State},
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

// Shared state holding the searcher
// We need Mutex because CodeSearcher::semantic_search takes &mut self
// (embedder inside it might need mutable access for internal buffers/onnx state)
#[derive(Clone)]
pub struct AppState {
    pub searcher: Arc<Mutex<CodeSearcher>>,
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

    // 1. Init Storage
    let storage = Storage::new(&config.db_path).await?;
    // Ensure table exists (optional, but good safety)
    if storage.get_indexed_metadata().await.is_err() {
        info!("Warning: Storage might not be initialized. Please run 'index' first.");
    }

    // 2. Init Embedder (with re-ranker)
    let embedder = Embedder::new(
        config.embedding_model.clone(),
        config.reranker_model.clone(),
        config.embedding_model_path,
        config.reranker_model_path,
        config.device.clone(),
    )?;
    embedder.init_reranker()?; // Pre-load re-ranker

    // 3. Create Searcher
    let bm25_index = BM25Index::new(&config.db_path, true, "log").ok();
    if bm25_index.is_none() {
        info!("Warning: BM25 index could not be opened. Falling back to pure vector search.");
    }

    let expander = if config.llm_enabled {
        let client = OllamaClient::new(&config.llm_host, &config.llm_model);
        Some(Arc::new(QueryExpander::new(
            Arc::new(client) as Arc<dyn crate::llm::client::LlmClient + Send + Sync>
        )))
    } else {
        None
    };

    let searcher = CodeSearcher::new(
        Some(storage),
        Some(embedder),
        bm25_index,
        expander,
        1.0,
        1.0,
        60.0,
    );
    let state = AppState {
        searcher: Arc::new(Mutex::new(searcher)),
    };

    // 4. Build Router
    let app = create_router(state);

    // 5. Run
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .route("/search", post(search_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        error!("Failed to encode metrics: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(axum::http::header::CONTENT_TYPE, "text/plain")],
            "Internal Server Error".into(),
        );
    }

    // Convert buffer to String to own the data
    let response_body =
        String::from_utf8(buffer).unwrap_or_else(|_| "Error encoding metrics".to_string());

    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4",
        )],
        response_body,
    )
}

async fn search_handler(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> impl IntoResponse {
    // Lock the searcher
    // We use tokio::sync::Mutex because we hold the lock across an .await point (semantic_search)
    let searcher = state.searcher.lock().await;

    info!(
        query = %payload.query,
        limit = payload.limit,
        ext = ?payload.ext,
        dir = ?payload.dir,

        no_rerank = payload.no_rerank,
        expand = payload.expand,
        "Handling search request"
    );

    match searcher
        .semantic_search(
            &payload.query,
            payload.limit,
            payload.ext.clone(),
            payload.dir.clone(),
            payload.no_rerank,
            None, // workspace (default/global for none)
            payload.max_tokens,
            payload.expand,
        )
        .await
    {
        Ok(results) => {
            info!("Search successful, {} results found.", results.len());
            Ok((StatusCode::OK, Json(SearchResponse { results })).into_response())
        }
        Err(e) => {
            error!("Search failed: {:?}", e);
            Err(CodeRagError::Search(e.to_string()))
        }
    }
}
