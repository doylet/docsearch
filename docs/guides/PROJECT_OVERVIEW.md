# Zero-Latency Document Search - Project Overview

**Last Updated**: December 2025
**Status**: Production-Ready
**Architecture**: Clean Architecture with Schema-First Contracts

---

## 🎯 What Is This Project?

**Zero-Latency Document Search** is a high-performance semantic search system for documentation that uses:
- **Local embeddings** (no API keys required)
- **Embedded vector storage** (SQLite-based, no external database needed)
- **Real-time indexing** (watches filesystem for changes)
- **Semantic search** (understands meaning, not just keywords)

It's built with **Rust** for the backend/CLI and **Next.js** for the web frontend, following clean architecture principles.

---

## 🏗️ Architecture Overview

### High-Level Components

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Frontend  │────▶│  API Server  │────▶│  Vector DB  │
│  (Next.js)  │     │ (doc-indexer)│     │  (SQLite)   │
└─────────────┘     └──────────────┘     └─────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │   Embeddings │
                    │ (ONNX Model) │
                    └──────────────┘
```

### Three Main Parts

1. **CLI Tool (`mdx`)** - Command-line interface for indexing and searching
2. **API Server (`doc-indexer`)** - HTTP API server that handles indexing and search requests
3. **Web Frontend** - Next.js application for browser-based search

---

## 📁 Project Structure

### Monorepo Organization

```
docsearch/
├── apps/
│   ├── frontend/          # Next.js web application
│   └── backend/           # Turborepo wrapper for Rust backend
│
├── crates/                # Shared Rust libraries
│   ├── cli/              # CLI application (mdx command)
│   ├── zero-latency-core/        # Core domain models
│   ├── zero-latency-search/      # Search algorithms
│   ├── zero-latency-vector/      # Vector operations
│   ├── zero-latency-config/      # Configuration management
│   ├── zero-latency-observability/ # Logging & metrics
│   ├── zero-latency-contracts/    # API contracts
│   └── zero-latency-api/          # OpenAPI client generation
│
├── services/
│   └── doc-indexer/       # Main API server & indexing engine
│       ├── application/   # Business logic layer
│       ├── infrastructure/ # External adapters (HTTP, vector DB)
│       └── domain/        # Core domain models
│
├── docs/                  # Comprehensive documentation
│   ├── architecture/     # Architecture decisions
│   ├── milestones/       # Development milestones
│   ├── implementation/   # Implementation guides
│   └── adr/              # Architecture Decision Records
│
└── api/                   # API specifications
    ├── public/           # OpenAPI specs for public API
    └── internal/         # gRPC specs for internal services
```

---

## 🔑 Key Concepts

### 1. **Collections**
- Organize documents into named collections
- Each collection has its own vector space
- Full CRUD operations (create, read, update, delete)

### 2. **Documents**
- Represent real files on the filesystem
- Read-only discovery (documents come from filesystem, not API)
- Automatically indexed when files change

### 3. **Semantic Search**
- Uses local ONNX model (`gte-small`, 384 dimensions)
- Understands meaning, not just keywords
- Fast response times (<100ms typical)

### 4. **Embedded Storage**
- SQLite-based vector database
- No external dependencies (no Qdrant needed)
- Self-contained binary deployment

---

## 🚀 How It Works

### Indexing Flow

```
1. File System Watcher detects change
   ↓
2. Document Parser extracts content & metadata
   ↓
3. Chunker splits into searchable chunks
   ↓
4. Embedding Generator creates vectors (local ONNX)
   ↓
5. Vector Store saves embeddings + metadata
```

### Search Flow

```
1. User enters query
   ↓
2. Query converted to embedding vector
   ↓
3. Vector similarity search in database
   ↓
4. Results ranked by relevance score
   ↓
