#[cfg(test)]
use crate::llm::client::mocks::MockLlmClient;
use crate::llm::QueryExpander;
use std::sync::Arc;

#[tokio::test]
async fn test_expand_query() {
    let mock_client = MockLlmClient::new("term1, term2");
    let expander = QueryExpander::new(
        Arc::new(mock_client) as Arc<dyn crate::llm::client::LlmClient + Send + Sync>
    );

    let expanded: Vec<String> = match expander.expand("query").await {
        Ok(v) => v,
        Err(e) => panic!("Expansion failed: {}", e),
    };
    assert_eq!(expanded.len(), 3); // original + 2 terms
    assert!(expanded.contains(&"query".to_string()));
    assert!(expanded.contains(&"term1".to_string()));
    assert!(expanded.contains(&"term2".to_string()));
}

#[tokio::test]
async fn test_expand_query_parse_error_fallback() {
    let mock_client = MockLlmClient::new("invalid format");
    let expander = QueryExpander::new(
        Arc::new(mock_client) as Arc<dyn crate::llm::client::LlmClient + Send + Sync>
    );

    // Should return original query + the "invalid" term (garbage in, garbage out)
    let expanded: Vec<String> = match expander.expand("query").await {
        Ok(v) => v,
        Err(e) => panic!("Expansion failed: {}", e),
    };
    assert_eq!(expanded.len(), 2);
    assert!(expanded.contains(&"query".to_string()));
    assert!(expanded.contains(&"invalid format".to_string()));
}
