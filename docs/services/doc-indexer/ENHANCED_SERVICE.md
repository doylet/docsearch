# Doc-Indexer Enhanced Tool Service

## 🎯 **Architecture Overview**

The doc-indexer service has been enhanced to serve as a **pure tool service** that can be called by MCP servers, automation tools, and other services. It's no longer an MCP server itself, but rather a tool that MCP servers can integrate with.

### **Correct Architecture:**
```
MCP Client (Claude, VS Code, etc.)
        ↕ MCP Protocol
MCP Server (separate service)
        ↕ HTTP/JSON-RPC/stdio calls  
doc-indexer (pure tool service)
```

## 🚀 **Transport Support**

### **1. HTTP REST API** (Existing)
- **Endpoints**: `/documents`, `/search`, `/health`, `/info`
- **Usage**: Direct HTTP calls for web integration
- **Backward Compatible**: Existing functionality preserved

### **2. JSON-RPC 2.0 over HTTP** (Enhanced)
- **Endpoint**: `/jsonrpc`
- **Batch Support**: `/jsonrpc/batch`
- **Standard Compliance**: Full JSON-RPC 2.0 specification
- **Methods**: `service.info`, `health.check`, `document.*`

### **3. HTTP Streaming** (New)
- **Server-Sent Events**: Real-time updates and progress
- **Endpoints**:
  - `/stream/index-progress` - Document indexing progress
  - `/stream/search-results` - Streaming search results
  - `/stream/health` - Live health monitoring
- **Use Cases**: Real-time dashboards, progress tracking

### **4. Stdio Transport** (New)
- **JSON-RPC over stdin/stdout**: Process-to-process communication
- **Modes**:
  - `--stdio` - Interactive JSON-RPC mode
  - `--batch` - Batch processing mode
- **Integration**: Perfect for subprocess calls, containers, automation

## 📋 **Available Methods**

### **Service Methods**
- `service.info` - Get service metadata and capabilities
- `health.check` - Health status and diagnostics

### **Document Methods**
- `document.index` - Index a new document
- `document.get` - Retrieve document by ID
- `document.update` - Update existing document
- `document.delete` - Remove document
- `document.search` - Search documents with vector similarity

## 🛠 **Usage Examples**

### **HTTP REST**
```bash
curl http://localhost:8081/info
curl -X POST http://localhost:8081/documents \
  -H "Content-Type: application/json" \
  -d '{"title": "Test", "content": "Document content"}'
```

### **JSON-RPC over HTTP**
```bash
curl http://localhost:8081/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "service.info", "id": 1}'
```

### **HTTP Streaming**
```bash
curl -N http://localhost:8081/stream/health
# Returns Server-Sent Events with real-time health updates
```

### **Stdio Transport**
```bash
# Interactive mode
cargo run -- --stdio
# Then send: {"jsonrpc": "2.0", "method": "service.info", "id": 1}

# Batch mode
echo '{"jsonrpc": "2.0", "method": "service.info", "id": 1}' | cargo run -- --batch
```

### **Container Usage**
```dockerfile
# Run as stdio service in container
docker run doc-indexer --stdio < requests.jsonl > responses.jsonl
```

## 🔧 **CLI Options**

```bash
doc-indexer [OPTIONS]

OPTIONS:
    --config <FILE>         Configuration file path
    --port <PORT>           HTTP server port (default: 8080)
    --log-level <LEVEL>     Log level (default: info)
    --structured-logs       Enable structured logging
    --stdio, -s             Enable stdio JSON-RPC mode
    --batch, -b             Enable batch processing mode
    --stdio-help            Show stdio usage information
    --env-example           Show environment variable examples
    --help                  Show help information
```

## 🏗 **MCP Server Integration**

An MCP server can integrate doc-indexer in multiple ways:

### **HTTP Integration**
```python
# MCP Server calls doc-indexer via HTTP
import requests

def search_documents(query: str):
    response = requests.post("http://doc-indexer:8081/jsonrpc", json={
        "jsonrpc": "2.0",
        "method": "document.search",
        "params": {"query": query},
        "id": 1
    })
    return response.json()["result"]
```

### **Subprocess Integration**
```python
# MCP Server launches doc-indexer as subprocess
import subprocess
import json

def search_documents(query: str):
    process = subprocess.Popen(
        ["doc-indexer", "--stdio"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True
    )
    
    request = {
        "jsonrpc": "2.0",
        "method": "document.search",
        "params": {"query": query},
        "id": 1
    }
    
    stdout, _ = process.communicate(json.dumps(request))
    return json.loads(stdout)["result"]
```

## 📊 **Real-time Capabilities**

### **Streaming Progress**
```javascript
// Connect to streaming endpoint
const eventSource = new EventSource('http://localhost:8081/stream/index-progress');

eventSource.onmessage = function(event) {
    const progress = JSON.parse(event.data);
    console.log(`Indexing progress: ${progress.progress}%`);
};
```

### **Live Health Monitoring**
```bash
# Monitor health in real-time
curl -N http://localhost:8081/stream/health
```

## 🔄 **Deployment Scenarios**

### **1. Standalone HTTP Service**
```bash
# Traditional web service deployment
cargo run -- --port 8081
```

### **2. Container Sidecar**
```yaml
# Kubernetes sidecar pattern
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: mcp-server
    image: my-mcp-server
  - name: doc-indexer
    image: doc-indexer
    args: ["--port", "8081"]
```

### **3. Process Communication**
```bash
# Direct process-to-process communication
my-mcp-server | doc-indexer --stdio | my-response-handler
```

### **4. Batch Processing**
```bash
# Batch file processing
cat requests.jsonl | doc-indexer --batch > responses.jsonl
```

## 🎯 **Next Steps**

This enhanced tool service is now ready for:

1. **MCP Server Integration** - Any MCP server can call doc-indexer via HTTP/stdio
2. **Automation Workflows** - Process-based automation with stdio transport
3. **Real-time Applications** - Streaming endpoints for live updates
4. **Container Orchestration** - Multiple deployment patterns supported
5. **Development Tools** - CLI and batch modes for testing and debugging

The service maintains full backward compatibility while adding powerful new transport options, making it a versatile tool for the MCP ecosystem and beyond!
