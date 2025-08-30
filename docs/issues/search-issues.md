# Search & Filtering Issues

**Component**: Search Pipeline, CLI Interface, Collection Filtering  
**Severity**: Medium  
**Status**: 📝 Documented  
**Discovered**: Various dates  

## 🎯 Problem Summary

Issues affecting search functionality and filtering capabilities that impact user experience but don't break core system functionality.

## ⚠️ Medium Priority Issues

### CLI Collection Filtering
**Status**: 📝 Documented  
**File**: `../implementation/CLI_SEARCH_COLLECTION_FILTERING_ISSUE.md`

**Problem**: CLI search with collection filters returns no results despite documents existing in the specified collection.

**Example**:
```bash
# This returns no results
mdx search "query" --collection zero_latency_docs

# This works fine  
mdx search "query"
```

**Root Cause**: Collection filtering logic in CLI search implementation doesn't properly handle collection parameter.

**Workaround**: Use CLI without collection filter, or use JSON-RPC API directly.

## 🔍 Investigation Status

### CLI Collection Filtering Deep Dive
**Component**: CLI Search Service  
**Files Affected**:
- CLI search command handler
- Search service collection filtering logic
- API client collection parameter passing

**Testing Evidence**:
- JSON-RPC API without collection filter: ✅ Works
- JSON-RPC API with collection filter: ❓ Needs testing
- CLI without collection filter: ✅ Works  
- CLI with collection filter: ❌ Fails (returns no results)

### API vs CLI Comparison
| Interface | Collection Filter | Status | Results |
|-----------|-------------------|--------|---------|
| JSON-RPC | None | ✅ Working | Returns documents |
| JSON-RPC | Specified | ❓ Unknown | Needs testing |
| CLI | None | ✅ Working | Returns documents |
| CLI | Specified | ❌ Broken | No results |

## 🔧 Recommended Investigation

### Next Steps
1. **Test JSON-RPC with Collection Filter**: Verify if the issue is CLI-specific or affects the entire collection filtering system
2. **CLI Parameter Tracing**: Add logging to trace how collection parameter flows from CLI to API
3. **Collection Validation**: Verify collection names and existence in filtering logic

### Potential Fixes
```rust
// CLI search service - ensure collection parameter is passed correctly
pub async fn search_with_collection(query: &str, collection: &str) -> Result<SearchResponse> {
    // Verify collection parameter is included in API request
    let request = SearchRequest::new(query).with_collection(collection);
    // Ensure API client properly serializes collection parameter
}
```

## 🧪 Testing Required

### Manual Testing
```bash
# Test JSON-RPC collection filtering
curl -X POST http://localhost:8081/jsonrpc \
  -d '{"method": "document.search", "params": {"query": "test", "collection": "zero_latency_docs"}}'

# Test CLI collection filtering with debugging
mdx search "test" --collection zero_latency_docs --verbose
```

### Automated Testing
```rust
#[tokio::test]
async fn test_cli_collection_filtering() {
    // Index documents in specific collection
    // Test CLI search with collection filter
    // Verify results are returned
}

#[tokio::test]
async fn test_json_rpc_collection_filtering() {
    // Test JSON-RPC search with collection parameter
    // Compare with unfiltered search
}
```

## 🔗 Related Issues
- [Metadata Issues](metadata-issues.md) - Collection association problems may be related
- [Search Limit Bug Fix](../implementation/SEARCH_LIMIT_BUG_FIX.md) - Recent search pipeline fix

## 📊 Impact Assessment

### User Impact
- **CLI Users**: Cannot filter search results by collection
- **API Users**: Unknown if collection filtering works at API level
- **Workflow Impact**: Users must manually filter results or use workarounds

### Priority Justification
- **Medium Priority**: Core search works, but filtering is a valuable feature
- **Not Critical**: System remains functional for primary use cases
- **UX Impact**: Reduces efficiency but doesn't break workflows entirely
