# Changelog

All notable changes to the Zero-Latency Documentation Search project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **BREAKING: real local embeddings.** `ZL_EMBEDDING_PROVIDER=local` (the default) now runs bge-small-en-v1.5 through ONNX Runtime instead of a hash of the text, so search results are ranked by meaning. Queries get bge's retrieval prefix.
  - **First run** downloads the model (~130 MB) from a pinned Hugging Face revision into `~/.zero-latency/models/bge-small-en-v1.5/` and checks its SHA-256; after that it runs offline. For air-gapped machines and containers, run `doc-indexer --fetch-model [dir]` elsewhere and set `ZL_EMBEDDING_LOCAL_MODEL_PATH`; the Docker image does this at build time
  - **Re-index once after upgrading.** Existing embedded stores hold hash vectors; on first start they are removed (your documents are not touched), a warning gives the count, and `GET /api/status` shows `"reindex_required": true`. Search returns nothing until you run `mdx reindex` (or `mdx index <path>`)
  - **Qdrant users**: collections built with the old local embeddings aren't detected. Delete the collection (or use a new `ZL_VECTOR_QDRANT_COLLECTION`) and re-index
  - **Rolling back**: the previous release has no model check and would search bge vectors with hash queries. Revert, delete `~/.zero-latency/vectors.db`, then re-index
- `ZL_EMBEDDING_LOCAL_DIMENSION` must be 384 with the local provider; other values are rejected
- **Search ranking follows similarity**: vector similarity now makes up at least 80% of a result's final score (previously 40%). The keyword and title heuristics only break near-ties. Previously a short, heading-like chunk could outrank a clearly more relevant passage, and the same query could rank differently from run to run

### Added
- `ZL_EMBEDDING_LOCAL_MODEL_PATH` (pre-supplied model directory, never downloaded into) and `ZL_EMBEDDING_LOCAL_MODEL_URL` (download mirror)
- `doc-indexer --fetch-model [dir]`: download and verify the model, print its directory, exit
- `ZL_EMBEDDING_PROVIDER=hash`: the old deterministic embedder, for tests only (not semantic)
- `reindex_required` on `GET /api/status` and in `mdx status`

### Fixed
- **`tantivy` feature builds**: the Tantivy BM25 adapter is ported to tantivy 0.22, opens or creates its index in an empty directory, and makes documents searchable as soon as indexing returns
- **`cloud` feature builds**: the OpenAI embedding provider now calls the OpenAI API (previously it returned placeholder vectors), and the Qdrant backend now stores, updates, deletes and counts vectors (previously only search worked), creating its collection on first insert
- **Vector backend and embedding provider are configurable**: `doc-indexer` read neither, so Qdrant and OpenAI could never be selected. Set them with `ZL_VECTOR_*` / `ZL_EMBEDDING_*` or `[vector]` / `[embedding]` in `zero-latency.toml`. `OPENAI_API_KEY` is honoured. The `DOC_INDEXER_*` names for these settings were documented but never read, and are removed from the docs
- **Multi-word `ZL_` settings** such as `ZL_SERVER_DOCS_PATH` now take effect (previously every `_` was read as nesting)
- **Builds without `embedded`** (`--no-default-features`) compile again
- **RUSTSEC-2026-0187**: `pdf-extract` upgraded to 0.12, which brings in a `lopdf` without the stack overflow on deeply nested PDFs

### Removed
- `zero-latency-search`'s `examples` feature, which gated a stub that never compiled
- The unused `qdrant-client` and `tonic` dependencies

### CI
- Clippy, tests and `cargo-deny` now cover every feature (`--all-features`)
- New `model-tests` job runs the ONNX embedding tests against the real model, cached per pinned revision; the main test job stays offline and model-free

## [1.1.2] - 2025-08-30

### Issues Discovered
- **Collection Metadata Missing in Search Results**: End-to-end testing revealed search results return empty metadata `{}` despite collection information being set during document indexing
- **Document ID Not Preserved During MCP Indexing**: MCP tools interface ignores provided document IDs and generates system UUIDs instead
- **Collection Association Lost**: Documents not properly associated with collections in search response metadata
- **Impact**: Core functionality works but collection-based organization and external ID tracking is compromised

### Documentation Added
- **Comprehensive Issue Analysis**: `docs/implementation/COLLECTION_METADATA_SEARCH_ISSUES.md`
  - Detailed root cause analysis with code investigation
  - Testing evidence and functional impact assessment
  - Recommended fixes for document ID preservation and metadata serialization
- **Known Issues Section**: Added to `docs/INDEX.md` and `docs/CURRENT_ARCHITECTURE.md`

### Sprint Planning
- **Sprint 004**: Metadata & Collection Management Issues Resolution (2 weeks, 47 story points)
- **Sprint 005**: Search & Filtering Issues Resolution (1.5 weeks, 37 story points)
- **Sprint 006**: Protocol Compliance & Standards Alignment (2 weeks, 46 story points)

