use crate::bm25::BM25Index;
use crate::embedding::Embedder;
use crate::indexer::CodeChunker;
use crate::storage::Storage;
use std::fs;
use std::path::Path;
use tracing::{error, info, warn};

pub struct CodeIndexer<'a> {
    storage: &'a Storage,
    embedder: &'a mut Embedder,
    bm25: &'a mut BM25Index,
    chunker: &'a CodeChunker,
    workspace: String,
}

impl<'a> CodeIndexer<'a> {
    pub fn new(
        storage: &'a Storage,
        embedder: &'a mut Embedder,
        bm25: &'a mut BM25Index,
        chunker: &'a CodeChunker,
        workspace: String,
    ) -> Self {
        Self {
            storage,
            embedder,
            bm25,
            chunker,
            workspace,
        }
    }

    /// Indexes a single file.
    /// 1. Checks if it's a supported code file.
    /// 2. Checks modification time (deltas) if needed.
    /// 3. Chunks the file.
    /// 4. Generates embeddings.
    /// 5. Stores chunks in LanceDB and BM25.
    pub async fn index_file(&mut self, path: &Path, mtime: i64) -> anyhow::Result<()> {
        let path_lossy = path.to_string_lossy();
        let fname_str = path_lossy.to_string();

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if CodeChunker::get_language(ext).is_none() {
            return Ok(()); // Skip unsupported files silently
        }

        // Clean up old entries first
        if let Err(e) = self
            .storage
            .delete_file_chunks(&fname_str, &self.workspace)
            .await
        {
            warn!("Error deleting old chunks for {}: {}", fname_str, e);
        }
        if let Err(e) = self.bm25.delete_file(&fname_str, &self.workspace) {
            warn!("Error deleting old BM25 docs for {}: {}", fname_str, e);
        }

        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to read file {}: {}", fname_str, e);
                return Ok(());
            }
        };
        let mut reader = std::io::BufReader::new(file);

        let chunks = match self.chunker.chunk_file(&fname_str, &mut reader, mtime) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to chunk file {}: {}", fname_str, e);
                return Ok(());
            }
        };

        if chunks.is_empty() {
            return Ok(());
        }

        let texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
        let embeddings = match self.embedder.embed(texts, Some(256)) {
            Ok(e) => e,
            Err(e) => {
                error!("Error generating embeddings for {}: {}", fname_str, e);
                return Ok(());
            }
        };

        let ids: Vec<String> = chunks
            .iter()
            .map(|c| format!("{}-{}-{}", c.filename, c.line_start, c.line_end))
            .collect();
        let filenames: Vec<String> = chunks.iter().map(|c| c.filename.clone()).collect();
        let codes: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
        let starts: Vec<i32> = chunks.iter().map(|c| c.line_start as i32).collect();
        let ends: Vec<i32> = chunks.iter().map(|c| c.line_end as i32).collect();
        let mtimes: Vec<i64> = chunks.iter().map(|c| c.last_modified).collect();
        let calls: Vec<Vec<String>> = chunks.iter().map(|c| c.calls.clone()).collect();

        if let Err(e) = self
            .storage
            .add_chunks(
                &self.workspace,
                ids,
                filenames,
                codes,
                starts,
                ends,
                mtimes,
                calls,
                embeddings,
            )
            .await
        {
            error!("Error storing chunks for {}: {}", fname_str, e);
        }

        if let Err(e) = self.bm25.add_chunks(&chunks, &self.workspace) {
            error!("Error adding to BM25 for {}: {}", fname_str, e);
        }

        info!("Indexed: {}", fname_str);
        Ok(())
    }

    /// Removes a file from the index.
    pub async fn remove_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let fname_str = path.to_string_lossy().to_string();

        self.storage
            .delete_file_chunks(&fname_str, &self.workspace)
            .await?;
        self.bm25.delete_file(&fname_str, &self.workspace)?;

        info!("Removed: {}", fname_str);
        Ok(())
    }
}
