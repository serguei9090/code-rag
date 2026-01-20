use anyhow::Result;
use code_rag::bm25::BM25Index;
use code_rag::indexer::CodeChunk;
// use std::fs;
use tempfile::TempDir;

#[test]
fn test_bm25_batch_delete() -> Result<()> {
    // Setup
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().to_str().unwrap();
    let index = BM25Index::new(db_path, false, "log")?;

    // Create dummy chunks
    let chunks = vec![
        CodeChunk {
            filename: "file1.rs".to_string(),
            code: "fn test1() {}".to_string(),
            line_start: 1,
            line_end: 10,
            last_modified: 100,
            calls: vec![],
        },
        CodeChunk {
            filename: "file2.rs".to_string(),
            code: "fn test2() {}".to_string(),
            line_start: 1,
            line_end: 10,
            last_modified: 100,
            calls: vec![],
        },
        CodeChunk {
            filename: "file3.rs".to_string(),
            code: "fn test3() {}".to_string(),
            line_start: 1,
            line_end: 10,
            last_modified: 100,
            calls: vec![],
        },
    ];

    // Index them
    index.add_chunks(&chunks, "default")?;
    index.commit()?;
    index.reload()?;

    println!("Num docs: {}", index.get_searcher().num_docs());

    // Verify they exist
    let results = index.search("test1", 10, Some("default"))?;
    println!("Results: {:?}", results);
    assert!(!results.is_empty(), "Should find at least test1");

    // Batch delete file1 and file3
    let to_delete = vec!["file1.rs".to_string(), "file3.rs".to_string()];
    index.batch_delete_files(&to_delete, "default")?;
    index.commit()?;
    index.reload()?;

    // Verify result
    let results_after = index.search("test2", 10, Some("default"))?;
    assert_eq!(results_after.len(), 1);
    assert_eq!(results_after[0].filename, "file2.rs");

    Ok(())
}
