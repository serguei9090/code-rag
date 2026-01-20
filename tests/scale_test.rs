use anyhow::Result;
use assert_cmd::prelude::*;
use assert_cmd::Command;
use std::fs;
use std::path::Path;
use std::time::Instant;
use tempfile::tempdir;
use tokio::fs as tokio_fs;

// Generate 10k files
async fn generate_large_dataset(root: &Path, count: usize) -> Result<()> {
    if root.exists() {
        fs::remove_dir_all(root)?;
    }
    fs::create_dir_all(root)?;

    let mut set = tokio::task::JoinSet::new();

    // Batch creation to avoid file handle limits if parallelized too aggressively
    let root_buf = root.to_path_buf();

    // We'll spawn chunks
    for i in 0..count {
        let r = root_buf.clone();
        set.spawn(async move {
            let file_path = r.join(format!("file_{}.rs", i));
            let content = format!(
                "fn function_{}() {{ println!(\"Hello from file {}\"); }}",
                i, i
            );
            let _ = tokio_fs::write(file_path, content).await;
        });
    }

    while (set.join_next().await).is_some() {}
    Ok(())
}

#[tokio::test]
#[ignore] // Run strictly on demand via `cargo test --test scale_test -- --ignored`
async fn test_indexing_scale_10k() -> Result<()> {
    let dir = tempdir()?;
    // Keep root outside tempdir if we want to inspect it? No, keep it clean.
    // Actually we need a large dataset. Tempdir is fine.

    let dataset_path = dir.path().join("src");
    let db_path = dir.path().join("db");
    let config_path = dir.path().join("code-rag.toml");

    // Create dummy config
    let config_content = format!(
        r#"
db_path = "{}"
default_index_path = "."
enable_server = false
enable_mcp = false
enable_watch = false
"#,
        db_path.to_string_lossy().replace("\\", "\\\\")
    );
    fs::write(&config_path, config_content)?;

    println!("Generating 10,000 files in {:?}...", dataset_path);
    generate_large_dataset(&dataset_path, 10_000).await?;

    let start = Instant::now();
    let status = Command::new(env!("CARGO_BIN_EXE_code-rag"))
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("index")
        .arg("--path")
        .arg(&dataset_path)
        .arg("--force")
        .ok()?;

    let duration = start.elapsed();
    status.assert().success();

    println!("Indexing 10,000 files took: {:?}", duration);

    // Environment-aware threshold:
    // - CI environments are resource-constrained and take longer (~465s observed in GitHub Actions)
    // - Local development machines should be faster
    let timeout_secs = if std::env::var("CI").is_ok() {
        600 // 10 minutes for CI
    } else {
        90 // 90 seconds for local development
    };

    assert!(
        duration.as_secs() < timeout_secs,
        "Indexing took longer than {}s (CI={}, actual={}s)",
        timeout_secs,
        std::env::var("CI").is_ok(),
        duration.as_secs()
    );

    Ok(())
}
