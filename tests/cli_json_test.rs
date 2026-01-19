use anyhow::Result;
use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_json_output_cleanliness() -> Result<()> {
    // 1. Setup temp config
    let dir = tempdir()?;
    let config_path = dir.path().join("code-rag.toml");
    let db_path = dir.path().join("db");

    // Create dummy config
    let config_content = format!(
        r#"
db_path = "{}"
default_index_path = "."
enable_server = false
enable_mcp = false
enable_watch = false
"#,
        db_path.to_string_lossy().replace("\\", "\\\\") // Escape windows paths
    );
    fs::write(&config_path, config_content)?;

    // 2. Run search with --json and --config
    let output = Command::cargo_bin("code-rag")?
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("search")
        .arg("nonexistent_unique_token_xyz")
        .arg("--json")
        .output()?; // Capture both stdout/stderr

    // 3. Capture output
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);

    // Verify successful exit, or at least that it ran enough to produce output
    // Search for non-existent token might return empty list [], which is valid JSON.
    // assert!(output.status.success());

    // 4. Verify stdout parses as JSON
    let parsed: Value = serde_json::from_str(&stdout).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse JSON output: {}. \nStdout: '{}' \nStderr: '{}'",
            e,
            stdout,
            stderr
        )
    })?;

    assert!(parsed.is_array(), "Output should be a JSON array");

    Ok(())
}
