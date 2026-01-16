use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub db_path: String,
    pub default_index_path: String,
    pub default_limit: usize,
    pub server_host: String,
    pub server_port: u16,
    pub exclusions: Vec<String>,
    pub log_level: String,
    pub log_format: String,
    pub embedding_model: String,
    pub reranker_model: String,
    pub embedding_model_path: Option<String>,
    pub reranker_model_path: Option<String>,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        // Defaults
        let mut s = Config::builder()
            .set_default("db_path", "./.lancedb")?
            .set_default("default_index_path", ".")?
            .set_default("default_limit", 5)?
            .set_default("server_host", "127.0.0.1")?
            .set_default("server_port", 3000)?
            .set_default("exclusions", Vec::<String>::new())?
            .set_default("log_level", "info")?
            .set_default("log_format", "text")?
            .set_default("embedding_model", "nomic-embed-text-v1.5")?
            .set_default("embedding_model", "nomic-embed-text-v1.5")?
            .set_default("reranker_model", "bge-reranker-base")?
            .set_default("chunk_size", 1024)?
            .set_default("chunk_overlap", 128)?;

        // 1. File: code-ragcnf.toml (Current Directory)
        if PathBuf::from("code-ragcnf.toml").exists() {
            s = s.add_source(File::with_name("code-ragcnf"));
        }

        // 2. File: ~/.config/code-rag/code-ragcnf.toml (User Config)
        if let Some(mut home) = dirs::config_dir() {
            home.push("code-rag");
            home.push("code-ragcnf");
            // Check for both without extension and with .toml extension
            s = s.add_source(File::from(home).required(false));
        }

        // 3. Environment: CODE_RAG__KEY=VALUE
        // e.g., CODE_RAG__DB_PATH=/tmp/db
        s = s.add_source(Environment::with_prefix("CODE_RAG").separator("__"));

        s.build()?.try_deserialize()
    }
}
