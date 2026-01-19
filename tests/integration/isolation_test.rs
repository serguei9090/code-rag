use crate::common;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use code_rag::server::workspace_manager::WorkspaceManager;
use code_rag::server::{create_router, AppState, ServerStartConfig};

use common::{cleanup_test_db, setup_test_env, TEST_ASSETS_PATH};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_workspace_isolation() {
    // 1. Setup Test Environment
    // We get a root temp dir.
    let (root_storage, embedder, chunker, root_db_path) = setup_test_env("isolation_test").await;
    // We won't use root_storage directly for adding chunks, but we needed the env setup.
    let embedder = Arc::new(embedder);

    // 2. Prepare Data for Workspace A
    // Logical Isolation: We use the SAME storage instance (root_storage)
    // but different workspace IDs ("workspace_a" and "workspace_b").

    // Index file for A
    let path_a = Path::new(TEST_ASSETS_PATH).join("test.rs");
    let code_a = fs::read_to_string(&path_a).expect("Failed to read test.rs");
    let mut reader_a = std::io::Cursor::new(code_a.as_bytes());
    let chunks_a = chunker.chunk_file("test_a.rs", &mut reader_a, 0).unwrap();

    let texts_a: Vec<String> = chunks_a.iter().map(|c| c.code.clone()).collect();
    let embeddings_a = embedder.embed(texts_a, None).expect("Embed failed A");
    let (ids_a, filenames_a, codes_a, starts_a, ends_a, mtimes_a, calls_a) =
        common::prepare_chunks(&chunks_a);

    root_storage
        .add_chunks(
            "workspace_a", // Must match the URL param later
            ids_a,
            filenames_a,
            codes_a,
            starts_a,
            ends_a,
            mtimes_a,
            calls_a,
            embeddings_a,
        )
        .await
        .expect("Add A failed");

    // 3. Prepare Data for Workspace B
    // Index DIFFERENT content for B
    let code_b = "fn unique_function_b() { println!(\"I am B\"); }";
    let mut reader_b = std::io::Cursor::new(code_b.as_bytes());
    let chunks_b = chunker.chunk_file("unique_b.rs", &mut reader_b, 0).unwrap();

    let texts_b: Vec<String> = chunks_b.iter().map(|c| c.code.clone()).collect();
    let embeddings_b = embedder.embed(texts_b, None).expect("Embed failed B");
    let (ids_b, filenames_b, codes_b, starts_b, ends_b, mtimes_b, calls_b) =
        common::prepare_chunks(&chunks_b);

    root_storage
        .add_chunks(
            "workspace_b", // Must match the URL param later
            ids_b,
            filenames_b,
            codes_b,
            starts_b,
            ends_b,
            mtimes_b,
            calls_b,
            embeddings_b,
        )
        .await
        .expect("Add B failed");

    // 4. Initialize Server WorkspaceManager
    let config = ServerStartConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        db_path: root_db_path.clone(), // Root containing workspace_a and workspace_b
        embedding_model: "dummy".to_string(),
        reranker_model: "dummy".to_string(),
        embedding_model_path: None,
        reranker_model_path: None,
        device: "cpu".to_string(),
        llm_enabled: false,
        llm_host: "".to_string(),
        llm_model: "".to_string(),
    };

    let manager = WorkspaceManager::new(config, embedder.clone(), None);
    let state = AppState {
        workspace_manager: Arc::new(manager),
    };
    let app = create_router(state);

    // 5. Query Workspace A
    let payload_a = serde_json::json!({
        "query": "rust function",
        "limit": 5
    });

    let req_a = Request::builder()
        .method("POST")
        .uri("/v1/workspace_a/search")
        .header("content-type", "application/json")
        .body(Body::from(payload_a.to_string()))
        .unwrap();

    let response_a = app.clone().oneshot(req_a).await.unwrap();
    assert_eq!(response_a.status(), StatusCode::OK);

    let body_a: serde_json::Value = serde_json::from_slice(
        &http_body_util::BodyExt::collect(response_a.into_body())
            .await
            .unwrap()
            .to_bytes(),
    )
    .unwrap();

    let results_a = body_a["results"].as_array().unwrap();
    // Should verify we found content from A
    let found_a = results_a
        .iter()
        .any(|r| r["filename"].as_str().unwrap() == "test_a.rs");
    assert!(found_a, "Should find content in workspace A");

    // 6. Query Workspace B
    let payload_b = serde_json::json!({
        "query": "unique_function_b",
        "limit": 5
    });

    let req_b = Request::builder()
        .method("POST")
        .uri("/v1/workspace_b/search")
        .header("content-type", "application/json")
        .body(Body::from(payload_b.to_string()))
        .unwrap();

    let response_b = app.clone().oneshot(req_b).await.unwrap();
    assert_eq!(response_b.status(), StatusCode::OK);

    let body_b: serde_json::Value = serde_json::from_slice(
        &http_body_util::BodyExt::collect(response_b.into_body())
            .await
            .unwrap()
            .to_bytes(),
    )
    .unwrap();

    let results_b = body_b["results"].as_array().unwrap();
    let found_b = results_b
        .iter()
        .any(|r| r["filename"].as_str().unwrap() == "unique_b.rs");
    assert!(found_b, "Should find content in workspace B");

    // 7. Verify Isolation (Search for B's unique content in A)
    let payload_cross = serde_json::json!({
        "query": "unique_function_b",
        "limit": 5
    });
    let req_cross = Request::builder()
        .method("POST")
        .uri("/v1/workspace_a/search")
        .header("content-type", "application/json")
        .body(Body::from(payload_cross.to_string()))
        .unwrap();

    let response_cross = app.clone().oneshot(req_cross).await.unwrap();
    assert_eq!(
        response_cross.status(),
        StatusCode::OK,
        "Cross-workspace search failed"
    );
    let body_cross: serde_json::Value = serde_json::from_slice(
        &http_body_util::BodyExt::collect(response_cross.into_body())
            .await
            .unwrap()
            .to_bytes(),
    )
    .unwrap();
    let results_cross = body_cross["results"].as_array().unwrap();

    // Should NOT find unique_b.rs in workspace_a
    let found_cross = results_cross
        .iter()
        .any(|r| r["filename"].as_str().unwrap() == "unique_b.rs");
    assert!(!found_cross, "Should NOT find B's content in A");

    // 8. Test Invalid Workspace
    let req_invalid = Request::builder()
        .method("POST")
        .uri("/v1/non_existent/search")
        .body(Body::from(payload_a.to_string()))
        .unwrap();

    let response_invalid = app.clone().oneshot(req_invalid).await.unwrap();
    // The handler might return 200 with error, or 500, or 404 depending on how we handled `Err` in `search_handler`.
    // In `server.rs`, `Err(e) => Err(CodeRagError::Search(e.to_string()))` converts to 500 or 400 usually.
    // Let's assert it is NOT 200 OK.
    assert_ne!(
        response_invalid.status(),
        StatusCode::OK,
        "Invalid workspace should fail"
    );

    cleanup_test_db(&root_db_path);
}
