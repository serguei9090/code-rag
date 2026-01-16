use code_rag::config::AppConfig;
use std::env;

#[test]
fn test_env_override() {
    // Set environment variable
    // CODE_RAG__DB_PATH -> "test_env_db"
    let key = "CODE_RAG__DB_PATH";
    let val = "test_env_db";
    env::set_var(key, val);

    // Initialize config
    let config = AppConfig::new().expect("Failed to load config");

    // Verify override
    assert_eq!(config.db_path, val);

    // Cleanup
    env::remove_var(key);
}

#[test]
fn test_default_values() {
    // Remove potentially conflicting env vars
    env::remove_var("CODE_RAG__CHUNK_SIZE");

    let config = AppConfig::new().expect("Failed to load config");

    // Check defaults matches specific values known in src/config.rs
    assert_eq!(config.chunk_size, 1024);
    assert_eq!(config.log_level, "info");
}
