# ADR-041: Schema-First Contract Architecture Strategy

**Status:** Proposed  
**Date:** August 28, 2025  
**Decision Makers:** Development Team  
**Supersedes:** ADR-008 (Contract-First Development Approach)  
**Technical Story:** Multi-Protocol API Contract Management and Schema Evolution

## Context

The Zero-Latency system currently implements a **multi-layered contract architecture** with different schema formats for different protocols:

1. **Manual Contract Constants** (`zero-latency-contracts` crate)
2. **OpenAPI Specifications** (minimal/placeholder)  
3. **Protocol Buffers** (gRPC internal communication)
4. **JSON-RPC 2.0 Schemas** (MCP compliance)
5. **Configuration Schemas** (environment/file-based)

### Current Implementation Analysis

**Current Approach:**
```rust
// Manual endpoint definitions
pub const COLLECTIONS: &str = "/api/collections";
pub const COLLECTION_BY_NAME: &str = "/api/collections/:name";

// Manual helper functions  
pub fn collection_by_name(name: &str) -> String {
    COLLECTION_BY_NAME.replace(":name", name)
}

// Separate type definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub document_count: usize,
}
```

**Identified Issues:**
- ❌ **Manual Maintenance Burden**: Hand-written constants and helpers
- ❌ **Schema Drift Risk**: No single source of truth across protocols
- ❌ **Limited Code Generation**: Missing client SDKs and documentation
- ❌ **Protocol Fragmentation**: Different schemas for REST, JSON-RPC, gRPC
- ❌ **Validation Inconsistency**: Manual validation logic across components

### Requirements Driving Multi-Protocol Support

The system legitimately requires multiple communication protocols:

1. **Model Context Protocol (MCP)**: JSON-RPC 2.0 for AI tool integration
2. **REST APIs**: HTTP for web client and external integrations  
3. **gRPC**: High-performance internal service communication
4. **WebSocket**: Real-time streaming for chat interfaces

## Decision

We will adopt a **Schema-First Contract Architecture** that uses **OpenAPI 3.1 as the canonical source of truth** with protocol-specific adapters generated from the central schema definition.

### Architecture Principles

#### 1. Single Source of Truth
All API contracts derive from a comprehensive OpenAPI 3.1 specification:

```yaml
# api/schemas/zero-latency-api.yaml
openapi: 3.1.0
info:
  title: Zero Latency API
  version: 1.0.0
  description: Unified API for document indexing and semantic search

components:
  schemas:
    Collection:
      type: object
      required: [name, document_count, created_at]
      properties:
        name: 
          type: string
          pattern: '^[a-zA-Z0-9_-]+$'
        document_count:
          type: integer
          minimum: 0
        created_at:
          type: string
          format: date-time

paths:
  /api/collections:
    get:
      operationId: listCollections
      summary: List all collections
      responses:
        '200':
          description: Collection list
          content:
            application/json:
              schema:
                type: object
                properties:
                  collections:
                    type: array
                    items:
                      $ref: '#/components/schemas/Collection'
```

#### 2. Code Generation Pipeline
Generate protocol-specific implementations from the central schema:

```bash
# Generate Rust types and traits
openapi-generator generate -i api/schemas/zero-latency-api.yaml \
  -g rust -o crates/zero-latency-api/

# Generate JSON-RPC schemas  
openapi-to-jsonrpc api/schemas/zero-latency-api.yaml \
  --output api/jsonrpc/

# Generate Protocol Buffer definitions
openapi-to-proto api/schemas/zero-latency-api.yaml \
  --output api/proto/zero_latency.proto

# Generate client SDKs
openapi-generator generate -i api/schemas/zero-latency-api.yaml \
  -g typescript-fetch -o clients/typescript/
```

#### 3. Protocol Adapter Pattern
Implement thin adapters that expose the same domain service through different protocols:

