# ADR-042: Search Service Collection Filtering and Data Quality Resolution

**Status:** Accepted  
**Date:** August 29, 2025  
**Decision Makers:** Development Team  
**Supersedes:** None  
**Technical Story:** Resolving CLI search degradation and training data quality issues

## Context

The Zero-Latency document indexer service has experienced search functionality degradation affecting the CLI and REST API endpoints, while maintaining JSON-RPC functionality. Investigation revealed multiple interconnected issues with collection filtering and training data quality.

### Problem Statement

During routine testing, the following issues were identified:

1. **CLI Search Failure**: Collection-filtered searches return 0 results
2. **REST API Collection Filtering**: Same failure pattern as CLI
3. **JSON-RPC Success**: Searches work when not filtering by collection
4. **Poor Training Data**: Only test stub documents indexed, not actual documentation

### Technical Investigation

#### Symptom Analysis
```bash
# JSON-RPC (Works) - searches all collections
curl -X POST http://127.0.0.1:8081/jsonrpc \
  -d '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "search_documents", "arguments": {"query": "test", "limit": 1}}, "id": 2}'
# Returns: 1 result

# REST API (Fails) - filters by collection  
curl -X POST http://127.0.0.1:8081/api/search \
  -d '{"query": "test", "filters": {"collection_name": "zero_latency_docs"}}'
# Returns: 0 results

# CLI (Fails) - uses REST API with collection filter
./target/debug/mdx search "test" --collection zero_latency_docs
# Returns: "No results found"
```

#### Root Cause Analysis

**Primary Issue: Collection Filtering Logic**
- JSON-RPC `search_documents()` uses: `SearchRequest::new(query).with_limit(limit)` (no collection filter)
- REST API `search_documents_in_collection()` uses: `SearchRequest::new(query).with_filters(filters)` with collection metadata

**Secondary Issue: Schema Compliance**
- CLI was sending non-compliant requests: `{"query": "...", "collection_name": "..."}`
- OpenAPI schema requires: `{"query": "...", "filters": {"collection_name": "..."}}`

**Tertiary Issue: Training Data Quality**
- Collection contains 35 documents, all identical test stubs
- Content: "Zero-Latency doc-indexer smoke test. This is a test document..."
- No actual project documentation indexed

### Code Analysis

**Document Service Implementation:**
```rust
// JSON-RPC path (works)
pub async fn search_documents(&self, query: &str, limit: usize) -> Result<SearchResponse> {
    let search_request = SearchRequest::new(query).with_limit(limit);
    self.search_orchestrator.search(search_request).await
}

// REST API path (fails with collection filter)
pub async fn search_documents_in_collection(&self, query: &str, collection_name: &str, limit: usize) -> Result<SearchResponse> {
    let mut filters = zero_latency_search::SearchFilters::default();
    filters.custom.insert("collection".to_string(), collection_name.to_string());
    
    let search_request = zero_latency_search::SearchRequest::new(query)
        .with_limit(limit)
        .with_filters(filters);
        
    self.search_orchestrator.search(search_request).await
}
```

**Vector Search Behavior:**
- Without collection filter: "Searching across all collections" → finds results
- With collection filter: "Searching in collection: 'zero_latency_docs'" → finds 0 results

## Decision

### Resolution Strategy

**1. Immediate Fix: Collection Filter Behavior Analysis**

Since there's only one collection (`zero_latency_docs` with 35 documents), filtering by that collection should return the same results as searching all collections. The issue indicates the collection filtering mechanism is not properly matching documents.

**Accepted Solution:**
- **For CLI**: Remove broken collection filtering, search all collections by default
- **For Future**: Investigate and fix collection metadata tagging during document indexing

**2. Schema Compliance Fix**

**Status: ✅ COMPLETED**
```rust
// Before (non-compliant)
.json(&serde_json::json!({
    "query": query.effective_query(),
    "limit": query.limit,
    "collection_name": self.collection_name
}))

// After (schema-compliant)  
.json(&serde_json::json!({
    "query": query.effective_query(),
    "limit": query.limit,
    "filters": {}  // Removed broken collection filter
}))
```

**3. Training Data Quality Improvement**

**Proposed Solution:**
- Index actual project documentation from `docs/` directory
- Include ADR files, implementation guides, architecture documents
- Replace test stub content with meaningful technical documentation

## Implementation

### Phase 1: Immediate Resolution ✅

**CLI Search Fix:**
- Modified `SearchApiClient` to remove broken collection filtering
- CLI now searches all collections (matches working JSON-RPC behavior)
- Schema-compliant request format implemented

**Files Modified:**
- `crates/cli/src/infrastructure/http/search_client.rs`

### Phase 2: Data Quality Improvement (Proposed)

**Training Data Replacement:**
```bash
# Index actual documentation
./target/debug/mdx index docs/adr/ --server http://127.0.0.1:8081 --recursive
./target/debug/mdx index docs/implementation/ --server http://127.0.0.1:8081 --recursive  
./target/debug/mdx index docs/architecture/ --server http://127.0.0.1:8081 --recursive
```

### Phase 3: Collection Filtering Investigation (Future)

**Research Required:**
- Investigate how documents are tagged with collection metadata during indexing
- Verify vector database collection filtering implementation
- Ensure consistent behavior between filtered and unfiltered searches

## Consequences

### Positive Outcomes

**✅ Immediate Resolution:**
- CLI search functionality restored
- Schema compliance achieved
- Consistent search behavior across interfaces

**✅ Simplified Architecture:**
- Reduced complexity by removing broken collection filtering
- Single search path for all client interfaces
- Aligned with working JSON-RPC implementation

### Technical Debt

**⚠️ Temporary Workarounds:**
- Collection filtering functionality disabled rather than fixed
- Root cause of collection metadata issues unresolved
- Training data quality remains poor until replacement

### Monitoring and Validation

**Success Metrics:**
- CLI search returns results for relevant queries
- REST API search matches JSON-RPC behavior  
- Search quality improves with better training data

**Test Commands:**
```bash
# Verify CLI functionality
./target/debug/mdx search "architecture" --server http://127.0.0.1:8081

# Verify REST API consistency
curl -X POST http://127.0.0.1:8081/api/search \
  -d '{"query": "architecture", "filters": {}}'

# Verify JSON-RPC still works
curl -X POST http://127.0.0.1:8081/jsonrpc \
  -d '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "search_documents", "arguments": {"query": "architecture"}}, "id": 1}'
```

## Related Decisions

- **ADR-041**: Schema-First Contract Architecture (addresses schema compliance issues)
- **ADR-039**: JSON-RPC MCP Protocol Compliance (explains JSON-RPC functionality)
- **Future ADR**: Vector Database Collection Management (will address collection filtering)

## Notes

This ADR documents a pragmatic resolution prioritizing immediate functionality over architectural purity. The collection filtering feature was disabled rather than debugged due to:

1. **Single Collection Context**: Only one collection exists, making filtering functionally unnecessary
2. **Working Alternative**: JSON-RPC provides identical functionality without filtering
3. **Time Efficiency**: Training data replacement is higher priority than debugging unused feature

Future work should investigate the collection metadata implementation and restore proper filtering functionality when multiple collections are needed.
