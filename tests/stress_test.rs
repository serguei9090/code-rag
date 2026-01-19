use anyhow::{Context, Result};
use assert_cmd::cargo::cargo_bin;
use reqwest::Client;
use std::fs;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tempfile::tempdir;
use tokio::time::sleep;

struct ServerGuard {
    process: Child,
    #[allow(dead_code)]
    config_path: std::path::PathBuf, // Keep it alive? tempdir cleans up
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

async fn wait_for_server(port: u16) -> Result<()> {
    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/health", port);

    for _ in 0..30 {
        // Wait up to 15s
        if client.get(&url).send().await.is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(500)).await;
    }
    Err(anyhow::anyhow!("Server failed to start on port {}", port))
}

#[tokio::test]
#[ignore] // Run on demand
async fn test_concurrent_load() -> Result<()> {
    // 1. Config Setup
    let dir = tempdir()?;
    let config_path = dir.path().join("code-rag.toml");
    let db_path = dir.path().join("db");

    let config_content = format!(
        r#"
db_path = "{}"
default_index_path = "."
enable_server = true
enable_mcp = false
enable_watch = false
"#,
        db_path.to_string_lossy().replace("\\", "\\\\")
    );
    fs::write(&config_path, config_content)?;

    // 2. Start Server
    let bin = cargo_bin("code-rag");
    let port = 8092;

    let process = Command::new(bin)
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("serve")
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn server")?;

    let _guard = ServerGuard {
        process,
        config_path: config_path.clone(),
    };

    // 3. Wait for Health
    println!("Waiting for server on port {}...", port);
    wait_for_server(port).await?;

    // 4. Generate Load
    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/v1/default/search?query=test", port);

    let concurrency = 50;
    let mut set = tokio::task::JoinSet::new();

    println!("Sending {} concurrent requests...", concurrency);
    let start = std::time::Instant::now();

    for i in 0..concurrency {
        let c = client.clone();
        let u = url.clone();
        set.spawn(async move {
            let resp = c.get(&u).send().await;
            (i, resp)
        });
    }

    let mut success_count = 0;
    while let Some(res) = set.join_next().await {
        let (_id, resp_result) = res?;
        if let Ok(resp) = resp_result {
            if resp.status().is_success() {
                success_count += 1;
            } else {
                eprintln!("Request failed with status: {}", resp.status());
            }
        } else {
            eprintln!("Request failed: {:?}", resp_result.err());
        }
    }

    let duration = start.elapsed();
    println!(
        "Processed {}/{} requests successfully in {:?}",
        success_count, concurrency, duration
    );

    assert_eq!(
        success_count, concurrency,
        "Not all concurrent requests succeeded"
    );

    Ok(())
}
