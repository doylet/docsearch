# API Reference - doc-indexer

**REST API for Zero-Latency Document Search**  
**Version**: v0.1.0  
**Updated**: August 24, 2025  
**Base URL**: `http://localhost:8081`  

## Overview

The doc-indexer API provides RESTful endpoints for document indexing, collection management, and semantic search operations. The API follows clean architecture principles with proper separation between collection management (full CRUD) and document discovery (read-only).

## Authentication

Currently, the API does not require authentication. All endpoints are publicly accessible.

## Content Types

- **Request**: `application/json`
- **Response**: `application/json`

## Error Handling

All error responses follow this format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": "Additional error context (optional)"
  },
  "timestamp": "2025-08-24T10:30:00Z",
  "path": "/api/endpoint"
}
```

### HTTP Status Codes

| Code | Description |
|------|-------------|
| 200 | OK - Request successful |
| 201 | Created - Resource created successfully |
| 400 | Bad Request - Invalid request format |
| 404 | Not Found - Resource not found |
| 409 | Conflict - Resource already exists |
| 500 | Internal Server Error - Server error |

## Collections API

Collections provide organization and isolation for document sets. Each collection maintains its own vector space and can be managed independently.

### List Collections

Retrieve all available collections.

```http
GET /collections
```

#### Response
```json
{
  "collections": [
    {
      "id": "default",
      "name": "default",
      "description": "Default document collection",
      "created_at": "2025-08-24T09:00:00Z",
      "document_count": 150,
      "total_size": 2048576
    },
    {
      "id": "api-docs",
      "name": "api-docs", 
      "description": "API documentation collection",
      "created_at": "2025-08-24T10:00:00Z",
      "document_count": 25,
      "total_size": 512000
    }
  ],
  "total": 2
}
```

#### Example
```bash
curl -X GET http://localhost:8081/collections
```

### Get Collection

Retrieve details for a specific collection.

```http
GET /collections/{collection_id}
```

#### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `collection_id` | string | Collection identifier |

#### Response
```json
{
  "id": "api-docs",
  "name": "api-docs",
  "description": "API documentation collection", 
  "created_at": "2025-08-24T10:00:00Z",
  "updated_at": "2025-08-24T12:30:00Z",
  "document_count": 25,
  "total_size": 512000,
  "metadata": {
    "indexed_extensions": ["md", "txt", "pdf"],
    "last_indexed": "2025-08-24T12:30:00Z"
  }
}
```

#### Example
```bash
curl -X GET http://localhost:8081/collections/api-docs
```

#### Error Responses
- `404` - Collection not found

### Create Collection

Create a new document collection.

```http
POST /collections
```

#### Request Body
```json
{
  "name": "tutorials",
  "description": "Programming tutorials and guides"
}
```

#### Request Parameters
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Collection name (3-50 chars, alphanumeric + hyphens) |
| `description` | string | No | Collection description (max 500 chars) |

#### Response
```json
{
  "id": "tutorials",
  "name": "tutorials",
  "description": "Programming tutorials and guides",
  "created_at": "2025-08-24T14:00:00Z",
  "document_count": 0,
  "total_size": 0
}
```

#### Example
```bash
curl -X POST http://localhost:8081/collections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "tutorials",
    "description": "Programming tutorials and guides"
  }'
