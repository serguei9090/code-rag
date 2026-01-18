use code_rag::indexer::CodeChunker;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use tempfile::tempdir;

#[test]
fn test_streaming_large_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("large_test.rs");

    // Create a large file
    {
        let file = File::create(&file_path).unwrap();
        let mut writer = BufWriter::new(file);
        for i in 0..10_000 {
            writeln!(writer, "fn func_{}() {{ println!(\"val={}\"); }}", i, i).unwrap();
        }
    }

    let file = File::open(&file_path).unwrap();
    let mut reader = BufReader::new(file);

    let chunker = CodeChunker::default();
    // pass fake filename with extension
    let chunks = chunker.chunk_file("large_test.rs", &mut reader, 0).unwrap();

    assert!(!chunks.is_empty());
    // 10,000 functions -> 10,000 chunks roughly (each fits in 1024 bytes)
    // We expect tree-sitter to find them all.
    assert!(
        chunks.len() >= 9_000,
        "Expected chunks > 9000, got {}",
        chunks.len()
    );

    // Check first and last
    assert!(chunks.iter().any(|c| c.code.contains("fn func_0")));
    assert!(chunks.iter().any(|c| c.code.contains("fn func_9999")));
}
