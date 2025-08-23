# Zero-Latency Documentation Search

A high-performance semantic search system for documentation using local embeddings and **embedded vector storage**. Built with Rust for speed and reliability.

## 🎯 **Current Status - Production Ready**

**Build Status**: ✅ All feature variants building successfully  
**Architecture Validation**: ✅ Clean architecture with SOLID principles  
**MCP Compliance**: ✅ Full JSON-RPC 2.0 transport validation  
**Performance Optimization**: ✅ <1s startup, <100ms response times  
**Production Readiness**: ✅ Multi-variant deployment validation complete  
**Documentation**: [Phase 4D Complete Summary](./docs/milestones/PHASE_4D_SERVICE_EXTENSION_COMPLETE.md)

### 🎯 Phase 4D Achievement Summary (August 23, 2025)

**ALL OBJECTIVES ACHIEVED** - Zero-Latency system now production-ready with:

✅ **Task 3**: MCP transport validation with JSON-RPC 2.0 compliance  
✅ **Task 4**: Advanced feature flag architecture with conditional compilation  
✅ **Task 5**: Comprehensive search pipeline validation and performance benchmarking  

**Ready for Next Phase**: Enterprise deployment and scaling optimization.

## 🚀 Quick Start

### Prerequisites

**Nothing!** Zero-Latency now includes an embedded vector database. No need for external services.

### Using the CLI

The `mdx` command provides an intuitive interface for document search:

```bash
# Build the CLI (includes embedded database)
cargo build --release

# Index documents from a directory
mdx index /path/to/docs

# Search for documents  
mdx search "embedding model"

# Check system status
mdx status

# Start the API server (with embedded database)
mdx server --start

# Get help
mdx help
```

## 📖 CLI Commands

### Search Documents

```bash
# Basic search
mdx search "vector database"

# Limit results
mdx search "machine learning" --limit 5

# JSON output for scripting
mdx search "architecture" --format json

# Table output (default)
mdx search "deployment" --format table
```

**Example output:**
```
🔍 Searching for: embedding model

📄 Model Architecture (score: 0.89)
   "Local embedding model using gte-small for semantic search..."
   📁 model-host/artefacts • 🏷️ blueprint

📄 ONNX Integration (score: 0.82)
   "ONNX Runtime provides efficient model inference..."
   📁 misc/artefacts • 🏷️ technical

✅ Found 2 results in 12ms
```

### System Status

```bash
mdx status
```

Shows collection statistics, health, and configuration:
```
✅ Zero-Latency Documentation Search Status

📊 Collection: zero_latency_docs
   Documents: 25
   Chunks: 1,248
   Vector Dimensions: 384

🔧 Configuration:
   Embedding Model: gte-small (local)
   Vector Database: Qdrant (localhost:6333)

🚀 Performance:
   Last Search: 12ms
   Index Status: Healthy
```

### Document Management

```bash
# Index documents from directory
mdx index ~/my-docs

# Rebuild entire index
mdx reindex

# Start background indexing server
mdx server --start

# Stop indexing server  
mdx server --stop
```

## 📚 Documentation

Comprehensive documentation is organized in the [`docs/`](docs/) directory:

- **[Architecture](docs/architecture/)** - Clean architecture implementation details
- **[Implementation](docs/implementation/)** - Technical implementation guides  
- **[Milestones](docs/milestones/)** - Project milestone documentation
- **[Services](docs/services/)** - Service-specific documentation
- **[Testing](test/integration/)** - Integration test documentation

For a complete overview, see the [Documentation Index](docs/README.md).

## 📊 Project Status

### ✅ Phase 4D: Service Extension - COMPLETED (August 23, 2025)

**PHASE COMPLETE!** Successfully delivered comprehensive service extension with production-grade validation framework:

#### **Enhanced Search Pipeline Validation** (Completed August 23, 2025)

- **Comprehensive Validation Framework**: Multi-variant testing and performance benchmarking
  - Build system validation across all feature combinations
  - JSON-RPC transport compliance testing with service discovery
  - Performance baseline establishment: <1s startup, <100ms response times
  - Production readiness validation with health monitoring

