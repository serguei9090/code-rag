use code_rag::context::ContextOptimizer;
use code_rag::search::SearchResult;

#[test]
fn test_context_optimizer_merging() {
    let results = vec![
        SearchResult {
            rank: 1,
            score: 0.9,
            filename: "test.rs".to_string(),
            code: "line1\nline2\n".to_string(),
            line_start: 10,
            line_end: 11,
            calls: vec![],
        },
        // Lines 12-13
        SearchResult {
            rank: 2,
            score: 0.85,
            filename: "test.rs".to_string(),
            code: "line3\nline4\n".to_string(),
            line_start: 12, // Adjacent to 11
            line_end: 13,
            calls: vec![],
        },
        // Another file
        SearchResult {
            rank: 3,
            score: 0.80,
            filename: "other.rs".to_string(),
            code: "other code\n".to_string(),
            line_start: 100,
            line_end: 101,
            calls: vec![],
        },
    ];

    let optimizer = ContextOptimizer::new(1000);
    let merged = optimizer.optimize(results).expect("Optimization failed");

    // Expecting 2 chunks: 1 merged for test.rs, 1 for other.rs
    assert_eq!(merged.len(), 2);

    // Check merged chunk
    let merged_chunk = merged.iter().find(|c| c.filename == "test.rs").unwrap();
    assert_eq!(merged_chunk.start_line, 10);
    assert_eq!(merged_chunk.end_line, 13);
    // Code should be joined with newline potentially?
    // Wait, implementation details of joining:
    // If adjacent, we join.
    // "line1\nline2\n" + "line3\nline4\n"
    assert!(merged_chunk.code.contains("line1"));
    assert!(merged_chunk.code.contains("line4"));
}

#[test]
fn test_context_optimizer_budgeting() {
    let mut results = vec![];

    // Create many small chunks
    for i in 0..10 {
        results.push(SearchResult {
            rank: i + 1,
            score: 1.0 - (i as f32 * 0.01),
            filename: format!("file{}.rs", i),
            code: "some tokens here".to_string(),
            line_start: 1,
            line_end: 2,
            calls: vec![],
        });
    }

    // Set a very small budget that can't fit all
    // say 10 tokens per chunk roughly? "some tokens here" is 3 words.
    // If we set limit to 10 tokens, we might get 1 or 2 chunks.
    let optimizer = ContextOptimizer::new(10);
    let optimized = optimizer.optimize(results).expect("Optimization failed");

    assert!(optimized.len() < 10);
    assert!(!optimized.is_empty());
}
