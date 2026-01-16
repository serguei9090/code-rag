use clap::{Parser, Subcommand};
use code_rag::bm25::BM25Index;
use code_rag::config::AppConfig;
use code_rag::embedding::Embedder;
use code_rag::indexer::CodeChunker;
use code_rag::reporting::generate_html_report;
use code_rag::search::CodeSearcher;
use code_rag::server::start_server;
use code_rag::storage::Storage;
use code_rag::watcher::start_watcher;
use colored::*;
use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{error, info, warn, Level};

#[derive(Parser, Debug)]
#[command(
    name = "code-rag",
    version,
    about = "A local-first code indexing and semantic search tool",
    long_about = "code-rag allows you to index your local source code into a vector database and perform semantic searches using natural language queries, as well as exact pattern matching via grep."
)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Index a directory of source code
    Index {
        /// Path to the directory to index
        path: Option<String>,
        /// Custom database path (default: ./.lancedb from config)
        #[arg(long)]
        db_path: Option<String>,
        /// Perform an incremental update (skip unchanged files)
        #[arg(long)]
        update: bool,
        /// Force a full re-index (delete existing database)
        #[arg(long)]
        force: bool,
    },
    /// Perform a semantic search using natural language
    Search {
        /// Natural language query
        query: String,
        /// Maximum number of results to return
        #[arg(short, long)]
        limit: Option<usize>,
        /// Custom database path
        #[arg(long)]
        db_path: Option<String>,
        /// Generate an HTML report (results.html)
        #[arg(long)]
        html: bool,
        /// Output results in JSON format
        #[arg(long)]
        json: bool,
        /// Filter results by file extension (e.g., rs, py)
        #[arg(long)]
        ext: Option<String>,
        /// Filter results by directory path
        #[arg(long)]
        dir: Option<String>,
        /// Skip the re-ranking step for faster results
        #[arg(long)]
        no_rerank: bool,
    },
    /// Perform a regex-based text search across the codebase
    Grep {
        /// Regex pattern to search for
        pattern: String,
        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Start a persistent HTTP server
    Serve {
        /// Port to listen on
        #[arg(long)]
        port: Option<u16>,
        /// Host to bind to
        #[arg(long)]
        host: Option<String>,
        /// Custom database path
        #[arg(long)]
        db_path: Option<String>,
    },
    /// Watch the directory for changes and update the index
    Watch {
        /// Path to watch (default: current directory or configured index path)
        path: Option<String>,
        /// Custom database path
        #[arg(long)]
        db_path: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = AppConfig::new()?;

    // Initialize Tracing (Logging)
    // We write logs to stderr to keep stdout clean for piped search results (JSON/Text).
    let log_level = config.log_level.parse::<Level>().unwrap_or(Level::INFO);
    let log_format = config.log_format.as_str();

    if log_format.eq_ignore_ascii_case("json") {
        tracing_subscriber::fmt()
            .with_max_level(log_level)
            .with_writer(std::io::stderr)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(log_level)
            .with_writer(std::io::stderr)
            .init();
    }

    match args.cmd {
        Commands::Index {
            path,
            db_path,
            update,
            force,
        } => {
            let actual_path = path.unwrap_or(config.default_index_path);
            let actual_db = db_path.unwrap_or(config.db_path);

            if force {
                info!("Force flag set. Removing database at: {}", actual_db);
                if Path::new(&actual_db).exists() {
                    fs::remove_dir_all(&actual_db)?;
                }
            }

            info!("Indexing path: {}", actual_path);
            let index_path = Path::new(&actual_path);

            // 1. Load Models with Spinner
            let pb_model = ProgressBar::new_spinner();
            pb_model.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}")?);
            pb_model.enable_steady_tick(std::time::Duration::from_millis(120));
            pb_model.set_message("Loading embedding model...");

            let mut embedder = Embedder::new(
                config.embedding_model.clone(),
                config.reranker_model.clone(),
                config.embedding_model_path.clone(),
                config.reranker_model_path.clone(),
            )?;

            pb_model.set_message("Warming up ONNX Runtime...");
            let warmup_text = vec!["warmup".to_string()];
            let _ = embedder.embed(warmup_text.clone(), None)?;
            // Reranker is not needed for indexing, so we don't init or warmup here.

            pb_model.finish_with_message("Models loaded.");

            // 2. Initialize Storage
            let storage = Storage::new(&actual_db).await?;
            storage.init(embedder.dim()).await?;

            // 3. Initialize BM25 Index
            let bm25_index = match BM25Index::new(&actual_db, false, &config.merge_policy) {
                Ok(idx) => idx,
                Err(e) => {
                    warn!(
                        "Failed to initialize BM25 index: {}. Hybrid search may be degraded.",
                        e
                    );
                    return Err(e);
                }
            };

            let chunker = CodeChunker::new(config.chunk_size, config.chunk_overlap);

            // 4. Scan Files (Collect first for determinate progress bar)
            let pb_scan = ProgressBar::new_spinner();
            pb_scan.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
            pb_scan.enable_steady_tick(std::time::Duration::from_millis(120));
            pb_scan.set_message("Scanning files...");

            // Use ignore::WalkBuilder with config exclusions if possible, or manual filter
            // Note: WalkBuilder respects .gitignore by default.
            // Config exclusions: config.exclusions
            let builder = WalkBuilder::new(index_path);
            // WalkBuilder doesn't easily take a Vec<String> of globs directly without overrides
            // For now, adhering to existing behavior + standard .gitignore
            let walker = builder.build();

            let mut entries = Vec::new();
            for entry in walker.flatten() {
                if entry.file_type().is_some_and(|ft| ft.is_file()) {
                    let path = entry.path();
                    let path_str = path.to_string_lossy();
                    // Simple exclusion check
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

            // 5. Indexing Loop (Determinate Progress Bar)
            let total_files = entries.len() as u64;
            let pb_index = ProgressBar::new(total_files);
            pb_index.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
                    .progress_chars("#>-"),
            );
            pb_index.set_message("Indexing...");

            // For incremental updates
            let existing_files = if update {
                pb_index.set_message("Fetching existing metadata...");
                storage.get_indexed_metadata().await?
            } else {
                HashMap::new()
            };

            let mut chunks_buffer = Vec::new();
            // Process files
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

                // Check modification time
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
                            // Changed: delete old chunks first
                            if let Err(e) = storage.delete_file_chunks(&fname_str).await {
                                warn!("Error deleting old chunks for {}: {}", fname_str, e);
                            }
                            if let Err(e) = bm25_index.delete_file(&fname_str) {
                                warn!("Error deleting old BM25 docs for {}: {}", fname_str, e);
                            }
                        }
                    }

                    // Read and Chunk
                    if let Ok(code) = fs::read_to_string(file_path) {
                        let new_chunks = chunker.chunk_file(&fname_str, &code, mtime);
                        chunks_buffer.extend(new_chunks);
                    }
                }

                // Flush buffer if large enough
                if chunks_buffer.len() >= 256 {
                    let chunk_slice = &chunks_buffer;
                    let texts: Vec<String> = chunk_slice.iter().map(|c| c.code.clone()).collect();
                    match embedder.embed(texts, Some(256)) {
                        Ok(embeddings) => {
                            // Unpack and store
                            let ids: Vec<String> = chunk_slice
                                .iter()
                                .map(|c| format!("{}-{}-{}", c.filename, c.line_start, c.line_end))
                                .collect();
                            let filenames: Vec<String> =
                                chunk_slice.iter().map(|c| c.filename.clone()).collect();
                            let codes: Vec<String> =
                                chunk_slice.iter().map(|c| c.code.clone()).collect();
                            let starts: Vec<i32> =
                                chunk_slice.iter().map(|c| c.line_start as i32).collect();
                            let ends: Vec<i32> =
                                chunk_slice.iter().map(|c| c.line_end as i32).collect();
                            let mtimes: Vec<i64> =
                                chunk_slice.iter().map(|c| c.last_modified).collect();
                            let calls: Vec<Vec<String>> =
                                chunk_slice.iter().map(|c| c.calls.clone()).collect();

                            if let Err(e) = storage
                                .add_chunks(
                                    ids, filenames, codes, starts, ends, mtimes, calls, embeddings,
                                )
                                .await
                            {
                                error!("Error storing chunks: {}", e);
                            }
                            if let Err(e) = bm25_index.add_chunks(chunk_slice) {
                                error!("Error adding to BM25: {}", e);
                            }
                        }
                        Err(e) => error!("Error generating embeddings: {}", e),
                    }
                    chunks_buffer.clear();
                }
            }

            // Flush remaining chunks
            if !chunks_buffer.is_empty() {
                pb_index.set_message("Flushing remaining chunks...");
                let texts: Vec<String> = chunks_buffer.iter().map(|c| c.code.clone()).collect();
                match embedder.embed(texts, Some(256)) {
                    Ok(embeddings) => {
                        let ids: Vec<String> = chunks_buffer
                            .iter()
                            .map(|c| format!("{}-{}-{}", c.filename, c.line_start, c.line_end))
                            .collect();
                        let filenames: Vec<String> =
                            chunks_buffer.iter().map(|c| c.filename.clone()).collect();
                        let codes: Vec<String> =
                            chunks_buffer.iter().map(|c| c.code.clone()).collect();
                        let starts: Vec<i32> =
                            chunks_buffer.iter().map(|c| c.line_start as i32).collect();
                        let ends: Vec<i32> =
                            chunks_buffer.iter().map(|c| c.line_end as i32).collect();
                        let mtimes: Vec<i64> =
                            chunks_buffer.iter().map(|c| c.last_modified).collect();
                        let calls: Vec<Vec<String>> =
                            chunks_buffer.iter().map(|c| c.calls.clone()).collect();

                        if let Err(e) = storage
                            .add_chunks(
                                ids, filenames, codes, starts, ends, mtimes, calls, embeddings,
                            )
                            .await
                        {
                            error!("Error storing remaining chunks: {}", e);
                        }
                        if let Err(e) = bm25_index.add_chunks(&chunks_buffer) {
                            error!("Error adding remaining chunks to BM25: {}", e);
                        }
                    }
                    Err(e) => error!("Error embedding remaining chunks: {}", e),
                }
            }

            pb_index.finish_with_message("Indexing complete.");

            info!("Optimizing index (creating filename index)...");
            if let Err(e) = storage.create_filename_index().await {
                warn!("Optimization warning: {}", e);
            }
        }
        Commands::Search {
            query,
            limit,
            db_path,
            html,
            json,
            ext,
            dir,
            no_rerank,
        } => {
            let actual_db = db_path.unwrap_or(config.db_path);
            let actual_limit = limit.unwrap_or(config.default_limit);

            let storage = Storage::new(&actual_db).await?;
            let embedder = if json {
                Embedder::new_with_quiet(
                    true,
                    config.embedding_model.clone(),
                    config.reranker_model.clone(),
                    config.embedding_model_path.clone(),
                    config.reranker_model_path.clone(),
                )?
            } else {
                Embedder::new(
                    config.embedding_model.clone(),
                    config.reranker_model.clone(),
                    config.embedding_model_path.clone(),
                    config.reranker_model_path.clone(),
                )?
            };

            // Initialize BM25 Index (Optional)
            // Pass "log" or any default, as readonly=true makes it irrelevant
            let bm25_index = BM25Index::new(&actual_db, true, "log").ok();
            if bm25_index.is_none() {
                warn!("BM25 index could not be opened. Falling back to pure vector search.");
            }

            // Init Searcher with BM25
            let mut searcher = CodeSearcher::new(
                Some(storage),
                Some(embedder),
                bm25_index,
                config.vector_weight,
                config.bm25_weight,
                config.rrf_k,
            );

            if !json {
                println!("Searching for: '{}'", query);
            }

            let search_results = searcher
                .semantic_search(&query, actual_limit, ext, dir, no_rerank)
                .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&search_results)?);
            } else if html {
                let report = generate_html_report(&query, &search_results);
                let report_path = "results.html";
                fs::write(report_path, report)?;
                println!(
                    "{} {}",
                    "HTML Report generated:".green().bold(),
                    report_path
                );
            } else {
                for res in search_results {
                    println!(
                        "\n{} {} (Score: {:.4})",
                        "Rank".bold(),
                        res.rank.to_string().cyan(),
                        res.score
                    );
                    println!(
                        "{} {}:{}-{}",
                        "File:".bold(),
                        res.filename.yellow(),
                        res.line_start,
                        res.line_end
                    );
                    let snippet: String =
                        res.code.lines().take(10).collect::<Vec<&str>>().join("\n");
                    println!("{}\n{}", "---".dimmed(), snippet);
                    println!("{}", "---".dimmed());
                }
            }
        }
        Commands::Grep { pattern, json } => {
            // For Grep, strict functionality relies on walkdir/regex or the CodeSearcher helper.
            let searcher = CodeSearcher::new(
                None,
                None,
                None,
                config.vector_weight,
                config.bm25_weight,
                config.rrf_k,
            );

            if !json {
                println!("Grepping for: '{}'", pattern);
            }

            match searcher.grep_search(&pattern, ".") {
                Ok(matches) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&matches)?);
                    } else {
                        for m in matches {
                            println!("{}", m);
                        }
                    }
                }
                Err(e) => error!("Grep failed: {}", e),
            }
        }
        Commands::Serve {
            port,
            host,
            db_path,
        } => {
            let actual_db = db_path.unwrap_or(config.db_path);
            let actual_port = port.unwrap_or(config.server_port);
            let actual_host = host.unwrap_or(config.server_host);

            info!("Starting server at {}:{}", actual_host, actual_port);
            start_server(
                actual_host,
                actual_port,
                actual_db,
                config.embedding_model.clone(),
                config.reranker_model.clone(),
                config.embedding_model_path.clone(),
                config.reranker_model_path.clone(),
            )
            .await?;
        }
        Commands::Watch { path, db_path } => {
            let actual_path = path.unwrap_or(config.default_index_path);
            let actual_db = db_path.unwrap_or(config.db_path);

            info!("Initializing watcher for path: {}", actual_path);

            // 1. Initialize Components
            let mut embedder = Embedder::new(
                config.embedding_model.clone(),
                config.reranker_model.clone(),
                config.embedding_model_path.clone(),
                config.reranker_model_path.clone(),
            )?;
            embedder.init_reranker()?;

            let storage = Storage::new(&actual_db).await?;
            storage.init(embedder.dim()).await?; // Ensure schema

            let bm25_index = match BM25Index::new(&actual_db, false, &config.merge_policy) {
                Ok(idx) => idx,
                Err(e) => {
                    error!("Failed to initialize BM25 index: {}", e);
                    return Err(e);
                }
            };

            let chunker = CodeChunker::new(config.chunk_size, config.chunk_overlap);

            // 2. Start Watcher
            // This is a blocking call (due to our channel loop implementation)
            start_watcher(&actual_path, storage, embedder, bm25_index, chunker).await?;
        }
    }

    Ok(())
}
