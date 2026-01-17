use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeRagError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Tantivy error: {0}")]
    Tantivy(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

// Helper to convert other errors to CodeRagError
impl From<lancedb::Error> for CodeRagError {
    fn from(err: lancedb::Error) -> Self {
        CodeRagError::Database(err.to_string())
    }
}

impl From<fastembed::Error> for CodeRagError {
    fn from(err: fastembed::Error) -> Self {
        CodeRagError::Embedding(err.to_string())
    }
}

impl From<tantivy::TantivyError> for CodeRagError {
    fn from(err: tantivy::TantivyError) -> Self {
        CodeRagError::Tantivy(err.to_string())
    }
}

impl axum::response::IntoResponse for CodeRagError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            CodeRagError::Io(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            CodeRagError::Config(e) => {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            CodeRagError::Database(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            CodeRagError::Embedding(e) => {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.clone())
            }
            CodeRagError::Search(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            CodeRagError::Server(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            CodeRagError::Serialization(e) => {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            CodeRagError::Tantivy(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            CodeRagError::Generic(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
        };

        let body = serde_json::json!({
            "error": message
        });

        (status, axum::Json(body)).into_response()
    }
}
