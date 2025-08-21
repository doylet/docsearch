# ADR-039: JSON-RPC and MCP Protocol Compliance Implementation

**Status:** Proposed  
**Date:** August 21, 2025  
**Decision Makers:** Development Team  
**Technical Story:** JSON-RPC/MCP Protocol Compliance Gap

## Context

The Zero-Latency HTTP API currently implements a standard REST API using Axum, but lacks compliance with JSON-RPC 2.0 and MCP (Model Context Protocol) standards. This creates integration barriers for:

1. **MCP-compliant clients** that expect JSON-RPC 2.0 transport
2. **Tool ecosystem integration** requiring MCP protocol methods
3. **Agent frameworks** that rely on MCP for tool execution
4. **Standard MCP tooling** that cannot connect to our API

### Current State
- REST API with custom JSON payloads
- JSON-RPC dependencies included but unused
- MCP directories created but empty
- No protocol compliance validation

### Business Impact
- Cannot integrate with MCP ecosystem
- Blocks agent platform development
- Prevents standard tooling adoption
- Creates developer friction

## Decision

We will implement **dual protocol support** by adding JSON-RPC 2.0 and MCP compliance while maintaining backward compatibility with the existing REST API.

### Implementation Strategy

#### Phase 1: JSON-RPC 2.0 Layer
- Add JSON-RPC wrapper around existing business logic
- Implement `/rpc` endpoint alongside REST endpoints
- Maintain shared service layer for both protocols
- Support batch requests and proper error handling

#### Phase 2: MCP Protocol Methods
- Implement required MCP methods (`tools/list`, `tools/call`, etc.)
- Create JSON Schema definitions for all tools
- Add MCP initialization and capability negotiation
- Support MCP-specific message types

#### Phase 3: Transport Extensions
- Add stdio transport for local MCP servers
- Implement WebSocket support for real-time communication
- Add SSE (Server-Sent Events) for streaming responses

## Architecture

### Protocol Layer Design

```rust
// JSON-RPC handler wrapping existing services
pub struct JsonRpcHandler {
    document_service: Arc<DocumentIndexingService>,
    health_service: Arc<HealthService>,
}

// MCP-specific protocol implementation
pub struct McpProtocolHandler {
    tool_registry: ToolRegistry,
    json_rpc_handler: JsonRpcHandler,
}

// Dual protocol router
Router::new()
    .route("/api/*", rest_handlers)      // Existing REST API
    .route("/rpc", json_rpc_handler)     // New JSON-RPC endpoint
    .route("/mcp", mcp_protocol_handler) // MCP-specific endpoint
```

### Tool Schema Structure

```
api/mcp/tools/
├── search_documents.schema.json
├── index_document.schema.json
├── get_document.schema.json
└── health_check.schema.json
```

### Protocol Mapping

| REST Endpoint | JSON-RPC Method | MCP Tool |
|---------------|-----------------|----------|
| `POST /documents/search` | `search_documents` | `search_documents` |
| `POST /documents` | `index_document` | `index_document` |
| `GET /documents/:id` | `get_document` | `get_document` |
| `GET /health` | `health_check` | `health_check` |

## Alternatives Considered

### 1. Full Migration to JSON-RPC Only
**Pros:** Single protocol, simpler maintenance  
**Cons:** Breaking change, migration burden for existing clients  
**Decision:** Rejected - too disruptive

### 2. Separate JSON-RPC Service
**Pros:** Clean separation, no existing code changes  
**Cons:** Code duplication, maintenance overhead  
**Decision:** Rejected - violates DRY principle

### 3. Protocol Adapter Pattern
**Pros:** Clean abstraction, easy to extend  
**Cons:** Additional complexity layer  
**Decision:** Considered but current dual approach simpler

## Implementation Plan

### Week 1: JSON-RPC Foundation
**Days 1-2:**
- Implement JsonRpcHandler wrapper
- Add `/rpc` endpoint to existing server
- Create JSON-RPC error handling
- Test basic method calls

**Day 3:**
- Add batch request support
- Implement request/response ID handling
- Add comprehensive error mapping
- Create protocol compliance tests