```rust
// Generated from OpenAPI
use zero_latency_api::{
    DocumentService,           // Generated trait
    Collection, SearchRequest, // Generated types
};

// Core business logic (protocol-agnostic)
pub struct DocumentServiceImpl {
    repository: Arc<dyn DocumentRepository>,
    search_engine: Arc<dyn SearchEngine>,
}

#[async_trait]
impl DocumentService for DocumentServiceImpl {
    async fn list_collections(&self) -> Result<Vec<Collection>> {
        // Business logic implementation
        self.repository.list_collections().await
    }
    
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        // Business logic implementation  
        self.search_engine.search(request).await
    }
}

// Protocol adapters (thin wrappers)
pub struct RestAdapter {
    service: Arc<dyn DocumentService>,
}

impl RestAdapter {
    pub fn router(&self) -> Router {
        Router::new()
            .route("/api/collections", get(self.list_collections_handler()))
            .route("/api/search", post(self.search_handler()))
    }
    
    async fn list_collections_handler(&self) -> impl IntoResponse {
        match self.service.list_collections().await {
            Ok(collections) => Json(collections).into_response(),
            Err(e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

pub struct JsonRpcAdapter {
    service: Arc<dyn DocumentService>,
}

impl JsonRpcAdapter {
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "listCollections" => {
                let collections = self.service.list_collections().await?;
                JsonRpcResponse::success(request.id, collections)
            }
            "search" => {
                let params: SearchRequest = serde_json::from_value(request.params)?;
                let result = self.service.search(params).await?;
                JsonRpcResponse::success(request.id, result)
            }
            _ => JsonRpcResponse::error(request.id, "Method not found"),
        }
    }
}
```

### Implementation Strategy

#### Phase 1: Schema Definition and Validation
```yaml
# Directory structure
api/
  schemas/
    zero-latency-api.yaml           # Master OpenAPI spec
    components/
      schemas/                      # Reusable schema components
        collection.yaml
        document.yaml  
        search.yaml
      responses/                    # Standard response schemas
        error.yaml
        pagination.yaml
  generated/                        # Generated artifacts (gitignored)
    rust/                          # Generated Rust code
    jsonrpc/                       # Generated JSON-RPC schemas
    proto/                         # Generated Protocol Buffers
    clients/                       # Generated client SDKs
```

#### Phase 2: Code Generation Integration
```toml
# Cargo.toml build script integration
[dependencies]
zero-latency-api = { path = "crates/zero-latency-api" }

# crates/zero-latency-api/build.rs
fn main() {
    // Regenerate types from OpenAPI spec if changed
    if schema_changed() {
        generate_rust_types();
        generate_jsonrpc_schemas();
        generate_proto_definitions();
    }
}
```

#### Phase 3: Protocol Adapter Implementation
```rust
// services/doc-indexer/src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    let service = Arc::new(DocumentServiceImpl::new(config).await?);
    
    // Start multiple protocol servers
    let rest_adapter = RestAdapter::new(service.clone());
    let jsonrpc_adapter = JsonRpcAdapter::new(service.clone());
    let grpc_adapter = GrpcAdapter::new(service.clone());
    
    tokio::try_join!(
        serve_rest_api(rest_adapter),
        serve_jsonrpc_api(jsonrpc_adapter),
        serve_grpc_api(grpc_adapter),
    )?;
    
    Ok(())
}
```

## Benefits

### Immediate Benefits
- ✅ **Single Source of Truth**: OpenAPI spec drives all protocol implementations
- ✅ **Automated Code Generation**: Types, clients, and documentation auto-generated
- ✅ **Schema Validation**: Built-in request/response validation
- ✅ **Breaking Change Detection**: Automated compatibility checking
- ✅ **Documentation**: Auto-generated API docs with examples

### Long-term Benefits  
- ✅ **Protocol Agnostic Business Logic**: Core services independent of transport
- ✅ **Easy Protocol Addition**: New protocols via adapter pattern
- ✅ **Client SDK Generation**: Multi-language client support
- ✅ **Contract Testing**: Automated contract compliance validation
- ✅ **Schema Evolution**: Versioned schema migration strategies

