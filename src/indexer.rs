use std::path::Path;
use tree_sitter::{Parser, Language, Node};

pub struct CodeChunk {
    pub filename: String,
    pub code: String,
    pub line_start: usize,
    pub line_end: usize,
    pub last_modified: i64,
}

pub struct CodeChunker {}

impl CodeChunker {
    pub fn new() -> Self {
        Self {}
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
            _ => None,
        }
    }

    pub fn chunk_file(&self, filename: &str, code: &str, mtime: i64) -> Vec<CodeChunk> {
        let path = Path::new(filename);
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        
        let language = match Self::get_language(ext) {
            Some(l) => l,
            None => return vec![], 
        };

        let mut parser = Parser::new();
        if parser.set_language(&language).is_err() {
             return vec![];
        }

        let tree = match parser.parse(code, None) {
             Some(t) => t,
             None => return vec![],
        };

        let mut chunks = Vec::new();
        let root = tree.root_node();
        
        self.traverse(&root, code, filename, &mut chunks, ext, mtime);
        
        chunks
    }

    fn traverse(&self, node: &Node, code: &str, filename: &str, chunks: &mut Vec<CodeChunk>, _ext: &str, mtime: i64) {
        let kind = node.kind();
        
        // Consolidated match arms to avoid "unreachable pattern" warnings
        // We list all chunkable types from all supported languages here
        let is_chunkable = matches!(kind,
            // Rust
             "function_item" | "impl_item" | "struct_item" | "enum_item" | "mod_item" |
            // Python, C/C++, generic
             "function_definition" | "class_definition" |
            // Go
             "function_declaration" | "method_declaration" | "type_declaration" |
            // C/C++ specific
             "class_specifier" | "struct_specifier" |
            // JS/TS
             "class_declaration" | "method_definition" | "arrow_function" |
            // Java/C#
             "interface_declaration" | "record_declaration" |
            // Ruby
             "method" | "class" | "module"
        );

        if is_chunkable {
            let start_byte = node.start_byte();
            let end_byte = node.end_byte();
            
            let chunk_content = &code[start_byte..end_byte];
            let start_position = node.start_position();
            let end_position = node.end_position();

            chunks.push(CodeChunk {
                filename: filename.to_string(),
                code: chunk_content.to_string(),
                line_start: start_position.row + 1, 
                line_end: end_position.row + 1,
                last_modified: mtime,
            });
            
            let is_container = kind.contains("class") || kind.contains("impl") || kind.contains("struct");
             if !is_container {
                 return;
             }
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
             self.traverse(&child, code, filename, chunks, _ext, mtime);
        }
    }
}

