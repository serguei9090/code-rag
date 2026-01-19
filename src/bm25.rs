use crate::indexer::CodeChunk;

use anyhow::{anyhow, Result};

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term};

/// Full-text search index using the BM25 ranking algorithm.
///
/// Provides efficient keyword-based search over code chunks with workspace isolation.
/// Uses Tantivy for the underlying inverted index implementation with configurable
/// merge policies for optimizing read vs write performance.
///
/// # Examples
///
/// ```no_run
/// use code_rag::bm25::BM25Index;
///
/// # fn main() -> anyhow::Result<()> {
/// let index = BM25Index::new("./bm25_db", false, "log")?;
/// let results = index.search("authentication", 10, Some("workspace1"))?;
/// # Ok(())
/// # }
/// ```
pub struct BM25Index {
    index: Index,
    reader: IndexReader,
    writer: Option<Arc<Mutex<IndexWriter>>>,
    schema: Schema,
    id_field: Field,
    filename_field: Field,
    code_field: Field,
    line_start_field: Field,
    line_end_field: Field,
    workspace_field: Field,
}

/// A single search result from the BM25 index.
///
/// Contains the matched code chunk with its file location and relevance score.
#[derive(Debug, Clone)]
pub struct BM25Result {
    /// Unique identifier for this code chunk
    pub id: String,
    /// Source file path
    pub filename: String,
    /// The actual code content
    pub code: String,
    /// Starting line number
    pub line_start: u64,
    /// Ending line number
    pub line_end: u64,
    /// BM25 relevance score (higher is better)
    pub score: f32,
}

impl BM25Index {
    /// Creates a new BM25 index.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Base directory for index storage
    /// * `readonly` - If true, index is read-only (no writer created)
    /// * `merge_policy_type` - Merge policy: "log", "fast-write", or "fast-search"
    pub fn new(db_path: &str, readonly: bool, merge_policy_type: &str) -> Result<Self> {
        let index_path = Path::new(db_path).join("bm25_index");
        if !index_path.exists() {
            fs::create_dir_all(&index_path)?;
        }

        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("id", STRING | STORED); // Unique ID
        schema_builder.add_text_field("filename", STRING | STORED); // Filename
        schema_builder.add_text_field("code", TEXT | STORED);
        schema_builder.add_u64_field("line_start", STORED);
        schema_builder.add_u64_field("line_end", STORED);
        schema_builder.add_text_field("workspace", STRING | STORED); // Workspace isolation

        let schema = schema_builder.build();

        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(&index_path)?,
            schema.clone(),
        )?;

        let writer = if readonly {
            None
        } else {
            match index.writer(200_000_000) {
                Ok(w) => {
                    // Apply Merge Policy
                    match merge_policy_type {
                        "log" | "fast-write" => {
                            let mut policy = tantivy::merge_policy::LogMergePolicy::default();
                            if merge_policy_type == "fast-write" {
                                policy.set_min_layer_size(10);
                            } else {
                                policy.set_min_layer_size(8);
                            }
                            w.set_merge_policy(Box::new(policy));
                        }
                        "fast-search" => {
                            // Default behavior
                        }
                        _ => {
                            let mut policy = tantivy::merge_policy::LogMergePolicy::default();
                            policy.set_min_layer_size(8);
                            w.set_merge_policy(Box::new(policy));
                        }
                    }
                    Some(Arc::new(Mutex::new(w)))
                }
                Err(e) => {
                    return Err(anyhow!(e));
                }
            }
        };

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;

        let id_field = schema.get_field("id")?;
        let filename_field = schema.get_field("filename")?;
        let code_field = schema.get_field("code")?;
        let line_start_field = schema.get_field("line_start")?;
        let line_end_field = schema.get_field("line_end")?;
        let workspace_field = schema.get_field("workspace")?;

