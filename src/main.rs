use clap::{Parser, Subcommand};
use code_rag::indexer::CodeChunker;
use code_rag::storage::Storage;
use code_rag::embedding::Embedder;
use code_rag::search::CodeSearcher;
use code_rag::config::AppConfig;
use code_rag::reporting::{SearchResult, generate_html_report};
use ignore::WalkBuilder;
use std::path::Path;
use std::fs;
use std::error::Error;
use indicatif::{ProgressBar, ProgressStyle};
use colored::*;
use std::collections::HashMap;
use arrow_array::{StringArray, Float32Array, Int32Array, ListArray, Array};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
     Index { 
        path: Option<String>,
        #[arg(long)]
        db_path: Option<String>,
        #[arg(long)]
        update: bool,
        #[arg(long)]
        force: bool,
    },
    Search {
        query: String,
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
        #[arg(long)]
        db_path: Option<String>,
        #[arg(long)]
        html: bool,
    },
    Grep { pattern: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config = AppConfig::new()?;

    match args.cmd {
        Commands::Index { path, db_path, update, force } => {
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
            
            let mut embedder = Embedder::new()?;
            let chunker = CodeChunker::new();

            println!("Scanning files...");
            let pb_scan = ProgressBar::new_spinner();
            pb_scan.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg} [{elapsed_precise}]")?);
            
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
                        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                            continue;
                        }
                        
                        let file_path = entry.path();
                        let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                        
                        if CodeChunker::get_language(ext).is_some() {
                             if let Ok(metadata) = fs::metadata(file_path) {
                                 let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                                 let mtime = modified.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
                                 let fname_str = file_path.to_string_lossy().to_string();

                                 if update {
                                     if let Some(stored_mtime) = existing_files.get(&fname_str) {
                                         if *stored_mtime == mtime {
                                             // Unchanged, skip
                                             continue;
                                         }
                                         // Changed, remove old chunks
                                         // Note: This is per-file. For batch efficiency we might want to batch deletes too, 
                                         // but for now deleting individually is safer logic-wise.
                                         // Ideally we defer deletes or just let them happen. 
                                         // LanceDB delete is async, we should await it.
                                         // But we are in a sync loop (walker). 
                                         // We need to collect files to delete or make the loop async-friendly?
                                         // Walker is sync.
                                         // We can't await here easily without block_on or refactoring.
                                         // Refactoring walker to be async is hard with 'ignore' crate.
                                     }
                                 }

                                 if let Ok(code) = fs::read_to_string(file_path) {
                                     file_count += 1;
                                     pb_scan.set_message(format!("Scanning: {} ({} chunks found)", file_path.display(), chunks_batch.len()));
                                     let new_chunks = chunker.chunk_file(&fname_str, &code, mtime);
                                     chunks_batch.extend(new_chunks);
                                 }
                             }
                        }
                    }
                    Err(err) => eprintln!("Error walking entry: {}", err),
                }
            }
            
            pb_scan.finish_with_message(format!("Scan complete. Found {} chunks across {} files.", chunks_batch.len(), file_count));
            
            if chunks_batch.is_empty() {
                return Ok(());
            }

            println!("Generating embeddings...");
            let pb_embed = ProgressBar::new(chunks_batch.len() as u64);
            pb_embed.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
                .progress_chars("#>-"));
            
            for chunk_slice in chunks_batch.chunks(100) {
                let texts: Vec<String> = chunk_slice.iter().map(|c| c.code.clone()).collect();
                let embeddings = embedder.embed(texts)?;
                
                pb_embed.inc(chunk_slice.len() as u64);
                
                // Simple ID generation
                let ids: Vec<String> = chunk_slice.iter().map(|c| format!("{}-{}-{}", c.filename, c.line_start, c.line_end)).collect();

                let filenames: Vec<String> = chunk_slice.iter().map(|c| c.filename.clone()).collect();
                let codes: Vec<String> = chunk_slice.iter().map(|c| c.code.clone()).collect();
                let starts: Vec<i32> = chunk_slice.iter().map(|c| c.line_start as i32).collect();
                let ends: Vec<i32> = chunk_slice.iter().map(|c| c.line_end as i32).collect();
                let mtimes: Vec<i64> = chunk_slice.iter().map(|c| c.last_modified).collect();
                let calls: Vec<Vec<String>> = chunk_slice.iter().map(|c| c.calls.clone()).collect();
                
                // Perform deletes for updated files (deduplicated)
                if update {
                    let unique_files: std::collections::HashSet<_> = filenames.iter().collect();
                    for file in unique_files {
                        // We await here because this loop IS async
                        let _ = storage.delete_file_chunks(file).await;
                    }
                }

                storage.add_chunks(ids, filenames, codes, starts, ends, mtimes, calls, embeddings).await?;
            }
            pb_embed.finish_with_message("Indexing complete.");
        }
        Commands::Search { query, limit, db_path, html } => {
            let actual_db = db_path.unwrap_or(config.db_path);
            let storage = Storage::new(&actual_db).await?;
            let embedder = Embedder::new()?;
            let mut searcher = CodeSearcher::new(Some(storage), Some(embedder));
            
            println!("Searching for: '{}'", query);
            let results = searcher.semantic_search(&query, limit).await?;
            
            let mut search_results = Vec::new();
            let mut rank = 1;

            for batch in results {
                 let filenames: &StringArray = batch.column_by_name("filename")
                     .expect("filename column missing")
                     .as_any().downcast_ref().expect("filename not string");
                 let codes: &StringArray = batch.column_by_name("code")
                     .expect("code column missing")
                     .as_any().downcast_ref().expect("code not string");
                 let line_starts: &Int32Array = batch.column_by_name("line_start")
                     .expect("line_start column missing")
                     .as_any().downcast_ref().expect("line_start not int32");
                 let line_ends: &Int32Array = batch.column_by_name("line_end")
                     .expect("line_end column missing")
                     .as_any().downcast_ref().expect("line_end not int32");
                 
                 let calls_col: Option<&ListArray> = batch.column_by_name("calls")
                     .and_then(|c| c.as_any().downcast_ref());
                     
                 let scores: Option<&Float32Array> = batch.column_by_name("_score")
                     .map(|c| c.as_any().downcast_ref().expect("_score not float32"));

                 for i in 0..batch.num_rows() {
                     let filename = filenames.value(i).to_string();
                     let code = codes.value(i).to_string();
                     let line_start = line_starts.value(i);
                     let line_end = line_ends.value(i);
                     let score = scores.map(|s| s.value(i)).unwrap_or(0.0);
                     
                     let mut calls_vec = Vec::new();
                     if let Some(calls_arr) = calls_col {
                         if !calls_arr.is_null(i) {
                             let list_val = calls_arr.value(i);
                             if let Some(str_arr) = list_val.as_any().downcast_ref::<StringArray>() {
                                 for s in str_arr.iter().flatten() {
                                     calls_vec.push(s.to_string());
                                 }
                             }
                         }
                     }
                     // Debug print

                     search_results.push(SearchResult {
                         rank,
                         score,
                         filename,
                         code,
                         line_start,
                         line_end,
                         calls: calls_vec,
                     });
                     rank += 1;
                 }
            }

            if html {
                let report = generate_html_report(&query, &search_results);
                let report_path = "results.html";
                fs::write(report_path, report)?;
                println!("{} {}", "HTML Report generated:".green().bold(), report_path);
                // Optional: Try to open it? println!("Open file://{}/{}", std::env::current_dir()?.display(), report_path);
            } else {
                 for res in search_results {
                     println!("\n{} {} (Score: {:.4})", "Rank".bold(), res.rank.to_string().cyan(), res.score);
                     println!("{} {}:{}-{}", "File:".bold(), res.filename.yellow(), res.line_start, res.line_end);
                     
                     let snippet: String = res.code.lines().take(10).collect::<Vec<&str>>().join("\n");
                     println!("{}\n{}", "---".dimmed(), snippet);
                     println!("{}", "---".dimmed());
                 }
            }
        }
        Commands::Grep { pattern } => {
            let searcher = CodeSearcher::new(None, None);
            println!("Grepping for: '{}'", pattern);
             match searcher.grep_search(&pattern, ".") {
                 Ok(matches) => {
                     for m in matches {
                         println!("{}", m);
                     }
                 },
                 Err(e) => eprintln!("Grep failed: {}", e),
             }
        }
    }
    
    Ok(())
}


