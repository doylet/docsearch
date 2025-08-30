# Search Filtering Troubleshooting Guide

**Document**: Search Filtering Issues & Solutions  
**Version**: v1.0  
**Updated**: August 30, 2025  
**Context**: Sprint 005 - Search & Filtering Issues Resolution  

---

## Overview

This guide provides solutions for common search filtering issues across all interfaces (CLI, REST API, JSON-RPC) in the Zero-Latency document search system.

## Quick Diagnosis

### Symptoms Checklist

- [ ] **No results returned**: Search returns empty results when you expect matches
- [ ] **Collection filtering not working**: Results from wrong collections appear
- [ ] **Inconsistent results**: Different interfaces return different results for same query
- [ ] **Performance issues**: Searches taking too long or timing out
- [ ] **Parameter errors**: Invalid parameter errors when using collection filtering

## Common Issues & Solutions

### 1. Collection Filtering Not Working

#### Symptoms
- Search returns results from all collections instead of specified collection
- Collection parameter seems to be ignored
- Getting results from unexpected collections

#### Diagnosis Steps
```bash
# Test if collection filtering is working
mdx search "test query" --collection zero_latency_docs
mdx search "test query"  # Compare with unfiltered search

# Verify collection exists
mdx collection list
mdx collection get zero_latency_docs
```

#### Solutions

**CLI Interface:**
```bash
# ✅ Correct: Use global --collection parameter
mdx search "your query" --collection collection_name

# ❌ Incorrect: Collection parameter after search command
mdx search "your query" --collection collection_name  # This won't work in some versions
```

**REST API:**
```bash
# ✅ Correct: Use collection parameter or filters.collection_name
curl "http://localhost:8081/search?q=query&collection=collection_name"

# ✅ Alternative: POST with filters
curl -X POST "http://localhost:8081/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "test", "filters": {"collection_name": "zero_latency_docs"}}'
```

**JSON-RPC API:**
```bash
# ✅ Correct: Use filters.collection in params
curl -X POST "http://localhost:8081/jsonrpc" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "document.search",
    "params": {
      "query": "test",
      "filters": {"collection": "zero_latency_docs"}
    },
    "id": 1
  }'
```

### 2. No Results Returned

#### Symptoms
- Search returns empty results for queries that should match
- Previously working searches now return nothing
- Collection exists but no documents found

#### Diagnosis Steps
```bash
# Check if collection has documents
mdx collection get collection_name

# Test with broader search terms
mdx search "common_word"

# Check if documents are indexed
mdx status
```

#### Solutions

**Check Collection Status:**
```bash
# Verify collection exists and has documents
mdx collection list --include-stats

# Expected output should show document count > 0
Collection: zero_latency_docs
Documents: 150
Status: active
```

**Re-index if Necessary:**
```bash
# If collection is empty, re-index documents
mdx index /path/to/documents --collection collection_name
```

**Adjust Search Parameters:**
```bash
# Lower similarity threshold
mdx search "query" --threshold 0.3 --collection collection_name

# Increase result limit
mdx search "query" --limit 50 --collection collection_name
```

### 3. Invalid Collection Names

#### Symptoms
- "Collection not found" errors
- Empty results for valid queries
- Parameter validation errors

#### Diagnosis Steps
```bash
# List available collections
mdx collection list

# Check exact collection name spelling
mdx collection get "exact_collection_name"
```

#### Solutions

**Verify Collection Names:**
```bash
# Get correct collection names
mdx collection list --format json | jq '.[] | .name'

# Use exact collection name (case-sensitive)
mdx search "query" --collection "zero_latency_docs"  # Not "Zero_Latency_Docs"
```

### 4. Cross-Interface Inconsistencies

#### Symptoms
- CLI returns different results than API
- REST API and JSON-RPC return different result counts
- Parameter naming confusion

#### Diagnosis Steps
```bash
# Test same query across interfaces
mdx search "test" --collection zero_latency_docs

curl "http://localhost:8081/search?q=test&collection=zero_latency_docs"

curl -X POST "http://localhost:8081/jsonrpc" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "document.search", "params": {"query": "test", "filters": {"collection": "zero_latency_docs"}}, "id": 1}'
```

