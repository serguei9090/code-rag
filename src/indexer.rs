use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use tree_sitter::{Language, Node, Parser};

/// A single logical unit of code extracted from a source file.
///
/// Contains the code content along with metadata for search and context optimization.
pub struct CodeChunk {
    /// Source file path (normalized)
    pub filename: String,
    /// The actual code snippet
    pub code: String,
    /// Start line number (1-indexed)
    pub line_start: usize,
    /// End line number (1-indexed)
    pub line_end: usize,
    /// Last modified timestamp of the source file
    pub last_modified: i64,
    /// List of function/method calls identified within this chunk
    pub calls: Vec<String>,
}

/// Handles the semantic chunking of source code files using Tree-sitter.
///
/// Supports various programming languages and applies language-specific
/// heuristics to extract functional units (functions, classes, etc.).
pub struct CodeChunker {
    /// Target size for chunks in bytes (soft limit, will respect semantic boundaries)
    pub max_chunk_size: usize,
    /// Number of bytes to overlap between adjacent chunks when splitting large blocks
    pub chunk_overlap: usize,
}

impl Default for CodeChunker {
    fn default() -> Self {
        Self::new(1024, 128)
    }
}

impl CodeChunker {
    pub fn new(max_chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            max_chunk_size,
            chunk_overlap,
        }
    }

    pub fn get_language(extension: &str) -> Option<Language> {
        match extension {
            "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
            "py" => Some(tree_sitter_python::LANGUAGE.into()),
            "go" => Some(tree_sitter_go::LANGUAGE.into()),
            "c" | "h" => Some(tree_sitter_c::LANGUAGE.into()),
            "cpp" | "hpp" | "cc" | "cxx" => Some(tree_sitter_cpp::LANGUAGE.into()),
            "js" | "jsx" => Some(tree_sitter_javascript::LANGUAGE.into()),
            "ts" | "tsx" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            "java" => Some(tree_sitter_java::LANGUAGE.into()),
            "cs" => Some(tree_sitter_c_sharp::LANGUAGE.into()),
            "rb" => Some(tree_sitter_ruby::LANGUAGE.into()),
            "php" => Some(tree_sitter_php::LANGUAGE_PHP.into()),
            "html" => Some(tree_sitter_html::LANGUAGE.into()),
            "css" => Some(tree_sitter_css::LANGUAGE.into()),
            "sh" | "bash" => Some(tree_sitter_bash::LANGUAGE.into()),
            "ps1" => Some(tree_sitter_powershell::language()),
            // "dockerfile" | "Dockerfile" => Some(tree_sitter_dockerfile::language()),
            "yaml" | "yml" => Some(tree_sitter_yaml::LANGUAGE.into()),
            "json" => Some(tree_sitter_json::LANGUAGE.into()),
            "zig" => Some(tree_sitter_zig::LANGUAGE.into()),
            "ex" | "exs" => Some(tree_sitter_elixir::LANGUAGE.into()),
            "hs" => Some(tree_sitter_haskell::LANGUAGE.into()),
            "sol" => Some(tree_sitter_solidity::LANGUAGE.into()),
            _ => None,
        }
    }

    pub fn chunk_file<R: Read + Seek>(
        &self,
        filename: &str,
        reader: &mut R,
        mtime: i64,
    ) -> std::io::Result<Vec<CodeChunk>> {
        let normalized_filename = filename.replace("\\", "/");
        let path = Path::new(&normalized_filename);
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        let language = match Self::get_language(ext) {
            Some(l) => l,
            None => return Ok(vec![]),
        };

        let mut parser = Parser::new();
        if parser.set_language(&language).is_err() {
            tracing::error!("Could not set language for extension: {}", ext);
            return Ok(vec![]);
        }

        // Check for binary content
        let mut check_buf = [0u8; 1024];
        let bytes_read = reader.read(&mut check_buf)?;
        reader.seek(SeekFrom::Start(0))?;

        if check_buf[..bytes_read].contains(&0) {
            tracing::debug!("Skipping binary file: {}", filename);
            return Ok(vec![]);
        }

        let mut chunks = Vec::new();

        // Use a buffer for tree-sitter callback
        // We need to return an owned slice-like object (Vec<u8> works)
        let mut buffer = Vec::new();

        let tree = parser.parse_with(
            &mut |byte_offset, _position| {
                if reader.seek(SeekFrom::Start(byte_offset as u64)).is_err() {
                    return Vec::new();
                }
                // Read a chunk (e.g. 8KB) to minimize small reads
                buffer.resize(8192, 0);
                match reader.read(&mut buffer) {
                    Ok(0) => Vec::new(),
                    Ok(n) => {
                        buffer.truncate(n);
                        buffer.clone()
                    }
                    Err(_) => Vec::new(),
                }
            },
            None,
        );

        let tree = match tree {
            Some(t) => t,
            None => return Ok(vec![]),
        };

        let root = tree.root_node();

        self.traverse(
            &root,
            reader,
            &normalized_filename,
            &mut chunks,
            ext,
            mtime,
            0,
        )?;

        Ok(chunks)
    }

    #[allow(clippy::too_many_arguments)]
    fn traverse<R: Read + Seek>(
        &self,
        node: &Node,
        reader: &mut R,
        filename: &str,
        chunks: &mut Vec<CodeChunk>,
        ext: &str,
        mtime: i64,
        depth: usize,
    ) -> std::io::Result<()> {
        let kind = node.kind();

        let is_script_lang = matches!(
            ext,
            "py" | "js" | "ts" | "jsx" | "tsx" | "rb" | "lua" | "sh" | "bash" | "ps1"
        );

        let is_semantic_chunk = matches!(
            kind,
            // Rust
            "function_item" | "impl_item" | "struct_item" | "enum_item" | "mod_item" | "const_item" | "static_item" |
            // Python, C/C++, generic, Bash, PS1
             "function_definition" | "class_definition" | "function_statement" |
            // Go
             "function_declaration" | "method_declaration" | "type_declaration" |
            // C/C++ specific
             "class_specifier" | "struct_specifier" |
            // JS/TS
             "class_declaration" | "method_definition" | "arrow_function" |
            // Java/C#
             "interface_declaration" | "record_declaration" |
            // Ruby
             "method" | "class" |
            // HTML
             "script_element" | "style_element" |
            // CSS
             "rule_set" | "media_statement" | "keyframes_statement" |
            // PowerShell extras
             "param_block" |
            // YAML / JSON
             "block_mapping_pair" | "pair" | "object" |
            // Zig
             "Decls" | "FnProto" | "ContainerField" |
            // Elixir
             "call" | "do_block" |
            // Haskell
             "signature" | "function" |
            // Solidity
             "contract_declaration" | "interface_definition" | "library_definition"
        );

        let is_ruby_module = ext == "rb" && kind == "module";

        // Scripts logic: chunk top-level logic, but avoid noise
        // Depth 1 means direct child of the root module/program
        // PS1 has a statement_list at depth 1, so top-level logic is at depth 2
        let is_script_chunk = is_script_lang
            && (depth == 1 || (ext == "ps1" && depth == 2))
            && matches!(
                kind,
                "if_statement"
                    | "flow_statement"
                    | "expression_statement"
                    | "assignment"
                    | "variable_declaration"
                    | "lexical_declaration"
                    | "function_call"
                    | "call"
                    | "command"
                    | "pipeline"
                    | "if_expression"
                    | "for_expression" // Bash/PS1 extras
            );

        let is_chunkable = is_semantic_chunk || is_ruby_module || is_script_chunk;

        if is_chunkable {
            // Restore debug printing for S-expressions
            tracing::trace!(
                "Processing chunks for node kind: {}, range: {}-{}",
                kind,
                node.start_byte(),
                node.end_byte()
            );

            let start_byte = node.start_byte();
            let end_byte = node.end_byte();

            // Read content from file/reader
            reader.seek(SeekFrom::Start(start_byte as u64))?;
            let len = end_byte.saturating_sub(start_byte);
            // Safety check: Prevent OOM on extremely large single nodes (e.g. > 10MB)
            // If a single semantic node is that large, it's likely not useful for embedding anyway.
            if len > 10 * 1024 * 1024 {
                tracing::warn!(
                    "Node too large to chunk in {} (size: {} bytes). Skipping.",
                    filename,
                    len
                );
                return Ok(());
            }

            if len > 0 {
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf)?;
                let chunk_content = String::from_utf8_lossy(&buf).to_string();

                let start_position = node.start_position();
                let end_position = node.end_position();

                // Extract calls
                let calls = self.find_calls(node, reader)?;

                if chunk_content.len() > self.max_chunk_size {
                    let sub_chunks = self.split_text(&chunk_content);
                    for sub_code in sub_chunks {
                        chunks.push(CodeChunk {
                            filename: filename.to_string(),
                            code: sub_code,
                            line_start: start_position.row + 1,
                            line_end: end_position.row + 1,
                            last_modified: mtime,
                            calls: calls.clone(),
                        });
                    }
                } else {
                    chunks.push(CodeChunk {
                        filename: filename.to_string(),
                        code: chunk_content,
                        line_start: start_position.row + 1,
                        line_end: end_position.row + 1,
                        last_modified: mtime,
                        calls,
                    });
                }

                let is_container = kind.contains("class")
                    || kind.contains("impl")
                    || kind.contains("struct")
                    || kind == "element"
                    || kind == "stylesheet";
                if !is_container {
                    return Ok(());
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse(&child, reader, filename, chunks, ext, mtime, depth + 1)?;
        }

        Ok(())
    }

    fn find_calls<R: Read + Seek>(
        &self,
        node: &Node,
        reader: &mut R,
    ) -> std::io::Result<Vec<String>> {
        let mut calls = Vec::new();
        let mut cursor = node.walk();

        // Simple recursive search looking for call-like nodes
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "call_expression" | "call" | "macro_invocation") {
                // Try to get identifier
                if let Some(name) = self.extract_name(&child, reader)? {
                    calls.push(name);
                }
            }
            // Recurse
            calls.extend(self.find_calls(&child, reader)?);
        }
        Ok(calls)
    }

    fn extract_name<R: Read + Seek>(
        &self,
        node: &Node,
        reader: &mut R,
    ) -> std::io::Result<Option<String>> {
        // Heuristic: finding the 'function' or 'identifier' child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let k = child.kind();
            if matches!(k, "identifier" | "field_expression" | "scoped_identifier") {
                let start = child.start_byte();
                let end = child.end_byte();

                reader.seek(SeekFrom::Start(start as u64))?;
                let len = end.saturating_sub(start);
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf)?;

                return Ok(Some(String::from_utf8_lossy(&buf).to_string()));
            }
        }
        Ok(None)
    }

    fn split_text(&self, text: &str) -> Vec<String> {
        if text.len() <= self.max_chunk_size {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let total_chars = chars.len();
        let mut start = 0;

        while start < total_chars {
            let end = std::cmp::min(start + self.max_chunk_size, total_chars);
            let s: String = chars[start..end].iter().collect();
            chunks.push(s);

            if end == total_chars {
                break;
            }

            // Ensure we move forward and respect overlap
            let step = if self.max_chunk_size > self.chunk_overlap {
                self.max_chunk_size - self.chunk_overlap
            } else {
                1
            };
            start += step;
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_chunk_overlap() {
        let chunker = CodeChunker::new(10, 2);
        let text = "1234567890EXTRA"; // 15 chars
        let chunks = chunker.split_text(text);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "1234567890");
        assert_eq!(chunks[1], "90EXTRA");
        assert!(chunks[1].starts_with("90"));
    }

    #[test]
    fn test_chunk_file_streaming() {
        let chunker = CodeChunker::default();
        let code = "fn main() { println!(\"Hello\"); }";
        let mut cursor = Cursor::new(code);

        let chunks = chunker.chunk_file("test.rs", &mut cursor, 0).unwrap();
        // Should find main function
        assert!(!chunks.is_empty());
        assert!(chunks.iter().any(|c| c.code.contains("fn main")));
    }

    #[test]
    fn test_exact_size_limit() {
        let chunker = CodeChunker::new(5, 0);
        let text = "1234567890";
        let chunks = chunker.split_text(text);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "12345");
        assert_eq!(chunks[1], "67890");
    }

    #[test]
    fn test_small_text_no_split() {
        let chunker = CodeChunker::new(100, 10);
        let text = "Short text";
        let chunks = chunker.split_text(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Short text");
    }

    #[test]
    fn test_binary_file_skip() {
        let chunker = CodeChunker::default();
        // Construct binary content with null bytes
        let binary_content = vec![0x00, 0xFF, 0xFE, 0x00, 0x41];
        let mut cursor = Cursor::new(binary_content);

        let chunks = chunker.chunk_file("test.rs", &mut cursor, 0).unwrap();
        assert!(
            chunks.is_empty(),
            "Binary file should be skipped even if extension matches"
        );
    }
}
