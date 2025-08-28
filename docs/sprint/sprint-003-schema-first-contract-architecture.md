# Sprint Plan: Schema-First Contract Architecture Implementation

**Sprint ID:** ZL-003  
**Sprint Name:** Schema-First Contract Architecture Implementation  
**Start Date:** August 29, 2025  
**End Date:** September 12, 2025  
**Duration:** 15 days (3 weeks)  
**Sprint Goal:** Transition from manual contract management to automated schema-first architecture with multi-protocol support  
**Related:** [ADR-041](../adr/041_schema-first-contract-architecture.md), [Artefact 052](../misc/artefacts/052_schema-first-contract-architecture-recommendation.md)  

---

## 🎯 Sprint Objective

Transform the Zero-Latency system from manual contract constants to a comprehensive schema-first architecture that auto-generates types, clients, and protocol adapters from a central OpenAPI specification. Establish foundation for multi-service and multi-tenant capabilities.

**Success Criteria:**
- [ ] Comprehensive OpenAPI 3.1 specification covering all current endpoints
- [ ] Automated code generation pipeline producing compilable Rust types
- [ ] REST and JSON-RPC protocol adapters functional with generated types
- [ ] CI/CD integration with schema validation and breaking change detection
- [ ] Zero manual endpoint definitions remaining in codebase
- [ ] Multi-tenant schema patterns established for future services
- [ ] Generated client SDKs for TypeScript and Python
- [ ] Auto-generated API documentation available

---

## 📋 Sprint Backlog

### **Epic 1: Schema Foundation & Code Generation**
**Story Points:** 21  
**Priority:** Critical  

#### **ZL-003-001: Create Master OpenAPI Specification**
**Story Points:** 8  
**Priority:** Critical  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] Complete OpenAPI 3.1 specification for all doc-indexer endpoints
- [ ] Tenant-aware schema patterns with `tenant_id` in all resources
- [ ] Comprehensive error response schemas with trace context
- [ ] Request/response validation schemas for all operations
- [ ] Multi-version support patterns (v1, v2, unversioned aliases)
- [ ] Domain-driven schema organization (collection, document, search domains)

**Tasks:**
- [ ] Create `api/schemas/zero-latency-api.yaml` master specification
- [ ] Define core schemas: Collection, Document, SearchRequest, SearchResponse
- [ ] Add tenant-aware resource patterns with TenantResource base schema
- [ ] Implement standardized error handling with ApiError schema
- [ ] Create domain-specific schema modules in `api/schemas/domains/`
- [ ] Add comprehensive examples for all request/response pairs
- [ ] Validate schema against OpenAPI 3.1 specification

**Definition of Done:**
- OpenAPI spec validates with swagger-codegen
- All current endpoints documented with complete schemas
- Tenant patterns established for future multi-tenant services
- Schema examples match current API behavior

---

#### **ZL-003-002: Implement Code Generation Pipeline**
**Story Points:** 8  
**Priority:** Critical  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] Automated Rust type generation from OpenAPI specification
- [ ] Generated types compile successfully and pass basic tests
- [ ] Build system integration with schema change detection
- [ ] Generated JSON-RPC schemas for MCP protocol compliance
- [ ] Client SDK generation for TypeScript and Python
- [ ] Auto-generated API documentation in multiple formats

**Tasks:**
- [ ] Set up openapi-generator-cli in build system
- [ ] Create `crates/zero-latency-api/build.rs` with generation logic
- [ ] Configure Rust code generation with proper serde derives
- [ ] Implement JSON-RPC schema generation for MCP compliance
- [ ] Add TypeScript and Python client generation
- [ ] Create API documentation generation (HTML, Markdown)
- [ ] Add schema validation in CI/CD pipeline
- [ ] Implement breaking change detection with oasdiff

**Definition of Done:**
- `make generate-schemas` produces all artifacts successfully
- Generated Rust types compile and integrate with existing code
- Client SDKs generate and include usage examples
- CI/CD pipeline validates schema changes and detects breaking changes

---

#### **ZL-003-003: Migrate Contract Constants to Generated Types**
**Story Points:** 5  
**Priority:** High  
**Status:** Blocked by ZL-003-002  