### Documentation Organization
- **Issues Folder**: Created `docs/issues/` with centralized issue tracking
  - `docs/issues/README.md` - Master issues index with status tracking
  - `docs/issues/metadata-issues.md` - Critical metadata and collection problems
  - `docs/issues/search-issues.md` - Search and filtering issues
  - `docs/issues/protocol-issues.md` - Protocol compliance review
- **Sprint Plans**: Added comprehensive sprint plans to `docs/sprint/` following established conventions

## [1.1.1] - 2025-08-24

### Fixed
- **Search Limit Parameter Bug**: CLI `--limit` parameter now properly respected throughout search pipeline
  - Enhanced `SearchQuery` domain model to include limit field with default of 10
  - Updated CLI service to pass limit parameter using `with_limit()` builder method
  - Fixed API client hardcoded limit of 10 to use actual query limit
  - Resolves issue where `mdx search "query" --limit 3` returned 10 results instead of 3
  - Eliminates duplicate results in search output
- **Collection Metadata Synchronization**: Fixed discrepancy between reported collection stats and actual vector count
  - Enhanced `CollectionService` to initialize from actual database vector count
  - Added async initialization pattern to ensure collection stats reflect database state
  - Resolves issue where collection showed 0 vectors despite 203,381 actual vectors in database

### Technical Details
- Request flow now correctly: CLI args → SearchCommand → SearchQuery → API request → VectorSearchStep
- Collection metadata properly synchronized with persistent vector storage on service startup
- Documentation added: `docs/implementation/SEARCH_LIMIT_BUG_FIX.md`

## [1.1.0] - 2025-08-23

### Added - Phase 4D Service Extension Complete

#### MCP Transport Validation
- **JSON-RPC 2.0 Compliance**: Full specification adherence for Model Context Protocol integration
- **Dual Transport Support**: stdio and HTTP protocols operational with seamless switching
- **Service Discovery**: Comprehensive capability reporting and health monitoring endpoints
- **Performance Validated**: <100ms response times across all transport methods

#### Advanced Feature Flag Architecture
- **Conditional Compilation**: Sophisticated feature flag system enabling deployment-specific builds
- **Multi-Variant Support**: embedded/cloud/full deployment configurations
- **Binary Optimization**: Feature-specific dependency inclusion for minimal footprint
- **Runtime Flexibility**: Environment-based feature selection without code changes

#### Enhanced Search Pipeline Validation
- **Comprehensive Testing Framework**: Multi-variant validation across all feature combinations
- **Performance Benchmarking**: Established baseline metrics for optimization decisions
- **Production Validation**: End-to-end workflow testing with quality assurance
- **Automated Testing**: Complete validation scripts for continuous integration

#### Quality Assurance Improvements
- **Production Readiness**: Validated deployment scenarios across all variants
- **Documentation Coverage**: Complete API documentation and deployment guides
- **Error Handling**: Graceful failure modes with proper recovery mechanisms
- **Health Monitoring**: Real-time subsystem monitoring with comprehensive status reporting

### Changed
- **Build System**: Enhanced with feature flag architecture for flexible deployment
- **Service Container**: Improved dependency injection with clean architecture principles
- **Configuration Management**: Environment-based feature selection system
- **Performance Characteristics**: Sub-second startup with comprehensive health monitoring

## [1.0.0] - 2025-08-22

### Added - Production Distribution

#### Self-Contained Binaries
- **ONNX Runtime Integration**: Embedded ONNX Runtime using `download-binaries` feature
- **Zero External Dependencies**: Self-contained binaries with no dylib requirements
- **Cross-Platform Model Inference**: Local embedding model with consistent performance

#### macOS App Bundle (`Zero-Latency.app`)
- **Professional App Bundle**: Native macOS application structure
- **GUI Control Panel**: User-friendly interface for daemon management
- **LaunchAgent Integration**: Automatic background service management
- **CLI Terminal Access**: Integrated terminal for command-line operations
- **Drag-and-Drop Installation**: Standard macOS installation experience

#### Distribution Packaging
- **DMG Installer**: Professional `Zero-Latency-v1.0.0.dmg` (6.4MB)
- **Build Automation**: Complete build and packaging scripts
  - `scripts/build-macos-app.sh`: Creates app bundle with LaunchAgent
  - `scripts/build-dmg.sh`: Packages into distributable DMG
  - `scripts/install.sh`: Command-line installation script

### Changed - Architecture Improvements

#### Dependency Management
- **Simplified ORT Configuration**: Removed manual dylib path configuration
- **Unified Build Process**: Single `cargo build --release` creates deployable binaries
- **Cleaner Cargo.toml**: Eliminated duplicate dependencies

