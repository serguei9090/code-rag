use std::collections::HashMap;
use std::fs;
use std::path::Path;

use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{error, info, warn};

use crate::bm25::BM25Index;
use crate::config::AppConfig;
use crate::core::CodeRagError;
use crate::embedding::Embedder;
use crate::indexer::CodeChunker;
use crate::storage::Storage;

pub struct IndexOptions {
    pub path: Option<String>,
    pub db_path: Option<String>,
    pub update: bool,
    pub force: bool,
    pub workspace: String,
    pub batch_size: Option<usize>,
    pub threads: Option<usize>,
}

pub async fn index_codebase(options: IndexOptions, config: &AppConfig) -> Result<(), CodeRagError> {
    let actual_path = options
        .path
        .unwrap_or_else(|| config.default_index_path.clone());
    let actual_db = options.db_path.unwrap_or_else(|| config.db_path.clone());

    let force = options.force;
    let update = options.update;
    let workspace = options.workspace;
    let batch_size = options.batch_size;

    if force {
        info!("Force flag set. Removing database at: {}", actual_db);
        if Path::new(&actual_db).exists() {
            fs::remove_dir_all(&actual_db).map_err(CodeRagError::Io)?;
        }
    }

    info!("Indexing path: {}", actual_path);
    let index_path = Path::new(&actual_path);

    // 1. Load Models with Spinner
    let pb_model = ProgressBar::new_spinner();
    pb_model.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .map_err(|e| CodeRagError::Tantivy(e.to_string()))?,
    );
    pb_model.enable_steady_tick(std::time::Duration::from_millis(120));
    pb_model.set_message("Loading embedding model...");

    let mut embedder = Embedder::new(
        config.embedding_model.clone(),
        config.reranker_model.clone(),
        config.embedding_model_path.clone(),
        config.reranker_model_path.clone(),
        config.device.clone(),
    )?;

    pb_model.set_message("Warming up ONNX Runtime...");
    let warmup_text = vec!["warmup".to_string()];
    let _ = embedder.embed(warmup_text.clone(), None)?;

    pb_model.finish_with_message("Models loaded.");

    // 2. Initialize Storage
    let storage = Storage::new(&actual_db)
        .await
        .map_err(|e| CodeRagError::Database(e.to_string()))?;
    storage
        .init(embedder.dim())
        .await
        .map_err(|e| CodeRagError::Database(e.to_string()))?;

    // 3. Initialize BM25 Index
    let bm25_index = match BM25Index::new(&actual_db, false, &config.merge_policy) {
        Ok(idx) => idx,
        Err(e) => {
            warn!(
                "Failed to initialize BM25 index: {}. Hybrid search may be degraded.",
                e
            );
            return Err(CodeRagError::Tantivy(e.to_string()));
        }
    };

    let chunker = CodeChunker::new(config.chunk_size, config.chunk_overlap);

    // 4. Scan Files
    // 4. Setup Progress Bar & Walker
    let pb_index = ProgressBar::new_spinner();
    pb_index.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {pos} files processed ({msg})")
            .map_err(|e| CodeRagError::Tantivy(e.to_string()))?,
    );
    pb_index.enable_steady_tick(std::time::Duration::from_millis(120));
    pb_index.set_message("Initializing...");

    let existing_files = if update {
        pb_index.set_message("Fetching existing metadata...");
        storage
            .get_indexed_metadata(&workspace)
            .await
            .map_err(|e| CodeRagError::Database(e.to_string()))?
    } else {
        HashMap::new()
    };

    let builder = WalkBuilder::new(index_path);
    let walker = builder.build();

    // 5. Indexing Loop (Streaming)
    let mut chunks_buffer = Vec::new();
    let mut pending_deletes = Vec::new();
    let mut visited_files = std::collections::HashSet::new();
    let batch_size_val = batch_size.unwrap_or(256);
    tracing::info!("Using batch size: {}", batch_size_val);

    for result in walker {
        match result {
            Ok(entry) => {
                if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                    continue;
                }

                let path = entry.path();
                let path_str = path.to_string_lossy();
                if config.exclusions.iter().any(|ex| path_str.contains(ex)) {
                    continue;
                }

                let fname_short = path.file_name().unwrap_or_default().to_string_lossy();
                pb_index.set_message(format!("Processing {}", fname_short));
                pb_index.inc(1);

                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if CodeChunker::get_language(ext).is_none() {
                    continue;
                }

                if let Ok(metadata) = fs::metadata(path) {
                    // OOM Protection: Skip large files
                    if metadata.len() > config.max_file_size_bytes as u64 {
                        warn!(
                            "Skipping file {} (size: {} bytes) - exceeds limit of {} bytes",
                            path_str,
                            metadata.len(),
                            config.max_file_size_bytes
                        );
                        continue;
                    }

                    let modified = metadata
                        .modified()
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    let mtime = modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64;
                    let fname_str = path_str.to_string();

                    // Track visited files for stale cleanup
                    visited_files.insert(fname_str.clone());

                    if update {
                        if let Some(stored_mtime) = existing_files.get(&fname_str) {
                            if *stored_mtime == mtime {
                                continue; // Unchanged
                            }
                            // File changed, mark old version for deletion
                            pending_deletes.push(fname_str.clone());
                        }
                    }

                    if let Ok(file) = fs::File::open(path) {
                        let mut reader = std::io::BufReader::new(file);
                        match chunker.chunk_file(&fname_str, &mut reader, mtime) {
                            Ok(new_chunks) => chunks_buffer.extend(new_chunks),
                            Err(e) => warn!("Error chunking file {}: {}", fname_str, e),
                        }
                    }
                }

                if chunks_buffer.len() >= batch_size_val || pending_deletes.len() >= batch_size_val
                {
                    let mut ctx = IndexingContext {
                        embedder: &mut embedder,
                        storage: &storage,
                        bm25_index: &bm25_index,
                        pb: &pb_index,
                        workspace: &workspace,
                    };
                    process_batch(&mut chunks_buffer, &mut pending_deletes, &mut ctx).await?;
                }
            }
            Err(err) => warn!("Error walking directory: {}", err),
        }
    }

    if !chunks_buffer.is_empty() || !pending_deletes.is_empty() {
        let mut ctx = IndexingContext {
            embedder: &mut embedder,
            storage: &storage,
            bm25_index: &bm25_index,
            pb: &pb_index,
            workspace: &workspace,
        };
        process_batch(&mut chunks_buffer, &mut pending_deletes, &mut ctx).await?;
    }

    // 6. Stale File Cleanup (Post-Indexing)
    if update {
        let stale_files: Vec<String> = existing_files
            .keys()
            .filter(|f| !visited_files.contains(*f))
            .cloned()
            .collect();

        if !stale_files.is_empty() {
            info!("Found {} stale files to remove.", stale_files.len());
            pb_index.set_message("Cleaning up stale files...");

            // Process in batches
            for chunk in stale_files.chunks(batch_size_val) {
                let batch: Vec<String> = chunk.to_vec();
                if let Err(e) = storage.batch_delete_files(&batch, &workspace).await {
                    error!("Error removing stale files from storage: {}", e);
                }
                if let Err(e) = bm25_index.batch_delete_files(&batch, &workspace) {
                    error!("Error removing stale files from BM25: {}", e);
                }
            }
        }
    }

    // Commit BM25 index once at the end (single expensive I/O operation)
    pb_index.set_message("Committing BM25 index...");
    if let Err(e) = bm25_index.commit() {
        warn!("Failed to commit BM25 index: {}", e);
    }

    pb_index.finish_with_message("Indexing complete.");

    info!("Optimizing index (creating filename index)...");
    if let Err(e) = storage.create_filename_index().await {
        warn!("Optimization warning: {}", e);
    }

    Ok(())
}

