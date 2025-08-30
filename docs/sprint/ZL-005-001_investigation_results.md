# ZL-005-001: JSON-RPC Collection Filtering Analysis Results

**Story ID:** ZL-005-001  
**Date:** August 30, 2025  
**Status:** COMPLETED ✅  

## Investigation Summary

Analysis completed for JSON-RPC collection filtering behavior to determine if collection filtering issues are CLI-specific or system-wide.

## Key Findings

### 1. JSON-RPC Collection Parameter Support

**Current JSON-RPC API Schema:**
```rust
#[derive(Debug, Deserialize)]
pub struct SearchDocumentsParams {
    pub query: String,
    pub limit: Option<usize>,
    pub filters: Option<HashMap<String, String>>,  // Collection should go here
    pub include_content: Option<bool>,
}
```

**Test Results:**
- ✅ JSON-RPC API accepts `collection` parameter in request
- ❌ Collection parameter is **IGNORED** by search handler
- ❌ Same results returned regardless of collection filter

### 2. Current Behavior Analysis

**Test 1: Unfiltered Search**
```bash
curl -X POST http://localhost:8081/jsonrpc \
  -d '{"method": "document.search", "params": {"query": "test"}}'
```
**Result:** 10 documents found across multiple collections

**Test 2: Collection-Filtered Search**
```bash
curl -X POST http://localhost:8081/jsonrpc \
  -d '{"method": "document.search", "params": {"query": "test", "collection": "docs"}}'
```
**Result:** 10 documents found (identical to unfiltered) - **Collection filter ignored**

### 3. Root Cause Identified

**Issue Location:** `services/doc-indexer/src/infrastructure/jsonrpc/handlers.rs:323-330`

**Current Implementation:**
```rust
pub async fn handle_search_documents(
    params: Option<Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    // ... parameter parsing ...
    match state
        .document_service
        .search_documents(&params.query, params.limit.unwrap_or(10))  // ❌ No collection parameter
        .await
    {
        // ...
    }
}
```

**Problem:** Handler calls `search_documents()` method which searches ALL collections, not `search_documents_in_collection()` method.

### 4. Available Methods Comparison

**Method 1: `search_documents()` (Currently Used)**
- Searches across ALL collections
- No collection filtering
- Used by JSON-RPC handler

**Method 2: `search_documents_in_collection()` (Available but Not Used)**
- Filters by specific collection
- Available in document service
- Not used by JSON-RPC handler

## Scope Determination

**Issue Scope:** System-wide collection filtering not implemented in JSON-RPC API

**Impact Assessment:**
- CLI: Uses broken collection filtering (different issue)
- JSON-RPC: Ignores collection parameter completely
- Both interfaces have collection filtering problems

## Recommended Fix

Modify JSON-RPC handler to:
1. Check for `collection` parameter in request
2. Call appropriate search method based on collection parameter presence
3. Use `search_documents_in_collection()` when collection specified

**Implementation Location:** `services/doc-indexer/src/infrastructure/jsonrpc/handlers.rs`

## Next Steps

- Implement collection parameter handling in JSON-RPC handler
- Test collection filtering with actual collection names
- Verify consistent behavior across all interfaces
- Update API documentation to reflect collection filtering support

---

**Validation Evidence:**
- JSON-RPC accepts collection parameter but ignores it
- Same search results returned regardless of collection filter
- Root cause: Handler doesn't implement collection filtering logic
- System-wide issue affecting JSON-RPC interface

**Status:** Investigation complete, ready for implementation phase
