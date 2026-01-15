# Supported Languages and Extracted Use Cases

`code-rag` uses Tree-sitter to parse source code into an Abstract Syntax Tree (AST) and extracts meaningful "chunks" based on language-specific nodes. This approach ensures that you index complete functions, classes, and modules rather than arbitrary lines of text.

## Core Languages

| Language | Extensions | Extracted Concepts (AST Nodes) |
|----------|-----------|--------------------------------|
| **Rust** | `.rs` | Functions (`function_item`), Implementations (`impl_item`), Structs (`struct_item`), Enums (`enum_item`), Modules (`mod_item`), Constants (`const_item`), Statics (`static_item`) |
| **Python** | `.py` | Functions (`function_definition`), Classes (`class_definition`), Async Functions (`function_statement`), Scripts (top-level logic) |
| **Go** | `.go` | Functions (`function_declaration`), Methods (`method_declaration`), Type Declarations (`type_declaration`) |
| **C/C++** | `.c`, `.cpp`, `.h`, `.hpp`, `.cc`, `.cxx` | Functions (`function_definition`), Classes (`class_specifier`), Structs (`struct_specifier`) |
| **JavaScript** | `.js`, `.jsx` | Classes (`class_declaration`), Methods (`method_definition`), Arrow Functions (`arrow_function`), Script logic |
| **TypeScript** | `.ts`, `.tsx` | Classes (`class_declaration`), Methods (`method_definition`), Interfaces (`interface_declaration`), Arrow Functions, Script logic |
| **Java** | `.java` | Classes (`class_declaration`), Methods (`method_declaration`), Interfaces (`interface_declaration`), Records (`record_declaration`) |
| **C#** | `.cs` | Classes (`class_declaration`), Methods (`method_declaration`), Interfaces (`interface_declaration`), Records (`record_declaration`) |
| **Ruby** | `.rb` | Methods (`method`), Classes (`class`), Modules (`module`), Script logic |
| **PHP** | `.php` | Functions (`function_definition`), Classes (`class_declaration`) |

## Web Technologies

| Language | Extensions | Extracted Concepts |
|----------|-----------|--------------------|
| **HTML** | `.html` | `<script>` blocks, `<style>` blocks |
| **CSS** | `.css` | Rule sets (`rule_set`), Media queries (`media_statement`), Keyframes (`keyframes_statement`) |

## Scripting & Configuration

| Language | Extensions | Extracted Concepts |
|----------|-----------|--------------------|
| **Bash** | `.sh`, `.bash` | Functions (`function_definition`), Top-level Commands (`command`, `pipeline`, `if_statement`, etc.) |
| **PowerShell** | `.ps1` | Functions (`function_definition`), Param Blocks (`param_block`), Top-level Commands (`command`, `pipeline`, `flow_statement`) |
| **YAML** | `.yaml`, `.yml` | Block mappings, Pairs (chunks based on top-level keys) |
| **JSON** | `.json` | Objects, Key-Value Pairs |

## Emerging & Specialized Languages

| Language | Extensions | Extracted Concepts |
|----------|-----------|--------------------|
| **Zig** | `.zig` | Declarations (`Decls`), Function Prototypes (`FnProto`), Fields (`ContainerField`) |
| **Elixir** | `.ex`, `.exs` | Function calls (`call`), Do blocks (`do_block` - used for def/defmodule) |
| **Haskell** | `.hs` | Functions (`function`), Signatures (`signature`) |
| **Solidity** | `.sol` | Contracts (`contract_declaration`), Libraries (`library_definition`), Interfaces (`interface_definition`) |

---

## How Extraction Works

1.  **Parsing**: The file is parsed into a full AST.
2.  **Traversal**: The chunker walks the tree looking for the specific nodes listed above.
3.  **Extraction**: When a node is found (e.g., a Python `def`), the entire byte range of that node is extracted as a single chunk.
4.  **Metadata**: The chunk is tagged with its filename, line numbers, and extracted function calls.

This ensures that if you search for "login logic", you get the full `login()` function, not just the line where the word "login" appears.