struct IndexingContext<'a> {
    embedder: &'a mut Embedder,
    storage: &'a Storage,
    bm25_index: &'a BM25Index,
    pb: &'a ProgressBar,
    workspace: &'a str,
}

async fn process_batch(
    chunks: &mut Vec<crate::indexer::CodeChunk>,
    pending_deletes: &mut Vec<String>,
    ctx: &mut IndexingContext<'_>,
) -> Result<(), CodeRagError> {
    // 1. Process Deletions
    if !pending_deletes.is_empty() {
        if let Err(e) = ctx
            .storage
            .batch_delete_files(pending_deletes, ctx.workspace)
            .await
        {
            error!("Error batch deleting chunks: {}", e);
        }
        if let Err(e) = ctx
            .bm25_index
            .batch_delete_files(pending_deletes, ctx.workspace)
        {
            error!("Error batch deleting BM25 docs: {}", e);
        }
        pending_deletes.clear();
    }

    if chunks.is_empty() {
        return Ok(());
    }

    ctx.pb.set_message("Embedding batch...");
    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();

    match ctx.embedder.embed(texts, None) {
        Ok(embeddings) => {
            let ids: Vec<String> = chunks
                .iter()
                .map(|c| format!("{}-{}-{}", c.filename, c.line_start, c.line_end))
                .collect();
            let filenames: Vec<String> = chunks.iter().map(|c| c.filename.clone()).collect();
            let codes: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
            let starts: Vec<i32> = chunks.iter().map(|c| c.line_start as i32).collect();
            let ends: Vec<i32> = chunks.iter().map(|c| c.line_end as i32).collect();
            let mtimes: Vec<i64> = chunks.iter().map(|c| c.last_modified).collect();
            let calls: Vec<Vec<String>> = chunks.iter().map(|c| c.calls.clone()).collect();

            if let Err(e) = ctx
                .storage
                .add_chunks(
                    ctx.workspace,
                    ids,
                    filenames,
                    codes,
                    starts,
                    ends,
                    mtimes,
                    calls,
                    embeddings,
                )
                .await
            {
                error!("Error storing chunks: {}", e);
            }
            if let Err(e) = ctx.bm25_index.add_chunks(chunks, ctx.workspace) {
                error!("Error adding to BM25: {}", e);
            }
        }
        Err(e) => error!("Error generating embeddings: {}", e),
    }
    chunks.clear();
    Ok(())
}
