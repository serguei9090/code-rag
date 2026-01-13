use clap::{Parser, Subcommand};
use code_rag::indexer::CodeChunker;
use code_rag::storage::Storage;
use code_rag::embedding::Embedder;
use code_rag::search::CodeSearcher;
use ignore::WalkBuilder;
use std::path::Path;
use std::fs;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Index { path: String },
    Search {
        query: String,
        #[arg(short, long, default_value_t = 5)]
        limit: usize // changed to usize for compatibility
    },
    Grep { pattern: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.cmd {
        Commands::Index { path } => {
            println!("Indexing path: {}", path);
            let index_path = Path::new(&path);
            
            // Initialize components
            let storage = Storage::new("./.lancedb").await?;
            storage.init().await?;
            
            let mut embedder = Embedder::new()?;
            let chunker = CodeChunker::new();

            let walker = WalkBuilder::new(index_path).build();
            let mut chunks_batch = Vec::new();
            
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
                                 // Use absolute path for robustness or relative? File path is from WalkBuilder.
                                 let new_chunks = chunker.chunk_file(file_path.to_string_lossy().as_ref(), &code);
                                 chunks_batch.extend(new_chunks);
                             }
                        }
                    }
                    Err(err) => eprintln!("Error walking entry: {}", err),
                }
            }
            
            println!("Found {} chunks. embedding...", chunks_batch.len());
            
            for chunk_slice in chunks_batch.chunks(100) {
                let texts: Vec<String> = chunk_slice.iter().map(|c| c.code.clone()).collect();
                let embeddings = embedder.embed(texts)?;
                
                // Simple ID generation
                let ids: Vec<String> = chunk_slice.iter().map(|c| format!("{}-{}-{}", c.filename, c.line_start, c.line_end)).collect();

                let filenames: Vec<String> = chunk_slice.iter().map(|c| c.filename.clone()).collect();
                let codes: Vec<String> = chunk_slice.iter().map(|c| c.code.clone()).collect();
                let starts: Vec<i32> = chunk_slice.iter().map(|c| c.line_start as i32).collect();
                let ends: Vec<i32> = chunk_slice.iter().map(|c| c.line_end as i32).collect();
                
                storage.add_chunks(ids, filenames, codes, starts, ends, embeddings).await?;
            }
            
            println!("Indexing complete.");
        }
        Commands::Search { query, limit } => {
            let storage = Storage::new("./.lancedb").await?;
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
                     println!("\nMatch {}: {}", i + 1, filenames.value(i));
                     let code = codes.value(i);
                     let snippet: String = code.chars().take(200).collect();
                     println!("{}\n...", snippet);
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


