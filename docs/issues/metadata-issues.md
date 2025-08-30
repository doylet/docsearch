# Metadata & Collection Management Issues

**Component**: Document Indexing, Search Pipeline, Collection Management  
**Severity**: High  
**Status**: 🔍 Open - Investigation Required  
**Discovered**: August 30, 2025  

## 🎯 Problem Summary

End-to-end testing revealed critical failures in metadata handling and document ID preservation that compromise the system's collection-based organization and external integration capabilities.

## 🚨 Critical Issues

### 1. Empty Search Metadata
**Impact**: All search results return empty metadata `{}`

**Expected Behavior**:
```json
{
  "metadata": {
    "collection": "zero_latency_docs",
    "custom": {"collection": "zero_latency_docs"},
    "document_id": "...",
    "title": "Document Title"
  }
}
```

**Actual Behavior**:
```json
{
  "metadata": {}
}
```

### 2. Document ID Not Preserved
**Impact**: MCP interface ignores provided document IDs

**Expected**: Document indexed with ID `550e8400-e29b-41d4-a716-446655440000`  
**Actual**: Document stored with ID `0f4570db-4edb-457a-b5cf-e76e3eebbe76`

### 3. Collection Association Lost
**Impact**: Documents not properly associated with collections in search results

**Collections API**: Reports "zero_latency_docs" with 35+ documents  
**Search Results**: Show no collection metadata or association

## 🔍 Root Cause Investigation

### Code Analysis
**File**: `services/doc-indexer/src/application/services/document_service.rs`

The indexing code correctly sets metadata:
```rust
// Lines 105-114: Metadata IS being set
let mut custom_metadata = chunk.metadata.custom.clone();
custom_metadata.insert("collection".to_string(), collection_name.to_string());

let vector_doc = VectorDocument {
    metadata: zero_latency_vector::VectorMetadata {
        collection: Some(collection_name.to_string()),  // ✅ Set correctly
        custom: custom_metadata,                         // ✅ Set correctly
        // ...
    },
};
```

### Suspected Failure Points
1. **MCP Interface**: Document ID mapping in MCP tools handler
2. **Search Serialization**: Metadata lost during result conversion
3. **Memory Adapter**: Metadata not preserved during search operations

## 🧪 Test Evidence

### Successful Indexing
```bash
# Document indexed successfully
curl -X POST http://localhost:8081/jsonrpc \
  -d '{"method": "tools/call", "params": {"name": "index_document", "arguments": {"id": "550e8400-e29b-41d4-a716-446655440000", "content": "Test"}}}'

Response: {"result": {"success": true, "document_id": "550e8400-e29b-41d4-a716-446655440000"}}
```

### Document Count Verification
```bash
# Document count increased from 35 to 36
curl http://localhost:8081/api/status
Response: {"total_documents": 36, "status": "healthy"}
```

### Metadata Failure
```bash
# Search shows empty metadata
curl -X POST http://localhost:8081/jsonrpc \
  -d '{"method": "document.search", "params": {"query": "test", "limit": 1}}'

Response: {"result": {"results": [{"metadata": {}, "content": "...", "id": "different-id"}]}}
```

## 📊 Impact Assessment

### Functional Impact
- ✅ **Core Search**: Working - documents indexed and searchable
- ❌ **Collection Organization**: Broken - no collection-based filtering
- ❌ **External Integration**: Broken - document IDs not preserved
- ❌ **Metadata Richness**: Lost - no contextual information in results

### User Experience Impact
- **CLI Users**: Cannot filter by collection
- **API Users**: Cannot rely on document IDs for tracking
- **Integrations**: Cannot associate search results with source systems

## 🔧 Recommended Fixes

### Priority 1: Document ID Preservation
**Target**: MCP interface document handling
```rust
// Ensure provided ID is preserved through indexing pipeline
// File: services/doc-indexer/src/infrastructure/mcp/handlers.rs
pub async fn index_document(params: IndexDocumentParams) -> Result<IndexResponse> {
    // Validate and preserve provided document ID
    let document_id = validate_uuid(&params.id)?;
    // Pass through to document service with preserved ID
}
```

### Priority 2: Metadata Serialization Fix
**Target**: Search result conversion
- Trace metadata flow: Vector Store → Search Results → API Response
- Ensure metadata survives serialization/deserialization
- Add metadata validation in tests

### Priority 3: Collection Association
**Target**: Collection metadata handling
- Verify collection metadata persists in vector storage
- Ensure search results include collection information
- Add collection-specific search tests

## 🧪 Testing Plan

### Integration Tests to Add
```rust
#[tokio::test]
async fn test_document_id_preservation() {
    let provided_id = "550e8400-e29b-41d4-a716-446655440000";
    // Index via MCP with specific ID
    // Search and verify same ID returned
}

#[tokio::test] 
async fn test_metadata_preservation() {
    // Index with collection metadata
    // Search and verify metadata in results
}

#[tokio::test]
async fn test_collection_association() {
    // Index in specific collection
    // Verify collection filtering works
}
```

## 🔗 Related Issues
- [CLI Collection Filtering](search-issues.md#cli-collection-filtering)
- [Protocol Compliance](protocol-issues.md)

## 📈 Next Steps
1. **Investigate MCP Handler**: Trace document ID flow in MCP interface
2. **Debug Metadata Flow**: Add logging to track metadata through search pipeline
3. **Memory Adapter Analysis**: Verify metadata preservation in vector storage
4. **Integration Tests**: Add comprehensive metadata and ID preservation tests
