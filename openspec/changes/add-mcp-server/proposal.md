## Why

The project's goal is to become a search and retrieval tool that AI assistants (Claude, Cursor and others) can call directly. The standard way to do that is the Model Context Protocol (MCP). doc-indexer already exposes `tools/list` and `tools/call` over JSON-RPC (via `--stdio` and HTTP), but it isn't MCP-compliant:

- There's no `initialize` handshake or protocol-version negotiation.
- There's no capability declaration.
- Tool results aren't in MCP's `content` format.
- There's no MCP-standard HTTP transport.

As a result, real MCP clients can't connect. This is the critical-path item in the Linear initiative "DocSearch: Cloud MCP RAG Platform".

## What Changes

- Add an MCP server to doc-indexer, built on the official Rust MCP SDK (`rmcp`). It supports:
  - the stdio transport, for local clients such as Claude Desktop and Claude Code
  - the Streamable HTTP transport at `/mcp`, for remote and cloud use
- Expose read-oriented tools with typed input schemas and MCP-format results: `search_documents`, `list_collections`, `get_document`, `get_health_status`.
- Expose `index_path` as a write tool. It's disabled unless explicitly enabled, and is subject to the same allowed-roots restriction as `/api/browse`.
- Add the CLI flag `--mcp` (stdio) and the config option `mcp.http_enabled` (HTTP route on the existing axum server).
- Leave the legacy JSON-RPC transport (`--stdio`, `jsonrpc` handlers) unchanged in this change, but mark it deprecated in docs. Removing it is a follow-up.

## Capabilities

### New Capabilities
- `mcp-server`: A standards-compliant MCP server exposing doc-indexer's search and collection operations as MCP tools over stdio and Streamable HTTP.

### Modified Capabilities
<!-- None: openspec/specs/ has no existing capability specs. -->

## Impact

- **Code**: new module `services/doc-indexer/src/infrastructure/mcp/`; `main.rs` (CLI flag); the HTTP router (mount `/mcp`); config.
- **Dependencies**: adds `rmcp` (with server, stdio and Streamable HTTP features) and `schemars` for tool input schemas.
- **APIs**: a new MCP endpoint `/mcp`. The existing REST and JSON-RPC endpoints don't change.
- **Docs**: new `docs/guides/MCP.md`, with client configuration snippets for Claude Desktop, Claude Code and Cursor.
- **Depends on**: `restore-project-health`. Shares the browse-roots setting from `finish-pdf-and-browse` (for `index_path`).
- **Out of scope**: authentication and multi-tenancy (Linear project "Cloud Infrastructure"); RAG answer generation (Linear project "RAG Pipeline").
