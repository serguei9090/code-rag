use colored::*;
use std::fs;
use tracing::{error, warn};

use crate::bm25::BM25Index;
use crate::config::AppConfig;
use crate::core::CodeRagError;
use crate::embedding::Embedder;
use crate::reporting::generate_html_report;
use crate::search::CodeSearcher;
use crate::storage::Storage;

pub async fn search_codebase(
    query: String,
    limit: Option<usize>,
    db_path: Option<String>,
    html: bool,
    json: bool,
    ext: Option<String>,
    dir: Option<String>,
    no_rerank: bool,
    config: &AppConfig,
) -> Result<(), CodeRagError> {
    let actual_db = db_path.unwrap_or_else(|| config.db_path.clone());
    let actual_limit = limit.unwrap_or(config.default_limit);

    let storage = Storage::new(&actual_db)
        .await
        .map_err(|e| CodeRagError::Database(e.to_string()))?;

    // Silence embedder logs if outputting JSON
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
    let bm25_index = BM25Index::new(&actual_db, true, "log").ok();
    if bm25_index.is_none() {
        warn!("BM25 index could not be opened. Falling back to pure vector search.");
    }

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
        .await
        .map_err(|e| CodeRagError::Search(e.to_string()))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&search_results)?);
    } else if html {
        let report = generate_html_report(&query, &search_results);
        let report_path = "results.html";
        fs::write(report_path, report).map_err(CodeRagError::Io)?;
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
            let snippet: String = res.code.lines().take(10).collect::<Vec<&str>>().join("\n");
            println!("{}\n{}", "---".dimmed(), snippet);
            println!("{}", "---".dimmed());
        }
    }

    Ok(())
}

pub fn grep_codebase(pattern: String, json: bool, config: &AppConfig) -> Result<(), CodeRagError> {
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

    Ok(())
}
