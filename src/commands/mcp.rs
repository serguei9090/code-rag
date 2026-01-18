use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, Write};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::commands::search::create_searcher;
use crate::config::AppConfig;
use crate::search::CodeSearcher;

// Basic JSON-RPC types
#[derive(Serialize, Deserialize, Debug)]
struct Request {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<Error>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Error {
    code: i32,
    message: String,
    data: Option<Value>,
}

struct McpState {
    config: AppConfig,
    searcher: Mutex<Option<CodeSearcher>>,
}

pub async fn run(config: &AppConfig) -> Result<()> {
    let state = Arc::new(McpState {
        config: config.clone(),
        searcher: Mutex::new(None),
    });

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    eprintln!("MCP Server starting (stdio transport)...");

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break; // EOF
        }

        let request: Request = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                continue;
            }
        };

        let state_clone = state.clone();
        // Spawn each request handling to avoid blocking parsing of next line
        // though stdio is sequential usually.
        tokio::spawn(async move {
            if let Err(e) = handle_request(request, state_clone).await {
                error!("Error handling request: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_request(req: Request, state: Arc<McpState>) -> Result<()> {
    let mut response = Response {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: None,
        id: req.id.clone(),
    };

    match req.method.as_str() {
        "initialize" => {
            response.result = Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    },
                    "resources": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": "code-rag",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }));
        }
        "notifications/initialized" => {
            return Ok(());
        }
        "tools/list" => {
            response.result = Some(json!({
                "tools": [
                    {
                        "name": "search",
                        "description": "Semantic search over the indexed codebase",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "The search query"
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "Maximum number of results (default 10)"
                                },
                                "workspace": {
                                    "type": "string",
                                    "description": "Workspace name (default 'default')"
                                }
                            },
                            "required": ["query"]
                        }
                    }
                ]
            }));
        }
        "tools/call" => {
            if let Some(params) = req.params {
                let name = params["name"].as_str().unwrap_or("").to_string();
                let args = params["arguments"].as_object();

                if name == "search" {
                    if let Some(args) = args {
                        let query = args
                            .get("query")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let limit = args
                            .get("limit")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as usize)
                            .unwrap_or(10);
                        let workspace = args
                            .get("workspace")
                            .and_then(|v| v.as_str())
                            .unwrap_or("default")
                            .to_string();

                        match perform_search(&state, query, limit, workspace).await {
                            Ok(results) => {
                                // Format as MCP tool result (text content)
                                let text_content = serde_json::to_string_pretty(&results)?;
                                response.result = Some(json!({
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": text_content
                                        }
                                    ]
                                }));
                            }
                            Err(e) => {
                                response.error = Some(Error {
                                    code: -32000, // app error
                                    message: format!("Search failed: {}", e),
                                    data: None,
                                });
                            }
                        }
                    } else {
                        response.error = Some(Error {
                            code: -32602,
                            message: "Missing arguments".to_string(),
                            data: None,
                        });
                    }
                } else {
                    response.error = Some(Error {
                        code: -32601,
                        message: format!("Tool not found: {}", name),
                        data: None,
                    });
                }
            } else {
                response.error = Some(Error {
                    code: -32602,
                    message: "Invalid params".to_string(),
                    data: None,
                });
            }
        }
        _ => {
            // Basic implementation: ignore other methods or return MethodNotFound
            // If id is present, we must invoke response. if not, it's notification.
            if req.id.is_some() {
                response.error = Some(Error {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                });
            } else {
                return Ok(());
            }
        }
    }

    if req.id.is_some() {
        let response_str = serde_json::to_string(&response)?;
        let mut stdout = io::stdout();
        writeln!(stdout, "{}", response_str)?;
        stdout.flush()?;
    }

    Ok(())
}

async fn perform_search(
    state: &McpState,
    query: String,
    limit: usize,
    workspace: String,
) -> Result<Vec<crate::search::SearchResult>> {
    let mut searcher_guard = state.searcher.lock().await;

    if searcher_guard.is_none() {
        info!("Initializing CodeSearcher for MCP...");
        let searcher = create_searcher(None, &state.config)
            .await
            .context("Failed to create searcher")?;
        *searcher_guard = Some(searcher);
    }

    if let Some(searcher) = searcher_guard.as_mut() {
        // semantic_search arguments: query, limit, ext, dir, no_rerank, workspace, max_tokens, expand
        searcher
            .semantic_search(
                &query,
                limit,
                None,  // ext
                None,  // dir
                false, // no_rerank
                Some(workspace),
                None,  // max_tokens
                false, // expand
            )
            .await
            .context("Semantic search failed")
    } else {
        Err(anyhow::anyhow!("Searcher failed to initialize"))
    }
}
