# ADR: Contract-First Development Approach for the Markdown Daemon

## Status
Proposed

## Context
We are building a Rust daemon that watches a local folder for Markdown file changes, chunks documents, embeds them, and indexes them into a vector database (Qdrant). The daemon also exposes search APIs (HTTP and JSON-RPC) and a CLI.  
To ensure stable interfaces and decoupled development, we want to adopt a **contract-first (spec-first)** development approach.

## Decision
We will define **contracts before implementations**, using formal schemas and specifications for all external and internal boundaries. These contracts will be versioned, tested, and validated in CI/CD.

### 1. Contract formats
- **HTTP APIs:** OpenAPI 3.1 + JSON Schemas for requests/responses
- **JSON-RPC APIs:** JSON Schemas for each `method.params` and `result`
- **Event bus / internal pipelines:** Protobuf or JSON Schema for messages
- **Vector store adapter boundary:** Rust traits with typed DTOs, documented in JSON Schema

### 2. External contracts
- Define OpenAPI spec for `/api/search`, `/api/embed`, `/api/docs/{doc_id}`.
- Define JSON-RPC schemas for `search`, `doc.meta`, `doc.purge`, `health.check`.
- Publish golden examples for each request/response.

### 3. Internal contracts
- Define Rust traits (ports) for `Chunker`, `Embedder`, `VectorStore`.
- Define DTOs that match JSON/Proto schemas (`Chunk`, `Embedding`, `Point`, etc.).
- Add `schema_version` field for future migrations.

### 4. Artifacts and stubs
- Generate SDKs and client libraries from OpenAPI/JSON-RPC specs.
- Provide mock servers (`embedder-mock`, `qdrant-mock`) for testing.
- Store example payloads in `/contracts/examples/`.

### 5. Contract tests
- Validate all examples against schemas in CI.
- Consumer-driven contracts (CDC): pact-style tests from CLI expectations.
- Golden-file tests ensure byte-for-byte equality with examples.

### 6. Implementation approach
- Implement thin adapters (`EmbedderHttp`, `QdrantClient`, `JsonRpcServer`, `HttpServer`) that satisfy the ports.
- Keep domain logic independent of adapters.
- Expose the same domain `SearchService` via both HTTP and JSON-RPC.

### 7. CI/CD gates
- Lint schemas (spectral for OpenAPI).
- Enforce backward compatibility checks on schemas.
- Require new endpoints to include golden request/response examples.
- Run contract and regression tests on every PR.

### 8. Versioning
- Semantic-version the API (`v1`, `v2`) and payload schema (`schema_version: 1`) separately.
- Migrations follow **add → dual-support → remove**.

### 9. Repository structure
```
daemon/
  contracts/
    openapi/
      search.yaml
      embed.yaml
    jsonrpc/
      search.schema.json
      doc.meta.schema.json
    examples/
      search_request.json
      search_response.json
  crates/
    domain/
    adapters/
      embedder_http/
      vectorstore_qdrant/
      jsonrpc_stdio/
      http_api/
    pipeline/
      indexer/
    mocks/
      embedder_mock/
      vectorstore_mock/
  cli/
    mdx/
  tests/
    contracts_http.rs
    contracts_jsonrpc.rs
    pipeline_golden.rs
```

### 10. Day-1 checklist
1. Write OpenAPI `search.yaml` + JSON-RPC schemas + golden examples.
2. Implement domain ports + DTOs.
3. Implement mock embedder and mock vector store.
4. Expose `/api/search` (HTTP) and `search` (JSON-RPC) via mocks.
5. Add contract tests with golden examples.
6. Hook up real adapters under feature flags.

## Consequences
- **Pros:**
  - Stable APIs early; clients can develop in parallel.
  - Reduced API drift and integration bugs.
  - Mock-first demos and CI enforcement.
- **Cons:**
  - More upfront work to write specs and schemas.
  - Requires discipline in versioning and contract governance.

---
Decision aligns with industry best practices for API-driven systems, especially where multiple clients (CLI, HTTP, JSON-RPC) must remain consistent.
