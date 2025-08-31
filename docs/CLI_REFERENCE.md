# CLI Reference - mdx

**Command Line Interface for Zero-Latency Document Search**  
**Version**: v0.1.0  
**Updated**: August 24, 2025  

## Installation

```bash
# Build from source
git clone <repository>
cd docsearch
cargo build --release

# The CLI binary will be available at:
./target/release/mdx
```

## Global Options

All commands support these global options:

| Option | Description | Default |
|--------|-------------|---------|
| `-v, --verbose` | Enable verbose output | `false` |
| `--server <URL>` | API server URL | `http://localhost:8081` |
| `--collection <NAME>` | Collection name | `zero_latency_docs` |
| `--config <PATH>` | Configuration file path | None |
| `-h, --help` | Show help information | - |
| `-V, --version` | Show version information | - |

## Commands Overview

```bash
mdx [OPTIONS] <COMMAND>

Commands:
  search      Search documents with semantic similarity
  index       Index documents from a directory  
  document    Document discovery operations (list, get)
  collection  Collection management operations (list, get, create, delete, stats)
  status      Show collection statistics and health
  server      Start the API server
  reindex     Rebuild the entire index
  help        Print help for commands
```

## 🔍 Search Command

Search for documents using natural language queries.

### Usage
```bash
mdx search [OPTIONS] <QUERY>
```

### Options
| Option | Description | Default |
|--------|-------------|---------|
| `<QUERY>` | Search query text | Required |
| `--limit <N>` | Maximum results to return | `10` |
| `--threshold <F>` | Minimum similarity score | `0.5` |
| `--format <FORMAT>` | Output format: json, table, yaml | `table` |

**Global Options for Search:**
| Option | Description | Default | Notes |
|--------|-------------|---------|-------|
| `--collection <NAME>` | Filter search to specific collection | `zero_latency_docs` | Use for targeted searches |
| `--server <URL>` | API server URL | `http://localhost:8081` | Override default server |

### Collection Filtering

Collection filtering allows you to search within a specific collection of documents, providing more targeted and relevant results.

#### How Collection Filtering Works
- **Purpose**: Narrow search scope to documents from a specific collection
- **Performance**: Faster searches when you know the target collection
- **Relevance**: More accurate results from related documents

#### Collection Filtering Examples
```bash
# Search only in documentation collection
mdx search "API endpoints" --collection zero_latency_docs

# Search in tutorial collection with higher threshold
mdx search "getting started" --collection tutorials --threshold 0.8

# Search across multiple queries in same collection
mdx --collection api-docs search "authentication"
mdx --collection api-docs search "rate limiting"

# Search with collection and output formatting
mdx search "configuration" --collection config-docs --format json --limit 20
```

#### Available Collections
To see available collections, use:
```bash
# List all collections
mdx collection list

# Get specific collection info
mdx collection get zero_latency_docs
```

#### Collection vs Default Search Comparison
```bash
# Default search (searches all collections)
mdx search "test methods"
# Result: May return results from docs, tutorials, API references, etc.

# Collection-filtered search (searches specific collection)
mdx search "test methods" --collection zero_latency_docs  
# Result: Only returns results from zero_latency_docs collection
```

### Examples
```bash
# Basic search
mdx search "machine learning algorithms"

# Search in specific collection with limit
mdx search "python tutorials" --collection tutorials --limit 5

# Search with custom threshold
mdx search "api documentation" --threshold 0.7

# JSON output for scripting
mdx search "configuration" --format json
```

### Output Formats

#### Table Format (Default)
```
┌─────────────────────────────────────┬─────────┬──────────────────────┬──────────────────────────────────────┐
│ Title                               │ Score   │ Path                 │ Summary                              │
├─────────────────────────────────────┼─────────┼──────────────────────┼──────────────────────────────────────┤
│ Machine Learning Introduction       │ 0.89    │ /docs/ml/intro.md    │ Comprehensive guide to ML concepts  │
│ ML Algorithms Overview              │ 0.82    │ /docs/ml/algos.md    │ Survey of common algorithms          │
└─────────────────────────────────────┴─────────┴──────────────────────┴──────────────────────────────────────┘
```

#### JSON Format
```json
{
  "query": "machine learning algorithms",
  "results": [
    {
      "id": "doc-123",
      "title": "Machine Learning Introduction",
      "score": 0.89,
      "path": "/docs/ml/intro.md",
      "summary": "Comprehensive guide to ML concepts"
    }
  ],
  "total": 1,
  "processing_time_ms": 45.2
}
```

## 📚 Index Command

Index documents from a filesystem directory into the vector store.

### Usage
```bash
mdx index [OPTIONS] <PATH>
```

### Options
| Option | Description | Default |
|--------|-------------|---------|
| `<PATH>` | Directory or file path to index | Required |
| `--recursive` | Index directories recursively | `true` |
| `--extensions <LIST>` | File extensions to include | `md,txt,pdf,doc,docx` |
| `--exclude <PATTERNS>` | Glob patterns to exclude | None |
| `--batch-size <N>` | Documents per batch | `100` |
| `--format <FORMAT>` | Output format: json, table, yaml | `table` |

