use crate::common;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use code_rag::search::CodeSearcher;
use code_rag::server::{create_router, AppState};
use common::{cleanup_test_db, prepare_chunks, setup_test_env, TEST_ASSETS_PATH};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let (storage, embedder, _, db_path) = setup_test_env("health_check").await;
    let searcher = CodeSearcher::new(Some(storage), Some(embedder), None, 1.0, 1.0, 60.0);
    let state = AppState {
        searcher: Arc::new(Mutex::new(searcher)),
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
    let (storage, mut embedder, chunker, db_path) = setup_test_env("server_search").await;

    // Index a file
    let path = Path::new(TEST_ASSETS_PATH).join("test.rs");
    let code = fs::read_to_string(&path).expect("Failed to read test.rs");
    let chunks = chunker.chunk_file("test.rs", &code, 0);

    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = embedder.embed(texts, None).expect("Embed failed");
    let (ids, filenames, codes, starts, ends, mtimes, calls) = prepare_chunks(&chunks);
    storage
        .add_chunks(
            ids, filenames, codes, starts, ends, mtimes, calls, embeddings,
        )
        .await
        .expect("Add failed");

    // Initialize Server
    let searcher = CodeSearcher::new(Some(storage), Some(embedder), None, 1.0, 1.0, 60.0);
    let state = AppState {
        searcher: Arc::new(Mutex::new(searcher)),
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
