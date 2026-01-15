use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub db_path: String,
    pub default_index_path: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::builder();

        // 1. Global config (~/.code-rag/config.toml)
        if let Some(mut home) = dirs::home_dir() {
            home.push(".code-rag");
            home.push("config.toml");
            if home.exists() {
                s = s.add_source(File::from(home).required(false));
            }
        }

        // 2. Local config (./code-rag.toml)
        s = s.add_source(File::with_name("code-rag").required(false));

        // 3. Environment variables (CODE_RAG_DB_PATH, etc.)
        s = s.add_source(Environment::with_prefix("CODE_RAG"));

        // Set defaults
        s = s.set_default("db_path", "./.lancedb")?;
        s = s.set_default("default_index_path", ".")?;

        let build = s.build()?;
        build.try_deserialize()
    }
}
