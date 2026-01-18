use crate::common;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use code_rag::server::workspace_manager::WorkspaceManager;
use code_rag::server::{create_router, AppState, ServerStartConfig};
use common::{cleanup_test_db, prepare_chunks, setup_test_env, TEST_ASSETS_PATH};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

fn create_test_config(db_path: &str) -> ServerStartConfig {
    ServerStartConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        db_path: db_path.to_string(),
        embedding_model: "dummy".to_string(),
        reranker_model: "dummy".to_string(),
        embedding_model_path: None,
        reranker_model_path: None,
        device: "cpu".to_string(),
        llm_enabled: false,
        llm_host: "".to_string(),
        llm_model: "".to_string(),
    }
}

#[tokio::test]
async fn test_health_check() {
    let (storage, embedder, _, db_path) = setup_test_env("health_check").await;
    // We don't need to manually create searcher anymore, WorkspaceManager constructs it.
    // However, we do need to pass the shared embedder.

    let config = create_test_config(&db_path);
    let manager = WorkspaceManager::new(config, Arc::new(embedder), None);
    // Explicitly insert a dummy searcher OR ensure `default` loads from the empty DB?
    // StartServer tries to load "default".
    // For health check, we don't strictly need a searcher loaded.

    let state = AppState {
        workspace_manager: Arc::new(manager),
    };

    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Cleanup
    cleanup_test_db(&db_path);
}

#[tokio::test]
async fn test_search_endpoint() {
    // Setup environment
    let (storage, embedder, chunker, db_path) = setup_test_env("server_search").await;

    // Index a file
    let path = Path::new(TEST_ASSETS_PATH).join("test.rs");
    let code = fs::read_to_string(&path).expect("Failed to read test.rs");
    let mut reader = std::io::Cursor::new(code.as_bytes());
    let chunks = chunker.chunk_file("test.rs", &mut reader, 0).unwrap();

    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = embedder.embed(texts, None).expect("Embed failed");
    let (ids, filenames, codes, starts, ends, mtimes, calls) = prepare_chunks(&chunks);
    storage
        .add_chunks(
            "default", ids, filenames, codes, starts, ends, mtimes, calls, embeddings,
        )
        .await
        .expect("Add failed");

    // Initialize Server via WorkspaceManager
    let config = create_test_config(&db_path);
    // We reuse the embedder. Note: we used it above, so we might need to clone if mutable?
    // Use the variable `embedder`. `embedder.embed` took &self.
    let manager = WorkspaceManager::new(config, Arc::new(embedder), None);

    // Ensure "default" workspace works
    // Since manually added chunks to "default" in storage, and config.db_path points to storage root,
    // accessing "default" via WorkspaceManager (which maps to db_path) should see the data.

    let state = AppState {
        workspace_manager: Arc::new(manager),
    };
    let app = create_router(state);

    // Prepare JSON payload
    let payload = serde_json::json!({
        "query": "rust function",
        "limit": 2
    });

    let req = Request::builder()
        .method("POST")
        .uri("/search")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse body
    let body_bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(body.get("results").is_some());
    let results = body["results"].as_array().unwrap();
    assert!(!results.is_empty(), "Expected search results");

    // Verify result content
    let first_result = &results[0];
    assert!(first_result["filename"]
        .as_str()
        .unwrap()
        .contains("test.rs"));

    cleanup_test_db(&db_path);
}

#[tokio::test]
async fn test_concurrent_searches() {
    // Setup environment
    let (storage, mut embedder, chunker, db_path) = setup_test_env("server_stress").await;

    // Index a file to search against
    let path = Path::new(TEST_ASSETS_PATH).join("test.rs");
    let code = fs::read_to_string(&path).expect("Failed to read test.rs");
    let mut reader = std::io::Cursor::new(code.as_bytes());
    let chunks = chunker.chunk_file("test.rs", &mut reader, 0).unwrap();
    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = embedder.embed(texts, None).expect("Embed failed");
    let (ids, filenames, codes, starts, ends, mtimes, calls) = prepare_chunks(&chunks);
    storage
        .add_chunks(
            "default", ids, filenames, codes, starts, ends, mtimes, calls, embeddings,
        )
        .await
        .expect("Add failed");

    // Initialize Server
    let config = create_test_config(&db_path);
    let manager = WorkspaceManager::new(config, Arc::new(embedder), None);

    let state = AppState {
        workspace_manager: Arc::new(manager),
    };
    let app = create_router(state);

    let mut handles = Vec::new();
    let num_requests = 20;

    for i in 0..num_requests {
        // Router is Clone
        let app_clone = app.clone();

        let payload = serde_json::json!({
            "query": format!("query {}", i),
            "limit": 1
        });

        let handle = tokio::spawn(async move {
            let req = Request::builder()
                .method("POST")
                .uri("/search")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap();

            app_clone.oneshot(req).await
        });
        handles.push(handle);
    }

    // Await all
    for handle in handles {
        let result = handle.await.unwrap(); // join error
        let response = result.unwrap(); // oneshot error/hyper error
        assert_eq!(response.status(), StatusCode::OK);
    }

    cleanup_test_db(&db_path);
}
