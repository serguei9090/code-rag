use colored::*;
use std::fs;
use tracing::{error, warn};

use crate::bm25::BM25Index;
use crate::config::AppConfig;
use crate::core::CodeRagError;
use crate::embedding::Embedder;
use crate::llm::client::OllamaClient;
use crate::llm::expander::QueryExpander;
use crate::reporting::generate_html_report;
use crate::search::CodeSearcher;
use crate::storage::Storage;
use std::sync::Arc;

pub struct SearchOptions {
    pub limit: Option<usize>,
    pub db_path: Option<String>,
    pub html: bool,
    pub json: bool,
    pub ext: Option<String>,
    pub dir: Option<String>,
    pub no_rerank: bool,
    pub workspace: Option<String>,

    pub max_tokens: Option<usize>,
    pub expand: bool,
}

pub async fn search_codebase(
    query: String,
    options: SearchOptions,
    config: &AppConfig,
) -> Result<(), CodeRagError> {
    let SearchOptions {
        limit,
        db_path,
        html,
        json,
        ext,
        dir,
        no_rerank,
        workspace,

        max_tokens,
        expand,
    } = options;

    let actual_limit = limit.unwrap_or(config.default_limit);
    let base_db = db_path.unwrap_or_else(|| config.db_path.clone());
    let (actual_db, table_name) = if let Some(ws) = workspace.clone() {
        if ws == "default" {
            (base_db, "code_chunks".to_string())
        } else {
            (
                std::path::Path::new(&base_db)
                    .join(&ws)
                    .to_string_lossy()
                    .to_string(),
                "code_chunks".to_string(),
            )
        }
    } else {
        (base_db, "code_chunks".to_string())
    };

    let storage = Storage::new(&actual_db, &table_name)
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
            config.device.clone(),
        )?
    } else {
        Embedder::new(
            config.embedding_model.clone(),
            config.reranker_model.clone(),
            config.embedding_model_path.clone(),
            config.reranker_model_path.clone(),
            config.device.clone(),
        )?
    };

    // Initialize BM25 Index (Optional)
    let bm25_index = BM25Index::new(&actual_db, true, "log").ok();
    if bm25_index.is_none() {
        warn!("BM25 index could not be opened. Falling back to pure vector search.");
        warn!("BM25 index could not be opened. Falling back to pure vector search.");
    }

    // Initialize Query Expander (Optional)
    let expander = if config.llm_enabled {
        let client = OllamaClient::new(&config.llm_host, &config.llm_model);
        Some(Arc::new(QueryExpander::new(Arc::new(client))))
    } else {
        None
    };

    let searcher = CodeSearcher::new(
        Some(Arc::new(storage)),
        Some(Arc::new(embedder)),
        bm25_index.map(Arc::new),
        expander,
        config.vector_weight,
        config.bm25_weight,
        config.rrf_k as f64,
    );

    if !json {
        println!("Searching for: '{}'", query);
    }

    let search_results = searcher
        .semantic_search(
            &query,
            actual_limit,
            ext,
            dir,
            no_rerank,
            workspace,
            max_tokens,
            expand,
        )
        .await
        .map_err(|e| CodeRagError::Search(e.to_string()))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&search_results)?);
    } else if html {
        let report = generate_html_report(&query, &search_results)
            .map_err(|e| CodeRagError::Search(e.to_string()))?;
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
        None,
        config.vector_weight,
        config.bm25_weight,
        config.rrf_k as f64,
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

/// Helper to create a CodeSearcher instance for API/MCP usage.
/// This skips the CLI spinners/logging but performs the same initialization.
pub async fn create_searcher(
    db_path: Option<String>,
    config: &AppConfig,
) -> Result<CodeSearcher, CodeRagError> {
    let actual_db = db_path.unwrap_or_else(|| config.db_path.clone());

    let storage = Storage::new(&actual_db, "code_chunks")
        .await
        .map_err(|e| CodeRagError::Database(e.to_string()))?;

    // Use quiet mode for Embedder to avoid polluting stdout/logs too much
    let embedder = Embedder::new_with_quiet(
        true,
        config.embedding_model.clone(),
        config.reranker_model.clone(),
        config.embedding_model_path.clone(),
        config.reranker_model_path.clone(),
        config.device.clone(),
    )?;

    let bm25_index = BM25Index::new(&actual_db, true, "log").ok();

    let expander = if config.llm_enabled {
        let client = crate::llm::client::OllamaClient::new(&config.llm_host, &config.llm_model);
        Some(std::sync::Arc::new(
            crate::llm::expander::QueryExpander::new(std::sync::Arc::new(client)),
        ))
    } else {
        None
    };

    Ok(CodeSearcher::new(
        Some(std::sync::Arc::new(storage)),
        Some(std::sync::Arc::new(embedder)),
        bm25_index.map(std::sync::Arc::new),
        expander,
        config.vector_weight,
        config.bm25_weight,
        config.rrf_k as f64,
    ))
}