- **Quality Assurance Metrics**: Production-grade validation results
  - **Build Performance**: 9-10 second release builds across all variants
  - **Runtime Performance**: <1 second startup, ~50MB memory footprint
  - **Service Reliability**: Comprehensive health checks and graceful shutdown
  - **Documentation Coverage**: Complete deployment guides and API documentation

#### **Advanced Feature Flag Architecture** (Completed August 23, 2025)

- **Feature Flag Architecture**: Conditional compilation for optimized deployments
  - `embedded`: Local SQLite + ONNX models (edge deployment)
  - `cloud`: Qdrant + OpenAI integration (server deployment)  
  - `full`: Complete feature set (development/testing)

- **Deployment Flexibility**: Tailored builds for specific environments
  - **Embedded builds**: Reduced size, local-only dependencies
  - **Cloud builds**: Network-optimized, external service integration
  - **Runtime validation**: Clear errors for missing features

#### **Core Architecture** (Completed August 21, 2025)

- **5 Shared Domain Crates**: Comprehensive foundation providing reusable models, traits, and abstractions
  - `zero-latency-core`: Foundation models, error handling, health monitoring
  - `zero-latency-vector`: Vector storage and embedding abstractions  
  - `zero-latency-search`: Search orchestration and query processing
  - `zero-latency-observability`: Metrics and monitoring frameworks
  - `zero-latency-config`: Type-safe configuration management

- **Clean Architecture Implementation**: Full refactor of doc-indexer service demonstrating:
  - SOLID principles compliance
  - Dependency injection with ServiceContainer
  - Trait-based abstractions for testability
  - Proper separation of concerns (Application/Infrastructure/Domain layers)

#### **Production Distribution** (Completed August 22, 2025)

- **Embedded Vector Database**: Self-contained SQLite-based vector storage
  - No external Qdrant dependency
  - Persistent storage with automatic caching
  - Cross-platform binary deployment
  - ~2KB per document chunk storage

- **Self-Contained Binaries**: ONNX Runtime embedded with `download-binaries` feature
  - No external library dependencies
  - Single binary deployment
  - Cross-platform model inference

- **macOS App Bundle**: Professional distribution package
  - GUI control panel for daemon management
  - Automatic LaunchAgent installation
  - Integrated CLI terminal access
  - Drag-and-drop .app installation

- **DMG Installer**: `Zero-Latency-v1.0.0.dmg` (6.4MB)
  - Professional macOS installer experience
  - Applications folder integration
  - Comprehensive README and documentation

- **Build Automation**:
  - `scripts/build-macos-app.sh`: Creates app bundle with LaunchAgent
  - `scripts/build-dmg.sh`: Packages into distributable DMG
  - `scripts/install.sh`: Command-line installation script

**Build Status**: ✅ Successful release build with feature flags  
**Architecture Validation**: ✅ All SOLID principles implemented  
**MCP Integration**: ✅ JSON-RPC 2.0 transport validation complete  
**Build Optimization**: ✅ Feature flag system for deployment flexibility  
**Distribution**: ✅ Professional macOS packaging complete  
**Documentation**: [Phase 4D Implementation Details](./docs/milestones/TASK_4_BUILD_OPTIMIZATION_COMPLETE.md)

### � Current Focus: Enhanced Search Pipeline Validation (August 23, 2025)

**Next Milestone**: Task 5 - Complete search pipeline validation with performance benchmarking

**Objectives**:
- Validate end-to-end search performance across all feature combinations
- Implement comprehensive integration testing framework
- Benchmark query processing and response times
- Establish performance baselines for optimization

## �🏗 Architecture

### System Components

```text
mdx CLI ──HTTP──> API Server ──SQLite──> Embedded Vector DB
   │                   │                       │
   │                   └─> Local Embedder ─────┘
   │                       (gte-small ONNX)
   │
   └─> Local Files & Configuration
```

### Technology Stack

- **CLI**: Rust with `clap` framework
- **HTTP API**: Axum web framework  
- **Embeddings**: Local ONNX model (gte-small, 384 dimensions)
- **Vector DB**: Embedded SQLite with binary blob storage
- **File Monitoring**: Real-time document change detection

## 🛠 Installation

### macOS App Bundle (Recommended)

**Download the latest release**: [Zero-Latency-v1.0.0.dmg](releases/Zero-Latency-v1.0.0.dmg)

