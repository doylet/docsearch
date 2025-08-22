# JSON-RPC/MCP Protocol Compliance Gap Analysis

**Date:** August 21, 2025  
**Status:** 🔴 NON-COMPLIANT  
**Priority:** HIGH  
**Effort:** Medium (2-3 days)  

## Executive Summary

The Zero-Latency HTTP API is currently **NOT compliant** with JSON-RPC 2.0 or MCP (Model Context Protocol) standards. The current implementation uses a standard REST API with Axum, while JSON-RPC dependencies are included but unused. This creates a compliance gap that needs to be addressed for proper MCP integration.

## Current State Analysis

### ❌ Current Implementation (REST API)

**Framework:** Axum HTTP server  
**Protocol:** Standard REST with JSON payloads  
**Endpoints:**
- `POST /documents` - Index document
- `POST /documents/search` - Search documents  
- `GET /health` - Health check
- `GET /documents/:id` - Get document

**Example Current API:**
```http
POST /documents/search HTTP/1.1
Content-Type: application/json

{
  "query": "search terms",
  "limit": 10
}
```

**Response:**
```json
{
  "query": "search terms",
  "results": [...],
  "total": 5
}
```

### ✅ Required JSON-RPC 2.0 Format

**Standard Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "search_documents",
  "params": {
    "query": "search terms",
    "limit": 10
  },
  "id": 1
}
```

**Standard Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "query": "search terms",
    "results": [...],
    "total": 5
  },
  "id": 1
}
```

### 🎯 Required MCP Protocol Extensions

**MCP-Specific Methods:**
- `tools/list` - List available tools
- `tools/call` - Execute tool with parameters
- `resources/list` - List available resources
- `resources/read` - Read resource content
- `initialize` - Protocol initialization
- `notifications/*` - Various notifications

**MCP Tool Schema Example:**
```json
{
  "name": "search_documents",
  "description": "Search through indexed documents",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": {"type": "string"},
      "limit": {"type": "integer", "default": 10}
    },
    "required": ["query"]
  }
}
```

## Evidence of Planned Support

### 1. Dependencies Already Included
```toml
# From services/doc-indexer/Cargo.toml
jsonrpc-core = "18.0"
jsonrpc-http-server = "18.0"
```

### 2. Documentation References
- **Roadmap:** "Search API (HTTP + JSON-RPC)"
- **Strategy:** "JSON-RPC interface for programmatic access"
- **Architecture:** References to MCP integration

### 3. Infrastructure Prepared
- `api/mcp/tools/` directory (currently empty)
- `edge/zerolatency-mcp/` directory (currently empty)
- MCP transport planning in model-host blueprints

## Compliance Gaps

### 1. **Protocol Layer** 
- ❌ No JSON-RPC 2.0 wrapper
- ❌ No request/response ID handling
- ❌ No error format compliance
- ❌ No batch request support

### 2. **MCP Specifications**
- ❌ No MCP method implementations
- ❌ No tool schema definitions
- ❌ No capability negotiation
- ❌ No MCP initialization sequence

### 3. **Transport Layer**
- ❌ No stdio transport for local MCP
- ❌ No SSE (Server-Sent Events) support
- ❌ No WebSocket transport

## Impact Assessment

### Business Impact
- **Integration Blocker:** Cannot integrate with MCP-compliant clients
- **Protocol Fragmentation:** Mixed REST/JSON-RPC creates confusion
- **Developer Experience:** Non-standard API patterns

### Technical Impact
- **Tool Integration:** MCP tools cannot connect to API
- **Client Libraries:** Standard MCP clients incompatible
- **Future Architecture:** Blocks planned MCP ecosystem integration

## Implementation Strategy

### Phase 1: JSON-RPC Layer (1-2 days)
1. **Wrapper Implementation**
   ```rust
   // Add JSON-RPC wrapper around existing handlers
   use jsonrpc_core::{IoHandler, Params, Value, Result};
   
   pub struct JsonRpcHandler {
       document_service: DocumentService,
   }
   
   impl JsonRpcHandler {
       pub fn create_io_handler(&self) -> IoHandler {
           let mut io = IoHandler::new();
           io.add_method("search_documents", |params| async {
               // Wrap existing search logic
           });
           io
       }
   }
   ```

2. **Dual Protocol Support**
   - Keep existing REST endpoints
   - Add `/rpc` endpoint for JSON-RPC
   - Share same business logic

### Phase 2: MCP Protocol (2-3 days)
1. **MCP Method Implementation**
   ```rust
   // MCP-specific methods
   io.add_method("tools/list", list_tools);
   io.add_method("tools/call", call_tool);
   io.add_method("resources/list", list_resources);
   io.add_method("initialize", initialize_mcp);
   ```

2. **Tool Schema Definitions**
   - Create JSON schemas in `api/mcp/tools/`
   - Define input/output schemas for each tool
   - Implement schema validation

### Phase 3: Transport Extensions (1-2 days)
1. **Additional Transports**
   - Stdio transport for local MCP servers
   - WebSocket support for real-time communication
   - SSE for streaming responses

## Files to Modify

### Core Implementation
- `services/doc-indexer/src/infrastructure/http/handlers.rs` - Add JSON-RPC wrapper
- `services/doc-indexer/src/infrastructure/http/server.rs` - Add RPC route
- `services/doc-indexer/src/infrastructure/http/mod.rs` - Export RPC types

### MCP Integration
- `api/mcp/tools/search.schema.json` - Tool schema
- `api/mcp/tools/index.schema.json` - Tool schema
- `edge/zerolatency-mcp/` - MCP server implementation

### Documentation
- `docs/api/json-rpc-specification.md` - API documentation
- `docs/api/mcp-protocol-guide.md` - MCP integration guide

## Success Criteria

### Technical Validation
- [ ] All endpoints accessible via JSON-RPC 2.0
- [ ] MCP protocol initialization successful
- [ ] Tool schemas validate correctly
- [ ] Error handling follows JSON-RPC spec
- [ ] Backward compatibility maintained

### Integration Testing
- [ ] Standard MCP client can connect
- [ ] Tool execution works end-to-end
- [ ] Batch requests supported
- [ ] Error scenarios handled properly

## Risks and Mitigations

### Risks
1. **Breaking Changes:** Modifying existing API
2. **Complexity:** Adding protocol layer overhead
3. **Performance:** Additional serialization/validation

### Mitigations
1. **Dual Support:** Keep REST and add JSON-RPC
2. **Incremental:** Phase implementation approach
3. **Testing:** Comprehensive protocol compliance tests

## Timeline

**Total Effort:** 4-5 days  
**Week 1 (Days 3-4):** JSON-RPC implementation  
**Week 2 (Days 1-3):** MCP protocol integration  

## Related Issues

- Architecture gap preventing MCP ecosystem integration
- Non-standard API patterns affecting developer adoption
- Missing tool schema definitions blocking automation
- Transport layer limitations preventing local MCP servers

## Next Steps

1. **Create Branch:** `feature/json-rpc-mcp-compliance`
2. **Phase 1:** Implement JSON-RPC wrapper layer
3. **Phase 2:** Add MCP protocol methods
4. **Phase 3:** Implement additional transports
5. **Testing:** Comprehensive protocol compliance validation

---

**Status:** 📋 DOCUMENTED - Ready for implementation  
**Branch:** `feature/json-rpc-mcp-compliance` (to be created)  
**Assignee:** Development Team  
**Priority:** HIGH - Blocking MCP ecosystem integration