5. Results returned with snippets & metadata
```

---

## 🛠️ Technology Stack

### Backend (Rust)
- **Framework**: Axum (HTTP server)
- **Embeddings**: ONNX Runtime (local inference)
- **Vector DB**: SQLite (embedded) or Qdrant (optional)
- **File Watching**: `notify` crate
- **CLI**: `clap` framework

### Frontend (TypeScript/React)
- **Framework**: Next.js 16 (App Router)
- **State**: React Query (server state), Zustand (client state)
- **Styling**: Tailwind CSS 4
- **Type Safety**: TypeScript 5 (strict mode)

### Build System
- **Monorepo**: Turborepo (caching & parallel builds)
- **Rust Workspace**: Cargo workspaces
- **Package Manager**: npm (for frontend), Cargo (for Rust)

---

## 📊 Current Status

### ✅ Production Ready Features

- **Collection Management**: Full CRUD operations
- **Document Indexing**: Real-time filesystem monitoring
- **Semantic Search**: Natural language queries
- **CLI Interface**: Complete command-line tool
- **REST API**: Comprehensive HTTP API
- **Web Frontend**: Next.js search interface
- **Clean Architecture**: Proper layer separation

### 🔄 Known Issues

- Collection metadata missing in some search results
- Document ID preservation in MCP interface
- CLI collection filtering returns no results (medium priority)

See `docs/issues/` for detailed issue tracking.

---

## 🎯 Design Principles

### 1. **Clean Architecture**
- Domain layer (business logic)
- Application layer (use cases)
- Infrastructure layer (external adapters)
- Presentation layer (UI/CLI)

### 2. **Schema-First Contracts**
- OpenAPI specs define API contracts
- Type generation from schemas
- Contract validation in CI/CD

### 3. **Filesystem as Source of Truth**
- Documents exist on filesystem
- Indexing is a derived state
- No virtual document creation

### 4. **Collection-First Design**
- Collections organize documents
- Each collection is isolated
- Documents discovered within collection context

---

## 📖 Key Documentation

### Getting Started
- **[README.md](./README.md)** - Installation and quick start
- **[MONOREPO.md](./MONOREPO.md)** - Monorepo structure and workflows
- **[DOCKER.md](./DOCKER.md)** - Docker setup and usage

### Architecture
- **[docs/CURRENT_ARCHITECTURE.md](./docs/CURRENT_ARCHITECTURE.md)** - Current system design
- **[docs/INDEX.md](./docs/INDEX.md)** - Documentation index
- **[docs/adr/](./docs/adr/)** - Architecture Decision Records

### Reference
- **[docs/CLI_REFERENCE.md](./docs/CLI_REFERENCE.md)** - CLI command reference
- **[docs/API_REFERENCE.md](./docs/API_REFERENCE.md)** - REST API specification

### Development
- **[docs/milestones/](./docs/milestones/)** - Development milestones
- **[docs/implementation/](./docs/implementation/)** - Implementation guides

---

## 🔧 Common Tasks

### Start Development Environment

```bash
# Start all services with Docker
make docker-up

# Or start individually
cd services/doc-indexer && cargo run
cd apps/frontend && npm run dev
```

### Index Documents

```bash
# Using CLI
mdx index /path/to/docs --collection my-docs

# Or via API
curl -X POST http://localhost:8081/api/index \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/docs", "collection": "my-docs"}'
```

### Search Documents

```bash
# Using CLI
mdx search "machine learning" --collection my-docs

# Or via API
curl "http://localhost:8081/api/search?q=machine+learning&collection=my-docs"
```

### Build Project

```bash
# Build everything (Turborepo)
npm run build

# Build Rust only
cargo build --release

# Build frontend only
cd apps/frontend && npm run build
```

---

## 🎓 Learning Path

### For New Developers

1. **Start Here**: Read [README.md](./README.md) and this overview
2. **Understand Architecture**: Review [docs/CURRENT_ARCHITECTURE.md](./docs/CURRENT_ARCHITECTURE.md)
3. **Try the CLI**: Run `mdx help` and explore commands
4. **Explore Code**: Start with `crates/cli/src/main.rs` and `services/doc-indexer/src/main.rs`
5. **Read ADRs**: Check [docs/adr/](./docs/adr/) for design decisions

### For Frontend Developers

1. Read [apps/frontend/README.md](./apps/frontend/README.md)
2. Understand clean architecture layers in `apps/frontend/`
3. Review API contracts in `api/public/openapi.yaml`
4. Check component structure in `apps/frontend/presentation/`

### For Backend Developers

1. Read [services/doc-indexer/README.md](./services/doc-indexer/README.md)
2. Understand crate structure in `crates/`
3. Review domain models in `zero-latency-core/`
4. Check API handlers in `services/doc-indexer/infrastructure/http/`

---

## 🚦 Project Status Indicators

- ✅ **Production Ready**: Core features complete and tested
- 🔄 **In Progress**: Active development on enhancements
- 📋 **Planned**: Features in roadmap
- 🐛 **Known Issues**: Documented problems being tracked

---

## 📝 Quick Reference

### Main Commands

```bash
# CLI
mdx search <query>              # Search documents
mdx index <path>                # Index documents
mdx collection list             # List collections
mdx status                      # System health

# Development
make docker-up                  # Start all services
make turbo-build               # Build everything
cargo test                     # Run tests
npm run dev                    # Start frontend dev server
```

### Key Files

- `Cargo.toml` - Rust workspace configuration
- `package.json` - Node.js workspace configuration
- `turbo.json` - Turborepo pipeline configuration
- `docker-compose.yml` - Docker services
- `api/public/openapi.yaml` - Public API specification

---

## 🤝 Contributing

This project follows clean architecture principles and schema-first contract design. See:
- [docs/architecture/](./docs/architecture/) for architecture patterns
- [docs/adr/](./docs/adr/) for design decisions
- [docs/issues/](./docs/issues/) for known issues

---

**Need Help?** Check the [Documentation Index](./docs/INDEX.md) for comprehensive guides and references.
