use clap::{Parser, Subcommand};
use code_rag::bm25::BM25Index;
use code_rag::config::AppConfig;
use code_rag::embedding::Embedder;
use code_rag::indexer::CodeChunker;
use code_rag::reporting::generate_html_report;
use code_rag::search::CodeSearcher;
use code_rag::server::start_server;
use code_rag::storage::Storage;
use colored::*;
use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

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
        /// Custom database path (default: ./.lancedb)
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
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
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
        #[arg(long, default_value_t = 3000)]
        port: u16,
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Custom database path
        #[arg(long)]
        db_path: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config = AppConfig::new()?;

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
                println!("Force flag set. Removing database at: {}", actual_db);
                if Path::new(&actual_db).exists() {
                    fs::remove_dir_all(&actual_db)?;
                }
            }

            println!("Indexing path: {}", actual_path);
            let index_path = Path::new(&actual_path);

            // Initialize components
            let storage = Storage::new(&actual_db).await?;
            storage.init().await?;

            // Initialize BM25 Index
            let bm25_index = match BM25Index::new(&actual_db) {
                Ok(idx) => idx,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to initialize BM25 index: {}. Hybrid search may be degraded.",
                        e
                    );
                    return Err(e);
                }
            };

            let mut embedder = Embedder::new()?;
            // Pre-cache Re-ranker
            embedder.init_reranker()?;

            // Warmup Models (Force load to RAM/ONNX Runtime init)
            println!("Running warmup query to initialize ONNX Runtime...");
            let warmup_text = vec!["warmup".to_string()];
            let _ = embedder.embed(warmup_text.clone(), None)?;
            let _ = embedder.rerank("query", warmup_text).is_ok();

            let chunker = CodeChunker::new();

            println!("Scanning files...");
            let pb_scan = ProgressBar::new_spinner();
            pb_scan.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg} [{elapsed_precise}]")?,
            );

            let walker = WalkBuilder::new(index_path).build();
            let mut chunks_batch = Vec::new();
            let mut file_count = 0;

            // For incremental updates
            let existing_files = if update {
                println!("Fetching existing index metadata...");
                storage.get_indexed_metadata().await?
            } else {
                HashMap::new()
            };

            for result in walker {
                match result {
                    Ok(entry) => {
                        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                            continue;
                        }

                        let file_path = entry.path();
                        let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");

                        if CodeChunker::get_language(ext).is_some() {
                            if let Ok(metadata) = fs::metadata(file_path) {
                                let modified = metadata
                                    .modified()
                                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                                let mtime = modified
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i64;
                                let fname_str = file_path.to_string_lossy().to_string();

                                if update {
                                    if let Some(stored_mtime) = existing_files.get(&fname_str) {
                                        if *stored_mtime == mtime {
                                            // Unchanged, skip
                                            continue;
                                        }
                                        // Changed, remove old contents
                                        // We await here because this loop IS async context (main is async)
                                        // But WalkBuilder is synchronous iterator.
                                        // We can't await inside the iterator loop nicely without collecting.
                                        // Ideally we collect updated files first.
                                        // For now, we will just delete later or ignore?
                                        // The 'storage.add_chunks' overwriting logic:
                                        // LanceDB append mostly. We need to delete old chunks.
                                        // Let's do a synchronous delete or collect to delete batch?
                                        // We'll collect 'files_to_delete' string list.
                                    }
                                }

                                if let Ok(code) = fs::read_to_string(file_path) {
                                    file_count += 1;
                                    pb_scan.set_message(format!(
                                        "Scanning: {} ({} chunks found)",
                                        file_path.display(),
                                        chunks_batch.len()
                                    ));
                                    let new_chunks = chunker.chunk_file(&fname_str, &code, mtime);
                                    chunks_batch.extend(new_chunks);
                                }
                            }
                        }
                    }
                    Err(err) => eprintln!("Error walking entry: {}", err),
                }
            }

            pb_scan.finish_and_clear();
            println!(
                "Scan complete. Found {} chunks across {} files.",
                chunks_batch.len(),
                file_count
            );

            if chunks_batch.is_empty() {
                return Ok(());
            }

            // Delete updated files if any
            if update {
                // Determine distinct filenames in the new batch that are also in existing db
                // and delete them first.
                // This naive approach deletes everything we found that we are about to re-index.
                let mut files_to_delete = std::collections::HashSet::new();
                for chunk in &chunks_batch {
                    if existing_files.contains_key(&chunk.filename) {
                        files_to_delete.insert(chunk.filename.clone());
                    }
                }

                for file in files_to_delete {
                    storage.delete_file_chunks(&file).await?;
                    bm25_index.delete_file(&file)?;
                }
            }

            println!("Generating embeddings...");
            let pb_embed = ProgressBar::new(chunks_batch.len() as u64);
            pb_embed.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}",
                    )?
                    .progress_chars("#>-"),
            );

            for chunk_slice in chunks_batch.chunks(256) {
                let texts: Vec<String> = chunk_slice.iter().map(|c| c.code.clone()).collect();
                let embeddings = embedder.embed(texts, Some(256))?;

                pb_embed.inc(chunk_slice.len() as u64);

                // Simple ID generation
                let ids: Vec<String> = chunk_slice
                    .iter()
                    .map(|c| format!("{}-{}-{}", c.filename, c.line_start, c.line_end))
                    .collect();

                let filenames: Vec<String> =
                    chunk_slice.iter().map(|c| c.filename.clone()).collect();
                let codes: Vec<String> = chunk_slice.iter().map(|c| c.code.clone()).collect();
                let starts: Vec<i32> = chunk_slice.iter().map(|c| c.line_start as i32).collect();
                let ends: Vec<i32> = chunk_slice.iter().map(|c| c.line_end as i32).collect();
                let mtimes: Vec<i64> = chunk_slice.iter().map(|c| c.last_modified).collect();
                let calls: Vec<Vec<String>> = chunk_slice.iter().map(|c| c.calls.clone()).collect();

                storage
                    .add_chunks(
                        ids, filenames, codes, starts, ends, mtimes, calls, embeddings,
                    )
                    .await?;

                if let Err(e) = bm25_index.add_chunks(chunk_slice) {
                    eprintln!("Warning: Failed to add chunks to BM25 index: {}", e);
                }
            }
            pb_embed.finish_with_message("Indexing complete.");

            println!("Optimizing index (creating filename index)...");
            if let Err(e) = storage.create_filename_index().await {
                eprintln!("Optimization warning: {}", e);
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
            let storage = Storage::new(&actual_db).await?;
            let embedder = Embedder::new()?;

            // Initialize BM25 Index (Optional)
            let bm25_index = BM25Index::new(&actual_db).ok();
            if bm25_index.is_none() {
                eprintln!(
                    "{}",
                    "Warning: BM25 index could not be opened. Falling back to pure vector search."
                        .yellow()
                );
            }

            // Init Searcher with BM25
            let mut searcher = CodeSearcher::new(Some(storage), Some(embedder), bm25_index);

            if !json {
                println!("Searching for: '{}'", query);
            }

            let search_results = searcher
                .semantic_search(&query, limit, ext, dir, no_rerank)
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
            // Assuming CodeSearcher has a simple grep implementation or we use grep crate.
            // But we need a searcher instance.
            // Since Grep might not need Storage/Embedder if strictly file-based:
            // But CodeSearcher::new takes them.
            // Let's create dummy ones or refactor `grep_search` to be static or standalone?
            // Checking previous knoweldge: `grep_search` loops over files.
            // Passing None is fine if `new` allows it.
            let searcher = CodeSearcher::new(None, None, None);

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
                Err(e) => eprintln!("Grep failed: {}", e),
            }
        }
        Commands::Serve {
            port,
            host,
            db_path,
        } => {
            let actual_db = db_path.unwrap_or(config.db_path);
            start_server(host, port, actual_db).await?;
        }
    }

    Ok(())
}