**Acceptance Criteria:**
- [ ] Replace `zero-latency-contracts` manual constants with generated types
- [ ] Update all service implementations to use generated types
- [ ] Maintain backward compatibility during migration
- [ ] Remove all manual endpoint and type definitions
- [ ] Update tests to use generated types and validation

**Tasks:**
- [ ] Replace endpoint constants with generated path definitions
- [ ] Update request/response types throughout codebase
- [ ] Migrate URL generation utilities to use generated helpers
- [ ] Update CLI clients to use generated types
- [ ] Refactor server handlers to use generated request/response types
- [ ] Update test fixtures to use generated types
- [ ] Remove deprecated manual contract definitions

**Definition of Done:**
- Zero manual endpoint definitions remain in codebase
- All compilation and tests pass with generated types
- Generated types provide same functionality as manual constants
- Code coverage maintained or improved

---

### **Epic 2: Protocol Adapter Implementation**
**Story Points:** 18  
**Priority:** High  

#### **ZL-003-004: Implement REST Protocol Adapter**
**Story Points:** 6  
**Priority:** High  
**Status:** Blocked by ZL-003-003  

**Acceptance Criteria:**
- [ ] REST adapter using generated types and traits
- [ ] Automatic request/response validation against schema
- [ ] Tenant context extraction and validation middleware
- [ ] Error handling with standardized API error responses
- [ ] Integration with existing axum-based server infrastructure

**Tasks:**
- [ ] Create `RestAdapter` struct with generated trait implementation
- [ ] Implement automatic request validation using generated schemas
- [ ] Add tenant extraction middleware with UUID validation
- [ ] Create standardized error response mapping
- [ ] Update existing handlers to use adapter pattern
- [ ] Add request/response logging with trace context
- [ ] Implement health check and metrics endpoints

**Definition of Done:**
- REST endpoints function identically to current implementation
- Request/response validation prevents invalid data
- Tenant context properly extracted and validated
- Error responses follow standardized schema format

---

#### **ZL-003-005: Implement JSON-RPC Protocol Adapter**
**Story Points:** 8  
**Priority:** High  
**Status:** Blocked by ZL-003-003  

**Acceptance Criteria:**
- [ ] JSON-RPC 2.0 adapter with MCP protocol compliance
- [ ] Generated method schemas for all operations
- [ ] Batch request support as per JSON-RPC specification
- [ ] Proper error handling with JSON-RPC error codes
- [ ] Tool schema generation for MCP ecosystem integration

**Tasks:**
- [ ] Create `JsonRpcAdapter` with generated method routing
- [ ] Implement JSON-RPC 2.0 request/response handling
- [ ] Add MCP protocol method mapping (search, listCollections, etc.)
- [ ] Generate tool schemas for MCP tool registration
- [ ] Implement batch request processing
- [ ] Add JSON-RPC specific error handling
- [ ] Create stdio transport for CLI integration

**Definition of Done:**
- JSON-RPC endpoints pass protocol compliance tests
- MCP tool schemas enable integration with AI platforms
- Batch requests process correctly with proper error isolation
- CLI can communicate via JSON-RPC transport

---

#### **ZL-003-006: Protocol Integration and Testing**
**Story Points:** 4  
**Priority:** Medium  
**Status:** Blocked by ZL-003-004, ZL-003-005  

**Acceptance Criteria:**
- [ ] Integration tests validating contract compliance across protocols
- [ ] Performance benchmarks for protocol adapter overhead
- [ ] Multi-protocol service startup and coordination
- [ ] Contract testing framework for regression prevention

**Tasks:**
- [ ] Create integration test suite for multi-protocol scenarios
- [ ] Add performance benchmarks comparing adapter vs direct implementation
- [ ] Implement multi-protocol server startup in main.rs
- [ ] Create contract testing framework with golden file validation
- [ ] Add smoke tests for all protocol combinations
- [ ] Document protocol adapter patterns for future services

**Definition of Done:**
- All protocol adapters pass integration tests
- Performance overhead is <5% compared to direct implementation
- Contract tests prevent schema drift regressions
- Documentation enables future service development

---

### **Epic 3: Tooling Integration & Documentation**
**Story Points:** 13  
**Priority:** Medium  

#### **ZL-003-007: CI/CD Integration and Validation**
**Story Points:** 5  
**Priority:** Medium  
**Status:** Blocked by ZL-003-002  

