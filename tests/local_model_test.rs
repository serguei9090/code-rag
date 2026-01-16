use code_rag::embedding::Embedder;
use code_rag::indexer::CodeChunker;
use code_rag::search::CodeSearcher;
use code_rag::storage::Storage;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

mod common;
use common::prepare_chunks;

#[tokio::test]
async fn test_local_model_loading() {
    let tmp_dir = tempdir().unwrap();
    let db_path = tmp_dir
        .path()
        .join("test_local.db")
        .to_str()
        .unwrap()
        .to_string();

    // Path to the local model fixture
    // Note: We use the absolute path for the model loading
    let current_dir = std::env::current_dir().unwrap();
    let model_path = current_dir
        .join("tests")
        .join("fixtures")
        .join("models")
        .join("bge-small-en-v1.5");
    let model_path_str = model_path.to_str().unwrap().to_string();

    // Ensure model path exists
    assert!(
        model_path.exists(),
        "Model path {} does not exist",
        model_path_str
    );

    // 1. Initialize Storage
    let storage = Storage::new(&db_path)
        .await
        .expect("Failed to create storage");
    // 2. Initialize Embedder with local path
    let mut embedder = Embedder::new(
        "unused".to_string(),
        "bge-reranker-base".to_string(),
        Some(model_path_str),
        None,
    )
    .expect("Failed to initialize embedder with local path");

    storage
        .init(embedder.dim())
        .await
        .expect("Failed to init storage"); // CRITICAL: Need to init for LanceDB

    // 3. Index some dummy content manually (reproducing main.rs logic)
    let test_file = tmp_dir.path().join("test.rs");
    let code = "fn hello_world() { println!(\"Hello!\"); }";
    fs::write(&test_file, code).unwrap();

    let chunker = CodeChunker::default();
    let chunks = chunker.chunk_file("test.rs", code, 0);
    assert!(!chunks.is_empty());

    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = embedder.embed(texts, None).expect("Embedding failed");
    let (ids, filenames, codes, line_starts, line_ends, last_modified, calls) =
        prepare_chunks(&chunks);

    storage
        .add_chunks(
            ids,
            filenames,
            codes,
            line_starts,
            line_ends,
            last_modified,
            calls,
            embeddings,
        )
        .await
        .expect("Failed to add chunks");

    // 4. Create Searcher
    let mut searcher = CodeSearcher::new(Some(storage), Some(embedder), None);

    // 5. Perform a search
    let results = searcher
        .semantic_search("hello", 1, None, None, true)
        .await
        .expect("Search failed");

    // 6. Verify results
    assert!(!results.is_empty(), "Should have found at least one result");
    assert!(results[0].code.contains("hello_world"));
}
