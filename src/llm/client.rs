use anyhow::Result;
use async_trait::async_trait;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};

/// Trait abstracting LLM interactions to allow for mocking and different backends.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Generates text based on the given prompt.
    async fn generate(&self, prompt: &str) -> Result<String>;
}

/// Client for interacting with a local Ollama instance.
pub struct OllamaClient {
    client: Ollama,
    model: String,
}

impl OllamaClient {
    /// Creates a new OllamaClient.
    pub fn new(host: &str, model: &str) -> Self {
        // Parse host string to URL for cleaner init, but Ollama::new takes protocol, host, port separately
        // For simplicity with ollama-rs 0.2, likely need to rely on default or parsing.
        // Actually ollama_rs::Ollama::new takes (host, port).
        // If config gives full URL "http://localhost:11434", we might need to parse it.
        // For now, assuming default localhost behavior if host parsing logic is complex
        // or just passing simple host/port.
        // Let's stick to default initialization if host is standard,
        // or parse the config `llm_host`.

        let url = url::Url::parse(host).unwrap_or_else(|_| {
            url::Url::parse("http://localhost:11434").expect("Hardcoded default URL must be valid")
        });
        let host_str = url.host_str().unwrap_or("localhost").to_string();
        let port = url.port().unwrap_or(11434);

        let client = Ollama::new(format!("{}://{}", url.scheme(), host_str), port);

        Self {
            client,
            model: model.to_string(),
        }
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let request = GenerationRequest::new(self.model.clone(), prompt.to_string());

        let response = self
            .client
            .generate(request)
            .await
            .map_err(|e| anyhow::anyhow!("Ollama generation failed: {}", e))?;

        Ok(response.response)
    }
}

#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::sync::Mutex;

    pub struct MockLlmClient {
        pub response: Mutex<String>,
    }

    impl MockLlmClient {
        pub fn new(response: &str) -> Self {
            Self {
                response: Mutex::new(response.to_string()),
            }
        }
    }

    #[async_trait]
    impl LlmClient for MockLlmClient {
        async fn generate(&self, _prompt: &str) -> Result<String> {
            Ok(self
                .response
                .lock()
                .map_err(|e| anyhow::anyhow!("Mutex lock poisoned: {}", e))?
                .clone())
        }
    }
}