**Acceptance Criteria:**
- [ ] Schema validation in CI/CD pipeline with clear error reporting
- [ ] Breaking change detection with automatic PR comments
- [ ] Automated generation and testing of all artifacts
- [ ] Schema lint rules enforcing consistency and best practices

**Tasks:**
- [ ] Add schema validation workflow to GitHub Actions
- [ ] Implement breaking change detection with oasdiff integration
- [ ] Create PR comment automation for schema changes
- [ ] Add spectral linting rules for OpenAPI consistency
- [ ] Integrate artifact generation testing in CI
- [ ] Add schema documentation deployment automation

**Definition of Done:**
- CI prevents invalid schema changes from merging
- Breaking changes require explicit approval and documentation
- All generated artifacts tested automatically
- Schema quality maintained through automated linting

---

#### **ZL-003-008: Client SDK Generation and Documentation**
**Story Points:** 5  
**Priority:** Medium  
**Status:** Blocked by ZL-003-002  

**Acceptance Criteria:**
- [ ] TypeScript client SDK with complete type definitions
- [ ] Python client SDK with async/await support
- [ ] Comprehensive API documentation with interactive examples
- [ ] Usage examples and integration guides for each SDK

**Tasks:**
- [ ] Generate TypeScript SDK with fetch-based HTTP client
- [ ] Generate Python SDK with aiohttp async client
- [ ] Create interactive API documentation with Swagger UI
- [ ] Write integration guides for each generated SDK
- [ ] Add SDK usage examples to documentation
- [ ] Set up automated SDK publication pipeline

**Definition of Done:**
- SDKs compile and pass basic integration tests
- API documentation is comprehensive and navigable
- Integration guides enable quick client development
- SDKs are versioned and published automatically

---

#### **ZL-003-009: Multi-Tenant Service Patterns**
**Story Points:** 3  
**Priority:** Low  
**Status:** Blocked by ZL-003-003  

**Acceptance Criteria:**
- [ ] Tenant-aware service implementation patterns documented
- [ ] Service registry schema for multi-service discovery
- [ ] Inter-service communication patterns with tenant context
- [ ] Migration guide for adding new services to the ecosystem

**Tasks:**
- [ ] Document tenant-aware service implementation patterns
- [ ] Create service registry schema and implementation example
- [ ] Define inter-service communication standards with tenant context
- [ ] Write guide for adding new services with schema-first approach
- [ ] Create tenant configuration management patterns
- [ ] Document schema evolution strategies for multi-tenant systems

**Definition of Done:**
- Service patterns documented with working examples
- Service registry enables multi-service discovery
- Inter-service communication maintains tenant isolation
- New service development guide is complete and tested

---

## 📊 Sprint Planning Details

### Story Point Distribution
- **Epic 1 (Foundation):** 21 points (50%)
- **Epic 2 (Adapters):** 18 points (43%)  
- **Epic 3 (Tooling):** 13 points (31%)
- **Total Sprint:** 52 points

### Weekly Breakdown

#### **Week 1 (Aug 29 - Sep 4): Foundation**
**Focus:** Schema definition and code generation pipeline
- ZL-003-001: Create Master OpenAPI Specification (8 pts)
- ZL-003-002: Implement Code Generation Pipeline (8 pts)
- **Goal:** Working code generation from comprehensive schema

#### **Week 2 (Sep 5 - Sep 11): Migration & Adapters**
**Focus:** Contract migration and protocol adapter implementation
- ZL-003-003: Migrate Contract Constants (5 pts)
- ZL-003-004: Implement REST Protocol Adapter (6 pts)
- ZL-003-005: Implement JSON-RPC Protocol Adapter (8 pts)
- **Goal:** All protocols functional with generated types

#### **Week 3 (Sep 12): Integration & Documentation**
**Focus:** Testing, tooling, and documentation completion
- ZL-003-006: Protocol Integration and Testing (4 pts)
- ZL-003-007: CI/CD Integration and Validation (5 pts)
- ZL-003-008: Client SDK Generation and Documentation (5 pts)
- ZL-003-009: Multi-Tenant Service Patterns (3 pts)
- **Goal:** Production-ready schema-first architecture

### Team Capacity and Allocation
- **Senior Developer:** Focus on Epic 1 & 2 (schema definition and core adapters)
- **Mid-level Developer:** Focus on Epic 3 (tooling and documentation)
- **Architecture Review:** Weekly checkpoints on schema design and patterns

