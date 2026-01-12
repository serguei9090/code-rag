---
trigger: always_on
---

High-Performance Offline Code RAG Stack

This document outlines the architecture for the code-rag CLI tool. The goal is to provide sub-millisecond code retrieval for AI agents without external API dependencies.

Component

Technology

Reasoning

Language

Rust

Zero Runtime Overhead. Unlike Python, it requires no interpreter startup time (critical for agents running repeated CLI commands). Handles parallel file walking (10k+ files) effortlessly.

Vector Database

LanceDB

Embedded & Serverless. It runs in-process and stores data in a local ./.lancedb folder. No Docker container or external service is required.

Embedding Model

nomic-embed-text-v1.5

8192 Token Context. Standard models (BERT/MiniLM) only support 512 tokens, which truncates long code files. Nomic allows embedding full functions/classes.

Inference Engine

fastembed-rs

ONNX Runtime. Runs the quantized embedding model locally on CPU with AVX512 optimization. No API keys or internet connection needed.

Chunking Strategy

Tree-sitter

AST Parsing. Instead of splitting text by arbitrary line numbers, we parse the Abstract Syntax Tree to extract complete functions, classes, and structs.

Relation Search

ripgrep (via grep crate)

Exact Matching. Vectors are fuzzy; regex is precise. We embed the ripgrep engine to allow the agent to find exact usage references (e.g., "Where is UserID imported?").

CLI Framework

clap

Robust Arguments. Provides strict typing and help generation, ensuring the AI agent constructs valid commands.

Dual-Mode Architecture

To solve the "Concept vs. Relation" problem, the tool implements two distinct search modes:

Semantic Mode (search): Uses Vector Embeddings (LanceDB) to find code based on meaning (e.g., "authentication logic").

Exact Mode (grep): Uses the embedded Ripgrep engine to find code based on usage (e.g., "files importing ./auth").

Data Flow

Indexing:
File System -> Tree-sitter (AST) -> FastEmbed (Vector) -> LanceDB (Storage)

Retrieval:
Agent Query -> Inference (Vector) -> LanceDB (ANN Search) -> JSON Output

Agent Implementation Guide

NOTICE TO BUILDER AGENT: Follow these protocols strictly when generating the application code.

1. Environment Setup

Initialize the project with the following dependencies to ensure compatibility and performance.

cargo new --bin code-rag
cargo add clap --features derive
cargo add fastembed
cargo add lancedb
cargo add arrow-array
cargo add tokio --features full
cargo add serde --features derive
cargo add serde_json
cargo add ignore # For respecting .gitignore

# --- Language Parsers (Tree-sitter) ---

# Core & Systems
cargo add tree-sitter tree-sitter-rust tree-sitter-python tree-sitter-c tree-sitter-cpp tree-sitter-go

# Web & Frontend
cargo add tree-sitter-javascript tree-sitter-typescript tree-sitter-html tree-sitter-css tree-sitter-vue tree-sitter-svelte

# Backend & Enterprise
cargo add tree-sitter-java tree-sitter-c-sharp tree-sitter-php tree-sitter-ruby tree-sitter-lua

# Mobile
cargo add tree-sitter-swift tree-sitter-kotlin

# DevOps & Config
cargo add tree-sitter-dockerfile tree-sitter-hcl tree-sitter-yaml tree-sitter-toml tree-sitter-make tree-sitter-json tree-sitter-bash

# Modern & Niche
cargo add tree-sitter-zig tree-sitter-elixir tree-sitter-haskell tree-sitter-solidity


2. Directory Structure Standard

Do not put all code in main.rs. Use a modular library structure for testability.

src/
├── main.rs            # Entry point, CLI argument parsing (Clap)
├── lib.rs             # Exports modules
├── indexer.rs         # Tree-sitter AST parsing & chunking logic
├── storage.rs         # LanceDB connection & schema definitions
├── embedding.rs       # FastEmbed model wrapper (singleton pattern)
└── search.rs          # Query logic (Semantic + Regex)


3. Security & Safety Protocols

A. The "No-Panic" Rule

Forbidden: Do not use .unwrap() or .expect() in any logic inside src/lib.rs or its modules.

Required: Return Result<T, Box<dyn Error>> and handle errors gracefully.

Exception: You may use unwrap() only in main.rs top-level error reporting or inside #[test] blocks.

B. File System Safety

Read-Only: The indexer module must open files in Read-Only mode.

Symlinks: Use std::fs::canonicalize to prevent infinite recursion in symlinks, but ensure the tool does not follow symlinks pointing outside the project root if strict isolation is required.

Git Ignore: You MUST use the ignore crate to respect .gitignore. Indexing node_modules or target directories is a critical failure.

C. Input Sanitation

When performing grep searches, escape user input if it contains special regex characters, unless the user explicitly requests raw regex mode.

4. Implementation Steps

Step 1: Define the Schema (storage.rs)

Create a strict Arrow schema for LanceDB.

Fields: id (String), filename (String), code (String), line_start (Int32), line_end (Int32), vector (FixedSizeList<Float32>).

Step 2: Build the AST Chunking (indexer.rs)

Implement a CodeChunker struct.

Logic: Load the correct Tree-sitter language based on file extension.

Strategy: traverse the tree. If a node is function_definition, class_definition, or impl_item, extract the byte range.

Step 3: Vector Pipeline (embedding.rs)

Initialize TextEmbedding with InitOptions { model_name: EmbeddingModel::NomicEmbedTextV15, ... }.

Optimization: Use model.embed_batch (not single embed) for performance.

Step 4: CLI Glue (main.rs)

Implement clap::Parser with subcommands:

Index { path: String }

Search { query: String, limit: u8 }

Grep { pattern: String }