#### Solutions

**Use Correct Parameter Names:**

| Interface | Collection Parameter | Example |
|-----------|---------------------|---------|
| CLI | `--collection name` | `mdx search "query" --collection zero_latency_docs` |
| REST API (GET) | `collection=name` | `curl "http://localhost:8081/search?q=query&collection=zero_latency_docs"` |
| REST API (POST) | `filters.collection_name` | `{"query": "test", "filters": {"collection_name": "zero_latency_docs"}}` |
| JSON-RPC | `filters.collection` | `{"params": {"query": "test", "filters": {"collection": "zero_latency_docs"}}}` |

### 5. Performance Issues

#### Symptoms
- Searches taking longer than expected
- Timeouts on collection-filtered searches
- High CPU/memory usage during searches

#### Diagnosis Steps
```bash
# Compare filtered vs unfiltered search times
time mdx search "query" --collection collection_name
time mdx search "query"

# Check collection size
mdx collection get collection_name --include-stats
```

#### Solutions

**Optimize Search Parameters:**
```bash
# Reduce result limit for faster searches
mdx search "query" --collection collection_name --limit 5

# Increase threshold to reduce processing
mdx search "query" --collection collection_name --threshold 0.7
```

**Check System Resources:**
```bash
# Monitor system during search
top -p $(pgrep doc-indexer)

# Check available memory
free -h
```

### 6. Service Connection Issues

#### Symptoms
- Connection refused errors
- API endpoint not responding
- Service not found errors

#### Diagnosis Steps
```bash
# Check if service is running
curl http://localhost:8081/health

# Check service status
ps aux | grep doc-indexer

# Test service connectivity
mdx --server http://localhost:8081 status
```

#### Solutions

**Start Service:**
```bash
# Start doc-indexer service
cargo run --bin doc-indexer -- --port 8081

# Or use background process
nohup cargo run --bin doc-indexer -- --port 8081 &
```

**Verify Service Health:**
```bash
# Health check
curl http://localhost:8081/health

# Expected response: {"status": "healthy", ...}
```

## Advanced Troubleshooting

### Enable Debug Logging

```bash
# CLI with verbose output
mdx --verbose search "query" --collection collection_name

# Service with debug logging
RUST_LOG=debug cargo run --bin doc-indexer -- --port 8081
```

### Validate API Responses

```bash
# Check JSON-RPC response format
curl -X POST "http://localhost:8081/jsonrpc" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "document.search", "params": {"query": "test"}, "id": 1}' | jq .

# Verify REST API response
curl "http://localhost:8081/search?q=test" | jq .
```

### Test Collection Filtering Manually

```bash
# Use comprehensive test suite
./test/run_search_filtering_tests.sh

# Run specific test types
./test/run_search_filtering_tests.sh python
./test/run_search_filtering_tests.sh rust
```

## Best Practices

### 1. Collection Management
- Use descriptive collection names
- Keep collection names consistent (lowercase, underscores)
- Regularly check collection statistics

### 2. Search Optimization
- Use specific collection filtering when possible
- Start with lower thresholds and adjust up
- Limit results for faster response times

### 3. Interface Selection
- **CLI**: Best for interactive use and scripting
- **REST API**: Best for web applications and general HTTP clients  
- **JSON-RPC**: Best for programmatic access and RPC-style applications

### 4. Error Handling
- Always check service health before operations
- Validate collection existence before searching
- Handle empty results gracefully in applications

## Getting Help

### Documentation
- `docs/CLI_REFERENCE.md` - Complete CLI documentation
- `docs/API_REFERENCE.md` - REST and JSON-RPC API documentation
- `docs/sprint/sprint-005-search-filtering-issues.md` - Implementation details

### Testing
- `test/run_search_filtering_tests.sh` - Comprehensive test suite
- `test/integration/test_search_filtering.py` - Python integration tests

### Support
- Check existing issues in project documentation
- Run diagnostic tests with verbose output
- Review service logs for error details

---

**Last Updated**: August 30, 2025  
**Related Sprint**: ZL-005 (Search & Filtering Issues Resolution)  
**Test Coverage**: 100% (11/11 test scenarios passing)
