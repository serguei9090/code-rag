use tracing::info;

use crate::config::AppConfig;
use crate::core::CodeRagError;
use crate::server::start_server;

pub async fn serve_api(
    port: Option<u16>,
    host: Option<String>,
    db_path: Option<String>,
    config: &AppConfig,
) -> Result<(), CodeRagError> {
    let actual_db = db_path.unwrap_or_else(|| config.db_path.clone());
    let actual_port = port.unwrap_or(config.server_port);
    let actual_host = host.unwrap_or_else(|| config.server_host.clone());

    info!("Starting server at {}:{}", actual_host, actual_port);
    start_server(crate::server::ServerStartConfig {
        host: actual_host,
        port: actual_port,
        db_path: actual_db,
        embedding_model: config.embedding_model.clone(),
        reranker_model: config.reranker_model.clone(),
        embedding_model_path: config.embedding_model_path.clone(),
        reranker_model_path: config.reranker_model_path.clone(),
        device: config.device.clone(),
        llm_enabled: config.llm_enabled,
        llm_host: config.llm_host.clone(),
        llm_model: config.llm_model.clone(),
    })
    .await
    .map_err(|e| CodeRagError::Server(e.to_string()))?;

    Ok(())
}
