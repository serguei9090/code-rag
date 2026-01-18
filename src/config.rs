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
    pub log_to_file: bool,
    pub log_dir: String,
    pub embedding_model: String,
    pub reranker_model: String,
    pub embedding_model_path: Option<String>,
    pub reranker_model_path: Option<String>,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub vector_weight: f32,
    pub bm25_weight: f32,
    pub rrf_k: f32,
    pub merge_policy: String, // "log", "sum", "replace"
    pub telemetry_enabled: bool,
    pub telemetry_endpoint: String,
    pub device: String, // "auto", "cpu", "cuda", "metal"
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        Self::load(true)
    }

    pub fn load(include_files: bool) -> Result<Self, ConfigError> {
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
            .set_default("log_to_file", false)?
            .set_default("log_dir", "logs")?
            .set_default("embedding_model", "nomic-embed-text-v1.5")?
            .set_default("reranker_model", "bge-reranker-base")?
            .set_default("chunk_size", 1024)?
            .set_default("chunk_overlap", 128)?
            .set_default("vector_weight", 1.0)?
            .set_default("bm25_weight", 1.0)?
            .set_default("rrf_k", 60.0)?
            .set_default("merge_policy", "log")?
            .set_default("telemetry_enabled", false)?
            .set_default("telemetry_endpoint", "http://localhost:4317")?
            .set_default("device", "auto")?;

        if include_files {
            // 1. File: ~/.config/code-rag/config_rag.toml (User Config)
            if let Some(mut home) = dirs::config_dir() {
                home.push("code-rag");
                home.push("config_rag");
                // Check for both without extension and with .toml extension
                s = s.add_source(File::from(home).required(false));
            }

            // 2. File: config_rag.toml (Current Directory) - takes precedence
            if PathBuf::from("config_rag.toml").exists() {
                s = s.add_source(File::with_name("config_rag"));
            }
        }

        // 3. Environment: CODE_RAG__KEY=VALUE
        // e.g., CODE_RAG__DB_PATH=/tmp/db
        s = s.add_source(Environment::with_prefix("CODE_RAG").separator("__"));

        s.build()?.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_sequential() {
        // Part 1: Default Logic
        // Ensure no env vars interfere
        env::remove_var("CODE_RAG__DB_PATH");
        env::remove_var("CODE_RAG__DEFAULT_LIMIT");

        let config = AppConfig::load(false).expect("Failed to load default config");
        assert_eq!(config.db_path, "./.lancedb");
        assert_eq!(config.default_limit, 5);
        assert_eq!(config.vector_weight, 1.0);

        // Part 2: Env Override Logic
        env::set_var("CODE_RAG__DB_PATH", "/tmp/test_db");
        env::set_var("CODE_RAG__DEFAULT_LIMIT", "10");

        let config = AppConfig::load(false).expect("Failed to load config with env vars");
        assert_eq!(config.db_path, "/tmp/test_db");
        assert_eq!(config.default_limit, 10);

        // Cleanup
        env::remove_var("CODE_RAG__DB_PATH");
        env::remove_var("CODE_RAG__DEFAULT_LIMIT");
    }
}
