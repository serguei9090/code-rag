use code_rag::embedding::Embedder;
use code_rag::indexer::CodeChunker;
use code_rag::storage::Storage;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const TEST_DB_BASE_PATH: &str = "./.lancedb-test";
pub const TEST_ASSETS_PATH: &str = "./test_assets";

/// Helper to clean up test database
pub fn cleanup_test_db(path: &str) {
    let _ = fs::remove_dir_all(path);
}

/// Helper to setup test environment with unique DB path
pub async fn setup_test_env(test_name: &str) -> (Storage, Embedder, CodeChunker, String) {
    // Generate unique path to allow parallel testing
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let db_path = format!("{}-{}-{}", TEST_DB_BASE_PATH, test_name, timestamp);

    if Path::new(&db_path).exists() {
        fs::remove_dir_all(&db_path).unwrap_or(());
    }

    let storage = Storage::new(&db_path)
        .await
        .expect("Failed to create storage");
    let embedder = Embedder::new(
        "nomic-embed-text-v1.5".to_string(),
        "bge-reranker-base".to_string(),
        None,
        None,
        "cpu".to_string(),
    )
    .expect("Failed to create embedder");
    storage
        .init(embedder.dim())
        .await
        .expect("Failed to init storage");
    let chunker = CodeChunker::default();
    (storage, embedder, chunker, db_path)
}

#[allow(clippy::type_complexity)]
pub fn prepare_chunks(
    chunks: &[code_rag::indexer::CodeChunk],
) -> (
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<i32>,
    Vec<i32>,
    Vec<i64>,
    Vec<Vec<String>>,
) {
    let ids = chunks
        .iter()
        .map(|c| format!("{}:{}", c.filename, c.line_start))
        .collect();
    let filenames = chunks.iter().map(|c| c.filename.clone()).collect();
    let codes = chunks.iter().map(|c| c.code.clone()).collect();
    let line_starts = chunks.iter().map(|c| c.line_start as i32).collect();
    let line_ends = chunks.iter().map(|c| c.line_end as i32).collect();
    let last_modified = chunks.iter().map(|c| c.last_modified).collect();
    let calls = chunks.iter().map(|c| c.calls.clone()).collect();
    (
        ids,
        filenames,
        codes,
        line_starts,
        line_ends,
        last_modified,
        calls,
    )
}
