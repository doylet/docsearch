# ZL-004-001: Document ID Preservation Investigation Results

## Sprint 004 Epic 1 - Task ZL-004-001 Results

### Investigation Summary
**Status**: ✅ COMPLETE - Root cause identified  
**Finding**: Document ID preservation is working correctly; metadata loss occurs in search response pipeline

### Root Cause Analysis

#### What's Working ✅
1. **MCP Interface**: Correctly receives and parses document ID from `params.id`
2. **JSON-RPC Handler**: Correctly validates UUID and creates Document with provided ID 
3. **Document Service**: Uses provided `document.id` correctly for chunk creation
4. **Vector Storage**: Stores chunks with correct `document_id` in VectorMetadata
5. **Collection Assignment**: Correctly stores collection name in VectorMetadata.collection

#### What's Broken ❌

**Issue 1: Search Result Metadata Loss**
- **Location**: `crates/zero-latency-search/src/vector_search.rs:101-120`
- **Problem**: VectorSearchStep ignores VectorMetadata.collection and VectorMetadata.custom when creating SearchResult objects
- **Impact**: Collection and custom metadata are lost during search result conversion

**Issue 2: Response Handler Metadata Override**  
- **Location**: `services/doc-indexer/src/infrastructure/jsonrpc/handlers.rs:347`
- **Problem**: Search response handler hardcodes `metadata: std::collections::HashMap::new()`
- **Impact**: Even if metadata were preserved in SearchResult, it would be overridden with empty HashMap

**Issue 3: SearchResult Schema Gap**
- **Location**: `crates/zero-latency-search/src/models.rs:83-93`
- **Problem**: SearchResult struct lacks metadata fields that exist in VectorMetadata
- **Impact**: No way to carry collection/custom metadata through search pipeline

### Technical Details

#### Data Flow Analysis
```
✅ MCP Request (params.id) 
    → ✅ JSON-RPC Handler (document.id)
    → ✅ Document Service (document.id) 
    → ✅ Chunk Creation (chunk.document_id)
    → ✅ Vector Storage (VectorMetadata.document_id + collection + custom)
    → ❌ Vector Search (SimilarityResult → SearchResult metadata loss)
    → ❌ JSON-RPC Response (hardcoded empty metadata)
```

#### Available Data Not Used
- `SimilarityResult.metadata.collection` contains correct collection name
- `SimilarityResult.metadata.custom` contains original custom metadata
- Both fields are completely ignored in vector_search.rs conversion

### Solution Architecture

**Phase 1: Extend SearchResult Schema**
- Add collection and custom metadata fields to SearchResult struct
- Maintain backward compatibility

**Phase 2: Fix Vector Search Conversion** 
- Update vector_search.rs to preserve collection and custom metadata
- Map SimilarityResult.metadata → SearchResult metadata fields

**Phase 3: Fix Response Handler**
- Update JSON-RPC handler to use SearchResult metadata instead of empty HashMap
- Ensure proper serialization of collection and custom fields

### Next Steps
- [ZL-004-002]: Implement SearchResult schema extension
- [ZL-004-003]: Fix VectorSearchStep metadata preservation  
- [ZL-004-004]: Update JSON-RPC response handler
- [ZL-004-005]: Integration testing and validation

### Files to Modify
1. `crates/zero-latency-search/src/models.rs` - Extend SearchResult
2. `crates/zero-latency-search/src/vector_search.rs` - Fix metadata conversion
3. `services/doc-indexer/src/infrastructure/jsonrpc/handlers.rs` - Fix response formatting

---
**Investigation Complete**: Root cause confirmed - metadata pipeline breaks at search result conversion, not document ID handling.
