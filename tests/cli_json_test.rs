use anyhow::Result;
use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_json_output_cleanliness() -> Result<()> {
    // 1. Setup temp directories
    let dir = tempdir()?;
    let config_path = dir.path().join("code-rag.toml");
    let db_path = dir.path().join("db");
    let test_index_dir = dir.path().join("test_src");

    // Create test source directory with a dummy file
    fs::create_dir_all(&test_index_dir)?;
    fs::write(test_index_dir.join("dummy.txt"), "test content")?;

    // Create config with telemetry disabled
    let config_content = format!(
        r#"
db_path = "{}"
default_index_path = "."
enable_server = false
enable_mcp = false
enable_watch = false
telemetry_enabled = false
"#,
        db_path.to_string_lossy().replace("\\", "\\\\")
    );
    fs::write(&config_path, config_content)?;

    // 2. Initialize empty database by running index command first
    // This prevents "Table not found" errors
    let index_output = Command::new(env!("CARGO_BIN_EXE_code-rag"))
        .env("RUST_LOG", "off")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("index")
        .arg("--path")
        .arg(test_index_dir.to_str().unwrap())
        .output()?;

    // Verify index succeeded
    assert!(
        index_output.status.success(),
        "Index command failed: {}",
        String::from_utf8_lossy(&index_output.stderr)
    );

    // 3. Run search with --json
    let output = Command::new(env!("CARGO_BIN_EXE_code-rag"))
        .env("RUST_LOG", "off")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("search")
        .arg("nonexistent_unique_token_xyz")
        .arg("--json")
        .output()?;

    // 4. Capture output
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // 5. THE MAIN TEST: Verify stdout contains ONLY clean JSON
    // (STDERR can have dependency logs, we don't care about those)
    let parsed: Value = serde_json::from_str(&stdout).map_err(|e| {
        eprintln!("=== TEST FAILED ===");
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
        eprintln!("==================");
        anyhow::anyhow!(
            "Failed to parse JSON output: {}. \nStdout: '{}' \nStderr: '{}'",
            e,
            stdout,
            stderr
        )
    })?;

    // Verify it's an array (empty array for non-existent query is expected)
    assert!(parsed.is_array(), "Output should be a JSON array");

    // Verify stdout starts with '[' (pure JSON, no log pollution)
    let trimmed_stdout = stdout.trim();
    assert!(
        trimmed_stdout.starts_with('['),
        "stdout should start with JSON array bracket, but starts with: '{}'",
        trimmed_stdout.chars().take(50).collect::<String>()
    );

    Ok(())
}
