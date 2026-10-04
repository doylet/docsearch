## Context

doc-indexer already has three ways in:

1. **REST** (axum 0.7), under `infrastructure/api/http/`.
2. **JSON-RPC 2.0 over HTTP**, under `infrastructure/api/jsonrpc/` (about 1,000 lines). Its method names are MCP-like (`tools/list`, `tools/call`, `collection.list`, `document.index`...), with hand-written JSON input schemas.
3. **JSON-RPC over stdio** (`--stdio`, `infrastructure/stdio.rs`), plus a batch mode.

All of them call the same application services through `AppState` / `ServiceContainer`. So the work is mostly protocol, not new business logic.

What's missing for MCP compliance: the `initialize` / `notifications/initialized` lifecycle, protocol-version negotiation, the `capabilities` declaration, tool results as `{ content: [...], isError }`, error semantics (tool failures go in the result, not as JSON-RPC errors), and the Streamable HTTP transport (session headers, SSE for streamed responses).

## Goals / Non-Goals

**Goals:**
- Claude Desktop, Claude Code and Cursor can connect over stdio and run searches with no adapter.
- A remote client can connect over Streamable HTTP to a running doc-indexer.
- Tool schemas are generated from Rust types, so they can't drift from the code.
- Tool results are compact and useful to a language model: ranked snippets with source, score and collection.

**Non-Goals:**
- Authentication, API keys or tenancy on `/mcp`. Until those exist, HTTP MCP binds to localhost by default.
- MCP resources, prompts or sampling. Tools only in this change.
- Removing or rewriting the legacy JSON-RPC transport.
- RAG answer generation, reranking or context-window packing.

## Decisions

**Use the official `rmcp` SDK rather than extending the hand-rolled JSON-RPC code.** `rmcp` implements the lifecycle, version negotiation, and both stdio and Streamable HTTP transports, which we'd otherwise have to build and then keep in step with spec revisions. It integrates with tokio and axum (the Streamable HTTP service mounts as an axum route). *Alternative:* add `initialize` and the content format to the existing `jsonrpc` module. Rejected: Streamable HTTP session handling and future spec changes would be ours to maintain forever.

**Use a thin adapter over application services.** Create `infrastructure/mcp/server.rs` with a `DocSearchMcp { state: AppState }` struct. Define tools with `rmcp`'s `#[tool]` macros. Input structs derive `serde::Deserialize` and `schemars::JsonSchema`. Each tool calls the same services the REST handlers use; MCP-specific code does no business logic.

**Choose a tool set for model use, not a mirror of the REST API:**

| Tool | Input | Output |
|---|---|---|
| `search_documents` | `query`, `limit` (default 10, max 50), optional `collection`, optional path `filters` | Text listing ranked results (title, path, score, snippet), plus `structuredContent` with the full results |
| `list_collections` | none | Collection names with document counts |
| `get_document` | `document_id` | Full document text (capped, with truncation noted) and metadata |
| `get_health_status` | none | Service status summary |
| `index_path` | `path`, optional `collection` | Indexed/skipped counts. **Only registered when `mcp.allow_indexing = true`**, and the path must be inside the browse roots |

Mark tools read-only or destructive with MCP tool annotations where `rmcp` supports them.

**Report tool failures as tool results.** Search failures, unknown collections and missing documents return `isError: true` with a readable message, so the model can recover. Protocol-level errors (unknown tool, schema validation failure) remain JSON-RPC errors, handled by `rmcp`.

**Configure transports separately:**
- **stdio**: `doc-indexer --mcp` builds `AppState` exactly as `--stdio` does today, then serves `DocSearchMcp` over stdin/stdout. Logging must go to **stderr only**, because any stdout output corrupts the protocol stream. Add a startup assertion or test for this.
- **Streamable HTTP**: mounted at `/mcp` on the existing axum router when `mcp.http_enabled = true` (default `false`). It inherits the server's bind address. The default bind stays `127.0.0.1`, and a warning is logged if `/mcp` is enabled on a non-loopback address without auth.

**Keep the legacy JSON-RPC transport and mark it deprecated.** It costs nothing to leave, and something may depend on it. Add a deprecation note to `--stdio-help` and the docs pointing at `--mcp`. Remove it in a later change once MCP is confirmed working with real clients.

## Risks / Trade-offs

- [`rmcp` API churn between versions] → Pin a minor version and wrap all `rmcp` usage in `infrastructure/mcp/`, so upgrades touch one module.
- [stdout pollution breaks stdio clients, for example a stray `println!` or ONNX Runtime logs] → Route the tracing subscriber to stderr in `--mcp` mode. Add an integration test that spawns `doc-indexer --mcp`, performs `initialize` → `tools/list`, and checks stdout is valid JSON-RPC only.
- [Cold start in stdio mode loads embedding models before the client's `initialize` times out] → Measure startup. If it's over a few seconds, initialise heavy services lazily on first tool call.
- [Unauthenticated HTTP endpoint] → Off by default, localhost by default, warning on non-loopback addresses. Real auth is tracked in the Linear "Cloud Infrastructure" project.
- [Large results overflow the model's context] → Cap `limit` at 50, cap snippet length, and truncate `get_document` with a marker.

## Migration Plan

1. Ship `--mcp` (stdio) first, and validate it with Claude Code (`claude mcp add docsearch -- doc-indexer --mcp`) and with MCP Inspector.
2. Enable `/mcp` behind config and validate it with MCP Inspector over HTTP.
3. Document both. Mark `--stdio` deprecated.
4. Rollback: the feature is additive. Leave out `--mcp` or set `mcp.http_enabled = false`.

## Open Questions

- Which `rmcp` version to pin? Check the latest release and its axum compatibility. The project uses axum 0.7, and `rmcp`'s Streamable HTTP server may need axum 0.8, which would force an axum upgrade or a separate listener.
- Should `search_documents` default to searching all collections, or only the default `zero_latency_docs`?
- Should the embedded (ONNX) or cloud embedding backend be the default for `--mcp`, given startup time?
