use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
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
    pub max_file_size_bytes: usize,
    pub vector_weight: f32,
    pub bm25_weight: f32,
    pub rrf_k: f32,
    pub merge_policy: String, // "log", "sum", "replace"
    pub telemetry_enabled: bool,
    pub telemetry_endpoint: String,
    pub device: String, // "auto", "cpu", "cuda", "metal"
    pub batch_size: usize,
    pub threads: Option<usize>,
    pub priority: String, // "low", "normal", "high"
    pub llm_enabled: bool,
    pub llm_model: String,
    pub llm_host: String,

    // Service Flags
    pub enable_server: bool,
    pub enable_mcp: bool,
    pub enable_watch: bool,

    // Multi-Workspace
    #[serde(default)]
    pub workspaces: std::collections::HashMap<String, String>,
}

impl AppConfig {
    /// Load default config (looks for code-rag.toml in current directory)
    pub fn new() -> Result<Self, ConfigError> {
        Self::from_path(None)
    }

    /// Load config from a specific file path
    pub fn from_path(custom_path: Option<String>) -> Result<Self, ConfigError> {
        // Set all defaults
        let mut builder = Config::builder()
            .set_default("db_path", "./.lancedb")?
            .set_default("default_index_path", ".")?
            .set_default("default_limit", 5)?
            .set_default("server_host", "127.0.0.1")?
            .set_default("server_port", 3000)?
            .set_default("exclusions", Vec::<String>::new())?
            .set_default("log_level", "warn")? // Changed from "info" to "warn"
            .set_default("log_format", "text")?
            .set_default("log_to_file", false)?
            .set_default("log_dir", "logs")?
            .set_default("embedding_model", "nomic-embed-text-v1.5")?
            .set_default("reranker_model", "bge-reranker-base")?
            .set_default("chunk_size", 1024)?
            .set_default("chunk_overlap", 128)?
            .set_default("max_file_size_bytes", 10 * 1024 * 1024)?
            .set_default("vector_weight", 1.0)?
            .set_default("bm25_weight", 1.0)?
            .set_default("rrf_k", 60.0)?
            .set_default("merge_policy", "log")?
            .set_default("telemetry_enabled", false)?
            .set_default("telemetry_endpoint", "http://localhost:4317")?
            .set_default("device", "auto")?
            .set_default("batch_size", 256)?
            .set_default("priority", "normal")?
            .set_default("llm_enabled", false)?
            .set_default("llm_model", "mistral")?
            .set_default("llm_host", "http://localhost:11434")?
            .set_default("enable_server", false)?
            .set_default("enable_mcp", false)?
            .set_default("enable_watch", false)?
            .set_default(
                "workspaces",
                std::collections::HashMap::<String, String>::new(),
            )?;

        // Load from file (custom path OR defaults)
        if let Some(path) = custom_path {
            // Custom config file specified via --config
            let path_buf = PathBuf::from(&path);

            if !path_buf.exists() {
                return Err(ConfigError::Message(format!(
                    "Config file not found: {}",
                    path
                )));
            }

            if path_buf.extension().and_then(|s| s.to_str()) != Some("toml") {
                return Err(ConfigError::Message(format!(
                    "Config file must have .toml extension: {}",
                    path
                )));
            }

            builder = builder.add_source(File::from(path_buf));
        } else {
            // No custom path - try standard locations
            // 1. File: ~/.config/code-rag/code-rag.toml (User Config)
            if let Some(mut home) = dirs::config_dir() {
                home.push("code-rag");
                home.push("code-rag.toml");
                builder = builder.add_source(File::from(home).required(false));
            }

            // 2. File: code-rag.toml (Current Directory) - takes precedence
            if PathBuf::from("code-rag.toml").exists() {
                builder = builder.add_source(File::with_name("code-rag"));
            }
        }

        // 3. Environment: CODE_RAG__KEY=VALUE (always checked, lowest precedence)
        builder = builder.add_source(Environment::with_prefix("CODE_RAG").separator("__"));

        // Build and deserialize with helpful error messages
        let config = builder.build()?;

        config.try_deserialize().map_err(|e| {
            // Provide helpful error for unknown fields
            let err_msg = e.to_string();
            if err_msg.contains("unknown field") {
                ConfigError::Message(format!(
                    "Invalid configuration key found.\n{}\n\nPlease check your config file for typos.\nRun 'code-rag --help' to see valid options.",
                    err_msg
                ))
            } else {
                e
            }
        })
    }

    /// For backward compatibility - old load function
    pub fn load(include_files: bool) -> Result<Self, ConfigError> {
        if include_files {
            Self::new()
        } else {
            // Load only defaults (for tests)
            Self::from_path(None)
        }
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
