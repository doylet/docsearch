# Integration Tests

This directory contains integration tests for the Zero-Latency project services.

## Test Files

- **`test_jsonrpc_compliance.py`** - Tests JSON-RPC 2.0 compliance for doc-indexer service
- **`test_mock_api.py`** - Mock API testing utilities  
- **`test_enhanced_service.py`** - Comprehensive enhanced service testing (if present)

## Running Tests

### Prerequisites
- Python 3.7+
- `requests` library: `pip install requests`
- Running doc-indexer service (for integration tests)

### Running Individual Tests

```bash
# Start the doc-indexer service first
cd services/doc-indexer
cargo run -- --port 8081

# In another terminal, run tests
cd test/integration
python test_jsonrpc_compliance.py
python test_enhanced_service.py
```

### Test Coverage

The integration tests cover:
- HTTP REST API endpoints
- JSON-RPC 2.0 protocol compliance  
- HTTP streaming (Server-Sent Events)
- Stdio transport functionality
- Batch processing
- Error handling

## Test Environment

Tests assume the service is running on `localhost:8081` by default. This can be configured by modifying the `SERVER_URL` variable in the test files.