```

#### Error Responses
- `400` - Invalid request format or parameters
- `409` - Collection already exists

### Delete Collection

Delete a collection and all its documents.

```http
DELETE /collections/{collection_id}
```

#### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `collection_id` | string | Collection identifier |

#### Response
```json
{
  "message": "Collection 'tutorials' deleted successfully",
  "deleted_documents": 45,
  "operation_time_ms": 234.5
}
```

#### Example
```bash
curl -X DELETE http://localhost:8081/collections/tutorials
```

#### Error Responses
- `404` - Collection not found
- `409` - Collection is not empty (if protection enabled)

### Collection Statistics

Get detailed statistics for a collection.

```http
GET /collections/{collection_id}/stats
```

#### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `collection_id` | string | Collection identifier |

#### Response
```json
{
  "collection": {
    "id": "api-docs",
    "name": "api-docs",
    "document_count": 150
  },
  "statistics": {
    "total_documents": 150,
    "total_size_bytes": 2048576,
    "average_document_size": 13657,
    "file_types": {
      "markdown": 120,
      "text": 20,
      "pdf": 10
    },
    "indexing_stats": {
      "total_vectors": 150,
      "vector_dimensions": 384,
      "last_indexed": "2025-08-24T12:30:00Z",
      "index_size_mb": 12.5
    },
    "performance": {
      "average_search_time_ms": 45.2,
      "total_searches": 1250,
      "cache_hit_rate": 0.85
    }
  }
}
```

#### Example
```bash
curl -X GET http://localhost:8081/collections/api-docs/stats
```

## Documents API

Document endpoints provide read-only discovery of indexed content. Documents represent filesystem files that have been processed and stored in the vector database.

### List Documents

Retrieve documents from a collection with pagination and filtering.

```http
GET /documents
```

#### Query Parameters
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `collection` | string | `default` | Collection to search in |
| `limit` | integer | `50` | Maximum documents to return (1-1000) |
| `offset` | integer | `0` | Number of documents to skip |
| `filter` | string | - | Filter by title or path pattern |
| `sort` | string | `created_at` | Sort field: `created_at`, `title`, `size` |
| `order` | string | `desc` | Sort order: `asc`, `desc` |

#### Response
```json
{
  "documents": [
    {
      "id": "doc-123",
      "title": "API Authentication Guide",
      "path": "/docs/api/auth.md",
      "size": 4096,
      "content_type": "text/markdown",
      "indexed_at": "2025-08-24T10:30:00Z",
      "summary": "Complete guide to API authentication methods...",
      "metadata": {
        "author": "developer",
        "last_modified": "2025-08-23T15:20:00Z",
        "tags": ["api", "security", "authentication"]
      }
    },
    {
      "id": "doc-124", 
      "title": "Rate Limiting Documentation",
      "path": "/docs/api/rate-limits.md",
      "size": 2048,
      "content_type": "text/markdown",
      "indexed_at": "2025-08-24T10:35:00Z",
      "summary": "Overview of API rate limiting policies..."
    }
  ],
  "pagination": {
    "total": 150,
    "limit": 50,
    "offset": 0,
    "has_next": true,
    "has_prev": false
  },
  "collection": "api-docs"
}
```

#### Example
```bash
# Basic listing
curl -X GET "http://localhost:8081/documents?collection=api-docs"

# With pagination
curl -X GET "http://localhost:8081/documents?collection=api-docs&limit=10&offset=20"

# With filtering
curl -X GET "http://localhost:8081/documents?filter=*.md&sort=title&order=asc"
```

### Get Document

Retrieve detailed information about a specific document.

```http
GET /documents/{document_id}
```

#### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `document_id` | string | Document identifier |

#### Query Parameters
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `include_content` | boolean | `false` | Include full document content |
| `collection` | string | `default` | Collection context |

#### Response
```json
{
  "id": "doc-123",
  "title": "API Authentication Guide",
  "path": "/docs/api/auth.md",
  "size": 4096,
  "content_type": "text/markdown",
  "indexed_at": "2025-08-24T10:30:00Z",
  "content": "# API Authentication\n\nThis guide covers...", // if include_content=true
  "summary": "Complete guide to API authentication methods...",
  "metadata": {
    "author": "developer",
    "last_modified": "2025-08-23T15:20:00Z",
    "tags": ["api", "security", "authentication"],
    "word_count": 1250,
    "reading_time_minutes": 5,
    "language": "en"
  },
  "vector_info": {
    "embedding_model": "sentence-transformers/all-MiniLM-L6-v2",
    "vector_dimensions": 384,
    "similarity_threshold": 0.5
  }
}
```

#### Example
```bash
# Get document metadata
curl -X GET http://localhost:8081/documents/doc-123

