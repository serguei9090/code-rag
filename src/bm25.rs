use crate::indexer::CodeChunk;

use anyhow::Result;

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument};

pub struct BM25Index {
    index: Index,
    reader: IndexReader,
    writer: Option<Arc<Mutex<IndexWriter>>>,
    schema: Schema,
}

#[derive(Debug, Clone)]
pub struct BM25Result {
    pub id: String,
    pub filename: String,
    pub code: String,
    pub line_start: u64,
    pub line_end: u64,
    pub score: f32,
}

impl BM25Index {
    pub fn new(db_path: &str, readonly: bool) -> Result<Self> {
        let index_path = Path::new(db_path).join("bm25_index");
        if !index_path.exists() {
            fs::create_dir_all(&index_path)?;
        }

        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("id", STRING | STORED); // Unique ID
        schema_builder.add_text_field("filename", STRING | STORED); // Filename (not tokenized for search, but for retrieving)
                                                                    // We use TEXT for code to allow full-text search.
                                                                    // We might want to use a specific tokenizer for code if desired, but standard is fine for now.
        schema_builder.add_text_field("code", TEXT | STORED);
        schema_builder.add_u64_field("line_start", STORED);
        schema_builder.add_u64_field("line_end", STORED);

        let schema = schema_builder.build();

        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(&index_path)?,
            schema.clone(),
        )?;

        let writer = if readonly {
            None
        } else {
            match index.writer(50_000_000) {
                Ok(w) => Some(Arc::new(Mutex::new(w))),
                Err(e) => {
                    // If we can't open writer (e.g. locked by another process), we can warn or fail.
                    // For the 'Index' and 'Watch' commands, this should fail.
                    // But if this was an opportunistic write, we could just be None.
                    // Since 'readonly=false' implies intent to write, we should propagate error.
                    return Err(anyhow::anyhow!(e));
                }
            }
        };

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;

        Ok(Self {
            index,
            reader,
            writer,
            schema,
        })
    }

    pub fn add_chunks(&self, chunks: &[CodeChunk]) -> Result<()> {
        let writer_arc = self
            .writer
            .as_ref()
            .ok_or(anyhow::anyhow!("Index is read-only"))?;
        let mut writer = writer_arc
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        let id_field = self.schema.get_field("id").expect("Schema invalid");
        let filename_field = self.schema.get_field("filename").expect("Schema invalid");
        let code_field = self.schema.get_field("code").expect("Schema invalid");
        let line_start_field = self.schema.get_field("line_start").expect("Schema invalid");
        let line_end_field = self.schema.get_field("line_end").expect("Schema invalid");

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

            writer.add_document(doc)?;
        }

        writer.commit()?;
        Ok(())
    }

    pub fn delete_file(&self, filename: &str) -> Result<()> {
        let writer_arc = self
            .writer
            .as_ref()
            .ok_or(anyhow::anyhow!("Index is read-only"))?;
        let mut writer = writer_arc
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let filename_field = self.schema.get_field("filename").expect("Schema invalid");
        // Since 'filename' is STRING (not analyzed), we can delete by exact match term
        writer.delete_term(Term::from_field_text(filename_field, filename));
        writer.commit()?;
        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<BM25Result>> {
        let searcher = self.reader.searcher();
        let code_field = self.schema.get_field("code").expect("Schema invalid");
        let filename_field = self.schema.get_field("filename").expect("Schema invalid"); // searching filenames too?
        let id_field = self.schema.get_field("id").expect("Schema invalid");
        let line_start_field = self.schema.get_field("line_start").expect("Schema invalid");
        let line_end_field = self.schema.get_field("line_end").expect("Schema invalid");

        let query_parser = QueryParser::for_index(&self.index, vec![code_field, filename_field]);
        let query = query_parser.parse_query(query_str)?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            let id = retrieved_doc
                .get_first(id_field)
                .and_then(|v| match v {
                    OwnedValue::Str(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or_default()
                .to_string();
            let filename = retrieved_doc
                .get_first(filename_field)
                .and_then(|v| match v {
                    OwnedValue::Str(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or_default()
                .to_string();
            let code = retrieved_doc
                .get_first(code_field)
                .and_then(|v| match v {
                    OwnedValue::Str(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or_default()
                .to_string();
            let line_start = retrieved_doc
                .get_first(line_start_field)
                .and_then(|v| match v {
                    OwnedValue::U64(n) => Some(*n),
                    _ => None,
                })
                .unwrap_or_default();
            let line_end = retrieved_doc
                .get_first(line_end_field)
                .and_then(|v| match v {
                    OwnedValue::U64(n) => Some(*n),
                    _ => None,
                })
                .unwrap_or_default();

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
        let index = BM25Index::new(db_path, false).expect("Failed to create index");
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

        index.add_chunks(&chunks).expect("Failed to add chunks");
        index.reader.reload().expect("Failed to reload");

        let results = index.search("test_func", 10).expect("Search failed");

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
        index.add_chunks(&chunks).expect("Failed to add chunks");
        index.reader.reload().expect("Failed to reload");

        let results = index.search("delete_me", 10).expect("Search failed");
        assert_eq!(results.len(), 1);

        index.delete_file("delete_me.rs").expect("Failed to delete");
        index.reader.reload().expect("Failed to reload");

        let results_after = index.search("delete_me", 10).expect("Search failed");
        assert!(
            results_after.is_empty(),
            "Should have deleted file contents"
        );
    }
}