### Examples
```bash
# Index a directory
mdx index /path/to/documents

# Index with specific collection
mdx index /docs/api --collection api-docs

# Index specific file types
mdx index /docs --extensions md,rst,txt

# Index with exclusions
mdx index /project --exclude "*.log,temp/*,build/*"

# Verbose indexing with JSON output
mdx index /docs --verbose --format json
```

### Output
```
Indexing documents from: /path/to/documents
Collection: zero_latency_docs

Progress: ████████████████████████████████ 100% (150/150)
┌─────────────────────┬──────────┐
│ Metric              │ Value    │
├─────────────────────┼──────────┤
│ Documents Processed │ 150      │
│ Documents Added     │ 142      │
│ Documents Skipped   │ 8        │
│ Processing Time     │ 12.4s    │
│ Average Time/Doc    │ 82.7ms   │
└─────────────────────┴──────────┘
```

## 📄 Document Commands

Read-only operations for discovering indexed documents.

### Usage
```bash
mdx document <SUBCOMMAND>
```

### Subcommands

#### `list` - List Documents
```bash
mdx document list [OPTIONS]

Options:
  --limit <N>          Maximum documents to list [default: 50]
  --offset <N>         Skip N documents [default: 0]  
  --format <FORMAT>    Output format: json, table, yaml [default: table]
  --filter <PATTERN>   Filter by title or path pattern
```

**Examples:**
```bash
# List all documents
mdx document list

# List with pagination
mdx document list --limit 20 --offset 40

# Filter documents
mdx document list --filter "*.md"

# JSON output
mdx document list --format json --limit 10
```

#### `get` - Get Document Details
```bash
mdx document get [OPTIONS] <DOCUMENT_ID>

Options:
  --format <FORMAT>    Output format: json, table, yaml [default: table]
  --include-content    Include full document content
```

**Examples:**
```bash
# Get document metadata
mdx document get doc-123

# Get document with full content
mdx document get doc-123 --include-content

# JSON output for scripting
mdx document get doc-123 --format json
```

## 🗂️ Collection Commands

Full CRUD operations for managing document collections.

### Usage
```bash
mdx collection <SUBCOMMAND>
```

### Subcommands

#### `list` - List Collections
```bash
mdx collection list [OPTIONS]

Options:
  --format <FORMAT>    Output format: json, table, yaml [default: table]
  --include-stats      Include document counts and sizes
```

**Examples:**
```bash
# List all collections
mdx collection list

# List with statistics
mdx collection list --include-stats

# JSON output
mdx collection list --format json
```

#### `get` - Get Collection Details
```bash
mdx collection get [OPTIONS] <COLLECTION_NAME>

Options:
  --format <FORMAT>    Output format: json, table, yaml [default: table]
```

**Examples:**
```bash
# Get collection info
mdx collection get my-docs

# JSON output
mdx collection get my-docs --format json
```

#### `create` - Create Collection
```bash
mdx collection create [OPTIONS] <COLLECTION_NAME>

Options:
  --description <TEXT>  Collection description
  --format <FORMAT>     Output format: json, table, yaml [default: table]
```

**Examples:**
```bash
# Create basic collection
mdx collection create api-docs

# Create with description
mdx collection create tutorials --description "Programming tutorials and guides"
```

#### `delete` - Delete Collection
```bash
mdx collection delete [OPTIONS] <COLLECTION_NAME>

Options:
  --force              Skip confirmation prompt
  --format <FORMAT>    Output format: json, table, yaml [default: table]
```

**Examples:**
```bash
# Delete with confirmation
mdx collection delete old-docs

# Force delete without confirmation
mdx collection delete temp-collection --force
```

#### `stats` - Collection Statistics
```bash
mdx collection stats [OPTIONS] <COLLECTION_NAME>

Options:
  --format <FORMAT>    Output format: json, table, yaml [default: table]
  --detailed           Include detailed statistics
```

**Examples:**
```bash
# Basic statistics
mdx collection stats my-docs

# Detailed statistics
mdx collection stats my-docs --detailed

# JSON output
mdx collection stats my-docs --format json
```

## 📊 Status Command

Show system health and collection statistics.

### Usage
```bash
mdx status [OPTIONS]

Options:
  --format <FORMAT>    Output format: json, table, yaml [default: table]
  --all-collections    Show stats for all collections
```

### Examples
```bash
# Basic system status
mdx status

# All collections status
mdx status --all-collections

# JSON output
mdx status --format json
```

### Output
```
System Status: Healthy
┌──────────────────────┬─────────────────┐
│ Metric               │ Value           │
├──────────────────────┼─────────────────┤
│ Server Version       │ v0.1.0          │
│ Uptime               │ 2h 15m 30s      │
│ Total Collections    │ 3               │
│ Total Documents      │ 1,250           │
│ Memory Usage         │ 256 MB          │
│ Storage Usage        │ 1.2 GB          │
└──────────────────────┴─────────────────┘
```