# Get document with content
curl -X GET "http://localhost:8081/documents/doc-123?include_content=true"
```

#### Error Responses
- `404` - Document not found

## Indexing API

The indexing API processes filesystem documents and adds them to collections.

### Index Documents from Path

Process and index documents from a filesystem path.

```http
POST /api/index
```

#### Request Body
```json
{
  "path": "/data/documents",
  "collection": "api-docs",
  "recursive": true,
  "extensions": ["md", "txt", "pdf"],
  "exclude_patterns": ["*.log", "temp/*", "build/*"],
  "batch_size": 100,
  "overwrite_existing": false
}
```

#### Request Parameters
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | string | Yes | Filesystem path to index |
| `collection` | string | No | Target collection (default: "default") |
| `recursive` | boolean | No | Process directories recursively (default: true) |
| `extensions` | array | No | File extensions to include (default: ["md","txt","pdf"]) |
| `exclude_patterns` | array | No | Glob patterns to exclude |
| `batch_size` | integer | No | Documents per processing batch (default: 100) |
| `overwrite_existing` | boolean | No | Overwrite existing documents (default: false) |

#### Response
```json
{
  "operation_id": "idx-789",
  "message": "Indexing completed successfully",
  "results": {
    "documents_processed": 150,
    "documents_added": 142,
    "documents_updated": 5,
    "documents_skipped": 3,
    "errors": []
  },
  "performance": {
    "processing_time_ms": 12450.5,
    "average_time_per_document_ms": 83.0,
    "throughput_docs_per_second": 12.0
  },
  "collection": "api-docs",
  "timestamp": "2025-08-24T14:30:00Z"
}
```

#### Example
```bash
curl -X POST http://localhost:8081/api/index \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/data/api-docs",
    "collection": "api-docs",
    "recursive": true,
    "extensions": ["md", "txt"],
    "batch_size": 50
  }'
```

#### Error Responses
- `400` - Invalid request parameters
- `404` - Path not found or collection not found
- `500` - Indexing operation failed

## Search API

Semantic search across indexed documents using natural language queries.

### Search Documents

Perform semantic search within a collection.

```http
GET /search
```

#### Query Parameters
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `q` | string | - | Search query (required) |
| `collection` | string | `default` | Collection to search in |
| `limit` | integer | `10` | Maximum results (1-100) |
| `threshold` | float | `0.5` | Minimum similarity score (0.0-1.0) |
| `include_content` | boolean | `false` | Include document content in results |

#### Response
```json
{
  "query": "API authentication methods",
  "results": [
    {
      "document": {
        "id": "doc-123",
        "title": "API Authentication Guide",
        "path": "/docs/api/auth.md",
        "summary": "Complete guide to API authentication..."
      },
      "score": 0.89,
      "highlights": [
        "API authentication methods include...",
        "OAuth 2.0 provides secure access..."
      ]
    },
    {
      "document": {
        "id": "doc-125",
        "title": "Security Best Practices", 
        "path": "/docs/security/best-practices.md",
        "summary": "Security guidelines for API development..."
      },
      "score": 0.76,
      "highlights": [
        "Authentication should always use...",
        "Secure token storage practices..."
      ]
    }
  ],
  "pagination": {
    "total": 8,
    "limit": 10,
    "offset": 0,
    "has_next": false,
    "has_prev": false
  },
  "performance": {
    "search_time_ms": 45.2,
    "vector_search_time_ms": 32.1,
    "highlight_time_ms": 13.1
  },
  "collection": "api-docs"
}
```

#### Example
```bash
# Basic search
curl -X GET "http://localhost:8081/search?q=authentication&collection=api-docs"

