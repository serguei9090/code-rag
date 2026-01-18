use code_rag::bm25::BM25Index;

use code_rag::indexer::CodeChunker;
use code_rag::search::CodeSearcher;

use std::fs;
use std::path::Path;

use crate::common;
use common::{cleanup_test_db, prepare_chunks, setup_test_env, TEST_ASSETS_PATH};

#[tokio::test]
async fn test_index_test_assets() {
    let (storage, embedder, chunker, db_path) = setup_test_env("index_assets").await;

    // Index all test assets
    let mut total_chunks = 0;
    let test_files = vec![
        "test.rs",
        "test.py",
        "test.go",
        "test.js",
        "test.java",
        "test.css",
        "test.html",
        "test.json",
        "test.yaml",
        "test.sh",
        "test.ps1",
    ];

    for file in test_files {
        let path = Path::new(TEST_ASSETS_PATH).join(file);
        // Initialize BM25 Index
        let _bm25_index =
            BM25Index::new(&db_path, false, "log").expect("Failed to create BM25 index");
        if path.exists() {
            let code = fs::read_to_string(&path).expect("Failed to read file");
            let mtime = fs::metadata(&path)
                .expect("Failed to get metadata")
                .modified()
                .expect("Failed to get mtime")
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time error")
                .as_secs() as i64;

            let mut reader = std::io::Cursor::new(code.as_bytes());
            let chunks = chunker
                .chunk_file(path.to_str().unwrap(), &mut reader, mtime)
                .unwrap();
            total_chunks += chunks.len();

            if !chunks.is_empty() {
                let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
                let embeddings = embedder.embed(texts, None).expect("Failed to embed");
                let (ids, filenames, codes, line_starts, line_ends, last_modified, calls) =
                    prepare_chunks(&chunks);
                storage
                    .add_chunks(
                        "default",
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
            }
        }
    }

    // Assert we indexed a reasonable number of chunks
    assert!(
        total_chunks > 20,
        "Expected at least 20 chunks, got {}",
        total_chunks
    );
    println!("✓ Indexed {} chunks from test assets", total_chunks);

    cleanup_test_db(&db_path);
}

#[tokio::test]
async fn test_search_rust_function() {
    let (storage, embedder, chunker, db_path) = setup_test_env("rust_search").await;

    // Index Rust test file
    let rust_path = Path::new(TEST_ASSETS_PATH).join("test.rs");
    let code = fs::read_to_string(&rust_path).expect("Failed to read Rust file");
    let mtime = 0;

    let mut reader = std::io::Cursor::new(code.as_bytes());
    let chunks = chunker
        .chunk_file(rust_path.to_str().unwrap(), &mut reader, mtime)
        .unwrap();
    assert!(!chunks.is_empty(), "No chunks found in test.rs");

    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = embedder.embed(texts, None).expect("Failed to embed");
    let (ids, filenames, codes, line_starts, line_ends, last_modified, calls) =
        prepare_chunks(&chunks);
    storage
        .add_chunks(
            "default",
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

    // Search for Rust function
    let mut searcher = CodeSearcher::new(
        Some(std::sync::Arc::new(storage)),
        Some(std::sync::Arc::new(embedder)),
        None,
        None,
        1.0,
        1.0,
        60.0,
    );
    let results = searcher
        .semantic_search(
            "rust function example",
            5,
            None,
            None,
            false,
            None,
            None,
            false,
        )
        .await
        .expect("Search failed");

    assert!(!results.is_empty(), "Search returned no results");
    assert!(
        results[0].filename.contains("test.rs"),
        "Top result should be from test.rs"
    );
    println!("✓ Found {} results for Rust function search", results.len());

    cleanup_test_db(&db_path);
}

#[tokio::test]
async fn test_search_python_class() {
    let (storage, embedder, chunker, db_path) = setup_test_env("py_search").await;

    // Index Python test file
    let py_path = Path::new(TEST_ASSETS_PATH).join("test.py");
    let code = fs::read_to_string(&py_path).expect("Failed to read Python file");
    let mtime = 0;

    let mut reader = std::io::Cursor::new(code.as_bytes());
    let chunks = chunker
        .chunk_file(py_path.to_str().unwrap(), &mut reader, mtime)
        .unwrap();
    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = embedder.embed(texts, None).expect("Failed to embed");
    let (ids, filenames, codes, line_starts, line_ends, last_modified, calls) =
        prepare_chunks(&chunks);
    storage
        .add_chunks(
            "default",
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

    // Search for Python content
    let mut searcher = CodeSearcher::new(
        Some(std::sync::Arc::new(storage)),
        Some(std::sync::Arc::new(embedder)),
        None,
        None,
        1.0,
        1.0,
        60.0,
    );
    let results = searcher
        .semantic_search("python function", 5, None, None, false, None, None, false)
        .await
        .expect("Search failed");

    assert!(!results.is_empty(), "Search returned no results for Python");
    println!("✓ Found {} results for Python search", results.len());

    cleanup_test_db(&db_path);
}

#[tokio::test]
async fn test_search_bash_script() {
    let (storage, embedder, chunker, db_path) = setup_test_env("bash_search").await;

    // Index Bash test file
    let bash_path = Path::new(TEST_ASSETS_PATH).join("test.sh");
    let code = fs::read_to_string(&bash_path).expect("Failed to read Bash file");
    let mtime = 0;

    let mut reader = std::io::Cursor::new(code.as_bytes());
    let chunks = chunker
        .chunk_file(bash_path.to_str().unwrap(), &mut reader, mtime)
        .unwrap();
    assert!(!chunks.is_empty(), "No chunks found in test.sh");

    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = embedder.embed(texts, None).expect("Failed to embed");
    let (ids, filenames, codes, line_starts, line_ends, last_modified, calls) =
        prepare_chunks(&chunks);
    storage
        .add_chunks(
            "default",
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

    // Search for Bash function
    let mut searcher = CodeSearcher::new(
        Some(std::sync::Arc::new(storage)),
        Some(std::sync::Arc::new(embedder)),
        None,
        None,
        1.0,
        1.0,
        60.0,
    );
    let results = searcher
        .semantic_search("backup logs", 5, None, None, false, None, None, false)
        .await
        .expect("Search failed");

    assert!(!results.is_empty(), "Search returned no results for Bash");
    assert!(
        results[0].code.contains("backup_logs") || results[0].code.contains("log"),
        "Result should contain backup or log related content"
    );
    println!("✓ Found {} results for Bash script search", results.len());

    cleanup_test_db(&db_path);
}

#[tokio::test]
async fn test_search_powershell_function() {
    let (storage, mut embedder, chunker, db_path) = setup_test_env("ps_search").await;

    // Index PowerShell test file
    let ps_path = Path::new(TEST_ASSETS_PATH).join("test.ps1");
    let code = fs::read_to_string(&ps_path).expect("Failed to read PowerShell file");
    let mtime = 0;

    let mut reader = std::io::Cursor::new(code.as_bytes());
    let chunks = chunker
        .chunk_file(ps_path.to_str().unwrap(), &mut reader, mtime)
        .unwrap();
    assert!(!chunks.is_empty(), "No chunks found in test.ps1");

    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = embedder.embed(texts, None).expect("Failed to embed");
    let (ids, filenames, codes, line_starts, line_ends, last_modified, calls) =
        prepare_chunks(&chunks);
    storage
        .add_chunks(
            "default",
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

    // Search for PowerShell function
    let mut searcher = CodeSearcher::new(
        Some(std::sync::Arc::new(storage)),
        Some(std::sync::Arc::new(embedder)),
        None,
        None,
        1.0,
        1.0,
        60.0,
    );
    let results = searcher
        .semantic_search("system status", 5, None, None, false, None, None, false)
        .await
        .expect("Search failed");

    assert!(
        !results.is_empty(),
        "Search returned no results for PowerShell"
    );
    println!("✓ Found {} results for PowerShell search", results.len());

    cleanup_test_db(&db_path);
}

#[tokio::test]
async fn test_search_json_config() {
    let (storage, mut embedder, chunker, db_path) = setup_test_env("json_search").await;

    // Index JSON test file
    let json_path = Path::new(TEST_ASSETS_PATH).join("test.json");
    let code = fs::read_to_string(&json_path).expect("Failed to read JSON file");
    let mtime = 0;

    let mut reader = std::io::Cursor::new(code.as_bytes());
    let chunks = chunker
        .chunk_file(json_path.to_str().unwrap(), &mut reader, mtime)
        .unwrap();
    assert!(!chunks.is_empty(), "No chunks found in test.json");

    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = embedder.embed(texts, None).expect("Failed to embed");
    let (ids, filenames, codes, line_starts, line_ends, last_modified, calls) =
        prepare_chunks(&chunks);
    storage
        .add_chunks(
            "default",
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

    // Search for JSON content
    let mut searcher = CodeSearcher::new(
        Some(std::sync::Arc::new(storage)),
        Some(std::sync::Arc::new(embedder)),
        None,
        None,
        1.0,
        1.0,
        60.0,
    );
    let results = searcher
        .semantic_search(
            "configuration database",
            5,
            None,
            None,
            false,
            None,
            None,
            false,
        )
        .await
        .expect("Search failed");

    assert!(!results.is_empty(), "Search returned no results for JSON");
    println!("✓ Found {} results for JSON search", results.len());

    cleanup_test_db(&db_path);
}

#[tokio::test]
async fn test_multi_language_search() {
    let (storage, mut embedder, chunker, db_path) = setup_test_env("multi_lang_search").await;

    // Index multiple languages
    let files = vec!["test.rs", "test.py", "test.go", "test.js"];
    let mut total_chunks = 0;

    for file in files {
        let path = Path::new(TEST_ASSETS_PATH).join(file);
        let code = fs::read_to_string(&path).expect("Failed to read file");
        let mut reader = std::io::Cursor::new(code.as_bytes());
        let chunks = chunker
            .chunk_file(path.to_str().unwrap(), &mut reader, 0)
            .unwrap();
        total_chunks += chunks.len();

        if !chunks.is_empty() {
            let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
            let embeddings = embedder.embed(texts, None).expect("Failed to embed");
            let (ids, filenames, codes, line_starts, line_ends, last_modified, calls) =
                prepare_chunks(&chunks);
            storage
                .add_chunks(
                    "default",
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
        }
    }

    // Search across all languages
    let mut searcher = CodeSearcher::new(
        Some(std::sync::Arc::new(storage)),
        Some(std::sync::Arc::new(embedder)),
        None,
        None,
        1.0,
        1.0,
        60.0,
    );
    let results = searcher
        .semantic_search(
            "function definition",
            10,
            None,
            None,
            false,
            None,
            None,
            false,
        )
        .await
        .expect("Search failed");

    assert!(
        !results.is_empty(),
        "Multi-language search returned no results"
    );
    assert!(total_chunks >= 4, "Expected at least 4 chunks from 4 files");
    println!(
        "✓ Multi-language search found {} results from {} chunks",
        results.len(),
        total_chunks
    );

    cleanup_test_db(&db_path);
}

#[test]
fn test_language_detection() {
    let _chunker = CodeChunker::default();

    // Test language detection for all supported extensions
    assert!(
        CodeChunker::get_language("rs").is_some(),
        "Rust not detected"
    );
    assert!(
        CodeChunker::get_language("py").is_some(),
        "Python not detected"
    );
    assert!(CodeChunker::get_language("go").is_some(), "Go not detected");
    assert!(
        CodeChunker::get_language("js").is_some(),
        "JavaScript not detected"
    );
    assert!(
        CodeChunker::get_language("sh").is_some(),
        "Bash not detected"
    );
    assert!(
        CodeChunker::get_language("ps1").is_some(),
        "PowerShell not detected"
    );
    assert!(
        CodeChunker::get_language("json").is_some(),
        "JSON not detected"
    );
    assert!(
        CodeChunker::get_language("yaml").is_some(),
        "YAML not detected"
    );
    assert!(
        CodeChunker::get_language("unknown").is_none(),
        "Unknown extension should return None"
    );

    println!("✓ All language detections passed");
}

#[test]
fn test_chunking_rust_file() {
    let chunker = CodeChunker::default();
    let rust_code = r#"
fn main() {
    println!("Hello, world!");
}

fn helper() {
    println!("Helper function");
}
"#;

    let mut reader = std::io::Cursor::new(rust_code.as_bytes());
    let chunks = chunker.chunk_file("test.rs", &mut reader, 0).unwrap();
    assert!(
        chunks.len() >= 2,
        "Expected at least 2 chunks (main + helper), got {}",
        chunks.len()
    );
    assert!(
        chunks.iter().any(|c| c.code.contains("main")),
        "Should contain main function"
    );
    assert!(
        chunks.iter().any(|c| c.code.contains("helper")),
        "Should contain helper function"
    );

    println!("✓ Rust chunking produced {} chunks", chunks.len());
}

#[test]
fn test_chunking_python_file() {
    let chunker = CodeChunker::default();
    let python_code = r#"
def greet(name):
    return f"Hello, {name}"

class Calculator:
    def add(self, a, b):
        return a + b
"#;

    let mut reader = std::io::Cursor::new(python_code.as_bytes());
    let chunks = chunker.chunk_file("test.py", &mut reader, 0).unwrap();
    assert!(!chunks.is_empty(), "Python file should produce chunks");
    assert!(
        chunks
            .iter()
            .any(|c| c.code.contains("greet") || c.code.contains("Calculator")),
        "Should contain function or class"
    );

    println!("✓ Python chunking produced {} chunks", chunks.len());
}

#[tokio::test]
async fn test_lancedb_filename_index() {
    let (storage, mut embedder, chunker, db_path) = setup_test_env("index_verification").await;

    // Index Rust test file
    let rust_path = Path::new(TEST_ASSETS_PATH).join("test.rs");
    let code = fs::read_to_string(&rust_path).expect("Failed to read Rust file");
    let mtime = 0;

    let mut reader = std::io::Cursor::new(code.as_bytes());
    let chunks = chunker
        .chunk_file(rust_path.to_str().unwrap(), &mut reader, mtime)
        .unwrap();
    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = embedder.embed(texts, None).expect("Failed to embed");
    let (ids, filenames, codes, line_starts, line_ends, last_modified, calls) =
        prepare_chunks(&chunks);
    storage
        .add_chunks(
            "default",
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

    // Create index (this should be automatic but testing explicitly)
    storage
        .create_filename_index()
        .await
        .expect("Failed to create filename index");

    // Verify index improves filtered search performance
    let mut searcher = CodeSearcher::new(
        Some(std::sync::Arc::new(storage)),
        Some(std::sync::Arc::new(embedder)),
        None,
        None,
        1.0,
        1.0,
        60.0,
    );

    // Search with extension filter (should use index)
    let results = searcher
        .semantic_search(
            "rust function",
            5,
            Some("rs".to_string()),
            None,
            false,
            None,
            None,
            false,
        )
        .await
        .expect("Filtered search failed");

    assert!(!results.is_empty(), "Filtered search returned no results");
    assert!(
        results[0].filename.ends_with(".rs"),
        "Filter did not correctly restrict to .rs files"
    );
    println!("✓ LanceDB filename index test passed");

    cleanup_test_db(&db_path);
}
