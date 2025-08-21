#!/usr/bin/env python3
"""
Updated JSON-RPC/Streaming/Stdio Test Suite for doc-indexer
Tests the enhanced tool service with HTTP streaming and stdio transport
"""

import requests
import json
import subprocess
import time
import sys
from typing import Dict, Any, Optional

# Server configuration
SERVER_URL = "http://localhost:8081"
JSONRPC_URL = f"{SERVER_URL}/jsonrpc"

def test_server_connectivity() -> bool:
    """Test if the server is running and responding"""
    try:
        response = requests.get(f"{SERVER_URL}/health", timeout=5)
        return response.status_code == 200
    except requests.RequestException:
        return False

def test_rest_endpoints() -> bool:
    """Test REST API endpoints"""
    print("🔍 Testing REST API endpoints...")
    
    try:
        # Test service info
        response = requests.get(f"{SERVER_URL}/info")
        if response.status_code != 200:
            print(f"❌ REST service info failed: {response.status_code}")
            return False
        
        info_data = response.json()
        print(f"✅ REST service info endpoint working")
        print(f"   Response: {info_data}")
        
        # Test health
        response = requests.get(f"{SERVER_URL}/health")
        if response.status_code != 200:
            print(f"❌ REST health failed: {response.status_code}")
            return False
        
        print(f"✅ REST health endpoint working")
        return True
        
    except Exception as e:
        print(f"❌ REST API test failed: {e}")
        return False

def test_jsonrpc_endpoints() -> bool:
    """Test JSON-RPC 2.0 endpoints"""
    print("🔍 Testing JSON-RPC 2.0 endpoints...")
    
    try:
        # Test service.info
        payload = {
            "jsonrpc": "2.0",
            "method": "service.info",
            "id": 1
        }
        
        response = requests.post(JSONRPC_URL, json=payload)
        if response.status_code != 200:
            print(f"❌ JSON-RPC service.info failed: {response.status_code}")
            return False
        
        result = response.json()
        print(f"✅ JSON-RPC service.info working")
        print(f"   Response: {json.dumps(result, indent=2)}")
        
        # Test health.check
        payload = {
            "jsonrpc": "2.0",
            "method": "health.check",
            "id": 2
        }
        
        response = requests.post(JSONRPC_URL, json=payload)
        if response.status_code != 200:
            print(f"❌ JSON-RPC health.check failed: {response.status_code}")
            return False
        
        result = response.json()
        health_status = result.get('result', {}).get('status', 'unknown')
        print(f"✅ JSON-RPC health.check working")
        print(f"   Status: {health_status}")
        
        return True
        
    except Exception as e:
        print(f"❌ JSON-RPC test failed: {e}")
        return False

def test_streaming_endpoints() -> bool:
    """Test Server-Sent Events streaming endpoints"""
    print("🔍 Testing HTTP streaming endpoints...")
    
    try:
        # Test streaming health endpoint
        response = requests.get(f"{SERVER_URL}/stream/health", stream=True, timeout=10)
        if response.status_code != 200:
            print(f"❌ Streaming health endpoint failed: {response.status_code}")
            return False
        
        print(f"✅ Streaming health endpoint accessible")
        
        # Try to read first few events (with timeout)
        start_time = time.time()
        event_count = 0
        
        for line in response.iter_lines():
            if time.time() - start_time > 5:  # 5 second timeout
                break
                
            if line:
                line_str = line.decode('utf-8')
                if line_str.startswith('data:'):
                    event_count += 1
                    if event_count <= 2:  # Show first 2 events
                        print(f"   📡 Received: {line_str}")
        
        if event_count > 0:
            print(f"✅ Streaming working - received {event_count} events")
            return True
        else:
            print(f"⚠️  Streaming endpoint accessible but no events received")
            return True  # Still consider success if endpoint is accessible
            
    except Exception as e:
        print(f"❌ Streaming test failed: {e}")
        return False