# Advanced search with options
curl -X GET "http://localhost:8081/search?q=rate%20limiting&limit=5&threshold=0.7&include_content=true"
```

### Search with POST

Alternative search endpoint using POST for complex queries.

```http
POST /search
```

#### Request Body
```json
{
  "query": "API authentication methods",
  "collection": "api-docs",
  "limit": 10,
  "threshold": 0.5,
  "include_content": false,
  "filters": {
    "content_type": "text/markdown",
    "tags": ["api", "security"],
    "date_range": {
      "from": "2025-01-01T00:00:00Z",
      "to": "2025-12-31T23:59:59Z"
    }
  },
  "boost": {
    "title": 2.0,
    "tags": 1.5,
    "recent": 1.2
  }
}
```

#### Response
Same format as GET `/search`

## Health and Status API

### Health Check

Check API server health status.

```http
GET /health
```

#### Response
```json
{
  "status": "healthy",
  "timestamp": "2025-08-24T14:30:00Z",
  "version": "v0.1.0",
  "uptime_seconds": 7890,
  "services": {
    "vector_store": "healthy",
    "embedding_service": "healthy", 
    "file_system": "healthy"
  },
  "metrics": {
    "total_collections": 3,
    "total_documents": 1250,
    "memory_usage_mb": 256,
    "storage_usage_mb": 1200
  }
}
```

### System Status

Get detailed system information and statistics.

```http
GET /status
```

#### Response
```json
{
  "system": {
    "version": "v0.1.0",
    "build_date": "2025-08-24T12:00:00Z",
    "rust_version": "1.70.0",
    "uptime_seconds": 7890
  },
  "performance": {
    "requests_per_second": 45.2,
    "average_response_time_ms": 125.3,
    "error_rate": 0.02
  },
  "resources": {
    "memory": {
      "used_mb": 256,
      "available_mb": 2048,
      "usage_percent": 12.5
    },
    "storage": {
      "used_mb": 1200,
      "available_mb": 10240,
      "usage_percent": 11.7
    }
  },
  "collections": [
    {
      "name": "api-docs",
      "documents": 150,
      "size_mb": 4.5
    }
  ]
}
```

## WebSocket API (Future)

**Note**: WebSocket support is planned for real-time features.

### Real-time Indexing Updates

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8081/ws/indexing');

// Listen for indexing progress
ws.onmessage = function(event) {
  const update = JSON.parse(event.data);
  console.log('Indexing progress:', update.progress);
};
```

## Rate Limiting

Current rate limits (subject to change):

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/search` | 100 requests | 1 minute |
| `/api/index` | 10 requests | 1 minute |
| Other endpoints | 1000 requests | 1 minute |

Rate limit headers are included in responses:
- `X-RateLimit-Limit`: Request limit
- `X-RateLimit-Remaining`: Remaining requests  
- `X-RateLimit-Reset`: Reset timestamp

## SDKs and Client Libraries

### Command Line Interface
The `mdx` CLI provides a full-featured client:

```bash
# Install CLI
cargo install --path crates/cli

# Use CLI as API client
mdx search "query" --server http://api-server:8081 --format json
```

### cURL Examples

Complete examples for common workflows:

```bash
#!/bin/bash

BASE_URL="http://localhost:8081"

# 1. Create collection
curl -X POST "$BASE_URL/collections" \
  -H "Content-Type: application/json" \
  -d '{"name": "docs", "description": "Documentation collection"}'

# 2. Index documents
curl -X POST "$BASE_URL/api/index" \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/data/docs",
    "collection": "docs",
    "recursive": true
  }'

# 3. Search documents
curl -X GET "$BASE_URL/search?q=authentication&collection=docs&limit=5"

# 4. List documents
curl -X GET "$BASE_URL/documents?collection=docs&limit=10"

# 5. Get document details
DOC_ID=$(curl -s "$BASE_URL/documents?collection=docs&limit=1" | jq -r '.documents[0].id')
curl -X GET "$BASE_URL/documents/$DOC_ID?include_content=true"

# 6. Collection statistics
curl -X GET "$BASE_URL/collections/docs/stats"
```

## Error Reference

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `COLLECTION_NOT_FOUND` | 404 | Collection does not exist |
| `DOCUMENT_NOT_FOUND` | 404 | Document does not exist |
| `COLLECTION_EXISTS` | 409 | Collection already exists |
| `INVALID_COLLECTION_NAME` | 400 | Collection name format invalid |
| `PATH_NOT_FOUND` | 404 | Filesystem path not found |
| `INDEXING_FAILED` | 500 | Document indexing operation failed |
| `SEARCH_FAILED` | 500 | Search operation failed |
| `INVALID_QUERY` | 400 | Search query format invalid |
| `RATE_LIMITED` | 429 | Request rate limit exceeded |

## Changelog

### v0.1.0 (August 24, 2025)
- Initial API release
- Collection CRUD operations
- Document discovery endpoints  
- Indexing API
- Semantic search functionality
- Health and status endpoints

## See Also

- [CLI Reference](CLI_REFERENCE.md) - Command-line interface documentation
- [Current Architecture](CURRENT_ARCHITECTURE.md) - System architecture overview
- [Installation Guide](../README.md) - Setup and deployment instructions
