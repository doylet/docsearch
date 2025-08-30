#!/usr/bin/env python3
"""
Sprint 005: Comprehensive Search Filtering Tests
Integration tests for collection filtering across all interfaces (CLI, REST, JSON-RPC)
"""

import json
import requests
import subprocess
import time
import sys
import os
from typing import Dict, List, Any, Optional

# Test configuration
BASE_URL = "http://localhost:8081"
CLI_BINARY = "./target/debug/mdx"

class SearchFilteringTests:
    def __init__(self, base_url: str = BASE_URL):
        self.base_url = base_url
        self.test_results = []
        
    def log_test(self, test_name: str, passed: bool, details: str = ""):
        """Log test result"""
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"{status}: {test_name}")
        if details:
            print(f"   {details}")
        self.test_results.append({
            "test": test_name,
            "passed": passed,
            "details": details
        })
    
    def test_cli_collection_filtering(self):
        """Test CLI collection filtering functionality"""
        print("\n🔍 Testing CLI Collection Filtering...")
        
        # Test 1: CLI with collection filtering
        try:
            result = subprocess.run([
                CLI_BINARY, "search", "test", "--collection", "zero_latency_docs"
            ], capture_output=True, text=True, timeout=30)
            
            if result.returncode == 0:
                output = result.stdout
                # Check for expected success indicators
                if "Search completed successfully!" in output:
                    self.log_test("CLI collection filtering execution", True, "Command executed successfully")
                else:
                    self.log_test("CLI collection filtering execution", False, f"Unexpected output: {output[:100]}")
            else:
                self.log_test("CLI collection filtering execution", False, f"Exit code: {result.returncode}, Error: {result.stderr}")
                
        except subprocess.TimeoutExpired:
            self.log_test("CLI collection filtering execution", False, "Command timed out")
        except Exception as e:
            self.log_test("CLI collection filtering execution", False, f"Exception: {e}")
        
        # Test 2: CLI without collection filtering (default behavior)
        try:
            result = subprocess.run([
                CLI_BINARY, "search", "test"
            ], capture_output=True, text=True, timeout=30)
            
            if result.returncode == 0 and "Search completed successfully!" in result.stdout:
                self.log_test("CLI default search", True, "Default search working")
            else:
                self.log_test("CLI default search", False, f"Exit code: {result.returncode}")
                
        except Exception as e:
            self.log_test("CLI default search", False, f"Exception: {e}")
    
    def test_rest_api_collection_filtering(self):
        """Test REST API collection filtering functionality"""
        print("\n🔍 Testing REST API Collection Filtering...")
        
        # Test 1: REST API with collection filtering
        try:
            payload = {
                "query": "test",
                "limit": 10,
                "filters": {"collection_name": "copilot-chat-dist"}
            }
            response = requests.post(f"{self.base_url}/api/search", json=payload, timeout=30)
            
            if response.status_code == 200:
                data = response.json()
                results = data.get("results", [])
                
                # Verify collection filtering worked
                if results:
                    collection_check = all(
                        result.get("collection") == "copilot-chat-dist" 
                        for result in results
                    )
                    if collection_check:
                        self.log_test("REST API collection filtering", True, f"Found {len(results)} results from correct collection")
                    else:
                        self.log_test("REST API collection filtering", False, "Results from wrong collection")
                else:
                    self.log_test("REST API collection filtering", False, "No results returned")
            else:
                self.log_test("REST API collection filtering", False, f"HTTP {response.status_code}: {response.text[:100]}")
                
        except Exception as e:
            self.log_test("REST API collection filtering", False, f"Exception: {e}")
        
        # Test 2: REST API without collection filtering
        try:
            payload = {"query": "test", "limit": 10}
            response = requests.post(f"{self.base_url}/api/search", json=payload, timeout=30)
            
            if response.status_code == 200:
                data = response.json()
                results = data.get("results", [])
                self.log_test("REST API default search", True, f"Default search returned {len(results)} results")
            else:
                self.log_test("REST API default search", False, f"HTTP {response.status_code}")
                
        except Exception as e:
            self.log_test("REST API default search", False, f"Exception: {e}")
    
    def test_jsonrpc_collection_filtering(self):
        """Test JSON-RPC collection filtering functionality"""
        print("\n🔍 Testing JSON-RPC Collection Filtering...")
        
        # Test 1: JSON-RPC with collection filtering
        try:
            payload = {
                "jsonrpc": "2.0",
                "method": "document.search",
                "params": {
                    "query": "test",
                    "filters": {"collection": "copilot-chat-dist"}
                },
                "id": 1
            }
            response = requests.post(f"{self.base_url}/jsonrpc", json=payload, timeout=30)
            
            if response.status_code == 200:
                data = response.json()
                if "result" in data and not data.get("error"):
                    results = data["result"].get("results", [])
                    
                    # Verify collection filtering
                    if results:
                        collection_check = all(
                            result.get("metadata", {}).get("collection") == "copilot-chat-dist"
                            for result in results
                        )
                        if collection_check:
                            self.log_test("JSON-RPC collection filtering", True, f"Found {len(results)} results from correct collection")
                        else:
                            self.log_test("JSON-RPC collection filtering", False, "Results from wrong collection")
                    else:
                        self.log_test("JSON-RPC collection filtering", False, "No results returned")
                else:
                    error = data.get("error", "Unknown error")
                    self.log_test("JSON-RPC collection filtering", False, f"JSON-RPC error: {error}")
            else:
                self.log_test("JSON-RPC collection filtering", False, f"HTTP {response.status_code}")
                
        except Exception as e:
            self.log_test("JSON-RPC collection filtering", False, f"Exception: {e}")
        
        # Test 2: JSON-RPC without collection filtering
        try:
            payload = {
                "jsonrpc": "2.0",
                "method": "document.search",
                "params": {"query": "test"},
                "id": 2
            }
            response = requests.post(f"{self.base_url}/jsonrpc", json=payload, timeout=30)
            
            if response.status_code == 200:
                data = response.json()
                if "result" in data:
                    results = data["result"].get("results", [])
                    self.log_test("JSON-RPC default search", True, f"Default search returned {len(results)} results")
                else:
                    self.log_test("JSON-RPC default search", False, f"Error: {data.get('error')}")
            else:
                self.log_test("JSON-RPC default search", False, f"HTTP {response.status_code}")
                
        except Exception as e:
            self.log_test("JSON-RPC default search", False, f"Exception: {e}")
    
    def test_edge_cases(self):
        """Test edge cases and error conditions"""
        print("\n🔍 Testing Edge Cases...")
        
        # Test 1: Invalid collection name (CLI)
        try:
            result = subprocess.run([
                CLI_BINARY, "search", "test", "--collection", "invalid_collection_name"
            ], capture_output=True, text=True, timeout=30)
            
            # Should succeed but return no results
            if result.returncode == 0:
                self.log_test("CLI invalid collection handling", True, "Gracefully handled invalid collection")
            else:
                self.log_test("CLI invalid collection handling", False, "Failed to handle invalid collection")
                
        except Exception as e:
            self.log_test("CLI invalid collection handling", False, f"Exception: {e}")
        
        # Test 2: Invalid collection name (REST API)
        try:
            payload = {
                "query": "test",
                "filters": {"collection_name": "invalid_collection_name"}
            }
            response = requests.post(f"{self.base_url}/api/search", json=payload, timeout=30)
            
            if response.status_code == 200:
                data = response.json()
                results = data.get("results", [])
                if len(results) == 0:
                    self.log_test("REST API invalid collection handling", True, "Returned empty results for invalid collection")
                else:
                    self.log_test("REST API invalid collection handling", False, f"Unexpected results: {len(results)}")
            else:
                self.log_test("REST API invalid collection handling", False, f"HTTP error: {response.status_code}")
                
        except Exception as e:
            self.log_test("REST API invalid collection handling", False, f"Exception: {e}")
        
        # Test 3: Invalid collection name (JSON-RPC)
        try:
            payload = {
                "jsonrpc": "2.0",
                "method": "document.search",
                "params": {
                    "query": "test",
                    "filters": {"collection": "invalid_collection_name"}
                },
                "id": 3
            }
            response = requests.post(f"{self.base_url}/jsonrpc", json=payload, timeout=30)
            
            if response.status_code == 200:
                data = response.json()
                if "result" in data:
                    results = data["result"].get("results", [])
                    if len(results) == 0:
                        self.log_test("JSON-RPC invalid collection handling", True, "Returned empty results for invalid collection")
                    else:
                        self.log_test("JSON-RPC invalid collection handling", False, f"Unexpected results: {len(results)}")
                else:
                    self.log_test("JSON-RPC invalid collection handling", False, f"Error response: {data.get('error')}")
            else:
                self.log_test("JSON-RPC invalid collection handling", False, f"HTTP error: {response.status_code}")
                
        except Exception as e:
            self.log_test("JSON-RPC invalid collection handling", False, f"Exception: {e}")
        
        # Test 4: Empty query
        try:
            payload = {
                "query": "",
                "filters": {"collection_name": "zero_latency_docs"}
            }
            response = requests.post(f"{self.base_url}/api/search", json=payload, timeout=30)
            
            # Should handle gracefully (either error or empty results)
            if response.status_code in [200, 400]:
                self.log_test("Empty query handling", True, "Gracefully handled empty query")
            else:
                self.log_test("Empty query handling", False, f"Unexpected status: {response.status_code}")
                
        except Exception as e:
            self.log_test("Empty query handling", False, f"Exception: {e}")
    
    def test_cross_interface_consistency(self):
        """Test consistency across interfaces"""
        print("\n🔍 Testing Cross-Interface Consistency...")
        
        query = "test"
        collection = "zero_latency_docs"
        
        # Get results from different interfaces
        rest_results = self._get_rest_results(query, collection)
        jsonrpc_results = self._get_jsonrpc_results(query, collection)
        
        if rest_results is not None and jsonrpc_results is not None:
            # Compare result counts (should be similar)
            rest_count = len(rest_results)
            jsonrpc_count = len(jsonrpc_results)
            
            # Allow for small differences due to timing/indexing
            if abs(rest_count - jsonrpc_count) <= 2:
                self.log_test("Cross-interface result consistency", True, f"REST: {rest_count}, JSON-RPC: {jsonrpc_count}")
            else:
                self.log_test("Cross-interface result consistency", False, f"Large difference: REST: {rest_count}, JSON-RPC: {jsonrpc_count}")
        else:
            self.log_test("Cross-interface result consistency", False, "Could not retrieve results from both interfaces")
    
    def _get_rest_results(self, query: str, collection: str) -> Optional[List[Dict]]:
        """Helper to get REST API results"""
        try:
            payload = {
                "query": query,
                "filters": {"collection_name": collection}
            }
            response = requests.post(f"{self.base_url}/api/search", json=payload, timeout=30)
            if response.status_code == 200:
                return response.json().get("results", [])
        except:
            pass
        return None
    
    def _get_jsonrpc_results(self, query: str, collection: str) -> Optional[List[Dict]]:
        """Helper to get JSON-RPC results"""
        try:
            payload = {
                "jsonrpc": "2.0",
                "method": "document.search",
                "params": {
                    "query": query,
                    "filters": {"collection": collection}
                },
                "id": 1
            }
            response = requests.post(f"{self.base_url}/jsonrpc", json=payload, timeout=30)
            if response.status_code == 200:
                data = response.json()
                if "result" in data:
                    return data["result"].get("results", [])
        except:
            pass
        return None
    
    def run_all_tests(self):
        """Run all search filtering tests"""
        print("🚀 Starting Sprint 005: Comprehensive Search Filtering Tests")
        print(f"Testing against: {self.base_url}")
        print(f"CLI binary: {CLI_BINARY}")
        
        # Check if CLI binary exists
        if not os.path.exists(CLI_BINARY):
            print(f"❌ CLI binary not found at {CLI_BINARY}")
            print("   Please build the project first: cargo build")
            return False
        
        # Run all test suites
        self.test_cli_collection_filtering()
        self.test_rest_api_collection_filtering()
        self.test_jsonrpc_collection_filtering()
        self.test_edge_cases()
        self.test_cross_interface_consistency()
        
        # Summary
        print("\n📊 Test Summary:")
        passed = sum(1 for result in self.test_results if result["passed"])
        total = len(self.test_results)
        
        print(f"   Passed: {passed}/{total}")
        print(f"   Success Rate: {(passed/total)*100:.1f}%")
        
        if passed == total:
            print("🎉 All tests passed!")
            return True
        else:
            print("⚠️  Some tests failed. Check details above.")
            return False

def main():
    """Main test runner"""
    if len(sys.argv) > 1:
        base_url = sys.argv[1]
    else:
        base_url = BASE_URL
    
    tester = SearchFilteringTests(base_url)
    success = tester.run_all_tests()
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