### Ecosystem Integration Benefits
- ✅ **Industry Standard Tooling**: Leverage OpenAPI ecosystem
- ✅ **Developer Experience**: Familiar patterns and tools
- ✅ **Interoperability**: Standard schemas enable easy integration
- ✅ **Governance**: Centralized contract management and approval workflows

## Multi-Tenant Architecture Support

The schema-first contract architecture **strongly supports multi-tenant systems** through several key design patterns and architectural considerations. This section addresses the extensibility requirements for adding new services to the Zero-Latency ecosystem.

### Multi-Tenant Schema Design Patterns

#### 1. Tenant-Aware Resource Identification
```yaml
# All resources include tenant context in schema
components:
  schemas:
    TenantResource:
      type: object
      required: [id, tenant_id, created_at]
      properties:
        id:
          type: string
          format: uuid
        tenant_id:
          type: string
          format: uuid
          description: "Unique identifier for the tenant"
        created_at:
          type: string
          format: date-time
    
    Collection:
      allOf:
        - $ref: '#/components/schemas/TenantResource'
        - type: object
          properties:
            name:
              type: string
              pattern: '^[a-zA-Z0-9_-]+$'
            document_count:
              type: integer
              minimum: 0

# Path parameters include tenant context
paths:
  /api/tenants/{tenant_id}/collections:
    parameters:
      - name: tenant_id
        in: path
        required: true
        schema:
          type: string
          format: uuid
    get:
      operationId: listCollectionsByTenant
      summary: List collections for a specific tenant
```

#### 2. Tenant-Specific Configuration Schema
```yaml
components:
  schemas:
    TenantConfiguration:
      type: object
      required: [tenant_id, settings, limits, features]
      properties:
        tenant_id:
          type: string
          format: uuid
        settings:
          $ref: '#/components/schemas/TenantSettings'
        limits:
          $ref: '#/components/schemas/TenantLimits'
        features:
          $ref: '#/components/schemas/TenantFeatures'
    
    TenantSettings:
      type: object
      properties:
        default_collection:
          type: string
        embedding_model:
          type: string
          enum: [openai-ada-002, local-bert, custom]
        search_configuration:
          $ref: '#/components/schemas/SearchConfiguration'
    
    TenantLimits:
      type: object
      properties:
        max_documents:
          type: integer
          minimum: 1
        max_collections:
          type: integer
          minimum: 1
        request_rate_limit:
          type: integer
          description: "Requests per minute"
        storage_quota_gb:
          type: number
          minimum: 0.1
    
    TenantFeatures:
      type: object
      properties:
        advanced_search:
          type: boolean
        real_time_indexing:
          type: boolean
        custom_embeddings:
          type: boolean
        api_access:
          type: boolean
```

### Multi-Service Architecture Patterns

#### 1. Service Registry and Discovery
```yaml
# Service definition schema for multi-tenant discovery
components:
  schemas:
    ServiceRegistration:
      type: object
      required: [service_id, service_type, tenant_support, endpoints]
      properties:
        service_id:
          type: string
          format: uuid
        service_type:
          type: string
          enum: [doc-indexer, search-engine, embedding-service, analytics-service]
        tenant_support:
          type: string
          enum: [single-tenant, multi-tenant, tenant-aware]
        endpoints:
          type: array
          items:
            $ref: '#/components/schemas/ServiceEndpoint'
        health_check:
          $ref: '#/components/schemas/HealthCheckEndpoint'
    
    ServiceEndpoint:
      type: object
      required: [protocol, url, capabilities]
      properties:
        protocol:
          type: string
          enum: [rest, grpc, jsonrpc, websocket]
        url:
          type: string
          format: uri
        capabilities:
          type: array
          items:
            type: string
        tenant_routing:
          type: string
          enum: [path-based, header-based, subdomain-based]

# Cross-service communication patterns
paths:
  /api/services/registry:
    get:
      operationId: listServices
      summary: Discover available services
      parameters:
        - name: tenant_id
          in: header
          required: true
          schema:
            type: string
            format: uuid
        - name: service_type
          in: query
          schema:
            type: string
```

