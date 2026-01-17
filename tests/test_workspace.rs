use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
#[allow(deprecated)]
fn test_workspace_isolation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("lancedb");
    let project_dir = temp_dir.path().join("project");
    let file_a = project_dir.join("file_a.rs");
    let file_b = project_dir.join("file_b.rs");

    fs::create_dir_all(&project_dir)?;
    fs::write(&file_a, "fn function_a() { println!(\"A\"); }")?;
    fs::write(&file_b, "fn function_b() { println!(\"B\"); }")?;

    // Index file_a into workspace "A"
    let mut cmd = Command::cargo_bin("code-rag")?;
    cmd.env("CODE_RAG__DB_PATH", &db_path)
        .arg("index")
        .arg("--path")
        .arg(&project_dir)
        .arg("--workspace")
        .arg("workspace_A")
        .assert()
        .success();

    // Index file_b into workspace "B"
    let mut cmd = Command::cargo_bin("code-rag")?;
    cmd.env("CODE_RAG__DB_PATH", &db_path)
        .arg("index")
        .arg("--path")
        .arg(&project_dir)
        .arg("--workspace")
        .arg("workspace_B")
        .assert()
        .success();

    // Search for "function_a" in workspace A -> Should find it
    let mut cmd = Command::cargo_bin("code-rag")?;
    cmd.env("CODE_RAG__DB_PATH", &db_path)
        .arg("search")
        .arg("function_a")
        .arg("--workspace")
        .arg("workspace_A")
        .assert()
        .success()
        .stdout(predicate::str::contains("file_a.rs"));

    // Search for "function_a" in workspace B -> Should find it (files are identical in this part)
    let mut cmd = Command::cargo_bin("code-rag")?;
    cmd.env("CODE_RAG__DB_PATH", &db_path)
        .arg("search")
        .arg("function_a")
        .arg("--workspace")
        .arg("workspace_B")
        .assert()
        .success()
        .stdout(predicate::str::contains("file_a.rs"));

    // START of Unique content test
    let dir_a = temp_dir.path().join("dir_a");
    let dir_b = temp_dir.path().join("dir_b");
    fs::create_dir_all(&dir_a)?;
    fs::create_dir_all(&dir_b)?;
    fs::write(dir_a.join("unique_a.rs"), "fn unique_a() {}")?;
    fs::write(dir_b.join("unique_b.rs"), "fn unique_b() {}")?;

    // Index A to WS A
    Command::cargo_bin("code-rag")?
        .env("CODE_RAG__DB_PATH", &db_path)
        .arg("index")
        .arg("--path")
        .arg(&dir_a)
        .arg("--workspace")
        .arg("workspace_A")
        .assert()
        .success();

    // Index B to WS B
    Command::cargo_bin("code-rag")?
        .env("CODE_RAG__DB_PATH", &db_path)
        .arg("index")
        .arg("--path")
        .arg(&dir_b)
        .arg("--workspace")
        .arg("workspace_B")
        .assert()
        .success();

    // Search specific UNIQUE content
    // Search for "unique_a" in workspace A -> YES
    Command::cargo_bin("code-rag")?
        .env("CODE_RAG__DB_PATH", &db_path)
        .arg("search")
        .arg("unique_a")
        .arg("--workspace")
        .arg("workspace_A")
        .assert()
        .success()
        .stdout(predicate::str::contains("unique_a.rs"));

    // Search for "unique_a" in workspace B -> NO
    Command::cargo_bin("code-rag")?
        .env("CODE_RAG__DB_PATH", &db_path)
        .arg("search")
        .arg("unique_a")
        .arg("--workspace")
        .arg("workspace_B")
        .assert()
        .success()
        .stdout(predicate::str::contains("unique_a.rs").not());

    // Search unique_a in Default WS -> NO (since not indexed there)
    Command::cargo_bin("code-rag")?
        .env("CODE_RAG__DB_PATH", &db_path)
        .arg("search")
        .arg("unique_a")
        // No workspace arg -> default
        .assert()
        .success()
        .stdout(predicate::str::contains("unique_a.rs").not());

    // Test Default Workspace
    let dir_default = temp_dir.path().join("dir_default");
    fs::create_dir_all(&dir_default)?;
    fs::write(dir_default.join("default_file.rs"), "fn default_func() {}")?;

    // Index to default workspace (no --workspace arg)
    Command::cargo_bin("code-rag")?
        .env("CODE_RAG__DB_PATH", &db_path)
        .arg("index")
        .arg("--path")
        .arg(&dir_default)
        .assert()
        .success();

    // Search in default workspace (no --workspace arg) -> YES
    Command::cargo_bin("code-rag")?
        .env("CODE_RAG__DB_PATH", &db_path)
        .arg("search")
        .arg("default_func")
        .assert()
        .success()
        .stdout(predicate::str::contains("default_file.rs"));

    // Search in workspace A -> NO
    Command::cargo_bin("code-rag")?
        .env("CODE_RAG__DB_PATH", &db_path)
        .arg("search")
        .arg("default_func")
        .arg("--workspace")
        .arg("workspace_A")
        .assert()
        .success()
        .stdout(predicate::str::contains("default_file.rs").not());

    Ok(())
}