## 🖥️ Server Command

Start the API server for remote access.

### Usage
```bash
mdx server [OPTIONS]

Options:
  --port <PORT>        Server port [default: 8081]
  --host <HOST>        Server host [default: 127.0.0.1]
  --docs-path <PATH>   Auto-monitor directory for changes
  --log-level <LEVEL>  Log level: trace, debug, info, warn, error [default: info]
```

### Examples
```bash
# Start basic server
mdx server

# Start on different port
mdx server --port 9000

# Start with auto-monitoring
mdx server --docs-path /data/documents

# Start with debug logging
mdx server --log-level debug --port 8081
```

## 🔄 Reindex Command

Rebuild the entire document index.

### Usage
```bash
mdx reindex [OPTIONS]

Options:
  --force              Skip confirmation prompt
  --batch-size <N>     Documents per batch [default: 100]
  --format <FORMAT>    Output format: json, table, yaml [default: table]
```

### Examples
```bash
# Reindex with confirmation
mdx reindex

# Force reindex without confirmation
mdx reindex --force

# Reindex with custom batch size
mdx reindex --batch-size 50
```

## 🎨 Output Formats

All commands support multiple output formats for different use cases:

### Table Format (Default)
- Human-readable tabular output
- Formatted for terminal display
- Includes headers and borders
- Good for interactive use

### JSON Format
- Machine-readable structured data
- Complete information preservation
- Suitable for scripting and APIs
- Can be piped to other tools

### YAML Format  
- Human and machine-readable
- Hierarchical structure
- Good for configuration files
- Easy to edit and version control

## 🔧 Configuration

### Configuration File
Create a configuration file to set default values:

```yaml
# ~/.mdx/config.yaml
server: "http://localhost:8081"
collection: "default"
verbose: false
format: "table"

# Search defaults
search:
  limit: 10
  threshold: 0.5

# Index defaults  
index:
  batch_size: 100
  extensions: ["md", "txt", "pdf", "doc", "docx"]
  exclude: ["*.log", "temp/*", "build/*"]
```

Use the configuration:
```bash
mdx --config ~/.mdx/config.yaml search "query"
```

### Environment Variables
```bash
export MDX_SERVER="http://localhost:8081"
export MDX_COLLECTION="default"
export MDX_VERBOSE="true"
```

## 📝 Examples & Workflows

### Complete Document Management Workflow
```bash
# 1. Start server
mdx server --port 8081 &

# 2. Create a new collection
mdx collection create project-docs --description "Project documentation"

# 3. Index documents
mdx index /project/docs --collection project-docs

# 4. Search documents
mdx search "API endpoints" --collection project-docs

# 5. List documents in collection
mdx document list --collection project-docs --limit 20

# 6. Get specific document
mdx document get doc-456 --include-content

# 7. Check collection statistics
mdx collection stats project-docs --detailed
```

### Scripting with JSON Output
```bash
#!/bin/bash

# Search and process results
results=$(mdx search "configuration" --format json --collection docs)
echo "$results" | jq '.results[] | select(.score > 0.8) | .path'

# List collections and get document counts
mdx collection list --format json | jq '.collections[] | "\(.name): \(.document_count) documents"'

# Batch process documents
mdx document list --format json --limit 1000 | jq -r '.documents[] | .id' | while read doc_id; do
  echo "Processing document: $doc_id"
  mdx document get "$doc_id" --format json | jq '.title'
done
```

### Development and Testing
```bash
# Index test documents
mdx index ./test/fixtures --collection test-docs

# Run searches with different thresholds
for threshold in 0.5 0.6 0.7 0.8; do
  echo "Threshold: $threshold"
  mdx search "example query" --threshold $threshold --format json | jq '.results | length'
done

# Monitor indexing performance
time mdx index /large/dataset --collection perf-test --verbose
```

## 🚨 Error Handling

Common error scenarios and solutions:

### Connection Errors
```bash
# Error: Connection refused
# Solution: Start the server first
mdx server &
sleep 2
mdx search "query"
```

### Collection Not Found
```bash
# Error: Collection 'missing' not found
# Solution: Create the collection or use existing one
mdx collection create missing
mdx search "query" --collection missing
```

### Index Errors
```bash
# Error: Path not found
# Solution: Check path exists and is readable
ls -la /path/to/docs
mdx index /path/to/docs
```

### Permission Errors
```bash
# Error: Permission denied
# Solution: Check file permissions
sudo chmod -R +r /path/to/docs
mdx index /path/to/docs
```

## 📚 See Also

- [Current Architecture](CURRENT_ARCHITECTURE.md) - System overview and design
- [API Reference](API_REFERENCE.md) - REST API documentation  
- [Installation Guide](../README.md) - Setup and deployment
- [Architecture Decisions](adr/) - Design decision records