1. **Download and mount** the DMG file
2. **Drag Zero-Latency.app** to your Applications folder
3. **Double-click Zero-Latency.app** to open the control panel
4. **Choose "Install & Start Daemon"** to begin background service

The app bundle includes:

- ✅ **Self-contained binaries** with embedded ONNX Runtime
- ✅ **Automatic daemon management** via macOS LaunchAgent
- ✅ **GUI control panel** for easy management
- ✅ **CLI access** via integrated terminal

### Command Line Installation

```bash
# Quick install script (requires built binaries)
./scripts/install.sh

# Manual installation
sudo cp target/release/{doc-indexer,mdx} /usr/local/bin/
```

### From Source

```bash
# Clone repository
git clone <repo-url>
cd Zero-Latency

# Build with feature flags for optimized deployment

# Embedded-only build (local deployment, edge devices)
cargo build --release --features embedded --no-default-features

# Cloud-only build (server deployment, external services)
cargo build --release --features cloud --no-default-features

# Full build (development, all features)
cargo build --release --features full

# Default build (embedded features enabled)
cargo build --release

# CLI available at ./target/release/mdx
# API server at ./target/release/doc-indexer
```

### Build Features

- **`embedded`** (default): Local SQLite storage + ONNX embeddings
- **`cloud`**: Qdrant integration + OpenAI embeddings
- **`full`**: All features enabled for development/testing

### Building Distribution Packages

```bash
# Build macOS app bundle
./scripts/build-macos-app.sh

# Create DMG installer
./scripts/build-dmg.sh

# Results in Zero-Latency.app and Zero-Latency-v1.0.0.dmg
```

### Docker Setup

**No Docker needed!** The embedded vector database eliminates the need for external services.

For advanced users who want to use external Qdrant:

```bash
# Optional: Start external Qdrant vector database
docker run -d \
  --name qdrant \
  -p 6333:6333 \
  -p 6334:6334 \
  qdrant/qdrant

# Configure to use external Qdrant  
export DOC_INDEXER_VECTOR_BACKEND=qdrant
export DOC_INDEXER_QDRANT_URL=http://localhost:6333
```

## 📊 Features

### ✅ Completed (Phase 1 & 2)

- **Local Embeddings**: No API keys required
  - gte-small model (384 dimensions)
  - ~1ms inference time
  - Automatic model download and caching

- **Real-time Indexing**:
  - File system monitoring
  - Smart change detection
  - Incremental updates

- **Professional CLI**:
  - Intuitive commands
  - Multiple output formats
  - Comprehensive error handling
  - Built-in help system

- **Robust Search**:
  - Semantic similarity search
  - Fast response times (<20ms)
  - Relevance scoring
  - Content preview

### 🔄 In Progress (Phase 2 Extensions)

- **Extended API**: Additional endpoints for document management
- **Enhanced CLI**: Server lifecycle management
- **Documentation**: Complete usage examples and troubleshooting

### 🎯 Planned (Future Phases)

- **Web Interface**: Browser-based search UI
- **Advanced Features**: Query filters, bulk operations
- **Integrations**: IDE plugins, CI/CD integration
- **Enterprise**: Multi-tenant, authentication, monitoring

## 🔧 Configuration

### Build-time Feature Selection

Choose the optimal build for your deployment:

```bash
# Embedded deployment (default) - Single binary, no external dependencies
cargo build --release --features embedded --no-default-features

# Cloud deployment - External services integration  
cargo build --release --features cloud --no-default-features

# Development build - All features available
cargo build --release --features full
```

### Environment Variables

```bash
# API server configuration
export MDX_API_URL="http://localhost:8081"
export RUST_LOG="info"

# Vector Storage (embedded build default)
export DOC_INDEXER_VECTOR_BACKEND="embedded"
export DOC_INDEXER_EMBEDDED_DB_PATH="~/.zero-latency/vectors.db"

# Cloud features (cloud build only)
export DOC_INDEXER_VECTOR_BACKEND="qdrant"
export QDRANT_URL="http://localhost:6333"
export OPENAI_API_KEY="your-api-key"

# CLI preferences
export MDX_DEFAULT_FORMAT="table"
export MDX_DEFAULT_LIMIT="10"
```

### Database Configuration

The embedded database automatically stores vectors in:

- **Default Location**: `~/.zero-latency/vectors.db`
- **Custom Location**: Set `DOC_INDEXER_EMBEDDED_DB_PATH`
- **Cache Size**: 10,000 vectors in memory by default
- **Performance**: ~2KB per document chunk

### Model Configuration

The system automatically downloads and caches the embedding model:

- **Model**: gte-small (384 dimensions)
- **Size**: ~126MB
- **Cache Location**: `~/.cache/zero-latency/models/`
- **Performance**: <1ms inference time

## 📈 Performance

### Benchmarks

- **Search Latency**: 10-20ms (typical)
- **Indexing Speed**: ~50 documents/second
- **Memory Usage**: ~100MB base + model size
- **Index Size**: ~2KB per document chunk

### Scaling Characteristics

- **Documents**: Tested with 1,000+ documents
- **Chunks**: Efficiently handles 10,000+ chunks
- **Concurrent Users**: Supports multiple CLI users
- **Storage**: Linear scaling with document count

## 🧪 Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific component tests
cargo test --package mdx-cli
cargo test --package doc-indexer

# Integration tests
cargo test --test integration
```

### Development Setup

1. **Start development services**:
   ```bash
   docker-compose up -d qdrant
   ```

2. **Run API server in development**:
   ```bash
   cargo run --bin doc-indexer -- --verbose
   ```

3. **Test CLI commands**:
   ```bash
   cargo run --bin mdx -- search "test query"
   ```bash
   ./scripts/build.sh
   ```

### Project Structure

```text
Zero-Latency/
├── crates/
│   └── cli/              # CLI application (mdx)
├── services/
│   └── doc-indexer/      # API server & indexing engine
├── docs/                 # Documentation (search target)
├── scripts/              # Development utilities
└── docker-compose.yml    # Development services
```

## 📝 API Reference

### Search Endpoint

```bash
curl -X POST "http://localhost:8081/api/search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "embedding model",
    "limit": 5
  }'
```

Response:

```json
{
  "results": [
    {
      "document_path": "docs/model-host/artefacts/001_blueprint.md",
      "content": "Local embedding model using gte-small...",
      "score": 0.89,
      "section": "Model Architecture",
      "doc_type": "blueprint"
    }
  ],
  "total": 1,
  "took_ms": 12
}
```

### Additional Endpoints

- `GET /api/status` - System health and statistics
- `GET /api/docs` - List indexed documents
- `POST /api/reindex` - Rebuild search index
- `DELETE /api/docs/{id}` - Remove document

## 🔍 Troubleshooting

### Common Issues

#### 1. CLI shows "API server is not reachable"

```bash
# Check if server is running
mdx status

# Start the server
mdx server --start
```

#### 2. Qdrant connection failed

```bash
# Verify Qdrant is running
docker ps | grep qdrant

# Check Qdrant health
curl http://localhost:6333/health

# Start Qdrant if not running
docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant
```

#### 3. Model download issues

```bash
# Check cache directory
ls ~/.cache/zero-latency/models/

# Clear cache if corrupted
rm -rf ~/.cache/zero-latency/models/
```

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
# API server debug mode
RUST_LOG=debug cargo run --bin doc-indexer

# CLI debug mode  
RUST_LOG=debug mdx search "query"
```

## 🤝 Contributing

### Development Workflow

1. **Fork and clone** the repository
2. **Create feature branch**: `git checkout -b feature/new-feature`
3. **Run tests**: `cargo test`
4. **Submit pull request** with clear description

### Code Standards

- **Rust formatting**: Use `cargo fmt`
- **Linting**: Run `cargo clippy`
- **Documentation**: Update relevant README sections
- **Tests**: Include tests for new functionality

## 📄 License

This project is licensed under the MIT License. See LICENSE file for details.

## 🎯 Roadmap

### Phase 3: Web Interface

- Browser-based search UI
- Real-time search suggestions
- Document preview and highlighting

### Phase 4: Advanced Features

- Query filters and faceted search
- Bulk document operations
- Search analytics and insights

### Phase 5: Enterprise

- Multi-tenant support
- Authentication and authorization
- Monitoring and alerting
- High availability deployment

---

**Status**: Phase 2 CLI Interface Complete ✅  
**Next**: API Extensions & Web Interface
