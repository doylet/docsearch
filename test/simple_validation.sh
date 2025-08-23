#!/bin/bash
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🚀 Simple Zero-Latency Pipeline Validation"
echo "=========================================="

# Test embedded variant
echo "📦 Testing embedded variant..."
cd services/doc-indexer
cargo build --release --features embedded --no-default-features --target-dir ../../target/test-embedded
EMBEDDED_BINARY="../../target/test-embedded/release/doc-indexer"

if [ -f "$EMBEDDED_BINARY" ]; then
    echo "✅ Embedded binary built successfully"
    
    # Start server
    echo "🚀 Starting embedded server..."
    $EMBEDDED_BINARY --port 18082 > /tmp/embedded_server.log 2>&1 &
    SERVER_PID=$!
    echo "Server PID: $SERVER_PID"
    
    # Wait for startup
    sleep 3
    
    # Test JSON-RPC
    echo "🔍 Testing JSON-RPC endpoints..."
    
    RESPONSE=$(curl -s -X POST "http://localhost:18082/jsonrpc" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"service.info","id":1}')
    
    echo "Service info response: $RESPONSE"
    
    if echo "$RESPONSE" | jq -e '.result.name' > /dev/null 2>&1; then
        echo "✅ JSON-RPC service.info successful"
    else
        echo "❌ JSON-RPC service.info failed"
    fi
    
    # Test health check
    HEALTH_RESPONSE=$(curl -s -X POST "http://localhost:18082/jsonrpc" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"health.check","id":2}')
    
    echo "Health check response: $HEALTH_RESPONSE"
    
    if echo "$HEALTH_RESPONSE" | jq -e '.result.status' > /dev/null 2>&1; then
        echo "✅ JSON-RPC health.check successful"
    else
        echo "❌ JSON-RPC health.check failed"
    fi
    
    # Cleanup
    kill $SERVER_PID
    echo "🧹 Server stopped"
    
else
    echo "❌ Embedded binary not found at $EMBEDDED_BINARY"
    exit 1
fi

echo ""
echo "✅ Simple validation complete!"
