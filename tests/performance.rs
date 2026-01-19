use anyhow::Result;
use assert_cmd::Command;
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

fn setup_large_project(file_count: usize) -> Result<(TempDir, std::path::PathBuf)> {
    let temp_dir = TempDir::new()?;
    let project_dir = temp_dir.path().join("large_project");
    fs::create_dir_all(&project_dir)?;

    for i in 0..file_count {
        let file_path = project_dir.join(format!("file_{}.rs", i));
        fs::write(
            &file_path,
            format!("fn function_{}() {{ println!(\"Function {}\"); }}", i, i),
        )?;
    }
    Ok((temp_dir, project_dir))
}

#[test]
#[ignore] // Start with ignore so it doesn't slow down standard `cargo test` unless requested
fn test_indexing_performance_100files() -> Result<()> {
    let (temp_dir, project_dir) = setup_large_project(100)?;
    let db_path = temp_dir.path().join("lancedb");

    let start = Instant::now();
    Command::new(env!("CARGO_BIN_EXE_code-rag"))
        .env("CODE_RAG__DB_PATH", &db_path)
        .arg("index")
        .arg("--path")
        .arg(&project_dir)
        .assert()
        .success();
    let duration = start.elapsed();

    println!("Indexing 100 files took: {:?}", duration);
    // Simple assertion to ensure it's not absurdly slow (e.g. > 30s for 100 simple files)
    assert!(duration.as_secs() < 30);
    Ok(())
}

#[test]
#[ignore]
fn test_search_performance() -> Result<()> {
    let (temp_dir, project_dir) = setup_large_project(50)?;
    let db_path = temp_dir.path().join("lancedb");

    // Pre-index
    Command::new(env!("CARGO_BIN_EXE_code-rag"))
        .env("CODE_RAG__DB_PATH", &db_path)
        .arg("index")
        .arg("--path")
        .arg(&project_dir)
        .assert()
        .success();

    let start = Instant::now();
    Command::new(env!("CARGO_BIN_EXE_code-rag"))
        .env("CODE_RAG__DB_PATH", &db_path)
        .arg("search")
        .arg("function_25")
        .assert()
        .success();
    let duration = start.elapsed();

    println!("Search took: {:?}", duration);
    assert!(duration.as_secs() < 15);
    Ok(())
}
