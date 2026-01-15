use crate::bm25::BM25Index;
use crate::embedding::Embedder;
use crate::search::{CodeSearcher, SearchResult};
use crate::storage::Storage;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

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
}

fn default_limit() -> usize {
    5
}

// Response payload
#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

pub async fn start_server(
    host: String,
    port: u16,
    db_path: String,
) -> Result<(), Box<dyn StdError>> {
    println!("Initializing server components...");

    // 1. Init Storage
    let storage = Storage::new(&db_path).await?;
    // Ensure table exists (optional, but good safety)
    if storage.get_indexed_metadata().await.is_err() {
        println!("Warning: Storage might not be initialized. Please run 'index' first.");
    }

    // 2. Init Embedder (with re-ranker)
    let mut embedder = Embedder::new()?;
    embedder.init_reranker()?; // Pre-load re-ranker

    // 3. Create Searcher
    let bm25_index = BM25Index::new(&db_path).ok();
    if bm25_index.is_none() {
        println!("Warning: BM25 index could not be opened. Falling back to pure vector search.");
    }
    let searcher = CodeSearcher::new(Some(storage), Some(embedder), bm25_index);
    let state = AppState {
        searcher: Arc::new(Mutex::new(searcher)),
    };

    // 4. Build Router
    let app = create_router(state);

    // 5. Run
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/search", post(search_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

async fn search_handler(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> impl IntoResponse {
    // Lock the searcher
    // We use tokio::sync::Mutex because we hold the lock across an .await point (semantic_search)
    let mut searcher = state.searcher.lock().await;

    println!(
        "Handling search: '{}' (limit: {})",
        payload.query, payload.limit
    );

    match searcher
        .semantic_search(
            &payload.query,
            payload.limit,
            payload.ext.clone(),
            payload.dir.clone(),
            payload.no_rerank,
        )
        .await
    {
        Ok(results) => (StatusCode::OK, Json(SearchResponse { results })),
        Err(e) => {
            eprintln!("Search error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SearchResponse { results: vec![] }),
            ) // Simplified error response
        }
    }
}
