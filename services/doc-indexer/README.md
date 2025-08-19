# Documentation Indexer

A Rust daemon that monitors the `/docs` directory and automatically maintains a vector database for semantic search of documentation content.

## Features

- **Real-time monitoring**: Watches for changes to markdown files in the docs directory
- **Semantic search**: Generates embeddings using OpenAI's API for advanced document retrieval
- **Vector database**: Uses Qdrant for efficient similarity search
- **Intelligent chunking**: Breaks documents into meaningful sections while preserving context
- **Document metadata**: Extracts and indexes document type, section, tags, and structure information
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

### Document Processing Pipeline

1. **File Discovery**: Recursively scans the docs directory for markdown files
2. **Content Parsing**: Extracts frontmatter, headings, and content structure
3. **Intelligent Chunking**: Splits documents into logical sections while preserving context
4. **Metadata Extraction**: Identifies document type (ADR, blueprint, whitepaper, etc.)
5. **Embedding Generation**: Creates vector embeddings using OpenAI's text-embedding-ada-002
6. **Vector Storage**: Stores embeddings and metadata in Qdrant for fast retrieval

### Real-time Monitoring

- Uses the `notify` crate for efficient file system watching
- Automatically reprocesses modified documents
- Removes deleted documents from the vector database
- Handles file renames and moves

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

### Vector Database Schema

Each document chunk is stored with the following metadata:

```json
{
  "document_id": "unique-document-identifier",
  "document_path": "/path/to/document.md",
  "document_title": "Document Title",
  "section": "Introduction",
  "doc_type": "adr",
  "tags": ["architecture", "decision"],
  "chunk_id": "chunk-identifier",
  "content": "chunk content text",
  "start_line": 1,
  "end_line": 10,
  "chunk_type": "Heading",
  "heading": "Section Title"
}
```

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
