# 010 - Plan: Doc-Indexer Step 3 - Search API and Real-time Embeddings Pipeline

**Date:** 19 August 2025  
**Status:** 📋 PLANNED  
**Phase:** Step 3 Implementation Plan  
**Dependencies:** Step 1 (Production Qdrant) ✅, Step 2 (Advanced Chunking) ✅

## Executive Summary

Step 3 transforms the doc-indexer from a pure indexing daemon into a complete semantic search service by implementing real-time embeddings generation and HTTP/JSON-RPC search APIs. This enables integration with the broader Zero-Latency platform and provides both programmatic and CLI access to semantic search capabilities.

## Objectives

### Primary Goals

- **Real-time Embeddings Pipeline**: Replace mock embeddings with production OpenAI API integration
- **HTTP Search API**: RESTful semantic search endpoint with filtering and ranking
- **JSON-RPC Interface**: CLI and programmatic access for search operations
- **Performance Optimization**: 200+ chunks/min embedding throughput
- **Production Readiness**: Comprehensive error handling, rate limiting, and monitoring

### Technical Deliverables

- ✅ OpenAI embeddings API client with batch processing
- ✅ HTTP REST API (`POST /api/search`) with filters and ranking
- ✅ JSON-RPC server for CLI integration
- ✅ CLI search command (`mdx search "query"`)
- ✅ Rate limiting and exponential backoff retry logic
- ✅ Search result snippet generation and highlighting
- ✅ Performance metrics and monitoring endpoints

## Current State Analysis

### Completed Foundations (Steps 1-2)

- **Production Qdrant Integration**: 384-dimensional vectors, batch operations, collection management
- **Advanced Chunking Pipeline**: Multiple strategies (ByHeading, BySize, Hybrid, Semantic)
- **Quality Framework**: Multi-dimensional chunk evaluation and optimization
- **Rich Metadata**: Heading breadcrumbs, structural context, document hierarchy
- **Trait-based Architecture**: Extensible VectorDatabase abstraction

### Missing Components for Step 3

- **Real Embeddings Generation**: Currently using simple mock vectors
- **Search API**: No external query interface available
- **CLI Integration**: No search commands for end users
- **Rate Limiting**: No production-ready API throttling
- **Performance Optimization**: Not tuned for high-throughput embedding generation

## Technical Architecture

### Embeddings Pipeline

#### OpenAI API Integration

**Client Implementation:**
```rust
pub struct OpenAIEmbedder {
    client: reqwest::Client,
    api_key: String,
    model: String,
    rate_limiter: RateLimiter,
}

impl EmbeddingProvider for OpenAIEmbedder {
    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<EmbeddingResponse>>;
}
```

**Batch Processing:**
- **Batch Size**: 16-64 chunks per API call (configurable)
- **Rate Limiting**: Token-aware throttling with `retry_after_ms` handling
- **Retry Logic**: Exponential backoff with jittered delays
- **Error Handling**: Comprehensive error classification and recovery

**API Contract:**
```json
POST https://api.openai.com/v1/embeddings
{
  "model": "text-embedding-3-small",
  "input": ["chunk 1", "chunk 2", "..."],
  "dimensions": 384
}
```

### Search API Layer

#### HTTP REST Endpoint

**Search Request:**
```json
POST /api/search
{
  "query": "vector databases rust implementation",
  "k": 10,
  "filters": {
    "path_prefix": "docs/",
    "tags": ["rust", "database"],
    "date_range": {
      "after": "2025-08-01",
      "before": "2025-08-20"
    }
  },
  "include_snippets": true,
  "highlight": true
}
```

**Search Response:**
```json
{
  "query": "vector databases rust implementation",
  "total_results": 47,
  "results": [
    {
      "score": 0.89,
      "chunk_id": "doc123:chunk042",
      "document_id": "doc123",
      "document_title": "Qdrant Integration Guide",
      "content": "Vector databases provide...",
      "snippet": "...implement <mark>vector databases</mark> in <mark>Rust</mark>...",
      "heading_path": ["# Database Design", "## Vector Storage"],
      "section": "Implementation Details",
      "doc_type": "technical_guide",
      "created_at": "2025-08-19T10:30:00Z",
      "metadata": {
        "chunk_index": 42,
        "chunk_total": 87,
        "file_path": "docs/database/qdrant-integration.md"
      }
    }
  ],
  "search_metadata": {
    "embedding_time_ms": 45,
    "search_time_ms": 12,
    "total_time_ms": 57,
    "model_used": "text-embedding-3-small"
  }
}
```

#### JSON-RPC Interface

**CLI Integration:**
```bash
mdx search "vector databases"
mdx search --filter="path:docs/" --limit=5 "rust implementation"
mdx stats
mdx health
```

**JSON-RPC Methods:**
- `search(query, filters, options)` - Semantic search
- `document.get(doc_id)` - Retrieve document metadata
- `collection.stats()` - Collection statistics
- `health.check()` - System health status

### Performance Architecture

#### Embedding Throughput Optimization