#### 2. Inter-Service Communication Schema
```yaml
# Standardized inter-service request/response patterns
components:
  schemas:
    InterServiceRequest:
      type: object
      required: [request_id, source_service, tenant_context, payload]
      properties:
        request_id:
          type: string
          format: uuid
        source_service:
          type: string
        tenant_context:
          $ref: '#/components/schemas/TenantContext'
        payload:
          type: object
        trace_context:
          $ref: '#/components/schemas/TraceContext'
    
    TenantContext:
      type: object
      required: [tenant_id, user_id]
      properties:
        tenant_id:
          type: string
          format: uuid
        user_id:
          type: string
          format: uuid
        permissions:
          type: array
          items:
            type: string
        feature_flags:
          type: object
          additionalProperties:
            type: boolean
    
    TraceContext:
      type: object
      properties:
        trace_id:
          type: string
        span_id:
          type: string
        parent_span_id:
          type: string
        sampling_decision:
          type: boolean
```

### Multi-Tenant Service Implementation

#### 1. Tenant-Aware Service Architecture
```rust
// Generated trait includes tenant context
#[async_trait]
pub trait DocumentService {
    async fn list_collections(
        &self, 
        tenant_id: Uuid,
        user_context: UserContext
    ) -> Result<Vec<Collection>>;
    
    async fn search(
        &self,
        tenant_id: Uuid,
        request: SearchRequest,
        user_context: UserContext
    ) -> Result<SearchResponse>;
}

// Multi-tenant service implementation
pub struct MultiTenantDocumentService {
    tenant_resolver: Arc<dyn TenantResolver>,
    repository_factory: Arc<dyn RepositoryFactory>,
    search_engine_factory: Arc<dyn SearchEngineFactory>,
    configuration_service: Arc<dyn ConfigurationService>,
}

impl MultiTenantDocumentService {
    async fn get_tenant_context(&self, tenant_id: Uuid) -> Result<TenantContext> {
        let tenant = self.tenant_resolver.resolve(tenant_id).await?;
        let config = self.configuration_service.get_tenant_config(tenant_id).await?;
        
        Ok(TenantContext {
            tenant,
            configuration: config,
            limits: self.get_tenant_limits(tenant_id).await?,
        })
    }
    
    async fn get_tenant_repository(
        &self, 
        tenant_id: Uuid
    ) -> Result<Arc<dyn DocumentRepository>> {
        let context = self.get_tenant_context(tenant_id).await?;
        self.repository_factory.create_for_tenant(context).await
    }
}

#[async_trait]
impl DocumentService for MultiTenantDocumentService {
    async fn list_collections(
        &self,
        tenant_id: Uuid,
        user_context: UserContext
    ) -> Result<Vec<Collection>> {
        // Validate tenant access
        self.validate_tenant_access(tenant_id, &user_context).await?;
        
        // Get tenant-specific repository
        let repository = self.get_tenant_repository(tenant_id).await?;
        
        // Apply tenant-specific filtering and limits
        let collections = repository.list_collections().await?;
        
        // Apply user-specific permissions
        self.filter_by_permissions(collections, &user_context).await
    }
}
```

#### 2. Protocol Adapter with Tenant Support
```rust
// Multi-tenant REST adapter
pub struct MultiTenantRestAdapter {
    service: Arc<dyn DocumentService>,
    tenant_extractor: Arc<dyn TenantExtractor>,
    auth_service: Arc<dyn AuthenticationService>,
}

impl MultiTenantRestAdapter {
    pub fn router(&self) -> Router {
        Router::new()
            .route("/api/tenants/:tenant_id/collections", 
                   get(self.list_collections_handler()))
            .route("/api/tenants/:tenant_id/search", 
                   post(self.search_handler()))
            .layer(AuthenticationLayer::new(self.auth_service.clone()))
            .layer(TenantValidationLayer::new(self.tenant_extractor.clone()))
    }
    
    async fn list_collections_handler(
        &self,
        Path(tenant_id): Path<Uuid>,
        Extension(user_context): Extension<UserContext>
    ) -> impl IntoResponse {
        match self.service.list_collections(tenant_id, user_context).await {
            Ok(collections) => Json(collections).into_response(),
            Err(e) => self.handle_error(e, tenant_id).into_response(),
        }
    }
}

// Tenant extraction middleware
#[async_trait]
impl<S> Layer<S> for TenantValidationLayer {
    type Service = TenantValidationService<S>;
    
    fn layer(&self, service: S) -> Self::Service {
        TenantValidationService::new(service, self.extractor.clone())
    }
}

impl TenantValidationService {
    async fn validate_request(&self, request: &Request) -> Result<Uuid> {
        // Extract tenant ID from path, header, or subdomain
        let tenant_id = self.extractor.extract_tenant_id(request).await?;
        
        // Validate tenant exists and is active
        self.extractor.validate_tenant(tenant_id).await?;
        
        Ok(tenant_id)
    }
}
```

