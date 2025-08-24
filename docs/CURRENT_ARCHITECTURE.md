# Zero-Latency Document Search - Current Architecture

**Document Status**: Current as of August 24, 2025  
**Version**: v0.1.0  
**Architecture**: Clean Architecture with Collection & Document Management  

## 🏗️ System Overview

The Zero-Latency Document Search system is a Rust-based CLI and server application providing semantic document search capabilities with a clean architecture that separates concerns between:

- **Filesystem Management**: Documents sourced from filesystem
- **Indexing Operations**: Processing and vectorizing documents
- **Collection Management**: Full CRUD operations for organizing document collections
- **Document Discovery**: Read-only operations for exploring indexed content

## 📦 Core Components

### CLI Application (`mdx`)

```bash
# Main commands available
mdx search <query>           # Semantic document search
mdx index <path>            # Index documents from directory
mdx collection <operation>   # Full collection management
mdx document <operation>     # Document discovery (read-only)
mdx status                  # System health and statistics
mdx server                  # Start API server
mdx reindex                 # Rebuild entire index
```

### API Server (`doc-indexer`)

**Base URL**: `http://localhost:8081`

#### Collection Management (Full CRUD)
- `GET /collections` - List all collections
- `GET /collections/{id}` - Get collection details
- `POST /collections` - Create new collection
- `DELETE /collections/{id}` - Delete collection
- `GET /collections/{id}/stats` - Collection statistics

#### Document Discovery (Read-Only)
- `GET /documents` - List indexed documents
- `GET /documents/{id}` - Get document details

#### Indexing Operations
- `POST /api/index` - Index documents from filesystem path

## 🎯 Key Architecture Principles

### 1. Filesystem as Source of Truth
Documents exist on the filesystem and are indexed into the vector store. The system does not create virtual documents - all documents represent real files.

### 2. Clean Separation of Concerns
```
Filesystem → Indexing API → Vector Store → Discovery API
     ↓            ↓             ↓           ↓
  Real Files → Processing → Embeddings → Read-Only View
```

### 3. Collection-First Design
Collections provide organization and isolation:
- Each collection has its own vector space
- Collections can be created, managed, and deleted independently
- Documents are discovered within collection contexts

### 4. Read-Only Document Operations
Document endpoints are intentionally read-only because:
- Documents represent filesystem artifacts
- Document lifecycle is managed through filesystem + indexing
- Prevents architectural confusion about document creation/deletion

## 🔧 Technical Implementation

### Crate Structure
```
crates/
├── cli/                    # Command-line interface
├── zero-latency-config/    # Configuration management
├── zero-latency-core/      # Core domain logic
├── zero-latency-observability/ # Logging and metrics
├── zero-latency-search/    # Search algorithms
└── zero-latency-vector/    # Vector operations
```

### Services Structure
```
services/
└── doc-indexer/           # Main API server
    ├── application/       # Business logic layer
    │   └── services/      # Domain services
    ├── infrastructure/    # External adapters
    │   ├── http/         # REST API handlers
    │   ├── vector/       # Vector store adapters
    │   └── embeddings/   # Embedding generation
    └── domain/           # Core domain models
```

## 📊 Current Capabilities

### ✅ Implemented Features
- **Full Collection CRUD**: Create, read, update, delete collections
- **Document Indexing**: Process filesystem documents into vector embeddings
- **Semantic Search**: Query documents using natural language
- **Read-Only Document Discovery**: Explore indexed documents
- **Multiple Output Formats**: JSON, table, YAML formatting
- **Health Monitoring**: System status and collection statistics
- **Clean Architecture**: Proper separation of concerns

### 🔄 Workflow Examples

#### Index and Search Workflow
```bash
# 1. Start the server
mdx server

# 2. Create a collection
mdx collection create my-docs

# 3. Index documents
mdx index /path/to/documents --collection my-docs

# 4. Search documents
mdx search "machine learning concepts" --collection my-docs

# 5. List indexed documents
mdx document list --collection my-docs
```

#### Collection Management Workflow
```bash
# List all collections
mdx collection list

# Get collection details
mdx collection get my-docs

# View collection statistics
mdx collection stats my-docs

# Delete collection
mdx collection delete my-docs
```

## 🎨 Design Patterns

### Command Query Responsibility Segregation (CQRS)
- **Commands**: Indexing operations, collection management
- **Queries**: Document discovery, search operations

### Repository Pattern
- `CollectionService`: Manages collection lifecycle
- `DocumentService`: Handles document indexing and discovery
- `VectorRepository`: Abstracts vector store operations

### Clean Architecture Layers
1. **Presentation**: CLI commands, HTTP handlers
2. **Application**: Business logic, use cases
3. **Domain**: Core entities, value objects
4. **Infrastructure**: External systems, databases

## 🔍 API Specifications

### Collection API

#### Create Collection
```http
POST /collections
Content-Type: application/json

{
  "name": "my-collection",
  "description": "My document collection"
}
```

#### List Collections
```http
GET /collections

Response:
{
  "collections": [
    {
      "id": "my-collection",
      "name": "my-collection", 
      "description": "My document collection",
      "created_at": "2025-08-24T10:00:00Z",
      "document_count": 42
    }
  ]
}
```

### Document API

#### List Documents
```http
GET /documents?collection=my-collection

Response:
{
  "documents": [
    {
      "id": "doc-123",
      "title": "Machine Learning Basics",
      "path": "/docs/ml-basics.md",
      "size": 2048,
      "indexed_at": "2025-08-24T10:30:00Z"
    }
  ]
}
```

#### Get Document
```http
GET /documents/doc-123

Response:
{
  "id": "doc-123",
  "title": "Machine Learning Basics",
  "content": "# Machine Learning Basics\n\n...",
  "metadata": {
    "path": "/docs/ml-basics.md",
    "size": 2048,
    "format": "markdown"
  }
}
```

## 🚀 Deployment Options

### Development
```bash
cargo run --bin doc-indexer -- --api-server --api-port 8081
```

### Production
```bash
cargo build --release
./target/release/doc-indexer --api-server --api-port 8081 --docs-path /data/docs
```

### CLI Usage
```bash
# Install CLI
cargo install --path crates/cli

# Use anywhere
mdx search "query" --server http://production-server:8081
```

## 📈 Performance Characteristics

- **Vector Search**: Sub-100ms response times for collections under 10K documents
- **Indexing Speed**: ~100 documents/second (varies by document size)
- **Memory Usage**: ~2GB RAM for 50K document collection
- **Storage**: Vector embeddings ~1.5KB per document

## 🔗 Related Documentation

- [Installation Guide](../README.md)
- [CLI Reference](CLI_REFERENCE.md)
- [API Reference](API_REFERENCE.md)
- [Architecture Decisions](adr/)
- [Implementation Notes](implementation/)