### Week 2: MCP Protocol Integration
**Days 1-2:**
- Implement MCP method handlers
- Create tool schema definitions
- Add MCP initialization sequence
- Implement capability negotiation

**Day 3:**
- Add stdio transport support
- Implement WebSocket transport
- Create MCP client integration tests
- Add protocol documentation

## Benefits

### Technical Benefits
1. **Protocol Compliance:** Full JSON-RPC 2.0 and MCP support
2. **Backward Compatibility:** Existing REST API unchanged
3. **Ecosystem Integration:** Compatible with MCP tooling
4. **Standard Transports:** Support for stdio, WebSocket, HTTP

### Business Benefits
1. **Tool Ecosystem:** Can integrate with MCP-based tools
2. **Agent Platforms:** Enables agent framework integration
3. **Developer Experience:** Standard protocol adoption
4. **Future-Proofing:** Aligned with emerging AI tool standards

## Risks and Mitigations

### Risk 1: Protocol Complexity
**Risk:** Multiple protocols increase maintenance burden  
**Mitigation:** Shared business logic, comprehensive testing

### Risk 2: Performance Impact
**Risk:** Additional protocol layers may impact performance  
**Mitigation:** Lightweight wrappers, performance monitoring

### Risk 3: Version Compatibility
**Risk:** JSON-RPC/MCP spec changes may require updates  
**Mitigation:** Version negotiation, backward compatibility testing

## Metrics and Monitoring

### Success Metrics
- [ ] 100% JSON-RPC 2.0 spec compliance
- [ ] MCP protocol initialization success rate
- [ ] Tool execution success rate via MCP
- [ ] Performance: <10ms additional latency
- [ ] Backward compatibility: 0 breaking changes

### Monitoring
- Protocol usage metrics (REST vs JSON-RPC vs MCP)
- Error rates by protocol
- Response time by protocol
- Client compatibility tracking

## Dependencies

### External Dependencies
- `jsonrpc-core = "18.0"` (already included)
- `jsonrpc-http-server = "18.0"` (already included)
- JSON Schema validation library
- WebSocket transport library

### Internal Dependencies
- Clean architecture implementation (Phase 4C/4D)
- Shared domain crates
- Application service layer
- Infrastructure adapters

## Migration Strategy

### For Existing Clients
1. **No immediate action required** - REST API maintained
2. **Optional migration** to JSON-RPC for better standards compliance
3. **Documentation provided** for protocol migration

### For New Integrations
1. **Prefer JSON-RPC/MCP** for new integrations
2. **Tool developers** should use MCP protocol
3. **Agent platforms** should use MCP transport

## Documentation Requirements

### Technical Documentation
- [ ] JSON-RPC API specification
- [ ] MCP protocol implementation guide
- [ ] Tool schema documentation
- [ ] Transport configuration guide

### Integration Guides
- [ ] Migrating from REST to JSON-RPC
- [ ] Building MCP-compatible tools
- [ ] Client library examples
- [ ] Testing and validation guides

## Validation Criteria

### Acceptance Criteria
- [ ] Standard MCP client can connect and execute tools
- [ ] JSON-RPC 2.0 spec compliance verified
- [ ] All existing REST endpoints remain functional
- [ ] Performance regression < 10%
- [ ] Comprehensive test coverage > 90%

### Integration Testing
- [ ] Standard MCP client integration
- [ ] JSON-RPC client library testing
- [ ] Protocol switching scenarios
- [ ] Error handling verification
- [ ] Batch request processing

## Timeline

**Total Duration:** 6 days  
**Start Date:** August 22, 2025  
**Target Completion:** August 29, 2025  

**Milestones:**
- Day 3: JSON-RPC 2.0 compliance achieved
- Day 6: MCP protocol integration complete
- Day 6: Documentation and testing complete

## Related ADRs

- [ADR-038: Phase 4D Service Extension Strategy](./ADR-038-phase-4d-service-extension-strategy.md)
- [ADR-036: Clean Architecture Implementation](./ADR-036-clean-architecture-implementation.md)

---

**Status:** ✅ Approved for Implementation  
**Next Steps:** Create feature branch and begin Phase 1 implementation  
**Review Date:** August 29, 2025
