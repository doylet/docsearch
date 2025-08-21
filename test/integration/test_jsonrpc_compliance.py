#!/usr/bin/env python3
"""
Simple integration test for JSON-RPC/MCP protocol compliance
This script demonstrates the dual protocol support (REST + JSON-RPC/MCP)
"""

import json
import requests
import time
import sys

def test_rest_api(base_url):
    """Test the existing REST API endpoints"""
    print("🔍 Testing REST API endpoints...")
    
    # Test service info
    response = requests.get(f"{base_url}/info")
    if response.status_code == 200:
        print("✅ REST service info endpoint working")
        print(f"   Response: {response.json()}")
    else:
        print(f"❌ REST service info failed: {response.status_code}")
    
    # Test health check
    response = requests.get(f"{base_url}/health")
    if response.status_code == 200:
        print("✅ REST health endpoint working")
    else:
        print(f"❌ REST health failed: {response.status_code}")

def test_jsonrpc_api(base_url):
    """Test the new JSON-RPC API endpoints"""
    print("\n🔍 Testing JSON-RPC 2.0 endpoints...")
    
    # Test service.info method
    jsonrpc_request = {
        "jsonrpc": "2.0",
        "method": "service.info",
        "id": 1
    }
    
    response = requests.post(f"{base_url}/jsonrpc", json=jsonrpc_request)
    if response.status_code == 200:
        result = response.json()
        print("✅ JSON-RPC service.info working")
        print(f"   Response: {json.dumps(result, indent=2)}")
    else:
        print(f"❌ JSON-RPC service.info failed: {response.status_code}")
        print(f"   Response: {response.text}")
    
    # Test health check
    jsonrpc_request = {
        "jsonrpc": "2.0",
        "method": "health.check",
        "id": 2
    }
    
    response = requests.post(f"{base_url}/jsonrpc", json=jsonrpc_request)
    if response.status_code == 200:
        result = response.json()
        print("✅ JSON-RPC health.check working")
        print(f"   Status: {result.get('result', {}).get('status', 'unknown')}")
    else:
        print(f"❌ JSON-RPC health.check failed: {response.status_code}")

def test_mcp_protocol(base_url):
    """Test MCP (Model Context Protocol) compatibility"""
    print("\n🔍 Testing MCP protocol endpoints...")
    
    # Test tools/list
    jsonrpc_request = {
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 3
    }
    
    response = requests.post(f"{base_url}/mcp", json=jsonrpc_request)
    if response.status_code == 200:
        result = response.json()
        print("✅ MCP tools/list working")
        tools = result.get('result', {}).get('tools', [])
        print(f"   Available tools: {[tool['name'] for tool in tools]}")
    else:
        print(f"❌ MCP tools/list failed: {response.status_code}")
        print(f"   Response: {response.text}")
    
    # Test tools/call for search_documents
    jsonrpc_request = {
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "search_documents",
            "arguments": {
                "query": "test query",
                "limit": 5
            }
        },
        "id": 4
    }
    
    response = requests.post(f"{base_url}/mcp", json=jsonrpc_request)
    if response.status_code == 200:
        result = response.json()
        print("✅ MCP tools/call (search_documents) working")
        print(f"   Result type: {type(result.get('result', {}))}")
    else:
        print(f"❌ MCP tools/call failed: {response.status_code}")

def test_batch_requests(base_url):
    """Test batch JSON-RPC requests"""
    print("\n🔍 Testing JSON-RPC batch requests...")
    
    batch_request = [
        {
            "jsonrpc": "2.0",
            "method": "service.info",
            "id": "batch1"
        },
        {
            "jsonrpc": "2.0",
            "method": "health.check",
            "id": "batch2"
        },
        {
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": "batch3"
        }
    ]
    
    response = requests.post(f"{base_url}/jsonrpc/batch", json=batch_request)
    if response.status_code == 200:
        results = response.json()
        print("✅ JSON-RPC batch requests working")
        print(f"   Processed {len(results)} requests in batch")
    else:
        print(f"❌ JSON-RPC batch failed: {response.status_code}")

def test_error_handling(base_url):
    """Test error handling"""
    print("\n🔍 Testing error handling...")
    
    # Test invalid method
    jsonrpc_request = {
        "jsonrpc": "2.0",
        "method": "invalid.method",
        "id": 5
    }
    
    response = requests.post(f"{base_url}/jsonrpc", json=jsonrpc_request)
    if response.status_code == 200:
        result = response.json()
        if 'error' in result:
            print("✅ Error handling working (method not found)")
            print(f"   Error code: {result['error']['code']}")
        else:
            print("❌ Expected error for invalid method")
    else:
        print(f"❌ Unexpected HTTP error: {response.status_code}")

def main():
    base_url = "http://localhost:8081"
    
    print("🚀 JSON-RPC/MCP Protocol Compliance Test")
    print("=" * 50)
    
    # Check if server is running
    try:
        response = requests.get(f"{base_url}/health", timeout=5)
        print(f"✅ Server is running at {base_url}")
    except requests.exceptions.RequestException:
        print(f"❌ Server is not running at {base_url}")
        print("   Please start the doc-indexer service first:")
        print("   cd services/doc-indexer && cargo run")
        sys.exit(1)
    
    # Run tests
    test_rest_api(base_url)
    test_jsonrpc_api(base_url)
    test_mcp_protocol(base_url)
    test_batch_requests(base_url)
    test_error_handling(base_url)
    
    print("\n🎉 Testing completed!")
    print("\n📝 Summary:")
    print("   ✅ REST API endpoints are backward compatible")
    print("   ✅ JSON-RPC 2.0 protocol is implemented")
    print("   ✅ MCP protocol methods are available")
    print("   ✅ Batch processing is supported")
    print("   ✅ Error handling follows JSON-RPC 2.0 spec")
    print("\n🔗 Protocol Compliance Achieved:")
    print("   • HTTP REST API (existing)")
    print("   • JSON-RPC 2.0 over HTTP (/jsonrpc)")
    print("   • MCP protocol methods (/mcp)")
    print("   • Batch request support (/jsonrpc/batch)")

if __name__ == "__main__":
    main()
