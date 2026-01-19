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
    let pb_scan = ProgressBar::new_spinner();
    pb_scan.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .map_err(|e| CodeRagError::Tantivy(e.to_string()))?,
    );
    pb_scan.enable_steady_tick(std::time::Duration::from_millis(120));
    pb_scan.set_message("Scanning files...");

    let builder = WalkBuilder::new(index_path);
    let walker = builder.build();

    let mut entries = Vec::new();
    for entry in walker.flatten() {
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            let path = entry.path();
            let path_str = path.to_string_lossy();
            let excluded = config.exclusions.iter().any(|ex| path_str.contains(ex));
            if !excluded {
                entries.push(entry);
                pb_scan.set_message(format!("Found {} files...", entries.len()));
            }
        }
    }
    pb_scan.finish_with_message(format!("Scanned {} files.", entries.len()));

    if entries.is_empty() {
        warn!("No files found to index.");
        return Ok(());
    }

    // 5. Indexing Loop
    let total_files = entries.len() as u64;
    let pb_index = ProgressBar::new(total_files);
    pb_index.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .map_err(|e| CodeRagError::Tantivy(e.to_string()))?
            .progress_chars("#>-"),
    );
    pb_index.enable_steady_tick(std::time::Duration::from_millis(120));
    pb_index.set_message("Indexing...");

    let existing_files = if update {
        pb_index.set_message("Fetching existing metadata...");
        storage
            .get_indexed_metadata(&workspace)
            .await
            .map_err(|e| CodeRagError::Database(e.to_string()))?
    } else {
        HashMap::new()
    };

    let mut chunks_buffer = Vec::new();
    let batch_size_val = batch_size.unwrap_or(256);
    tracing::info!("Using batch size: {}", batch_size_val);

    for entry in entries {
        let file_path = entry.path();
        let fname_lossy = file_path.to_string_lossy();
        let fname_short = file_path.file_name().unwrap_or_default().to_string_lossy();

        pb_index.set_message(format!("Processing {}", fname_short));
        pb_index.inc(1);

        let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if CodeChunker::get_language(ext).is_none() {
            continue;
        }

        if let Ok(metadata) = fs::metadata(file_path) {
            let modified = metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let mtime = modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let fname_str = fname_lossy.to_string();

            if update {
                if let Some(stored_mtime) = existing_files.get(&fname_str) {
                    if *stored_mtime == mtime {
                        continue; // Unchanged
                    }
                    if let Err(e) = storage.delete_file_chunks(&fname_str, &workspace).await {
                        warn!("Error deleting old chunks for {}: {}", fname_str, e);
                    }
                    if let Err(e) = bm25_index.delete_file(&fname_str, &workspace) {
                        warn!("Error deleting old BM25 docs for {}: {}", fname_str, e);
                    }
                }
            }

            if let Ok(file) = fs::File::open(file_path) {
                let mut reader = std::io::BufReader::new(file);
                match chunker.chunk_file(&fname_str, &mut reader, mtime) {
                    Ok(new_chunks) => chunks_buffer.extend(new_chunks),
                    Err(e) => warn!("Error chunking file {}: {}", fname_str, e),
                }
            }
        }

        if chunks_buffer.len() >= batch_size_val {
            process_batch(
                &mut chunks_buffer,
                &mut embedder,
                &storage,
                &bm25_index,
                &pb_index,
                &workspace,
                batch_size_val,
            )
            .await?;
        }
    }

    if !chunks_buffer.is_empty() {
        process_batch(
            &mut chunks_buffer,
            &mut embedder,
            &storage,
            &bm25_index,
            &pb_index,
            &workspace,
            batch_size_val,
        )
        .await?;
    }

    pb_index.finish_with_message("Indexing complete.");

    info!("Optimizing index (creating filename index)...");
    if let Err(e) = storage.create_filename_index().await {
        warn!("Optimization warning: {}", e);
    }

    Ok(())
}

async fn process_batch(
    chunks: &mut Vec<crate::indexer::CodeChunk>,
    embedder: &mut Embedder,
    storage: &Storage,
    bm25_index: &BM25Index,
    pb: &ProgressBar,
    workspace: &str,
    batch_size: usize,
) -> Result<(), CodeRagError> {
    pb.set_message("Embedding batch...");
    let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();

    match embedder.embed(texts, Some(batch_size)) {
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

            if let Err(e) = storage
                .add_chunks(
                    workspace, ids, filenames, codes, starts, ends, mtimes, calls, embeddings,
                )
                .await
            {
                error!("Error storing chunks: {}", e);
            }
            if let Err(e) = bm25_index.add_chunks(chunks, workspace) {
                error!("Error adding to BM25: {}", e);
            }
        }
        Err(e) => error!("Error generating embeddings: {}", e),
    }
    chunks.clear();
    Ok(())
}
