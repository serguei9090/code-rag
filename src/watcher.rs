use crate::bm25::BM25Index;
use crate::embedding::Embedder;
use crate::indexer::CodeChunker;
use crate::ops::indexer::CodeIndexer;
use crate::storage::Storage;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::path::Path;
use std::time::Duration;
use tracing::{error, info};

pub async fn start_watcher(
    path: &str,
    storage: Storage,
    mut embedder: Embedder,
    mut bm25: BM25Index,
    chunker: CodeChunker,
    workspace: String,
) -> anyhow::Result<()> {
    info!("Starting watcher on: {}", path);

    let (tx, rx) = std::sync::mpsc::channel();

    // Create a debouncer with 2 seconds timeout
    let mut debouncer = new_debouncer(Duration::from_secs(2), tx)?;

    debouncer
        .watcher()
        .watch(Path::new(path), RecursiveMode::Recursive)?;

    // We need to keep the components alive and mutable.
    // Since notify runs in a separate thread (or system event loop) but communicates via channel,
    // we can process events in the main async loop.

    // However, rx is blocking. We should use a blocking task or a non-blocking channel if we want strict async.
    // For simplicity in this CLI tool, we can loop over the channel in a blocking_spawn or similar,
    // but better is to iterate and await inside the loop.

    // Since we need to call async methods on storage/indexer, we can't easily be in a blocking loop unless we block_on.
    // Let's use a standard loop checking the channel.

    let mut indexer = CodeIndexer::new(&storage, &mut embedder, &mut bm25, &chunker, workspace);

    for result in rx {
        match result {
            Ok(events) => {
                for event in events {
                    let path = event.path;
                    let path_lossy = path.to_string_lossy();

                    // Simple exclusion for .git and target/lancedb
                    if path_lossy.contains(".git")
                        || path_lossy.contains("node_modules")
                        || path_lossy.contains("target")
                        || path_lossy.contains(".lancedb")
                    {
                        continue;
                    }

                    // Check if file still exists (Modification vs Deletion)
                    if path.exists() {
                        // It's a Create or Write
                        match std::fs::metadata(&path) {
                            Ok(metadata) => {
                                let mtime = metadata
                                    .modified()
                                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i64;

                                if let Err(e) = indexer.index_file(&path, mtime).await {
                                    error!("Failed to re-index {}: {}", path.display(), e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to read metadata for {}: {}", path.display(), e)
                            }
                        }
                    } else {
                        // It's a Remove (or Move away)
                        if let Err(e) = indexer.remove_file(&path).await {
                            error!("Failed to remove index for {}: {}", path.display(), e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Watch error: {:?}", e);
            }
        }
    }

    Ok(())
}
