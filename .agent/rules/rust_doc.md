---
trigger: glob
globs: *rs
---

# Rust Coding Standards & Documentation Rules

This document defines the mandatory coding standards for the `code-rag` Rust project.

## 📐 Code Formatting & Structure

### Indentation & Spacing
- **Indentation**: 4 spaces (enforced by `rustfmt`)
- **Line Width**: Maximum 100 characters
- **Trailing Commas**: Required in multi-line items
- **Import Groups**: Separate std, external crates, and internal modules with blank lines

```rust
// ✅ CORRECT
use std::collections::HashMap;
use std::error::Error;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::embedding::Embedder;
use crate::storage::Storage;
```

### Braces & Punctuation
- **Opening Brace**: Same line as declaration
- **Closing Brace**: Own line, aligned with declaration
- **Semicolons**: Required after statements (not expressions)

```rust
// ✅ CORRECT
fn example() -> Result<()> {
    let x = if condition {
        5
    } else {
        10
    };
    Ok(())
}

// ❌ WRONG
fn example() -> Result<()> 
{
    let x = if condition 
    {
        5
    } 
    else 
    {
        10
    };
    Ok(())
}
```

### Naming Conventions
- **Types** (Struct, Enum, Trait): `UpperCamelCase`
- **Functions, Methods, Variables**: `snake_case`
- **Constants, Statics**: `SCREAMING_SNAKE_CASE`
- **Type Parameters**: Single uppercase letter or `UpperCamelCase`

```rust
// ✅ CORRECT
pub struct CodeSearcher { /* ... */ }
pub enum SearchMode { /* ... */ }
const MAX_RETRIES: u32 = 3;
fn semantic_search() { /* ... */ }

// ❌ WRONG
pub struct code_searcher { /* ... */ }  // Wrong case
const maxRetries: u32 = 3;             // Wrong case
fn SemanticSearch() { /* ... */ }      // Wrong case
```

## 📝 Documentation Requirements

### Public Item Documentation
**ALL** public items (`pub`) MUST have documentation comments (`///`).

```rust
/// Performs semantic search over indexed code chunks.
///
/// This function uses vector embeddings to find semantically similar code
/// regardless of exact keyword matches.
///
/// # Arguments
///
/// * `query` - The search query string
/// * `limit` - Maximum number of results to return
/// * `ext` - Optional file extension filter (e.g., "rs", "py")
/// * `dir` - Optional directory filter
/// * `no_rerank` - Skip reranking for faster (but less accurate) results
///
/// # Returns
///
/// Returns a `Vec<SearchResult>` ordered by relevance score (highest first).
///
/// # Errors
///
/// Returns an error if:
/// - Storage is not initialized
/// - Embedder fails to generate query embedding
/// - Database query fails
///
/// # Examples
///
/// ```
/// let mut searcher = CodeSearcher::new(Some(storage), Some(embedder), None);
/// let results = searcher
///     .semantic_search("authentication logic", 10, None, None, false)
///     .await?;
/// ```
pub async fn semantic_search(
    &mut self,
    query: &str,
    limit: usize,
    ext: Option<String>,
    dir: Option<String>,
    no_rerank: bool,
) -> Result<Vec<SearchResult>> {
    // Implementation...
}
```

### Module Documentation
Add module-level documentation at the top of each file:

```rust
//! Code search functionality using hybrid BM25 + vector search.
//!
//! This module implements semantic code search combining traditional
//! keyword-based search (BM25) with modern vector embeddings for
//! improved relevance ranking.

use crate::embedding::Embedder;
// ...
```

### Inline Comments
- **Why**, not **what**: Explain non-obvious logic, not obvious code
- **TODO/FIXME**: Must reference an issue or task number
- **SAFETY**: Required for every `unsafe` block

```rust
// ✅ CORRECT
// Batch embeddings to avoid OOM with large codebases
if chunks_buffer.len() >= 256 {
    // Process batch...
}

// ❌ WRONG
// Check if buffer is greater than or equal to 256
if chunks_buffer.len() >= 256 {
    // ...
}
```

## 🛡️ Error Handling

### Library Code (src/*.rs)
**NEVER** use `.unwrap()` or `.expect()` in library code.

```rust
// ✅ CORRECT
pub fn process_file(path: &str) -> Result<Vec<Chunk>> {
    let content = fs::read_to_string(path)
        .context("Failed to read file")?;
    Ok(parse_content(&content))
}

// ❌ WRONG
pub fn process_file(path: &str) -> Vec<Chunk> {
    let content = fs::read_to_string(path)
        .expect("Failed to read file");  // FORBIDDEN!
    parse_content(&content)
}
```

### Application Code (main.rs, bin/*.rs)
May use `.expect()` with descriptive messages, but `Result` is preferred.

```rust
// ✅ ACCEPTABLE (but Result is better)
let config = load_config()
    .expect("Configuration file is required");

// ✅ BETTER
fn main() -> anyhow::Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;
    Ok(())
}
```

### Error Types
- **Applications**: Use `anyhow::Result<T>`
- **Libraries**: Use `Result<T, Box<dyn Error>>` or custom error types with `thiserror`

## 🧪 Testing Standards

### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### Test Naming
- Prefix with `test_`
- Use descriptive names: `test_search_with_extension_filter_returns_only_matching_files`
- One assertion concept per test

### Integration Tests
Place in `tests/*.rs` (not `src/`):
```rust
// tests/integration_tests.rs
use code_rag::search::CodeSearcher;

#[tokio::test]
async fn test_end_to_end_search_workflow() {
    // Full workflow test...
}
```

## 🔒 Safety & Security

### Unsafe Code
```rust
// ✅ CORRECT (only if absolutely necessary)
unsafe {
    // SAFETY: Pointer is guaranteed to be valid because we just allocated it
    // on line X and haven't moved ownership. The lifetime is bounded by
    // the enclosing scope which holds the allocation.
    *ptr = value;
}

// ❌ WRONG
unsafe {
    *ptr = value;  // No SAFETY comment!
}
```

### Input Validation
Always validate user input before processing:
```rust
pub fn search_with_limit(query: &str, limit: usize) -> Result<Vec<Result>> {
    // Validate inputs
    if query.trim().is_empty() {
        return Err(anyhow!("Query cannot be empty"));
    }
    if limit == 0 || limit > 1000 {
        return Err(anyhow!("Limit must be between 1 and 1000"));
    }
    
    // Process...
}
```

## 🎯 Best Practices Checklist

Before submitting code, verify:

- [ ] All `pub` items have `///` documentation
- [ ] Examples in documentation compile and work
- [ ] No `.unwrap()` or `.expect()` in library code
- [ ] All errors return `Result<T, E>`
- [ ] Input validation for all public functions
- [ ] No `unsafe` without `// SAFETY:` comment
- [ ] `cargo fmt` applied
- [ ] `cargo clippy` shows zero warnings
- [ ] Tests added for new functionality
- [ ] Integration tests pass

## 📚 Additional Resources

### Required Reading
1. **MEMORY[rust_bp.md]** - Comprehensive Rust best practices (MUST READ FIRST)
2. [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
3. [Rust Style Guide](https://doc.rust-lang.org/beta/style-guide/)

### Project-Specific Guidelines
- See `MEMORY[project_setup.md]` for architecture decisions
- See `docs/architecture.md` for system design
- See existing code for established patterns

---

**Remember**: These rules are not optional. They ensure code quality, maintainability, and safety.
