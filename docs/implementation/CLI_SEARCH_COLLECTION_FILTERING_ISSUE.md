# CLI Search Collection Filtering Issue

## Problem Statement
The CLI search functionality returns no results when filtering by collection, despite the same documents being found via JSON-RPC API without collection filtering.

## Root Cause Analysis

### API Endpoint Comparison
1. **JSON-RPC Search (Working)**: `search_documents()` without collection filter
   - Searches across all collections
   - Returns 10 results for "architecture" query
   - Uses `SearchRequest::new(query).with_limit(limit)`

2. **REST API Search (Not Working)**: `search_documents_in_collection()` with collection filter
   - Filters by specific collection: "zero_latency_docs"
   - Returns 0 results for same query
   - Uses `SearchRequest::new(query).with_limit(limit).with_filters(filters)` where filters contain collection name

### Collection Filtering Logic Issue
The collection filtering mechanism in the vector search is not working correctly:
- Documents exist in the vector database (confirmed by JSON-RPC returning results)
- Collection filter `filters.custom.insert("collection".to_string(), collection_name.to_string())` is not matching documents
- Documents may not be properly tagged with collection metadata during indexing

## Schema Compliance Fix
Fixed CLI to send schema-compliant requests:
```rust
// Before (non-compliant):
{
  "query": "...",
  "collection_name": "zero_latency_docs"
}

// After (schema-compliant):
{
  "query": "...",
  "filters": {
    "collection_name": "zero_latency_docs"
  }
}
```

## Next Steps
1. **Investigate Vector Database**: Check how documents are stored and if collection metadata is properly indexed
2. **Fix Collection Filtering**: Ensure the search filter logic correctly matches collection-tagged documents
3. **Schema-First Enforcement**: Implement contract-first validation to prevent future API divergence

## Files Modified
- `/crates/cli/src/infrastructure/http/search_client.rs` - Fixed to use schema-compliant request format

## Testing Commands
```bash
# Test JSON-RPC (works):
curl -X POST http://127.0.0.1:8081/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "search_documents", "arguments": {"query": "architecture", "limit": 10}}, "id": 5}'

# Test REST API (doesn't work with collection filter):
./target/debug/mdx search "architecture" --server http://127.0.0.1:8081 --collection zero_latency_docs

# Test REST API direct (doesn't work with collection filter):
curl -X POST http://127.0.0.1:8081/api/search \
  -H "Content-Type: application/json" \
  -d '{"query": "architecture", "filters": {"collection_name": "zero_latency_docs"}}'
```