---

## 🚨 Risk Management

### High-Risk Items
1. **Generated Code Complexity (ZL-003-002)**
   - **Risk:** Generated types don't compile or integrate properly
   - **Mitigation:** Start with simple schemas; iterative complexity addition
   - **Contingency:** Manual type definitions as fallback

2. **Protocol Adapter Performance (ZL-003-004, ZL-003-005)**
   - **Risk:** Adapter pattern introduces unacceptable overhead
   - **Mitigation:** Benchmark early; optimize critical paths
   - **Contingency:** Direct implementation for performance-critical endpoints

3. **Migration Compatibility (ZL-003-003)**
   - **Risk:** Breaking changes during contract migration
   - **Mitigation:** Parallel implementation; gradual cutover
   - **Contingency:** Rollback to manual contracts if critical issues

### Dependencies and Blockers
- **External:** OpenAPI tooling stability and Rust generator quality
- **Internal:** Completion of configuration architecture (Sprint 002)
- **Team:** Learning curve for OpenAPI patterns and tooling

---

## 📈 Success Metrics

### Technical Metrics
- **Schema Coverage:** 100% of current API endpoints in OpenAPI spec
- **Generation Success:** All artifacts generate without errors
- **Type Safety:** Zero runtime type errors in generated code
- **Performance:** <5% overhead for protocol adapters vs direct implementation

### Quality Metrics
- **Test Coverage:** >90% coverage maintained after migration
- **Documentation Currency:** 100% of endpoints documented automatically
- **Breaking Change Prevention:** 100% breaking changes caught in CI
- **Contract Compliance:** All protocols pass contract validation tests

### Developer Experience Metrics
- **Endpoint Addition Time:** <30 minutes from schema to working endpoint
- **Client Integration:** <1 hour for new client using generated SDK
- **Service Development:** <1 day to add new service with full protocol support

---

## 🎯 Definition of Sprint Success

### Minimum Viable Success
- [ ] OpenAPI specification covers all current endpoints
- [ ] Generated Rust types compile and replace manual contracts
- [ ] REST adapter functions identically to current implementation
- [ ] CI/CD validates schema changes

### Target Success  
- [ ] All success criteria met
- [ ] JSON-RPC adapter enables MCP ecosystem integration
- [ ] Client SDKs generated and documented
- [ ] Multi-tenant patterns established

### Stretch Goals
- [ ] gRPC adapter prototype
- [ ] Advanced schema evolution strategies
- [ ] Service mesh integration patterns
- [ ] Performance optimization beyond target metrics

---

## 📋 Sprint Ceremonies

### Daily Standups
- **Focus:** Progress on current epic items
- **Blockers:** Particularly around generated code integration
- **Coordination:** Schema changes affecting multiple team members

### Weekly Sprint Reviews
- **Week 1:** Schema design review and generation pipeline demo
- **Week 2:** Protocol adapter demo and integration testing
- **Week 3:** Complete system demo with documentation review

### Sprint Retrospective
- **Focus:** Schema-first development process effectiveness
- **Lessons:** Code generation tooling and patterns
- **Improvements:** Process refinement for future schema evolution

---

## 🔗 Related Documentation

### ADRs
- [ADR-041: Schema-First Contract Architecture](../adr/041_schema-first-contract-architecture.md)
- [ADR-040: Configuration Architecture Centralization](../adr/040-configuration-architecture-centralization.md)
- [ADR-039: JSON-RPC and MCP Protocol Compliance](../adr/039-json-rpc-mcp-protocol-compliance.md)

### Artefacts
- [052: Schema-First Contract Architecture Recommendation](../misc/artefacts/052_schema-first-contract-architecture-recommendation.md)
- [051: Implementation Status Report](../misc/artefacts/051_implementation-status-report-august-27.md)

### Previous Sprints
- [Sprint 002: Configuration Architecture Implementation](sprint-002-configuration-architecture-implementation.md)
- [Sprint 001: Advanced Search Pipeline Activation](sprint-001-advanced-search-pipeline-activation.md)

---

**Status:** 📋 SPRINT PLAN READY  
**Next:** Sprint execution and daily tracking  
**Priority:** HIGH - Foundation for multi-service architecture  
**Impact:** CRITICAL - Enables scalable contract management
