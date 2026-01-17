pub mod bm25;
pub mod commands;
pub mod config;
pub mod core;
pub mod embedding;
pub mod indexer;
pub mod ops;
pub mod reporting;
pub mod search;
pub mod server;
pub mod storage;
pub mod watcher;

// Re-export core types for convenience
pub use config::AppConfig;
pub use core::CodeRagError;
