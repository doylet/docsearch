# Zero-Latency Documentation Search

A high-performance semantic search system for documentation using local embeddings and vector similarity search. Built with Rust for speed and reliability.

## 🚀 Quick Start

### Prerequisites

1. **Start Qdrant Vector Database**:
   ```bash
   docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant
   ```

2. **Build the CLI**:
   ```bash
   cargo build --release
   ```

### Using the CLI

The `mdx` command provides an intuitive interface for document search:

```bash
# Search for documents
mdx search "embedding model"

# Check system status
mdx status

# Index documents from a directory
mdx index /path/to/docs

# Rebuild the entire index
mdx reindex

# Start the API server
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

## � Project Status

### ✅ Phase 4C: Clean Architecture - COMPLETED (August 21, 2025)

**Major Milestone Achieved!** Successfully implemented enterprise-grade clean architecture with:

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

- **Production-Ready Features**:
  - Vector storage adapters (Qdrant + In-Memory)
  - Embedding adapters (OpenAI + Local deterministic)
  - HTTP REST API with comprehensive error handling
  - Health monitoring (readiness/liveness checks)
  - Configuration-driven adapter selection

**Build Status**: ✅ Successful compilation  
**Architecture Validation**: ✅ All SOLID principles implemented  
**Documentation**: [Phase 4C Implementation Details](./docs/milestones/Phase_4C_Clean_Architecture_Implementation.md)

### 🚀 Next Phase: Phase 4D Service Extension (Starting August 21, 2025)

**Decision**: [ADR-038 Phase 4D Service Extension Strategy](./docs/misc/artefacts/038_adr-phase-4d-service-extension.md)

**Objectives**:
- Apply clean architecture patterns to all remaining services
- Implement comprehensive integration testing across services  
- Deploy production-grade observability and monitoring
- Establish operational excellence patterns for scaling

## �🏗 Architecture

### System Components

```
mdx CLI ──HTTP──> API Server ──gRPC──> Qdrant Vector DB
   │                   │                      │
   │                   └─> Local Embedder ────┘
   │                       (gte-small ONNX)
   │
   └─> Local Files & Configuration
```

### Technology Stack

- **CLI**: Rust with `clap` framework
- **HTTP API**: Axum web framework  
- **Embeddings**: Local ONNX model (gte-small, 384 dimensions)
- **Vector DB**: Qdrant for similarity search
- **File Monitoring**: Real-time document change detection

## 🛠 Installation

### From Source

```bash
# Clone repository
git clone <repo-url>
cd Zero-Latency

# Build CLI and API server
cargo build --release

# CLI available at ./target/release/mdx
# API server at ./target/release/doc-indexer
```

### Docker Setup

```bash
# Start Qdrant vector database
docker run -d \
  --name qdrant \
  -p 6333:6333 \
  -p 6334:6334 \
  qdrant/qdrant

# Verify Qdrant is running
curl http://localhost:6333/health
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

### Environment Variables

```bash
# API server configuration
export MDX_API_URL="http://localhost:8081"
export QDRANT_URL="http://localhost:6333"
export RUST_LOG="info"

# CLI preferences
export MDX_DEFAULT_FORMAT="table"
export MDX_DEFAULT_LIMIT="10"
```

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
   ```

### Project Structure

```
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

**1. CLI shows "API server is not reachable"**
```bash
# Check if server is running
mdx status

# Start the server
mdx server --start

# Or start manually
cargo run --bin doc-indexer
```

**2. Qdrant connection failed**
```bash
# Verify Qdrant is running
docker ps | grep qdrant

# Check Qdrant health
curl http://localhost:6333/health

# Start Qdrant if not running
docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant
```

**3. Model download issues**
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