#### Service Architecture
- **Enhanced Error Handling**: Comprehensive error propagation in service layers
- **Improved Configuration**: Type-safe configuration management with validation
- **Production-Ready Logging**: Structured logging with multiple output formats

### Technical Details

#### Build System
- **Release Build**: Optimized binaries (~8.4MB each)
- **Model Embedding**: gte-small ONNX model automatically downloaded and cached
- **Packaging Scripts**: Automated macOS distribution packaging

#### Dependencies
- **ort v1.16**: ONNX Runtime with `download-binaries` feature
- **tokenizers v0.15**: Text tokenization for embeddings
- **Clean Architecture Crates**: All shared domain crates integrated

### Installation Methods

1. **macOS App Bundle** (Recommended)
   - Download `Zero-Latency-v1.0.0.dmg`
   - Drag to Applications folder
   - Launch for GUI daemon management

2. **Command Line Installation**
   - Use `scripts/install.sh` for automated setup
   - Manual binary copying to `/usr/local/bin`

3. **From Source**
   - `cargo build --release` creates self-contained binaries
   - No additional configuration required

### Performance
- **Binary Size**: ~8.4MB per binary (self-contained)
- **DMG Size**: 6.4MB (complete distribution)
- **Model Cache**: ~/.cache/zero-latency/models/ (~126MB)
- **Search Latency**: 10-20ms typical response time

### Documentation
- **Updated README**: Comprehensive installation and usage documentation
- **Build Scripts**: Fully documented packaging and distribution processes
- **Professional Presentation**: DMG includes README and usage instructions

---

## [0.4.0] - 2025-08-21

### Added - Clean Architecture Implementation

#### Shared Domain Crates
- **zero-latency-core**: Foundation models, error handling, health monitoring
- **zero-latency-vector**: Vector storage and embedding abstractions
- **zero-latency-search**: Search orchestration and query processing
- **zero-latency-observability**: Metrics and monitoring frameworks
- **zero-latency-config**: Type-safe configuration management

#### Architecture Patterns
- **SOLID Principles**: Complete compliance with clean architecture principles
- **Dependency Injection**: ServiceContainer for loose coupling
- **Trait Abstractions**: Testable and mockable service interfaces
- **Layer Separation**: Clear Application/Infrastructure/Domain boundaries

#### Production Features
- **Vector Storage Adapters**: Qdrant + In-Memory implementations
- **Embedding Adapters**: OpenAI + Local deterministic options
- **HTTP REST API**: Comprehensive error handling and validation
- **Health Monitoring**: Readiness/liveness checks for deployment
- **Configuration-Driven**: Runtime adapter selection via configuration

### Changed
- **Service Refactoring**: Complete doc-indexer service restructure
- **Error Handling**: Comprehensive error propagation and user-friendly messages
- **API Design**: RESTful endpoints with proper HTTP status codes

---

## [0.3.0] - 2025-08-20

### Added - CLI Interface and API Extensions

#### CLI Features
- **mdx Command**: Intuitive command-line interface
- **Multiple Output Formats**: Table, JSON output options
- **Server Management**: Start/stop API server via CLI
- **Status Monitoring**: System health and collection statistics

#### API Enhancements
- **RESTful Endpoints**: /api/search, /api/status, /api/docs
- **JSON-RPC Support**: Alternative API protocol
- **Performance Monitoring**: Request timing and metrics

### Changed
- **Improved Search**: Enhanced relevance scoring and result formatting
- **Better Error Messages**: User-friendly error reporting
- **Documentation**: Comprehensive usage examples

---

## [0.2.0] - 2025-08-19

### Added - Core Search Functionality

#### Local Embeddings
- **gte-small Model**: 384-dimension embeddings
- **ONNX Runtime**: Local model inference (~1ms)
- **Automatic Caching**: Model download and local storage

#### Vector Search
- **Qdrant Integration**: Vector similarity search
- **Real-time Indexing**: File system monitoring
- **Incremental Updates**: Smart change detection

#### Document Processing
- **Markdown Support**: Comprehensive markdown parsing
- **Chunking Strategy**: Semantic document segmentation
- **Metadata Extraction**: Document type and section detection

### Performance
- **Search Speed**: 10-20ms typical response time
- **Indexing Rate**: ~50 documents/second
- **Memory Efficiency**: ~100MB base + model size

---

## [0.1.0] - 2025-08-19

### Added - Project Foundation

#### Core Architecture
- **Rust Implementation**: High-performance system programming
- **Modular Design**: Separated CLI and service components
- **Docker Integration**: Qdrant vector database containerization

#### Basic Features
- **Document Indexing**: File system scanning and processing
- **HTTP API**: Basic search endpoint
- **Configuration**: Environment-based configuration

#### Development Setup
- **Cargo Workspace**: Multi-crate project structure
- **Development Tools**: Testing, linting, and formatting setup
- **Documentation**: Initial project documentation and examples

---

## Legend

- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Vulnerability fixes
