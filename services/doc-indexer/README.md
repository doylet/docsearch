# Documentation Indexer

A Rust daemon that monitors the `/docs` directory and automatically maintains a vector database for semantic search of documentation content.

## Features

- **Real-time monitoring**: Watches for changes to markdown files with intelligent event coalescing
- **Content versioning**: SHA256-based stable document IDs with xxHash content revision tracking
- **Smart change detection**: Only reprocesses documents when content actually changes
- **Event coalescing**: 300ms debouncing prevents file system thrashing during rapid changes
- **Semantic search**: Generates embeddings using OpenAI's API for advanced document retrieval
- **Vector database**: Uses Qdrant for efficient similarity search with tombstone deletion semantics
- **Enhanced chunking**: Byte-offset precise chunking with heading breadcrumb paths
- **Document metadata**: Extracts and indexes document type, section, tags, and structure information
- **Schema versioning**: Built-in migration support for future enhancements
- **Production-ready**: Thread-safe operations with comprehensive error handling
- **Configurable**: Supports various configuration options via CLI arguments and environment variables

## Prerequisites

1. **Qdrant Vector Database**: The indexer requires a running Qdrant instance
   ```bash
   # Using Docker
   docker run -p 6333:6333 qdrant/qdrant
   
   # Or using Docker Compose (see docker-compose.yml in project root)
   docker-compose up qdrant
   ```

2. **OpenAI API Key**: Required for generating embeddings
   ```bash
   export OPENAI_API_KEY="your-api-key-here"
   ```

## Installation

```bash
# Build the indexer
cargo build --release

# Or run directly with cargo
cargo run -- --help
```

## Usage

### Basic Usage

```bash
# Index all documents once and start watching for changes
./target/release/doc-indexer

# Index only (don't watch for changes)
./target/release/doc-indexer --index-only

# Custom docs path and Qdrant URL
./target/release/doc-indexer \
  --docs-path /path/to/docs \
  --qdrant-url http://localhost:6333 \
  --collection-name my_docs
```

### Command Line Options

- `--docs-path`: Path to documentation directory (default: `./docs`)
- `--qdrant-url`: Qdrant server URL (default: `http://localhost:6333`)
- `--collection-name`: Vector database collection name (default: `zero_latency_docs`)
- `--openai-api-key`: OpenAI API key (can also use `OPENAI_API_KEY` env var)
- `--index-only`: Perform initial indexing then exit
- `--verbose`: Enable debug logging

## Architecture

### Enhanced Document Processing Pipeline

1. **File Discovery**: Recursively scans the docs directory for markdown files
2. **Content Versioning**: Generates stable SHA256 document IDs and xxHash content revisions
3. **Change Detection**: Only processes documents when content revision changes
4. **Content Parsing**: Extracts frontmatter, headings, and content structure
5. **Enhanced Chunking**: Splits documents with precise byte offsets and heading breadcrumb paths
6. **Metadata Extraction**: Identifies document type (ADR, blueprint, whitepaper, etc.) with proper timestamps
7. **Embedding Generation**: Creates vector embeddings using OpenAI's text-embedding-ada-002
8. **Vector Storage**: Stores embeddings and metadata in Qdrant with tombstone deletion semantics

### Real-time Monitoring with Event Coalescing

- Uses the `notify` crate for efficient file system watching
- **Event Coalescing**: 300ms debouncing prevents rapid-fire processing thrashing
- **Smart Aggregation**: HashMap-based event coalescence handles multiple rapid changes
- Automatically reprocesses modified documents only when content changes
- Safely removes deleted documents using tombstone semantics
- Handles file renames and moves with stable document ID tracking

### Document Types Supported

The indexer recognizes and categorizes these document types:

- **ADR (Architecture Decision Records)**: Technical decisions and their rationale
- **Blueprints**: Technical design documents and specifications
- **Whitepapers**: Research and analysis documents
- **Roadmaps**: Planning and timeline documents
- **Reviews**: Assessment and evaluation documents
- **Generic**: Any other markdown documentation

## Configuration

### Environment Variables

- `OPENAI_API_KEY`: Required for embedding generation
- `RUST_LOG`: Controls logging level (e.g., `doc_indexer=debug`)

### Enhanced Vector Database Schema

Each document chunk is stored with comprehensive metadata and versioning:

```json
{
  "doc_id": "sha256-hash-of-absolute-path",
  "rev_id": "xxhash-of-content",
  "document_path": "/absolute/path/to/document.md",
  "relative_path": "docs/adr/001_example.md",
  "document_title": "Document Title",
  "section": "Introduction",
  "doc_type": "adr",
  "tags": ["architecture", "decision"],
  "chunk_id": "doc-id:00001",
  "content": "chunk content text",
  "start_byte": 1024,
  "end_byte": 2048,
  "chunk_index": 0,
  "chunk_total": 5,
  "chunk_type": "Heading",
  "h_path": ["Section Title", "Subsection"],
  "created_at": "2024-01-01T12:00:00Z",
  "updated_at": "2024-01-01T12:00:00Z",
  "schema_version": 1,
  "embedding_model": "text-embedding-ada-002"
}
```

### Document Registry and Versioning

- **Stable IDs**: SHA256 of absolute path ensures consistent document identity
- **Content Tracking**: xxHash revision IDs detect actual content changes
- **Tombstone Semantics**: Deleted documents are marked, not removed
- **Migration Support**: Schema versioning enables future enhancements

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_document_processing
```

### Local Development Setup

1. Start Qdrant locally:
   ```bash
   docker run -p 6333:6333 qdrant/qdrant
   ```

2. Set up environment:
   ```bash
   export OPENAI_API_KEY="your-key"
   export RUST_LOG="doc_indexer=debug"
   ```

3. Run the indexer:
   ```bash
   cargo run -- --verbose --docs-path ./docs
   ```

## Monitoring and Observability

The indexer provides structured logging with the following information:

- Document indexing progress and statistics
- File system events (creates, modifications, deletions)
- Vector database operations and performance
- Error handling and recovery

Example log output:
```
2024-01-01T12:00:00Z INFO  Starting Zero Latency Documentation Indexer
2024-01-01T12:00:00Z INFO  Monitoring: ./docs
2024-01-01T12:00:00Z INFO  Vector DB: http://localhost:6333 (collection: zero_latency_docs)
2024-01-01T12:00:01Z INFO  Initial indexing complete. Indexed: 15, Errors: 0
2024-01-01T12:00:01Z INFO  Collection 'zero_latency_docs' now contains 342 points (342 vectors)
2024-01-01T12:00:01Z INFO  Started watching directory: ./docs
```

## Troubleshooting

### Common Issues

1. **Qdrant Connection Failed**
   - Ensure Qdrant is running and accessible
   - Check the `--qdrant-url` parameter
   - Verify network connectivity

2. **OpenAI API Errors**
   - Verify your API key is valid and has sufficient credits
   - Check rate limits if processing many documents
   - Ensure network access to OpenAI's API

3. **File Permission Issues**
   - Ensure read access to the docs directory
   - Check that the indexer can create the collection in Qdrant

4. **Memory Usage**
   - Large documents may require significant memory for processing
   - Consider chunking very large files or increasing available memory

### Performance Tuning

- **Batch Processing**: The indexer processes documents sequentially to avoid rate limits
- **Embedding Caching**: Consider implementing local caching for repeated content
- **Qdrant Configuration**: Tune Qdrant settings based on your dataset size and query patterns

## License

This project is part of the Zero-Latency framework. See the project root for license information.
