# API Contract Formalization Strategy

**Date:** August 25, 2025  
**Purpose:** Prevent regressions through formalized contracts between CLI and doc-indexer service  
**Priority:** HIGH - Critical for system stability  

## 🚨 Current Regression Analysis

### Root Cause: Contract Drift
1. **Port Mismatch**: CLI default (8080) vs Server default (8080) vs CLI server command (8081)
2. **Route Mismatch**: Collection endpoints missing `/api` prefix in server routes
3. **No Contract Validation**: No automated testing of client-server contract compliance

### Identified Issues
- CLI configuration: `http://localhost:8080` (fixed to 8081)
- Server routes: `/collections` vs Client expects: `/api/collections` (fixed)
- No shared contract definition between CLI and server
- No integration tests validating end-to-end functionality

## 🎯 Contract Formalization Strategy

### Phase 1: Shared Contract Definition

Create a shared contract crate that both CLI and server depend on:

```rust
// crates/zero-latency-contracts/src/lib.rs
pub mod api {
    pub const DEFAULT_PORT: u16 = 8081;
    pub const API_PREFIX: &str = "/api";
    
    pub mod endpoints {
        use super::API_PREFIX;
        
        // Server endpoints
        pub const STATUS: &str = const_format::concatcp!(API_PREFIX, "/status");
        pub const SEARCH: &str = const_format::concatcp!(API_PREFIX, "/search");
        pub const INDEX: &str = const_format::concatcp!(API_PREFIX, "/index");
        
        // Collection endpoints
        pub const COLLECTIONS: &str = const_format::concatcp!(API_PREFIX, "/collections");
        pub const COLLECTION: &str = const_format::concatcp!(API_PREFIX, "/collections/{name}");
        pub const COLLECTION_STATS: &str = const_format::concatcp!(API_PREFIX, "/collections/{name}/stats");
        
        // Document endpoints
        pub const DOCUMENTS: &str = const_format::concatcp!(API_PREFIX, "/documents");
        pub const DOCUMENT: &str = const_format::concatcp!(API_PREFIX, "/documents/{id}");
    }
}
```

### Phase 2: Contract-Driven Development

#### Server Implementation
```rust
// services/doc-indexer/src/infrastructure/http/handlers.rs
use zero_latency_contracts::api::endpoints;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route(endpoints::STATUS, get(api_status))
        .route(endpoints::SEARCH, post(search_documents))
        .route(endpoints::INDEX, post(index_documents_from_path))
        .route(endpoints::COLLECTIONS, get(list_collections))
        .route(endpoints::COLLECTIONS, post(create_collection))
        .route(endpoints::COLLECTION, get(get_collection))
        .route(endpoints::COLLECTION, delete(delete_collection))
        .route(endpoints::COLLECTION_STATS, get(get_collection_stats))
        // ... etc
}
```

#### Client Implementation
```rust
// crates/cli/src/infrastructure/http/collection_client.rs
use zero_latency_contracts::api::endpoints;

impl CollectionApiClient {
    pub async fn list_collections(&self) -> ZeroLatencyResult<Vec<CollectionInfo>> {
        let url = format!("{}{}", self.base_url, endpoints::COLLECTIONS);
        // ... rest of implementation
    }
}
```

### Phase 3: Configuration Contracts

#### Shared Configuration Constants
```rust
// crates/zero-latency-contracts/src/config.rs
pub struct DefaultConfig {
    pub const DEFAULT_SERVER_PORT: u16 = 8081;
    pub const DEFAULT_SERVER_HOST: &str = "localhost";
    pub const DEFAULT_COLLECTION_NAME: &str = "zero_latency_docs";
    pub const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 30000;
}

pub fn default_server_url() -> String {
    format!("http://{}:{}", DefaultConfig::DEFAULT_SERVER_HOST, DefaultConfig::DEFAULT_SERVER_PORT)
}
```

#### CLI Configuration Update
```rust
// crates/cli/src/config.rs
use zero_latency_contracts::config::{default_server_url, DefaultConfig};

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            server_url: default_server_url(),
            collection_name: DefaultConfig::DEFAULT_COLLECTION_NAME.to_string(),
            // ... etc
        }
    }
}
```

### Phase 4: Contract Testing

#### Integration Test Suite
```rust
// tests/contract_tests.rs
use zero_latency_contracts::api::endpoints;

#[tokio::test]
async fn test_all_endpoints_respond() {
    let server_url = "http://localhost:8081";
    
    // Test each endpoint exists and responds appropriately
    test_endpoint_exists(&format!("{}{}", server_url, endpoints::STATUS)).await;
    test_endpoint_exists(&format!("{}{}", server_url, endpoints::COLLECTIONS)).await;
    // ... test all endpoints
}

#[tokio::test]
async fn test_cli_server_communication() {
    // Start server
    let _server = start_test_server().await;
    
    // Test CLI commands work end-to-end
    test_cli_command("mdx status").await;
    test_cli_command("mdx collection list").await;
    test_cli_command("mdx search 'test'").await;
}
```