**Pipeline Design:**
```rust
pub struct EmbeddingPipeline {
    chunker: AdvancedChunker,
    embedder: Box<dyn EmbeddingProvider>,
    vectordb: Box<dyn VectorDatabase>,
    batch_processor: BatchProcessor,
}
```

**Throughput Targets:**
- **Baseline**: 200+ chunks/min on M-series Mac
- **Batch Optimization**: 16-64 chunks per API call
- **Concurrent Processing**: Parallel embedding and indexing
- **Memory Efficiency**: Streaming processing for large documents

**Rate Limiting Strategy:**
- **Token Bucket**: Smooth rate limiting with burst capacity
- **Backoff**: Exponential delays with jitter (200ms base)
- **Circuit Breaker**: Automatic fallback on repeated failures
- **Monitoring**: Real-time throughput and error rate tracking

#### Search Performance

**Query Optimization:**
- **Vector Search**: Sub-100ms Qdrant queries for k≤20
- **Filtering**: Efficient payload-based filtering before vector search
- **Snippet Generation**: Fast text highlighting with context preservation
- **Caching**: Query result caching for frequent searches

## Implementation Plan

### Phase 3.1: Real-time Embeddings Foundation (Week 1)

#### Day 1-2: OpenAI API Integration
- **EmbeddingProvider Trait**: Define interface for embedding services
- **OpenAIEmbedder**: Implement OpenAI API client with authentication
- **Configuration**: Add API key management and model selection
- **Basic Integration**: Replace mock embeddings in document processing

#### Day 3-4: Batch Processing and Rate Limiting
- **BatchProcessor**: Implement configurable batch size and queueing
- **RateLimiter**: Token bucket implementation with backoff
- **Error Handling**: Comprehensive retry logic and error classification
- **Testing**: Unit tests for batch processing and rate limiting

#### Day 5: Integration and Testing
- **End-to-End**: Full document indexing with real embeddings
- **Performance Testing**: Validate 200+ chunks/min throughput
- **Error Scenarios**: Test rate limiting, API failures, and recovery
- **Documentation**: Update configuration and deployment guides

### Phase 3.2: Search API Implementation (Week 2)

#### Day 1-2: HTTP REST API
- **Axum Server**: HTTP server with search endpoint
- **Request/Response**: JSON schema validation and error handling
- **Search Service**: Core search logic with filtering and ranking
- **Snippet Generation**: Text highlighting and context extraction

#### Day 3-4: JSON-RPC Interface
- **JSON-RPC Server**: Stdio and HTTP JSON-RPC endpoints
- **CLI Integration**: Search commands for terminal usage
- **Method Implementation**: Search, stats, health, and document methods
- **Protocol Testing**: Validate JSON-RPC compliance and error handling

#### Day 5: Advanced Search Features
- **Filtering System**: Path, tag, date range, and content type filters
- **Result Ranking**: Relevance scoring with metadata boosting
- **Performance Optimization**: Query optimization and caching
- **Monitoring**: Search analytics and performance metrics

### Phase 3.3: Production Readiness (Week 3)

#### Day 1-2: CLI and User Experience
- **mdx CLI**: Complete command-line interface with search, stats, health
- **Output Formatting**: Human-readable and JSON output modes
- **Help System**: Comprehensive help and examples
- **Shell Completion**: Bash/zsh completion for commands and options

#### Day 3-4: Monitoring and Observability
- **Metrics Collection**: Prometheus-compatible metrics export
- **Health Endpoints**: Detailed component status reporting
- **Logging**: Structured logging for search and embedding operations
- **Profiling**: Performance profiling and bottleneck identification

#### Day 5: Documentation and Testing
- **API Documentation**: OpenAPI spec and examples
- **Integration Tests**: End-to-end testing with real API calls
- **Performance Benchmarks**: Baseline establishment and regression tests
- **Deployment Guide**: Production deployment and configuration

## API Specifications

### Search API Contract

#### POST /api/search

**Request Schema:**
```json
{
  "type": "object",
  "required": ["query"],
  "properties": {
    "query": {"type": "string", "minLength": 1},
    "k": {"type": "integer", "minimum": 1, "maximum": 100, "default": 10},
    "filters": {
      "type": "object",
      "properties": {
        "path_prefix": {"type": "string"},
        "tags": {"type": "array", "items": {"type": "string"}},
        "date_range": {
          "type": "object",
          "properties": {
            "after": {"type": "string", "format": "date-time"},
            "before": {"type": "string", "format": "date-time"}
          }
        }
      }
    },
    "include_snippets": {"type": "boolean", "default": true},
    "highlight": {"type": "boolean", "default": false}
  }
}
```

#### GET /api/health

**Response Schema:**
```json
{
  "status": "healthy",
  "components": {
    "embedder": {"status": "healthy", "latency_ms": 45},
    "vectordb": {"status": "healthy", "collection_size": 1234},
    "file_watcher": {"status": "healthy", "files_watched": 567}
  },
  "system": {
    "uptime_seconds": 86400,
    "memory_usage_mb": 128,
    "embeddings_generated": 5678,
    "searches_completed": 234
  }
}
```

