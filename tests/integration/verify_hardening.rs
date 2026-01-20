use crate::common::{cleanup_test_db, setup_test_env};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use code_rag::server::{
    create_router,
    workspace_manager::{WorkspaceManager, WorkspaceStats},
    AppState, ServerStartConfig,
};
use std::sync::Arc;
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
async fn test_server_status_endpoint() {
    // 1. Setup Environment
    let (_storage, embedder, _chunker, db_path) = setup_test_env("hardening_status").await;

    // 2. Initialize Manager
    let config = create_test_config(&db_path);
    let manager = WorkspaceManager::new(config, Arc::new(embedder), None);

    // 3. Create Router
    let state = AppState {
        workspace_manager: Arc::new(manager),
    };
    let app = create_router(state);

    // 4. Test /status
    let response = app
        .oneshot(
            Request::builder()
                .uri("/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Verify Response Structure
    let body_bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    let stats: WorkspaceStats =
        serde_json::from_slice(&body_bytes).expect("Failed to parse status");

    // Initially, no workspaces loaded or locked
    assert_eq!(stats.loaded_workspaces, 0);
    assert_eq!(stats.active_locks, 0);

    cleanup_test_db(&db_path);
}