### Schema Evolution in Multi-Tenant Context

#### 1. Tenant-Aware Schema Versioning
```yaml
# Support different schema versions per tenant
components:
  schemas:
    TenantSchemaVersion:
      type: object
      properties:
        tenant_id:
          type: string
          format: uuid
        api_version:
          type: string
          pattern: '^v[0-9]+$'
        schema_version:
          type: string
          pattern: '^[0-9]+\.[0-9]+\.[0-9]+$'
        migration_status:
          type: string
          enum: [pending, in_progress, completed, failed]
        supported_features:
          type: array
          items:
            type: string

# Version-aware endpoints
paths:
  /api/v1/tenants/{tenant_id}/collections:
    # Legacy version for older tenants
  /api/v2/tenants/{tenant_id}/collections:
    # Current version
  /api/tenants/{tenant_id}/collections:
    # Auto-routing based on tenant's preferred version
```

#### 2. Gradual Migration Strategies
```rust
// Schema version-aware service implementation
pub struct VersionAwareDocumentService {
    v1_service: Arc<dyn DocumentServiceV1>,
    v2_service: Arc<dyn DocumentServiceV2>,
    tenant_version_resolver: Arc<dyn TenantVersionResolver>,
}

impl VersionAwareDocumentService {
    async fn route_request(&self, tenant_id: Uuid) -> Arc<dyn DocumentService> {
        match self.tenant_version_resolver.get_version(tenant_id).await {
            Ok(Version::V1) => self.v1_service.clone(),
            Ok(Version::V2) | Err(_) => self.v2_service.clone(), // Default to latest
        }
    }
}

// Migration coordination
pub struct TenantMigrationCoordinator {
    schema_migrator: Arc<dyn SchemaMigrator>,
    data_migrator: Arc<dyn DataMigrator>,
    notification_service: Arc<dyn NotificationService>,
}

impl TenantMigrationCoordinator {
    pub async fn migrate_tenant(
        &self,
        tenant_id: Uuid,
        from_version: SchemaVersion,
        to_version: SchemaVersion
    ) -> Result<MigrationResult> {
        // 1. Validate migration path
        self.validate_migration_path(from_version, to_version).await?;
        
        // 2. Create backup
        self.create_backup(tenant_id).await?;
        
        // 3. Migrate schema
        self.schema_migrator.migrate(tenant_id, from_version, to_version).await?;
        
        // 4. Migrate data
        self.data_migrator.migrate(tenant_id, from_version, to_version).await?;
        
        // 5. Validate migration
        self.validate_migration(tenant_id, to_version).await?;
        
        // 6. Update tenant version
        self.update_tenant_version(tenant_id, to_version).await?;
        
        // 7. Notify stakeholders
        self.notification_service.notify_migration_complete(tenant_id).await?;
        
        Ok(MigrationResult::Success)
    }
}
```

### Multi-Tenant Benefits and Capabilities

#### 1. Tenant Isolation and Security
- **Data Isolation**: Schema-enforced tenant boundaries prevent cross-tenant data access
- **Configuration Isolation**: Tenant-specific settings and feature flags
- **Resource Isolation**: Tenant-specific quotas and rate limiting
- **Security Context**: Per-request tenant and user context validation