### JSON-RPC Methods

#### search
```json
{
  "jsonrpc": "2.0",
  "method": "search",
  "params": {
    "query": "vector databases",
    "k": 5,
    "filters": {"path_prefix": "docs/"}
  },
  "id": 1
}
```

#### collection.stats
```json
{
  "jsonrpc": "2.0", 
  "method": "collection.stats",
  "params": {},
  "id": 2
}
```

## Quality Assurance

### Testing Strategy

#### Unit Tests
- **EmbeddingProvider**: Mock API responses and error conditions
- **SearchService**: Query processing and filtering logic
- **RateLimiter**: Throughput limits and backoff behavior
- **JSON-RPC**: Protocol compliance and method implementation

#### Integration Tests
- **End-to-End**: Document indexing through search with real APIs
- **Performance**: Throughput and latency under load
- **Error Handling**: API failures, network issues, and recovery
- **CLI**: Command-line interface functionality and output

#### Acceptance Tests
- **Golden File Tests**: Consistent API responses for regression testing
- **Performance Benchmarks**: 200+ chunks/min embedding throughput
- **Search Quality**: Relevance scoring and result ranking validation
- **Production Scenarios**: Real-world usage patterns and edge cases

### Performance Targets

#### Embedding Pipeline
- **Throughput**: ≥200 chunks/min sustained processing
- **Latency**: ≤500ms average per batch (16-64 chunks)
- **Error Rate**: ≤1% failed embedding requests
- **Recovery Time**: ≤30s automatic recovery from API failures

#### Search API
- **Response Time**: ≤100ms for k≤10 results
- **Concurrent Users**: Support 10+ simultaneous searches
- **Availability**: ≥99.9% uptime with graceful degradation
- **Scalability**: Linear performance scaling with corpus size

## Security and Safety

### API Security
- **Local Binding**: Default bind to 127.0.0.1, explicit `--listen 0.0.0.0` required
- **Rate Limiting**: Per-client request throttling and abuse prevention
- **Input Validation**: Comprehensive query sanitization and bounds checking
- **Error Handling**: No sensitive information in error responses

### Data Protection
- **API Key Management**: Secure storage and rotation of OpenAI credentials
- **Query Logging**: Optional query anonymization and retention policies
- **Content Filtering**: Regex-based scrubbing of sensitive data before embedding
- **Access Control**: Path allowlisting and ignore pattern enforcement

## Success Metrics

### Technical Metrics
- **Embedding Throughput**: 200+ chunks/min sustained
- **Search Latency**: <100ms average response time
- **API Reliability**: 99.9+ % success rate
- **CLI Usability**: Complete feature parity with HTTP API

### User Experience Metrics
- **Search Relevance**: Qualitative evaluation with test queries
- **Result Quality**: Snippet accuracy and context preservation
- **Developer Experience**: Clear API documentation and examples
- **Operational Visibility**: Comprehensive monitoring and debugging

## Future Enhancements

### Short-term (Post Step 3)
- **Search Analytics**: Query patterns and usage statistics
- **Result Caching**: Intelligent caching for frequent searches
- **Advanced Filters**: Content type, author, and semantic similarity filters
- **Bulk Operations**: Batch search and document management APIs

### Medium-term
- **Web Interface**: React-based search dashboard
- **Search Suggestions**: Auto-complete and query expansion
- **Document Recommendations**: Similarity-based content discovery
- **Integration APIs**: Webhooks and event streaming for external systems

### Long-term
- **Hybrid Search**: Combining semantic and lexical search
- **Multi-modal**: Support for images, diagrams, and rich content
- **Personalization**: User-specific ranking and preferences
- **Federated Search**: Cross-repository and cross-platform search

## Risk Assessment

### Technical Risks
- **API Rate Limits**: OpenAI API quotas and cost management
- **Performance Bottlenecks**: Embedding generation latency at scale
- **Memory Usage**: Large document corpus memory requirements
- **Network Dependencies**: Reliability on external embedding services

### Mitigation Strategies
- **Fallback Embeddings**: Local model integration for API failures
- **Caching Strategy**: Persistent embedding cache to reduce API calls
- **Resource Monitoring**: Automatic scaling and resource management
- **Circuit Breakers**: Graceful degradation and error recovery

## Conclusion

Step 3 completes the transformation of doc-indexer from an indexing daemon to a full-featured semantic search service. The implementation provides production-ready embeddings generation, comprehensive search APIs, and seamless CLI integration while maintaining the quality and performance standards established in previous steps.

The planned architecture ensures scalability, reliability, and extensibility while providing the foundation for advanced search features and platform integration in future phases.

---

**Dependencies:** Step 1 ✅, Step 2 ✅  
**Next Phase:** Step 4 - Web Interface and Advanced Features  
**Implementation Timeline:** 3 weeks (19 Aug - 9 Sep 2025)
