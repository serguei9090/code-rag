use std::path::Path;
use tree_sitter::{Parser, Language, Node};

pub struct CodeChunk {
    pub filename: String,
    pub code: String,
    pub line_start: usize,
    pub line_end: usize,
    pub last_modified: i64,
    pub calls: Vec<String>,
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
             "sh" | "bash" => Some(tree_sitter_bash::LANGUAGE.into()),
             "ps1" => Some(tree_sitter_powershell::language().into()),
             // "dockerfile" | "Dockerfile" => Some(tree_sitter_dockerfile::language().into()),
             "yaml" | "yml" => Some(tree_sitter_yaml::LANGUAGE.into()),
             "json" => Some(tree_sitter_json::LANGUAGE.into()),
             "zig" => Some(tree_sitter_zig::LANGUAGE.into()),
             "ex" | "exs" => Some(tree_sitter_elixir::LANGUAGE.into()),
             "hs" => Some(tree_sitter_haskell::LANGUAGE.into()),
             "sol" => Some(tree_sitter_solidity::LANGUAGE.into()),
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
             eprintln!("ERROR: Could not set language for extension: {}", ext);
             return vec![];
        }

        let tree = match parser.parse(code, None) {
             Some(t) => t,
             None => return vec![],
        };

        let mut chunks = Vec::new();
        let root = tree.root_node();
        
        self.traverse(&root, code, filename, &mut chunks, ext, mtime, 0);
        
        chunks
    }

    fn traverse(&self, node: &Node, code: &str, filename: &str, chunks: &mut Vec<CodeChunk>, ext: &str, mtime: i64, depth: usize) {
        let kind = node.kind();
        
        let is_script_lang = matches!(ext, "py" | "js" | "ts" | "jsx" | "tsx" | "rb" | "lua" | "sh" | "bash" | "ps1");
        
        let is_semantic_chunk = matches!(kind,
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
        let is_script_chunk = is_script_lang && (depth == 1 || (ext == "ps1" && depth == 2)) && matches!(kind,
            "if_statement" | "flow_statement" | "expression_statement" | "assignment" | 
            "variable_declaration" | "lexical_declaration" | "function_call" | "call" |
            "command" | "pipeline" | "if_expression" | "for_expression" // Bash/PS1 extras
        );

        let is_chunkable = is_semantic_chunk || is_ruby_module || is_script_chunk;

        if is_chunkable {
            // DEBUG: Print S-expression
            // eprintln!("DEBUG AST: {}", node.to_sexp());
            let start_byte = node.start_byte();
            let end_byte = node.end_byte();
            
            let chunk_content = &code[start_byte..end_byte];
            let start_position = node.start_position();
            let end_position = node.end_position();

            // Extract calls
            let calls = self.find_calls(node, code);

            chunks.push(CodeChunk {
                filename: filename.to_string(),
                code: chunk_content.to_string(),
                line_start: start_position.row + 1, 
                line_end: end_position.row + 1,
                last_modified: mtime,
                calls,
            });
            
            let is_container = kind.contains("class") || kind.contains("impl") || kind.contains("struct") || kind == "element" || kind == "stylesheet"; 
             if !is_container {
                 return;
             }
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
             self.traverse(&child, code, filename, chunks, ext, mtime, depth + 1);
        }
    }

    fn find_calls(&self, node: &Node, code: &str) -> Vec<String> {
        let mut calls = Vec::new();
        let mut cursor = node.walk();
        
        // Simple recursive search looking for call-like nodes
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            // println!("DEBUG visiting kind: {}", kind); 
            if matches!(kind, "call_expression" | "call" | "macro_invocation") {
                // Try to get identifier
                if let Some(name) = self.extract_name(&child, code) {
                    calls.push(name);
                }
            }
            // Recurse
            calls.extend(self.find_calls(&child, code));
        }
        calls
    }

    fn extract_name(&self, node: &Node, code: &str) -> Option<String> {
        // Heuristic: finding the 'function' or 'identifier' child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
             let k = child.kind();
             if matches!(k, "identifier" | "field_expression" | "scoped_identifier") {
                 return Some(code[child.start_byte()..child.end_byte()].to_string());
             }
        }
        None
    }
}

