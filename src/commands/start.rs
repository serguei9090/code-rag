use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::task::JoinSet;
use tracing::{error, info};

use crate::commands::{mcp, serve, watch};
use crate::config::AppConfig;

pub async fn run(config: &AppConfig) -> Result<()> {
    if !config.enable_server && !config.enable_mcp && !config.enable_watch {
        return Err(anyhow::anyhow!(
            "No services enabled. Please set enable_server, enable_mcp, or enable_watch to true in config."
        ));
    }

    let mut set: JoinSet<Result<()>> = JoinSet::new();

    // Pre-initialize BM25 indexes to avoid race conditions
    if config.enable_server || config.enable_watch {
        let mut index_targets = Vec::new();

        if config.workspaces.is_empty() {
            // Default workspace
            index_targets.push(PathBuf::from(&config.db_path));
        } else {
            for name in config.workspaces.keys() {
                if name == "default" {
                    index_targets.push(PathBuf::from(&config.db_path));
                } else {
                    index_targets.push(Path::new(&config.db_path).join(name));
                }
            }
        }

        for db_path in index_targets {
            let path_str = db_path.to_string_lossy();
            info!("Ensuring BM25 index exists at {}", path_str);
            if let Err(e) = crate::bm25::BM25Index::new(&path_str, false, &config.merge_policy) {
                error!("Failed to pre-initialize BM25 index at {}: {}", path_str, e);
            }
        }
    }

    // 1. Start Server
    if config.enable_server {
        let config_clone = config.clone();
        set.spawn(async move {
            info!(
                "Starting API Server on {}:{}",
                config_clone.server_host, config_clone.server_port
            );
            serve::serve_api(
                Some(config_clone.server_port),
                Some(config_clone.server_host.clone()),
                None,
                &config_clone,
            )
            .await
            .context("Server task failed")
        });
    }

    // 2. Start MCP
    if config.enable_mcp {
        let config_clone = config.clone();
        set.spawn(async move {
            info!("Starting MCP Server (Stdio)...");
            mcp::run(&config_clone).await.context("MCP task failed")
        });
    }

    // 3. Start Watcher
    if config.enable_watch {
        if config.workspaces.is_empty() {
            let config_clone = config.clone();
            set.spawn(async move {
                info!("Starting File Watcher (Default)...");
                let path = Some(config_clone.default_index_path.clone());
                watch::watch_codebase(path, None, "default".to_string(), &config_clone)
                    .await
                    .context("Watcher task failed")
            });
        } else {
            for (name, path_str) in &config.workspaces {
                let config_clone = config.clone();
                let name = name.clone();
                let path_to_watch = path_str.clone();

                // Replicate logic from specific WorkspaceManager to align DB paths
                let db_path_buf = if name == "default" {
                    PathBuf::from(&config.db_path)
                } else {
                    Path::new(&config.db_path).join(&name)
                };
                let db_path = db_path_buf.to_string_lossy().to_string();

                set.spawn(async move {
                    info!(
                        "Starting File Watcher for workspace '{}' at '{}'",
                        name, path_to_watch
                    );
                    watch::watch_codebase(Some(path_to_watch), Some(db_path), name, &config_clone)
                        .await
                        .context("Watcher task failed")
                });
            }
        }
    }

    // Wait for tasks
    // If any critical service fails, we might want to shut down everything?
    // For now, just log completions.
    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok(())) => info!("A service task completed successfully."),
            Ok(Err(e)) => error!("A service task failed with error: {:#}", e),
            Err(e) => error!("A service task panicked or was cancelled: {:#}", e),
        }
    }

    Ok(())
}
