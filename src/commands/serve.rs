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
    start_server(
        actual_host,
        actual_port,
        actual_db,
        config.embedding_model.clone(),
        config.reranker_model.clone(),
        config.embedding_model_path.clone(),
        config.reranker_model_path.clone(),
        config.device.clone(),
    )
    .await
    .map_err(|e| CodeRagError::Server(e.to_string()))?;

    Ok(())
}