#### Contract Validation CI
```yaml
# .github/workflows/contract-tests.yml
name: API Contract Validation
on: [push, pull_request]

jobs:
  contract-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
      - name: Run Contract Tests
        run: |
          cargo test --test contract_tests
          cargo test --test integration_tests
```

### Phase 5: Schema Validation

#### Request/Response Schemas
```rust
// crates/zero-latency-contracts/src/schemas.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<u32>,
    pub collection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: u64,
    pub query_time_ms: u64,
}

// Implement JSON Schema validation
impl SearchRequest {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.query.is_empty() {
            return Err(ValidationError::EmptyQuery);
        }
        if let Some(limit) = self.limit {
            if limit == 0 || limit > 1000 {
                return Err(ValidationError::InvalidLimit);
            }
        }
        Ok(())
    }
}
```

### Phase 6: Runtime Contract Validation

#### Server Middleware
```rust
// services/doc-indexer/src/infrastructure/middleware/contract_validation.rs
pub async fn validate_request_schema<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Validate request against schema
    if let Err(e) = validate_request_body(&request).await {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let response = next.run(request).await;
    
    // Validate response against schema
    validate_response_schema(&response)?;
    
    Ok(response)
}
```

#### Client Validation
```rust
// crates/cli/src/infrastructure/http/validation.rs
pub trait ContractValidation {
    fn validate_request<T: Serialize>(&self, request: &T) -> Result<(), ContractError>;
    fn validate_response<T: DeserializeOwned>(&self, response: &str) -> Result<T, ContractError>;
}

impl ContractValidation for CollectionApiClient {
    fn validate_request<T: Serialize>(&self, request: &T) -> Result<(), ContractError> {
        // Validate against schema
        Ok(())
    }
}
```

## 🔧 Implementation Plan

### Week 1: Foundation
- [ ] Create `zero-latency-contracts` crate
- [ ] Define endpoint constants and configuration contracts
- [ ] Update server routes to use contract constants
- [ ] Update CLI clients to use contract constants

### Week 2: Testing Infrastructure
- [ ] Implement contract validation tests
- [ ] Create integration test suite
- [ ] Set up CI contract validation
- [ ] Add schema validation for requests/responses

### Week 3: Runtime Validation
- [ ] Implement server-side contract middleware
- [ ] Add client-side request/response validation
- [ ] Create contract violation reporting
- [ ] Add metrics for contract compliance

### Week 4: Documentation & Tooling
- [ ] Generate API documentation from contracts
- [ ] Create contract diff tools for breaking changes
- [ ] Implement contract versioning strategy
- [ ] Add contract compliance dashboard

## 📊 Success Metrics

### Immediate (Week 1)
- [ ] Zero port/route mismatches
- [ ] All endpoints accessible via CLI
- [ ] Shared constants used throughout codebase

### Medium-term (Month 1)
- [ ] 100% contract test coverage
- [ ] Automated regression detection
- [ ] Breaking change prevention in CI

### Long-term (Quarter 1)
- [ ] Zero contract violations in production
- [ ] Automated API documentation
- [ ] Backward compatibility guarantees

## 🚨 Breaking Change Prevention

### Pre-commit Hooks
```bash
#!/bin/bash
# .git/hooks/pre-commit
echo "🔍 Validating API contracts..."
cargo test --test contract_tests --quiet
if [ $? -ne 0 ]; then
    echo "❌ Contract validation failed. Commit rejected."
    exit 1
fi
echo "✅ Contract validation passed."
```

### Pull Request Validation
- Automatic contract diff generation
- Breaking change detection and warnings
- Require manual approval for contract changes

### Semantic Versioning
- Major version bump for breaking contract changes
- Minor version bump for backward-compatible additions
- Patch version bump for implementations-only changes

## 📝 Contract Documentation

### Auto-generated API Docs
```rust
// Use contract definitions to generate OpenAPI specs
#[derive(OpenApi)]
#[openapi(
    paths(list_collections, get_collection, create_collection),
    components(schemas(CollectionInfo, CreateCollectionRequest))
)]
struct ApiDoc;
```

### Contract Registry
Maintain a central registry of all API contracts with:
- Version history
- Breaking change tracking
- Client compatibility matrix
- Migration guides

## 🎯 Next Steps

1. **Immediate**: Create contracts crate and fix current regressions
2. **Short-term**: Implement contract testing infrastructure  
3. **Medium-term**: Add runtime validation and monitoring
4. **Long-term**: Full contract governance and versioning

This strategy ensures we never have port mismatches, route inconsistencies, or breaking changes without proper validation and migration paths.

---

**Status**: Ready for implementation  
**Priority**: HIGH - Prevents production regressions  
**Impact**: Eliminates entire class of integration bugs
