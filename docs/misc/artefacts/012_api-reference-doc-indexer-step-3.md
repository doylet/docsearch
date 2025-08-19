# Doc-Indexer Step 3: API Reference and Usage Guide

## Table of Contents

1. [HTTP API Reference](#http-api-reference)
2. [CLI Reference](#cli-reference)
3. [Configuration](#configuration)
4. [Development Guide](#development-guide)
5. [Troubleshooting](#troubleshooting)

## HTTP API Reference

### Base URL

Default: `http://127.0.0.1:3000`

### Authentication

Currently no authentication required. OpenAI API key is configured server-side.

### Content Type

All requests and responses use `application/json`.

### Endpoints

#### Search Documents

**POST** `/api/search`

Search for documents using semantic similarity.

**Request Body:**

```json
{
  "query": "string",              // Required: Search query text
  "k": 10,                        // Optional: Number of results (default: 10, max: 100)
  "filters": {                    // Optional: Filter criteria
    "path_contains": ["string"],  // Filter by file path substrings
    "tags": ["string"],          // Filter by document tags
    "date_after": "2025-01-01",  // Filter by creation/modification date
    "date_before": "2025-12-31"  // Filter by creation/modification date
  },
  "include_snippets": true        // Optional: Include highlighted snippets (default: true)
}
```

**Response:**

```json
{
  "query": "vector database implementation",
  "total_results": 5,
  "results": [
    {
      "document_id": "doc_abc123",
      "title": "Vector Database Implementation Guide",
      "score": 0.92,
      "snippet": "The **vector database** implementation uses Qdrant...",
      "heading_path": ["Architecture", "Database Layer", "Implementation"],
      "file_path": "./docs/architecture/database.md",
      "metadata": {
        "tags": ["rust", "database", "qdrant"],
        "last_updated": "2025-08-19T10:30:00Z",
        "author": "thomas.doyle",
        "word_count": 1250
      }
    }
  ],
  "search_metadata": {
    "embedding_time_ms": 45,
    "search_time_ms": 12,
    "filter_time_ms": 3,
    "total_time_ms": 60,
    "model_used": "text-embedding-3-small",
    "filtered_from": 150,
    "ranked_results": 25
  }
}
```

**Error Responses:**

```json
// Bad Request (400)
{
  "error": "Invalid request format",
  "details": "Field 'query' is required",
  "timestamp": "2025-08-19T12:30:00Z"
}

// Internal Server Error (500)
{
  "error": "Search operation failed",
  "details": "Vector database connection timeout",
  "timestamp": "2025-08-19T12:30:00Z"
}

// Rate Limited (429)
{
  "error": "Rate limit exceeded",
  "details": "OpenAI API rate limit reached. Retry after 60 seconds.",
  "timestamp": "2025-08-19T12:30:00Z"
}
```

#### Health Check

**GET** `/api/health`

Check system health and component status.

**Response:**

```json
{
  "status": "healthy",
  "components": {
    "vector_database": {
      "status": "healthy",
      "details": "Connected to Qdrant at localhost:6333",
      "response_time_ms": 5
    },
    "embedding_provider": {
      "status": "healthy", 
      "details": "OpenAI API responding normally",
      "rate_limit_remaining": 45
    },
    "search_service": {
      "status": "healthy",
      "details": "All components operational"
    }
  },
  "timestamp": "2025-08-19T12:15:00Z",
  "uptime_seconds": 3600,
  "version": "0.3.0"
}
```

**Degraded Health Example:**

```json
{
  "status": "degraded",
  "components": {
    "vector_database": {
      "status": "healthy",
      "details": "Connected to mock database",
      "response_time_ms": 1
    },
    "embedding_provider": {
      "status": "degraded",
      "details": "Using mock embedder - no OpenAI API key provided",
      "rate_limit_remaining": null
    },
    "search_service": {
      "status": "healthy",
      "details": "Operating with mock components"
    }
  },
  "timestamp": "2025-08-19T12:15:00Z"
}
```

#### API Information

**GET** `/`

Get basic API information and documentation links.

**Response:**

```json
{
  "name": "Zero Latency Documentation Search API",
  "version": "0.3.0",
  "description": "Semantic search API for Zero Latency documentation",
  "endpoints": {
    "search": "POST /api/search",
    "health": "GET /api/health"
  },
  "documentation": "https://github.com/your-org/zero-latency/docs"
}
```

## CLI Reference

### Basic Usage

```bash
doc-indexer [OPTIONS]
```

### Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--docs-path` | | Path | `./docs` | Path to documentation directory |
| `--qdrant-url` | | String | `http://localhost:6333` | Qdrant server URL |
| `--collection-name` | | String | `zero_latency_docs` | Qdrant collection name |
| `--openai-api-key` | | String | | OpenAI API key (or use OPENAI_API_KEY env var) |
| `--index-only` | | Flag | | Run indexing then exit (no watching) |
| `--api-server` | | Flag | | Start HTTP API server mode |
| `--api-port` | | Number | `3000` | HTTP API server port |
| `--verbose` | `-v` | Flag | | Enable verbose logging |
| `--help` | `-h` | Flag | | Show help information |

### Usage Examples

#### Traditional Indexing

```bash
# Monitor ./docs and index changes
doc-indexer

# Monitor custom path with verbose output
doc-indexer --docs-path /path/to/docs --verbose

# Index once and exit
doc-indexer --index-only
```

#### API Server Mode

```bash
# Start API server with defaults
doc-indexer --api-server

# Custom port and verbose logging
doc-indexer --api-server --api-port 8080 --verbose

# With custom Qdrant instance
doc-indexer --api-server --qdrant-url http://production-qdrant:6333

# Mock mode for testing (no Qdrant required)
doc-indexer --api-server --qdrant-url "mock://localhost:6333"
```

#### With OpenAI Integration

```bash
# Using environment variable
export OPENAI_API_KEY=sk-your-key-here
doc-indexer --api-server

# Using command line argument
doc-indexer --api-server --openai-api-key sk-your-key-here
```

## Configuration

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `OPENAI_API_KEY` | OpenAI API key for embeddings | `sk-abc123...` |
| `RUST_LOG` | Logging configuration | `doc_indexer=debug,warn` |

### Mock vs Production Mode

The system automatically switches between mock and production mode based on configuration:

**Mock Mode** (for development/testing):
- Use `--qdrant-url "mock://localhost:6333"`
- No external dependencies required
- Uses deterministic mock embeddings
- Instant responses for testing

**Production Mode**:
- Use real Qdrant URL: `--qdrant-url "http://localhost:6333"`
- Requires running Qdrant instance
- Requires OpenAI API key for real embeddings
- Real vector similarity search

## Development Guide

### Building and Running

```bash
# Build debug version
cargo build

# Build release version
cargo build --release

# Run with cargo
cargo run -- --api-server --verbose

# Run tests
cargo test

# Check code without building
cargo check
```

### Testing API Endpoints

```bash
# Health check
curl -s http://127.0.0.1:3000/api/health | jq .

# Basic search
curl -s -X POST http://127.0.0.1:3000/api/search \
  -H "Content-Type: application/json" \
  -d '{"query": "rust implementation", "k": 5}' | jq .

# Advanced search with filters
curl -s -X POST http://127.0.0.1:3000/api/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "vector database",
    "k": 10,
    "filters": {
      "path_contains": ["architecture"],
      "tags": ["rust"],
      "date_after": "2025-01-01"
    },
    "include_snippets": true
  }' | jq .
```

### Mock Development Setup

For development without external dependencies:

```bash
# Start in mock mode
doc-indexer --api-server --qdrant-url "mock://localhost:6333" --verbose

# Mock mode features:
# - No Qdrant required
# - Deterministic embeddings
# - Instant responses
# - All API endpoints functional
```

### Adding New Features

1. **Embedding Providers**: Implement the `EmbeddingProvider` trait
2. **Search Filters**: Extend `SearchFilters` struct in `search_service.rs`
3. **API Endpoints**: Add routes in `api_server.rs`
4. **CLI Options**: Update `Cli` struct in `main.rs`

## Troubleshooting

### Common Issues

#### 1. "Failed to initialize Qdrant vector database"

**Cause**: Qdrant server not running or URL incorrect

**Solutions**:
- Start Qdrant: `docker run -p 6333:6333 qdrant/qdrant`
- Use mock mode: `--qdrant-url "mock://localhost:6333"`
- Check URL: `--qdrant-url "http://your-qdrant:6333"`

#### 2. "No OpenAI API key provided, using mock embedder"

**Cause**: OpenAI API key not configured

**Solutions**:
- Set environment variable: `export OPENAI_API_KEY=sk-...`
- Use CLI argument: `--openai-api-key sk-...`
- Continue with mock embedder for testing

#### 3. "Rate limit exceeded"

**Cause**: OpenAI API rate limits reached

**Solutions**:
- Wait for rate limit reset (typically 1 minute)
- Use smaller batch sizes
- Switch to mock mode for development

#### 4. "Port already in use"

**Cause**: API server port conflicts

**Solutions**:
- Use different port: `--api-port 8080`
- Kill existing process: `lsof -ti:3000 | xargs kill`

#### 5. Search returns no results

**Causes and Solutions**:
- **No documents indexed**: Check `--docs-path` points to correct directory
- **Mock mode**: Expected behavior - mock database has no real content
- **Filters too restrictive**: Remove or adjust filter criteria
- **Vector database empty**: Run initial indexing first

### Logging and Debugging

Enable verbose logging to diagnose issues:

```bash
# Verbose output
doc-indexer --api-server --verbose

# Debug level logging
RUST_LOG=doc_indexer=debug doc-indexer --api-server

# Trace all operations
RUST_LOG=trace doc-indexer --api-server
```

### Performance Monitoring

Monitor API performance:

```bash
# Check response times
curl -w "@curl-format.txt" -s -X POST http://127.0.0.1:3000/api/search \
  -H "Content-Type: application/json" \
  -d '{"query": "test", "k": 5}'

# Health check with timing
time curl -s http://127.0.0.1:3000/api/health
```

Where `curl-format.txt` contains:
```
     time_namelookup:  %{time_namelookup}\n
        time_connect:  %{time_connect}\n
     time_appconnect:  %{time_appconnect}\n
    time_pretransfer:  %{time_pretransfer}\n
       time_redirect:  %{time_redirect}\n
  time_starttransfer:  %{time_starttransfer}\n
                     ----------\n
          time_total:  %{time_total}\n
```

### Getting Help

1. **CLI Help**: `doc-indexer --help`
2. **Health Check**: `GET /api/health` for system status  
3. **Logs**: Enable `--verbose` for detailed operation logs
4. **Mock Mode**: Use `--qdrant-url "mock://localhost:6333"` for isolated testing
