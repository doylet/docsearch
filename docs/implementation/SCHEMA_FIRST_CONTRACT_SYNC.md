# Schema-First Contract Architecture Implementation Plan

## Overview
Implement schema-first design to ensure CLI remains synchronized with doc-indexer server using OpenAPI specifications as the single source of truth.

## Current State Analysis

### Schema Definition
- **OpenAPI Schema**: `api/schemas/zero-latency-api.yaml` defines REST API contracts
- **Generated Types**: `crates/zero-latency-api/` contains auto-generated Rust types
- **Build Integration**: `build.rs` generates types from OpenAPI schema

### API Endpoint Variants
1. **REST API**: `/api/search` - Uses OpenAPI-defined schemas
2. **JSON-RPC API**: `/jsonrpc` - Uses custom MCP tool definitions
3. **CLI Client**: HTTP client making REST API calls

## Schema Synchronization Issues

### 1. Request Format Mismatch (FIXED)
- **Issue**: CLI was sending `collection_name` as top-level property
- **Schema**: Requires `filters.collection_name` nested structure
- **Fix**: Updated `SearchApiClient` to use schema-compliant format

### 2. JSON-RPC vs REST API Divergence
- **JSON-RPC**: Uses custom `SearchDocumentsParams` type
- **REST API**: Uses OpenAPI-generated `SearchRequest` type
- **Result**: Different behavior for same logical operation

## Implementation Strategy

### Phase 1: Schema-First Type Generation
```rust
// In build.rs - enhance existing type generation
fn generate_api_types() {
    // Current: Generate REST API types from OpenAPI
    generate_openapi_types();
    
    // New: Generate JSON-RPC types from same schema
    generate_jsonrpc_types_from_openapi();
    
    // New: Generate CLI client types
    generate_cli_types_from_openapi();
}
```

### Phase 2: Unified Request/Response Types
```rust
// Use OpenAPI-generated types for all endpoints
use zero_latency_api::models::{SearchRequest, SearchResponse};

// REST Handler
async fn search_documents(Json(request): Json<SearchRequest>) -> Json<SearchResponse> {
    // Implementation
}

// JSON-RPC Handler  
async fn handle_search_documents(params: SearchRequest) -> SearchResponse {
    // Same implementation, different transport
}
```

### Phase 3: Contract Testing
```rust
// Generate contract tests from OpenAPI schema
#[cfg(test)]
mod contract_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_search_request_schema_compliance() {
        // Validate REST API against schema
        // Validate JSON-RPC API against schema
        // Validate CLI client against schema
    }
}
```

## Schema-First Enforcement Tools

### 1. Build-Time Validation
```toml
# In Cargo.toml
[build-dependencies]
openapi-generator = "1.0"
schemars = "0.8"
```

### 2. Runtime Schema Validation
```rust
// Middleware for request validation
pub struct SchemaValidationMiddleware {
    schema: OpenApiSchema,
}

impl SchemaValidationMiddleware {
    pub fn validate_request<T>(&self, request: &T) -> Result<(), ValidationError> {
        // Validate against OpenAPI schema
    }
}
```

### 3. CLI Schema Sync Check
```bash
# New CLI command
mdx config validate-schema --server http://localhost:8081
# Fetches server's OpenAPI spec and validates client compatibility
```

## Benefits

1. **Single Source of Truth**: OpenAPI schema drives all API implementations
2. **Automatic Sync**: Generated types prevent drift between client and server
3. **Contract Testing**: Automated validation ensures compliance
4. **Developer Experience**: IDE completion and compile-time validation
5. **Documentation**: Self-documenting API with examples

## Implementation Files

### New Files
- `crates/zero-latency-api/src/contract.rs` - Contract validation utilities
- `crates/zero-latency-api/src/jsonrpc.rs` - JSON-RPC type mappings
- `tools/schema-validator/` - Standalone schema validation tool

### Modified Files
- `crates/zero-latency-api/build.rs` - Enhanced type generation
- `services/doc-indexer/src/infrastructure/jsonrpc/handlers.rs` - Use shared types
- `crates/cli/src/infrastructure/http/` - Schema-validated clients

## Migration Path

1. **Immediate**: Fix CLI schema compliance (DONE)
2. **Phase 1**: Generate unified types from OpenAPI schema
3. **Phase 2**: Migrate JSON-RPC handlers to use OpenAPI types
4. **Phase 3**: Add contract testing and validation
5. **Phase 4**: Implement schema evolution and versioning

## Example Usage

```rust
// Schema-first API client
use zero_latency_api::client::SchemaValidatedClient;

let client = SchemaValidatedClient::new("http://localhost:8081")?;
let request = SearchRequest {
    query: "architecture".to_string(),
    filters: Some(SearchFilters {
        collection_name: Some("zero_latency_docs".to_string()),
        ..Default::default()
    }),
    ..Default::default()
};

let response = client.search(request).await?;
// Compile-time guarantee of schema compliance
```
