use clap::{Parser, Subcommand};
use code_rag::indexer::CodeChunker;
use code_rag::storage::Storage;
use code_rag::embedding::Embedder;
use code_rag::search::CodeSearcher;
use code_rag::config::AppConfig;
use ignore::WalkBuilder;
use std::path::Path;
use std::fs;
use std::error::Error;
use indicatif::{ProgressBar, ProgressStyle};
use colored::*;

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
    },
    Search {
        query: String,
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
        #[arg(long)]
        db_path: Option<String>,
    },
    Grep { pattern: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config = AppConfig::new()?;

    match args.cmd {
        Commands::Index { path, db_path } => {
            let actual_path = path.unwrap_or(config.default_index_path);
            let actual_db = db_path.unwrap_or(config.db_path);

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
            
            for result in walker {
                match result {
                    Ok(entry) => {
                        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                            continue;
                        }
                        
                        let file_path = entry.path();
                        let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                        
                        if CodeChunker::get_language(ext).is_some() {
                             if let Ok(code) = fs::read_to_string(file_path) {
                                 file_count += 1;
                                 pb_scan.set_message(format!("Scanning: {} ({} chunks found)", file_path.display(), chunks_batch.len()));
                                 let new_chunks = chunker.chunk_file(file_path.to_string_lossy().as_ref(), &code);
                                 chunks_batch.extend(new_chunks);
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
                
                storage.add_chunks(ids, filenames, codes, starts, ends, embeddings).await?;
            }
            pb_embed.finish_with_message("Indexing complete.");
        }
        Commands::Search { query, limit, db_path } => {
            let actual_db = db_path.unwrap_or(config.db_path);
            let storage = Storage::new(&actual_db).await?;
            let embedder = Embedder::new()?;
            let mut searcher = CodeSearcher::new(Some(storage), Some(embedder));
            
            println!("Searching for: '{}'", query);
            let results = searcher.semantic_search(&query, limit).await?;
            
            for batch in results {
                 let filenames: &arrow_array::StringArray = batch.column_by_name("filename")
                     .expect("filename column missing")
                     .as_any()
                     .downcast_ref()
                     .expect("filename not string");
                 let codes: &arrow_array::StringArray = batch.column_by_name("code")
                     .expect("code column missing")
                     .as_any()
                     .downcast_ref()
                     .expect("code not string");
                     
                 for i in 0..batch.num_rows() {
                     let filename = filenames.value(i);
                     let score = if batch.column_by_name("_score").is_some() {
                         let scores: &arrow_array::Float32Array = batch.column_by_name("_score")
                             .unwrap()
                             .as_any()
                             .downcast_ref()
                             .unwrap();
                         scores.value(i)
                     } else { 0.0 };

                     println!("\n{} {} (Score: {:.4})", "Rank".bold(), (i + 1).to_string().cyan(), score);
                     println!("{} {}", "File:".bold(), filename.yellow());
                     
                     let code = codes.value(i);
                     let snippet: String = code.lines().take(10).collect::<Vec<&str>>().join("\n");
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