#### 2. Service Extensibility
- **Service Registry**: Auto-discovery of new services with tenant capability declaration
- **Protocol Flexibility**: New services can choose optimal protocols while maintaining contract compatibility
- **Feature Rollouts**: Tenant-specific feature flags enable gradual rollouts
- **A/B Testing**: Schema-supported experimentation across tenant segments

#### 3. Operational Excellence
- **Monitoring**: Tenant-aware observability and alerting
- **Scaling**: Independent scaling per tenant or service
- **Deployment**: Rolling deployments with tenant-specific validation
- **Compliance**: Tenant-specific data residency and compliance requirements

## Best Practice Implementation

### Schema Design Patterns

#### 1. Versioned Schema Evolution
```yaml
# Support multiple API versions
openapi: 3.1.0
info:
  version: 2.0.0
paths:
  /api/v1/collections:    # Legacy endpoint
  /api/v2/collections:    # Current endpoint  
  /api/collections:       # Alias to current version

components:
  schemas:
    CollectionV1:         # Legacy schema
    Collection:           # Current schema (v2)
      allOf:
        - $ref: '#/components/schemas/CollectionV1'
        - type: object
          properties:
            metadata:     # New field in v2
              type: object
```

#### 2. Domain-Driven Schema Organization
```yaml
# Organize schemas by domain boundaries
components:
  schemas:
    # Collection domain
    Collection:
      $ref: './domains/collection/collection.yaml'
    CollectionStats:
      $ref: './domains/collection/stats.yaml'
    
    # Document domain  
    Document:
      $ref: './domains/document/document.yaml'
    DocumentMetadata:
      $ref: './domains/document/metadata.yaml'
    
    # Search domain
    SearchRequest:
      $ref: './domains/search/request.yaml'
    SearchResponse:
      $ref: './domains/search/response.yaml'
```

#### 3. Error Handling Standards
```yaml
components:
  schemas:
    ApiError:
      type: object
      required: [code, message, timestamp]
      properties:
        code:
          type: string
          enum: [VALIDATION_ERROR, NOT_FOUND, INTERNAL_ERROR]
        message:
          type: string
        details:
          type: object
        timestamp:
          type: string
          format: date-time
        trace_id:
          type: string
          format: uuid
          
  responses:
    BadRequest:
      description: Invalid request
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/ApiError'
```

### Tooling Integration

#### 1. Development Workflow
```bash
# Developer workflow with schema-first approach
make generate-schemas     # Regenerate all artifacts from OpenAPI
make validate-schemas     # Validate schema correctness
make test-contracts      # Run contract compliance tests
make build               # Build with generated types
```

#### 2. CI/CD Pipeline Integration
```yaml
# .github/workflows/schema-validation.yml
name: Schema Validation
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - name: Validate OpenAPI Schema
        run: swagger-codegen validate -i api/schemas/zero-latency-api.yaml
      
      - name: Check Breaking Changes
        run: oasdiff breaking api/schemas/zero-latency-api.yaml main
        
      - name: Generate and Test Artifacts
        run: |
          make generate-schemas
          cargo test --package zero-latency-api
```

#### 3. Documentation Generation
```yaml
# Auto-generate comprehensive documentation
docs:
  api-reference:
    generated: api/docs/reference/     # Auto-generated from OpenAPI
    source: api/schemas/zero-latency-api.yaml
  
  client-guides:
    typescript: docs/clients/typescript.md
    python: docs/clients/python.md
    rust: docs/clients/rust.md
```

## Alternatives Considered

### Alternative 1: Protocol Buffers-First
**Approach:** Use Protocol Buffers as the primary schema definition language.

**Pros:**
- Strong typing across languages
- Efficient binary serialization
- Built-in evolution support

**Cons:**
- Limited REST API generation  
- Less web ecosystem integration
- More complex for JSON-based protocols

**Verdict:** Rejected due to REST API requirements and web integration needs.

### Alternative 2: JSON Schema-First  
**Approach:** Use JSON Schema as the canonical definition.

