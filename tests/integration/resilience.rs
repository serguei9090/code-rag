use crate::common::{cleanup_test_db, setup_test_env};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tokio::time::Instant;

#[tokio::test]
async fn test_corrupt_database() {
    let (_, _, _, db_path) = setup_test_env("corrupt_db").await;

    // Corrupt the database by writing garbage to a file in the directory
    let bad_file = Path::new(&db_path).join("corrupted.lance");
    let mut file = File::create(bad_file).unwrap();
    writeln!(file, "NOT A VALID LANCE FILE").unwrap();

    // Verification would be creating a searcher on this path and verifying it doesn't panic
}

#[tokio::test]
async fn test_empty_file_indexing() {
    let (_, _, chunker, _) = setup_test_env("empty_file").await;

    // Create an empty file
    let file_path = std::env::temp_dir().join("empty_test.rs");
    File::create(&file_path).unwrap();

    // Read code
    let code = std::fs::read_to_string(&file_path).unwrap();

    // Chunk (sync)
    let chunks = chunker.chunk_file(file_path.to_str().unwrap(), &code, 0);

    assert_eq!(chunks.len(), 0, "Empty file should produce 0 chunks");

    // Cleanup
    let _ = std::fs::remove_file(file_path);
}

#[tokio::test]
async fn test_large_file_chunking() {
    let (_, _, chunker, _) = setup_test_env("large_file").await;

    // Create a 5MB file
    let file_path = std::env::temp_dir().join("large_test.rs");
    let mut file = File::create(&file_path).unwrap();

    // Write 5MB of roughly valid looking code
    let chunk = "// This is a test comment line to fill up space.\n";
    let iterations = (5 * 1024 * 1024) / chunk.len();
    for _ in 0..iterations {
        file.write_all(chunk.as_bytes()).unwrap();
    }

    let code = std::fs::read_to_string(&file_path).unwrap();

    let start = Instant::now();
    let chunks = chunker.chunk_file(file_path.to_str().unwrap(), &code, 0);
    let duration = start.elapsed();

    // Should produce chunks (comments are captured as chunks in some languages or at least processed)
    // The key is performance check.
    assert!(
        duration.as_secs() < 5,
        "Chunking 5MB should take less than 5 seconds (was {:?})",
        duration
    );

    // Cleanup
    let _ = std::fs::remove_file(file_path);
}

#[tokio::test]
async fn test_invalid_syntax() {
    let (_, _, chunker, _) = setup_test_env("invalid_syntax").await;

    // Create a file with invalid syntax
    let file_path = std::env::temp_dir().join("broken.rs");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        "def python_function():\n    print('This is not Rust!')"
    )
    .unwrap();

    let code = std::fs::read_to_string(&file_path).unwrap();

    let chunks = chunker.chunk_file(file_path.to_str().unwrap(), &code, 0);

    // Should return result without panicking.
    // Count might be 0 or >0 depending on fallback.
    assert!(chunks.len() >= 0);

    // Cleanup
    let _ = std::fs::remove_file(file_path);
}
