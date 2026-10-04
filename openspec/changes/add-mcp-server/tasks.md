## 1. Spike and dependencies

- [ ] 1.1 Check the latest `rmcp` release, its feature names (server, stdio, Streamable HTTP) and its axum version requirement; resolve the axum 0.7 compatibility open question
- [ ] 1.2 Add `rmcp` (pinned minor version) and `schemars` to `services/doc-indexer/Cargo.toml`
- [ ] 1.3 Build a minimal `DocSearchMcp` with one `get_health_status` tool over stdio, and confirm the handshake with MCP Inspector

## 2. MCP server module

- [ ] 2.1 Create `src/infrastructure/mcp/{mod.rs,server.rs,tools.rs}`; `DocSearchMcp` holds `AppState`
- [ ] 2.2 Define input types (`SearchArgs`, `GetDocumentArgs`, `IndexPathArgs`) deriving `Deserialize` and `JsonSchema`, with field docs used as descriptions
- [ ] 2.3 Implement `search_documents` over the existing search service: clamp `limit`, cap snippet length, format ranked text plus `structuredContent`
- [ ] 2.4 Implement `list_collections`, `get_document` (with content cap and truncation notice) and `get_health_status`
- [ ] 2.5 Map domain errors (not found, unknown collection, search failure) to `isError: true` tool results
- [ ] 2.6 Implement `index_path`, registered only when `mcp.allow_indexing` is true and enforcing browse roots (reuse the canonicalising check from `finish-pdf-and-browse`)
- [ ] 2.7 Add read-only and destructive tool annotations, if `rmcp` supports them

## 3. Transports and config

- [ ] 3.1 Add an `mcp` config section: `http_enabled` (default false), `allow_indexing` (default false), `max_document_chars`
- [ ] 3.2 Add the `--mcp` CLI flag in `main.rs`; build `AppState` as `--stdio` does and serve over stdio
- [ ] 3.3 In `--mcp` mode, initialise tracing to write to stderr only, and audit for `println!` in startup paths
- [ ] 3.4 Mount the Streamable HTTP service at `/mcp` on the axum router when `http_enabled`; log a warning on non-loopback binds
- [ ] 3.5 Measure cold-start time for `--mcp`; if it's slow, initialise embedding and vector services lazily

## 4. Tests

- [ ] 4.1 Unit tests per tool against an in-memory or test `AppState` (success, `isError` paths, limit clamping, truncation)
- [ ] 4.2 Integration test: spawn `doc-indexer --mcp`, run `initialize` → `tools/list` → `tools/call search_documents`, and assert stdout is JSON-RPC only
- [ ] 4.3 Integration test: start the HTTP server with `http_enabled`, and run the same flow against `/mcp`; assert `404` when disabled
- [ ] 4.4 Test that `index_path` is hidden when disabled and refuses paths outside the roots

## 5. Client validation and docs

- [ ] 5.1 Manual check with Claude Code: `claude mcp add docsearch -- <path>/doc-indexer --mcp`, then run a search
- [ ] 5.2 Manual check with Claude Desktop and Cursor configs
- [ ] 5.3 Write `docs/guides/MCP.md`: setup, config options, client snippets, security notes
- [ ] 5.4 Add a deprecation note for `--stdio` JSON-RPC to `--stdio-help` and `docs/CLI_REFERENCE.md`
- [ ] 5.5 Update the Linear project "MCP Server: Full Protocol Implementation" with status and follow-ups (auth, removing the legacy transport, resources)