        Ok(Self {
            index,
            reader,
            writer,
            schema,
            id_field,
            filename_field,
            code_field,
            line_start_field,
            line_end_field,
            workspace_field,
        })
    }

    /// Indexes code chunks with workspace isolation.
    ///
    /// Deletes existing chunks with the same ID to prevent duplicates.
    ///
    /// **Note**: This method does NOT commit changes. Caller must call `commit()` when done.
    pub fn add_chunks(&self, chunks: &[CodeChunk], workspace: &str) -> Result<()> {
        let writer_arc = self
            .writer
            .as_ref()
            .ok_or(anyhow::anyhow!("Index is read-only"))?;
        let mut writer = writer_arc
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        let id_field = self.id_field;
        let filename_field = self.filename_field;
        let code_field = self.code_field;
        let line_start_field = self.line_start_field;
        let line_end_field = self.line_end_field;
        let workspace_field = self.workspace_field;

        for chunk in chunks {
            let chunk_id = format!("{}-{}-{}", chunk.filename, chunk.line_start, chunk.line_end);

            // Delete existing document with this ID to prevent duplicates (though upstream logic might handle this)
            // tantivy delete is term-based.
            let _ = writer.delete_term(Term::from_field_text(id_field, &chunk_id));

            let mut doc = TantivyDocument::default();

            doc.add_text(id_field, &chunk_id);
            doc.add_text(filename_field, &chunk.filename);
            doc.add_text(code_field, &chunk.code);
            doc.add_u64(line_start_field, chunk.line_start as u64);
            doc.add_u64(line_end_field, chunk.line_end as u64);
            doc.add_text(workspace_field, workspace);

            writer.add_document(doc)?;
        }

        // Commit removed for performance - caller must call commit() explicitly
        Ok(())
    }

    /// Deletes all indexed chunks from a specific file.
    ///
    /// **Note**: This method does NOT commit changes. Caller must call `commit()` when done.
    ///
    /// # Errors
    ///
    /// Returns an error if the index is read-only or if the deletion query fails.
    pub fn delete_file(&self, filename: &str, workspace: &str) -> Result<()> {
        let writer_arc = self
            .writer
            .as_ref()
            .ok_or(anyhow::anyhow!("Index is read-only"))?;
        let mut writer = writer_arc
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        let filename_field = self.filename_field;
        let workspace_field = self.workspace_field;

        let filename_term = Term::from_field_text(filename_field, filename);
        let workspace_term = Term::from_field_text(workspace_field, workspace);

        let query = tantivy::query::BooleanQuery::new(vec![
            (
                tantivy::query::Occur::Must,
                Box::new(tantivy::query::TermQuery::new(
                    filename_term,
                    IndexRecordOption::Basic,
                )),
            ),
            (
                tantivy::query::Occur::Must,
                Box::new(tantivy::query::TermQuery::new(
                    workspace_term,
                    IndexRecordOption::Basic,
                )),
            ),
        ]);

        writer.delete_query(Box::new(query))?;
        // Commit removed for performance - caller must call commit() explicitly
        Ok(())
    }

    /// Commits all pending write operations to disk.
    ///
    /// This is an expensive I/O operation that flushes the entire write buffer.
    /// Should only be called once at the end of a batch indexing operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the index is read-only or if the commit fails.
    pub fn commit(&self) -> Result<()> {
        let writer_arc = self
            .writer
            .as_ref()
            .ok_or(anyhow::anyhow!("Index is read-only"))?;
        let mut writer = writer_arc
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        writer.commit()?;
        Ok(())
    }

    /// Returns a lightweight searcher for concurrent read access.
    ///
    /// Multiple threads can call this simultaneously without blocking.
    /// Each searcher is independent and can be used in parallel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use code_rag::bm25::BM25Index;
    /// # fn main() -> anyhow::Result<()> {
    /// let index = BM25Index::new("./db", true, "log")?;
    /// let searcher = index.get_searcher();
    /// // Use searcher for queries (thread-safe)
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_searcher(&self) -> tantivy::Searcher {
        self.reader.searcher()
    }

    /// Reloads the index reader to see newly committed data.
    ///
    /// Call this after committing new documents to make them visible
    /// to subsequent searcher instances.
    ///
    /// # Errors
    ///
    /// Returns an error if the reload operation fails.
    pub fn reload(&self) -> Result<()> {
        self.reader.reload()?;
        Ok(())
    }

    /// Searches the index using BM25 ranking.
    ///
    /// # Arguments
    ///
    /// * `query_str` - Search query
    /// * `limit` - Maximum number of results  
    /// * `workspace` - Optional workspace filter for isolation
    pub fn search(
        &self,
        query_str: &str,
        limit: usize,
        workspace: Option<&str>,
    ) -> Result<Vec<BM25Result>> {
        let searcher = self.reader.searcher();
        let id_field = self.id_field;
        let filename_field = self.filename_field;
        let code_field = self.code_field;
        let line_start_field = self.line_start_field;
        let line_end_field = self.line_end_field;
        let workspace_field = self.workspace_field;

        let query_parser = QueryParser::for_index(&self.index, vec![code_field, filename_field]);
        let mut query = query_parser.parse_query(query_str)?;

        if let Some(ws) = workspace {
            let term = Term::from_field_text(workspace_field, ws);
            let term_query = tantivy::query::TermQuery::new(term, IndexRecordOption::Basic);
            query = Box::new(tantivy::query::BooleanQuery::new(vec![
                (tantivy::query::Occur::Must, query),
                (tantivy::query::Occur::Must, Box::new(term_query)),
            ]));
        }

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            let id = retrieved_doc
                .get_first(id_field)
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing or invalid 'id' field in document"))?
                .to_string();
            let filename = retrieved_doc
                .get_first(filename_field)
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing or invalid 'filename' field in document"))?
                .to_string();
            let code = retrieved_doc
                .get_first(code_field)
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing or invalid 'code' field in document"))?
                .to_string();
            let line_start = retrieved_doc
                .get_first(line_start_field)
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow!("Missing or invalid 'line_start' field in document"))?;
            let line_end = retrieved_doc
                .get_first(line_end_field)
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow!("Missing or invalid 'line_end' field in document"))?;

            results.push(BM25Result {
                id,
                filename,
                code,
                line_start,
                line_end,
                score,
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::CodeChunk;
    use tempfile::TempDir;

    fn setup_test_index() -> (BM25Index, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().to_str().unwrap();
        let index = BM25Index::new(db_path, false, "log").expect("Failed to create index");
        (index, temp_dir)
    }

    #[test]
    fn test_initialization() {
        let (_index, temp_dir) = setup_test_index();
        let index_path = temp_dir.path().join("bm25_index");
        assert!(index_path.exists());
    }

    #[test]
    fn test_indexing_and_search() {
        let (index, _temp_dir) = setup_test_index();

        let chunks = vec![
            CodeChunk {
                filename: "test.rs".to_string(),
                code: "fn test_func() { println!(\"Hello\"); }".to_string(),
                line_start: 1,
                line_end: 3,
                last_modified: 0,
                calls: vec![],
            },
            CodeChunk {
                filename: "test.py".to_string(),
                code: "def test_func(): print(\"Hello\")".to_string(),
                line_start: 1,
                line_end: 2,
                last_modified: 0,
                calls: vec![],
            },
        ];

        index
            .add_chunks(&chunks, "default")
            .expect("Failed to add chunks");
        index.reader.reload().expect("Failed to reload");

        let results = index
            .search("test_func", 10, Some("default"))
            .expect("Search failed");

        // With Manual policy, we must reload.
        assert!(!results.is_empty(), "Should find results after indexing");
        assert!(results.iter().any(|r| r.filename == "test.rs"));
        assert!(results.iter().any(|r| r.filename == "test.py"));
    }

    #[test]
    fn test_deletion() {
        let (index, _temp_dir) = setup_test_index();

        let chunks = vec![CodeChunk {
            filename: "delete_me.rs".to_string(),
            code: "fn delete_me() {}".to_string(),
            line_start: 1,
            line_end: 3,
            last_modified: 0,
            calls: vec![],
        }];
        index
            .add_chunks(&chunks, "default")
            .expect("Failed to add chunks");
        index.reader.reload().expect("Failed to reload");

        let results = index
            .search("delete_me", 10, Some("default"))
            .expect("Search failed");
        assert_eq!(results.len(), 1);

        index
            .delete_file("delete_me.rs", "default")
            .expect("Failed to delete");
        index.reader.reload().expect("Failed to reload");

        let results_after = index
            .search("delete_me", 10, Some("default"))
            .expect("Search failed");
        assert!(
            results_after.is_empty(),
            "Should have deleted file contents"
        );
    }
}
