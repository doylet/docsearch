# Phase 2 Plan: CLI Interface and Complete API Contract

**Date:** August 20, 2025  
**Status:** 🚀 STARTING  
**Branch:** `feature/phase-2-cli-interface`  
**Previous Phase:** ✅ Phase 1 Complete (Working search functionality)

## 🎯 Mission: User-Friendly CLI Interface

**Phase 2** focuses on creating an intuitive CLI interface (`mdx`) that makes the search functionality easily accessible to users, plus expanding the API contract for full document management.

## 📋 Phase 2 Deliverables

### **1. CLI Interface (`mdx` binary)**

- ✅ **Foundation**: Use existing HTTP API server as backend
- 🔄 **Commands to implement:**
  - `mdx search "query text"` - Search documents with semantic similarity
  - `mdx index /path/to/docs` - Index documents from a directory
  - `mdx status` - Show collection statistics and health
  - `mdx reindex` - Rebuild the entire index
  - `mdx server` - Start the HTTP API server in background
  - `mdx help` - Show usage information

### **2. Extended API Contract**

- ✅ **Existing**: `POST /api/search` (working in Phase 1)
- 🔄 **New endpoints:**
  - `GET /api/docs` - List all indexed documents
  - `GET /api/docs/{id}` - Get specific document details
  - `DELETE /api/docs/{id}` - Remove document from index
  - `POST /api/reindex` - Rebuild index from source documents
  - `GET /api/status` - Collection statistics and health
  - `GET /api/health` - Simple health check

### **3. JSON-RPC Interface Layer**

- 🔄 **Structured communication** between CLI and API server
- 🔄 **Error handling** with proper status codes and messages
- 🔄 **Request/response validation** with proper schemas

## 🏗 Technical Architecture

### **CLI Design Pattern**

```text
mdx (CLI binary)
    ↓ HTTP requests
API Server (from Phase 1)
    ↓ vector operations  
Qdrant Database
```

### **Project Structure**

```text
crates/
  cli/               # New CLI application
    src/
      main.rs        # CLI entry point and command parsing
      commands/      # Individual command implementations
        search.rs    # Search command logic
        index.rs     # Indexing command logic
        status.rs    # Status command logic
        server.rs    # Server command logic
      client.rs      # HTTP client for API communication
      config.rs      # CLI configuration management
      output.rs      # Result formatting and display
  
services/doc-indexer/ # Existing API server (Phase 1)
    # No major changes - extend API endpoints
```

## 🎯 User Experience Goals

### **Simple Commands**

```bash
# Quick search
mdx search "embedding model"

# Index documents  
mdx index ~/docs

# Check system status
mdx status

# Start server in background
mdx server --daemon
```

### **Rich Output**

```bash
$ mdx search "vector database"
🔍 Searching for: "vector database"

📄 001_blueprint_rust-model-host.md (score: 0.89)
   "The vector database stores embeddings for semantic search..."
   📁 model-host/artefacts • 🏷️ blueprint

📄 015_qdrant_integration.md (score: 0.82)  
   "Qdrant provides high-performance vector similarity search..."
   📁 misc/artefacts • 🏷️ technical

✅ Found 2 results in 15ms
```

### **Error Handling**

```bash
$ mdx search "test"
❌ Error: API server not running
💡 Try: mdx server --start

$ mdx index /invalid/path  
❌ Error: Directory not found: /invalid/path
💡 Check the path and try again
```

## 🛠 Implementation Strategy

### **Step 1: CLI Foundation** (Day 1)

- Create `crates/cli` with basic structure
- Implement command parsing with `clap`
- Add HTTP client for API communication
- Implement `mdx search` command

### **Step 2: Core Commands** (Day 1-2)

- Implement `mdx index` command
- Implement `mdx status` command  
- Add configuration management
- Rich output formatting

### **Step 3: Extended API** (Day 2)

- Add missing API endpoints to doc-indexer
- Implement document management endpoints
- Add proper error responses
- Update API documentation

### **Step 4: Polish & Testing** (Day 2)

- Add `mdx server` command with daemon mode
- Comprehensive error handling
- Integration testing
- Documentation and examples

## 📊 Success Criteria

### **CLI Usability**

- ✅ **Users can search with a single command**: `mdx search "query"`
- ✅ **Index management is simple**: `mdx index /path`
- ✅ **Status is transparent**: `mdx status` shows collection health
- ✅ **Self-contained**: No complex setup required

### **API Completeness**

- ✅ **Full CRUD operations** for document management
- ✅ **Rich metadata** in responses (timing, stats, errors)
- ✅ **Proper HTTP status codes** and error messages
- ✅ **OpenAPI documentation** (bonus)

### **Developer Experience**

- ✅ **Clear command structure** following Unix conventions
- ✅ **Helpful error messages** with actionable suggestions
- ✅ **Fast response times** (CLI overhead < 50ms)
- ✅ **Consistent output formatting** across commands

## 🔧 Technical Dependencies

### **CLI Dependencies**

- `clap` - Command line argument parsing
- `tokio` - Async runtime for HTTP client
- `reqwest` - HTTP client for API communication
- `serde_json` - JSON serialization/deserialization
- `anyhow` - Error handling
- `colored` - Terminal color output
- `indicatif` - Progress bars and spinners

### **API Extensions**

- Reuse existing Axum/Tower stack from Phase 1
- Add new route handlers
- Extend existing data models
- No major architectural changes

## 📈 Phase 2 Timeline

### **Day 1: CLI Foundation**

- ✅ Morning: Create CLI crate and basic structure
- ✅ Afternoon: Implement `mdx search` command
- ✅ Evening: Add HTTP client and configuration

### **Day 2: API Extension & Polish**

- ✅ Morning: Extend API with new endpoints
- ✅ Afternoon: Implement remaining CLI commands
- ✅ Evening: Testing, error handling, documentation

## 🎉 Success Metrics

### **Phase 2 Complete When:**

1. **✅ CLI commands work end-to-end** - Users can search, index, and check status
2. **✅ API contract is complete** - All documented endpoints implemented
3. **✅ Error handling is robust** - Graceful failures with helpful messages
4. **✅ Documentation is comprehensive** - README with examples and troubleshooting

### **User Value Delivered:**

- **Simplified workflow** - No need to remember HTTP endpoints
- **Better discoverability** - `mdx help` shows all options
- **Richer feedback** - Progress indicators and formatted output
- **Production readiness** - Complete document lifecycle management

## 🚀 Getting Started

The foundation from Phase 1 is solid:

- ✅ Search API working perfectly
- ✅ Document indexing pipeline stable  
- ✅ Vector database operations validated
- ✅ Local embeddings integration complete

**Next step:** Create the CLI crate and implement the first command (`mdx search`)

---

**Phase 2 Focus:** Transform working functionality into delightful user experience 🎯