**Pros:**
- Native JSON support
- Excellent validation capabilities
- Language agnostic

**Cons:**
- Limited REST API generation
- No built-in HTTP semantics
- Fragmented tooling ecosystem

**Verdict:** Rejected due to lack of HTTP operation semantics.

### Alternative 3: Multi-Source Approach (Current)
**Approach:** Maintain separate schemas for each protocol.

**Pros:**
- Protocol-specific optimization
- No cross-protocol constraints
- Maximum flexibility

**Cons:**
- Schema drift risk
- Manual synchronization
- Maintenance burden
- No single source of truth

**Verdict:** Rejected due to maintenance complexity and drift risk.

## Implementation Plan

### Phase 1: Foundation (Week 1-2)
- [ ] Create comprehensive OpenAPI 3.1 specification
- [ ] Set up code generation pipeline  
- [ ] Migrate `zero-latency-contracts` to generated types
- [ ] Implement basic schema validation

### Phase 2: Protocol Adapters (Week 3-4)  
- [ ] Implement REST adapter with generated types
- [ ] Implement JSON-RPC adapter with generated schemas
- [ ] Add protocol adapter integration tests
- [ ] Migrate existing handlers to use adapters

### Phase 3: Tooling Integration (Week 5-6)
- [ ] Integrate schema generation into build process
- [ ] Add CI/CD schema validation and breaking change detection
- [ ] Generate client SDKs for primary languages
- [ ] Create auto-generated API documentation

### Phase 4: Advanced Features (Week 7-8)
- [ ] Implement schema versioning and migration
- [ ] Add runtime contract validation middleware
- [ ] Create contract testing framework
- [ ] Add performance monitoring for generated code

## Success Metrics

### Technical Metrics
- **Schema Coverage**: 100% of API surface covered by OpenAPI spec
- **Generation Success**: All artifacts generate successfully from schema
- **Contract Compliance**: 0% schema drift between protocols
- **Breaking Change Detection**: 100% breaking changes caught in CI

### Developer Experience Metrics  
- **Time to Add Endpoint**: <30 minutes (schema + generation)
- **Client Integration Time**: <1 hour with generated SDKs
- **Documentation Currency**: 100% auto-generated and current
- **Bug Reduction**: 90% reduction in protocol mismatch bugs

### Ecosystem Metrics
- **Multi-Language Support**: Client SDKs for 3+ languages
- **Integration Ease**: External integrations use generated clients
- **Standards Compliance**: Full OpenAPI 3.1 and JSON-RPC 2.0 compliance
- **Tool Ecosystem**: Leverage standard OpenAPI tooling

## Risk Mitigation

### Risk: Generated Code Complexity
**Mitigation:** Use proven generators (openapi-generator) with extensive configuration options.

### Risk: Build Process Complexity  
**Mitigation:** Isolate generation in dedicated build scripts with clear error handling.

### Risk: Protocol Adapter Performance
**Mitigation:** Keep adapters thin; benchmark and optimize critical paths.

### Risk: Schema Evolution Challenges
**Mitigation:** Implement comprehensive versioning strategy with migration tooling.

## Related ADRs

- **ADR-008**: Contract-First Development Approach (superseded)
- **ADR-039**: JSON-RPC and MCP Protocol Compliance  
- **ADR-040**: Configuration Architecture Centralization

## Validation Criteria

### Ready for Implementation
- [ ] OpenAPI 3.1 specification covers all current endpoints
- [ ] Code generation pipeline produces compilable artifacts
- [ ] Protocol adapters pass integration tests
- [ ] CI/CD pipeline validates schema changes

### Implementation Success
- [ ] All services use generated types exclusively
- [ ] Zero manual endpoint/type definitions in codebase
- [ ] Breaking change detection prevents regressions
- [ ] Client SDKs successfully integrate with external systems

---

**Status:** Proposed  
**Priority:** HIGH - Foundation for scalable multi-protocol architecture  
**Impact:** Eliminates schema drift, reduces maintenance burden, enables ecosystem integration