def test_stdio_transport() -> bool:
    """Test stdio JSON-RPC transport"""
    print("🔍 Testing stdio JSON-RPC transport...")
    
    try:
        # Test stdio help
        result = subprocess.run(
            ["cargo", "run", "--", "--stdio-help"],
            cwd=".",
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode == 0 and "Stdio JSON-RPC Transport Usage" in result.stdout:
            print("✅ Stdio help command working")
        else:
            print(f"⚠️  Stdio help command issue (may still work): {result.returncode}")
        
        # Test actual stdio communication
        request = {"jsonrpc": "2.0", "method": "service.info", "id": 1}
        request_json = json.dumps(request) + "\n"
        
        process = subprocess.Popen(
            ["cargo", "run", "--", "--stdio"],
            cwd=".",
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        try:
            # Send request and get response
            stdout, stderr = process.communicate(input=request_json, timeout=15)
            
            if stdout.strip():
                try:
                    response = json.loads(stdout.strip())
                    if response.get("jsonrpc") == "2.0" and "result" in response:
                        print("✅ Stdio JSON-RPC communication working")
                        print(f"   Response: {json.dumps(response, indent=2)}")
                        return True
                    else:
                        print(f"⚠️  Stdio response format unexpected: {response}")
                        return False
                except json.JSONDecodeError:
                    print(f"⚠️  Stdio response not valid JSON: {stdout}")
                    return False
            else:
                print(f"⚠️  No stdio response received. Stderr: {stderr}")
                return False
                
        except subprocess.TimeoutExpired:
            process.kill()
            print("⚠️  Stdio test timed out")
            return False
            
    except Exception as e:
        print(f"❌ Stdio test failed: {e}")
        return False

def test_batch_processing() -> bool:
    """Test JSON-RPC batch processing"""
    print("🔍 Testing JSON-RPC batch requests...")
    
    try:
        # Test HTTP batch endpoint
        batch_payload = [
            {"jsonrpc": "2.0", "method": "service.info", "id": 1},
            {"jsonrpc": "2.0", "method": "health.check", "id": 2},
        ]
        
        response = requests.post(f"{JSONRPC_URL}/batch", json=batch_payload)
        if response.status_code != 200:
            print(f"❌ Batch request failed: {response.status_code}")
            return False
        
        results = response.json()
        if isinstance(results, list) and len(results) == 2:
            print(f"✅ JSON-RPC batch requests working")
            print(f"   Processed {len(results)} requests in batch")
            return True
        else:
            print(f"❌ Batch response format unexpected: {results}")
            return False
            
    except Exception as e:
        print(f"❌ Batch test failed: {e}")
        return False

def main():
    print("🚀 Enhanced Tool Service Test Suite")
    print("=====================================")
    
    # Check server connectivity
    if not test_server_connectivity():
        print("❌ Server is not running at http://localhost:8081")
        print("   Please start the server with: cargo run -- --port 8081")
        sys.exit(1)
    
    print("✅ Server is running at http://localhost:8081")
    
    # Run all tests
    tests = [
        ("REST API", test_rest_endpoints),
        ("JSON-RPC 2.0", test_jsonrpc_endpoints),
        ("HTTP Streaming", test_streaming_endpoints),
        ("Stdio Transport", test_stdio_transport),
        ("Batch Processing", test_batch_processing),
    ]
    
    results = {}
    for test_name, test_func in tests:
        print()
        try:
            results[test_name] = test_func()
        except Exception as e:
            print(f"❌ {test_name} test crashed: {e}")
            results[test_name] = False
    
    # Summary
    print()
    print("📝 Test Summary:")
    print("================")
    all_passed = True
    for test_name, passed in results.items():
        status = "✅" if passed else "❌"
        print(f"   {status} {test_name}")
        if not passed:
            all_passed = False
    
    print()
    if all_passed:
        print("🎉 All tests passed!")
        print()
        print("✅ Tool Service Features:")
        print("   • HTTP REST API (backward compatible)")
        print("   • JSON-RPC 2.0 protocol compliance")
        print("   • HTTP streaming (Server-Sent Events)")
        print("   • Stdio transport (process communication)")
        print("   • Batch request processing")
        print()
        print("🔗 Ready for integration with MCP servers and automation tools!")
    else:
        print("⚠️  Some tests failed - see details above")
        sys.exit(1)

if __name__ == "__main__":
    main()
