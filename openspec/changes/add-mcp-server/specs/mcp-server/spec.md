## ADDED Requirements

### Requirement: MCP lifecycle over stdio
When started with `--mcp`, doc-indexer SHALL act as an MCP server on stdin/stdout. It SHALL complete the `initialize` handshake, negotiate a supported protocol version, and declare the `tools` capability. It SHALL write nothing to stdout except MCP JSON-RPC messages.

#### Scenario: Client handshake
- **WHEN** an MCP client sends `initialize` with a supported `protocolVersion`, then `notifications/initialized`
- **THEN** the server responds with its `serverInfo`, the negotiated `protocolVersion`, and `capabilities.tools`

#### Scenario: Clean stdout
- **WHEN** doc-indexer runs with `--mcp` at log level `debug`
- **THEN** every line written to stdout parses as a JSON-RPC message
- **AND** log output appears only on stderr

### Requirement: MCP over Streamable HTTP
When `mcp.http_enabled` is true, doc-indexer SHALL serve MCP using the Streamable HTTP transport at `/mcp` on its HTTP server. The endpoint SHALL be disabled by default.

#### Scenario: Disabled by default
- **WHEN** doc-indexer starts with default configuration and a client POSTs to `/mcp`
- **THEN** the response is `404`

#### Scenario: HTTP handshake and tool call
- **WHEN** `mcp.http_enabled` is true and an MCP client connects to `http://127.0.0.1:<port>/mcp`
- **THEN** the client can complete `initialize`, list tools, and call `search_documents`

#### Scenario: Non-loopback warning
- **WHEN** `mcp.http_enabled` is true and the server binds to a non-loopback address
- **THEN** a warning is logged stating that `/mcp` has no authentication

### Requirement: Read tools are listed with generated schemas
The server SHALL list the tools `search_documents`, `list_collections`, `get_document` and `get_health_status`, each with a JSON Schema input definition generated from its Rust input type and a human-readable description.

#### Scenario: Tool listing
- **WHEN** a client calls `tools/list`
- **THEN** the result includes those four tools
- **AND** `search_documents.inputSchema` marks `query` as required

### Requirement: Search tool returns model-ready results
`search_documents` SHALL run a search over the indexed collections and return, in MCP `content`, a text summary of ranked results giving title, source path, score and snippet for each. It SHALL respect `limit` (default 10, maximum 50) and an optional `collection`.

#### Scenario: Successful search
- **WHEN** a client calls `search_documents` with `{ "query": "vector search", "limit": 3 }` against a populated index
- **THEN** the result has `isError` false
- **AND** contains at most 3 ranked results, each with title, source path, score and snippet

#### Scenario: Limit above maximum
- **WHEN** a client calls `search_documents` with `limit` 500
- **THEN** at most 50 results are returned

#### Scenario: Unknown collection
- **WHEN** a client calls `search_documents` with a `collection` that does not exist
- **THEN** the result has `isError` true and a message naming the missing collection

### Requirement: Document and collection tools
`list_collections` SHALL return every collection's name and document count. `get_document` SHALL return a document's text and metadata by ID, truncating content above a configured cap and saying so in the output.

#### Scenario: List collections
- **WHEN** a client calls `list_collections` and two collections exist
- **THEN** the result names both, with their document counts

#### Scenario: Missing document
- **WHEN** a client calls `get_document` with an ID that does not exist
- **THEN** the result has `isError` true and a not-found message

#### Scenario: Large document
- **WHEN** a client calls `get_document` for a document longer than the content cap
- **THEN** the returned text is cut at the cap and ends with a truncation notice

### Requirement: Indexing tool is opt-in and path-restricted
The `index_path` tool SHALL be listed only when `mcp.allow_indexing` is true, and SHALL refuse any path outside the configured browse roots.

#### Scenario: Indexing disabled
- **WHEN** `mcp.allow_indexing` is false and a client calls `tools/list`
- **THEN** `index_path` is not listed

#### Scenario: Path outside roots
- **WHEN** `mcp.allow_indexing` is true and a client calls `index_path` with `/etc`
- **THEN** the result has `isError` true and nothing is indexed

#### Scenario: Path inside roots
- **WHEN** `mcp.allow_indexing` is true and a client calls `index_path` with a directory inside a browse root
- **THEN** the directory is indexed and the result reports indexed and skipped counts
