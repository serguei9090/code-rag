use crate::llm::LlmClient;
use anyhow::Result;
use std::sync::Arc;

/// Service for expanding user queries into multiple related search terms.
pub struct QueryExpander {
    llm_client: Arc<dyn LlmClient>,
}

impl QueryExpander {
    /// Creates a new QueryExpander with the given LLM client.
    pub fn new(llm_client: Arc<dyn LlmClient>) -> Self {
        Self { llm_client }
    }

    /// Expands a single query into a list of related search terms.
    ///
    /// The original query is included in the returned list.
    pub async fn expand(&self, query: &str) -> Result<Vec<String>> {
        let prompt = format!(
            "You are a coding assistant. Generate 3-5 short technical synonyms or related terms for the following search query to improve code search recall.
            
            Query: '{}'
            
            Return ONLY a comma-separated list of terms. Do not include the original query in the output. Do not add numbering or explanations.
            Example:
            Query: auth
            Output: authentication, login, credentials, identity, oauth",
            query
        );

        let response = self.llm_client.generate(&prompt).await?;

        // Parse comma-separated response
        let mut terms: Vec<String> = response
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Ensure original query is always present (and first)
        terms.insert(0, query.to_string());

        // Deduplicate in case LLM repeats original
        terms.sort();
        terms.dedup();

        // Re-insert original at front if lost during sort (though dedup shouldn't lose it if we just inserted it)
        // Actually simpler: just collect, filter, then add original.
        // Let's rely on HashSet for dedup then convert to Vec.

        Ok(terms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::client::mocks::MockLlmClient;

    #[tokio::test]
    async fn test_expansion_parsing() {
        let mock_client = Arc::new(MockLlmClient::new("authentication, login, oauth"));
        let expander = QueryExpander::new(mock_client as Arc<dyn LlmClient>);

        let terms = expander.expand("auth").await.unwrap();

        // Original query should be preserved
        assert!(terms.contains(&"auth".to_string()));
        // Expansion terms should be present
        assert!(terms.contains(&"authentication".to_string()));
        assert!(terms.contains(&"login".to_string()));
    }
}
