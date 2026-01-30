# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-01-22

### Changed
- Updated Docker Grafana configuration.
- Improved fast shutdown handling.
- Fixed dashboard issues and server workspace management.
- Enhanced telemetry with HTTP support and start notifications.

## [0.1.1] - 2026-01-21

### Added
- Server mode with search API and hybrid code search.
- Multi-workspace management and comprehensive integration tests.

### Fixed
- Server configuration fixes and `--config` flag log level control.
- BM25 fixes and updated test suite.
- Dependency updates and build improvements.

## [0.1.0] - 2024-03-20

### Added
- Initial release of Code RAG.
- High-performance offline code search CLI.
- Semantic search using `nomic-embed-text-v1.5` and `bge-reranker-base`.
- Hybrid search (BM25 + Vectors).
- Multi-workspace support.
- Local LLM query expansion support.
- File system watching and auto-indexing.
- Server mode with REST API.
