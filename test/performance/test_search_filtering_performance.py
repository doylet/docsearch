#!/usr/bin/env python3
"""
Sprint 005: Performance & Regression Testing for Search Filtering
ZL-005-008: Comprehensive performance testing and regression validation
"""

import json
import requests
import subprocess
import time
import statistics
import sys
import os
from typing import Dict, List, Any, Optional, Tuple
from concurrent.futures import ThreadPoolExecutor, as_completed
import threading

# Configuration
BASE_URL = "http://localhost:8081"
CLI_BINARY = "./target/debug/mdx"

class PerformanceTester:
    def __init__(self, base_url: str = BASE_URL):
        self.base_url = base_url
        self.results = {}
        self.lock = threading.Lock()
        
    def log_result(self, test_name: str, metric: str, value: float, unit: str = "ms"):
        """Log performance result"""
        with self.lock:
            if test_name not in self.results:
                self.results[test_name] = {}
            self.results[test_name][metric] = {"value": value, "unit": unit}
        print(f"📊 {test_name}: {metric} = {value:.2f}{unit}")
    
    def benchmark_rest_api_search(self, iterations: int = 10) -> Dict[str, float]:
        """Benchmark REST API search performance"""
        print("\n🔍 Benchmarking REST API Search Performance...")
        
        # Test scenarios
        scenarios = [
            ("Default Search", {"query": "test", "limit": 10}),
            ("Collection Filtered", {"query": "test", "filters": {"collection_name": "zero_latency_docs"}, "limit": 10}),
            ("High Threshold", {"query": "test", "threshold": 0.8, "limit": 10}),
            ("Large Limit", {"query": "test", "limit": 50}),
        ]
        
        results = {}
        
        for scenario_name, payload in scenarios:
            times = []
            
            for i in range(iterations):
                start_time = time.time()
                try:
                    response = requests.post(f"{self.base_url}/api/search", json=payload, timeout=30)
                    response.raise_for_status()
                    end_time = time.time()
                    
                    response_time = (end_time - start_time) * 1000  # Convert to ms
                    times.append(response_time)
                    
                except Exception as e:
                    print(f"❌ Error in {scenario_name} iteration {i+1}: {e}")
                    continue
            
            if times:
                avg_time = statistics.mean(times)
                median_time = statistics.median(times)
                min_time = min(times)
                max_time = max(times)
                
                self.log_result(f"REST API - {scenario_name}", "avg_response_time", avg_time)
                self.log_result(f"REST API - {scenario_name}", "median_response_time", median_time)
                self.log_result(f"REST API - {scenario_name}", "min_response_time", min_time)
                self.log_result(f"REST API - {scenario_name}", "max_response_time", max_time)
                
                results[scenario_name] = {
                    "avg": avg_time,
                    "median": median_time,
                    "min": min_time,
                    "max": max_time,
                    "samples": len(times)
                }
        
        return results
    
    def benchmark_jsonrpc_search(self, iterations: int = 10) -> Dict[str, float]:
        """Benchmark JSON-RPC search performance"""
        print("\n🔍 Benchmarking JSON-RPC Search Performance...")
        
        scenarios = [
            ("Default Search", {
                "jsonrpc": "2.0",
                "method": "document.search",
                "params": {"query": "test"},
                "id": 1
            }),
            ("Collection Filtered", {
                "jsonrpc": "2.0",
                "method": "document.search",
                "params": {"query": "test", "filters": {"collection": "zero_latency_docs"}},
                "id": 2
            }),
            ("High Threshold", {
                "jsonrpc": "2.0",
                "method": "document.search",
                "params": {"query": "test", "threshold": 0.8},
                "id": 3
            }),
        ]
        
        results = {}
        
        for scenario_name, payload in scenarios:
            times = []
            
            for i in range(iterations):
                start_time = time.time()
                try:
                    response = requests.post(f"{self.base_url}/jsonrpc", json=payload, timeout=30)
                    response.raise_for_status()
                    end_time = time.time()
                    
                    response_time = (end_time - start_time) * 1000
                    times.append(response_time)
                    
                except Exception as e:
                    print(f"❌ Error in {scenario_name} iteration {i+1}: {e}")
                    continue
            
            if times:
                avg_time = statistics.mean(times)
                median_time = statistics.median(times)
                
                self.log_result(f"JSON-RPC - {scenario_name}", "avg_response_time", avg_time)
                self.log_result(f"JSON-RPC - {scenario_name}", "median_response_time", median_time)
                
                results[scenario_name] = {
                    "avg": avg_time,
                    "median": median_time,
                    "samples": len(times)
                }
        
        return results
    
    def benchmark_cli_search(self, iterations: int = 5) -> Dict[str, float]:
        """Benchmark CLI search performance"""
        print("\n🔍 Benchmarking CLI Search Performance...")
        
        scenarios = [
            ("Default Search", ["search", "test"]),
            ("Collection Filtered", ["search", "test", "--collection", "zero_latency_docs"]),
            ("High Threshold", ["search", "test", "--threshold", "0.8"]),
        ]
        
        results = {}
        
        for scenario_name, args in scenarios:
            times = []
            
            for i in range(iterations):
                start_time = time.time()
                try:
                    result = subprocess.run([CLI_BINARY] + args, 
                                          capture_output=True, text=True, timeout=30)
                    end_time = time.time()
                    
                    if result.returncode == 0:
                        response_time = (end_time - start_time) * 1000
                        times.append(response_time)
                    else:
                        print(f"❌ CLI error in {scenario_name}: {result.stderr}")
                        
                except Exception as e:
                    print(f"❌ Error in CLI {scenario_name} iteration {i+1}: {e}")
                    continue
            
            if times:
                avg_time = statistics.mean(times)
                self.log_result(f"CLI - {scenario_name}", "avg_response_time", avg_time)
                
                results[scenario_name] = {
                    "avg": avg_time,
                    "samples": len(times)
                }
        
        return results
    
    def concurrent_load_test(self, concurrent_users: int = 5, requests_per_user: int = 10):
        """Test performance under concurrent load"""
        print(f"\n🚀 Concurrent Load Test: {concurrent_users} users, {requests_per_user} requests each...")
        
        def user_session(user_id: int) -> List[float]:
            times = []
            payload = {"query": "test", "filters": {"collection_name": "zero_latency_docs"}}
            
            for i in range(requests_per_user):
                start_time = time.time()
                try:
                    response = requests.post(f"{self.base_url}/api/search", json=payload, timeout=30)
                    response.raise_for_status()
                    end_time = time.time()
                    
                    response_time = (end_time - start_time) * 1000
                    times.append(response_time)
                    
                except Exception as e:
                    print(f"❌ User {user_id} request {i+1} failed: {e}")
                    
            return times
        
        all_times = []
        
        with ThreadPoolExecutor(max_workers=concurrent_users) as executor:
            futures = [executor.submit(user_session, i) for i in range(concurrent_users)]
            
            for future in as_completed(futures):
                try:
                    user_times = future.result()
                    all_times.extend(user_times)
                except Exception as e:
                    print(f"❌ User session failed: {e}")
        
        if all_times:
            avg_time = statistics.mean(all_times)
            median_time = statistics.median(all_times)
            p95_time = statistics.quantiles(all_times, n=20)[18]  # 95th percentile
            
            self.log_result("Concurrent Load Test", "avg_response_time", avg_time)
            self.log_result("Concurrent Load Test", "median_response_time", median_time)
            self.log_result("Concurrent Load Test", "p95_response_time", p95_time)
            self.log_result("Concurrent Load Test", "total_requests", len(all_times), "requests")
    
    def memory_usage_test(self):
        """Test memory usage during searches"""
        print("\n💾 Memory Usage Test...")
        
        try:
            # Get initial memory usage
            result = subprocess.run(["ps", "-o", "rss", "-p", "$(pgrep doc-indexer)"], 
                                  capture_output=True, text=True, shell=True)
            
            if result.returncode == 0:
                memory_lines = result.stdout.strip().split('\n')
                if len(memory_lines) > 1:
                    initial_memory = int(memory_lines[1])  # RSS in KB
                    self.log_result("Memory Usage", "initial_memory", initial_memory, "KB")
                    
                    # Perform intensive searches
                    for i in range(20):
                        payload = {"query": f"test query {i}", "limit": 50}
                        requests.post(f"{self.base_url}/api/search", json=payload, timeout=30)
                    
                    # Check memory after intensive usage
                    result = subprocess.run(["ps", "-o", "rss", "-p", "$(pgrep doc-indexer)"], 
                                          capture_output=True, text=True, shell=True)
                    
                    if result.returncode == 0:
                        memory_lines = result.stdout.strip().split('\n')
                        if len(memory_lines) > 1:
                            final_memory = int(memory_lines[1])
                            memory_increase = final_memory - initial_memory
                            
                            self.log_result("Memory Usage", "final_memory", final_memory, "KB")
                            self.log_result("Memory Usage", "memory_increase", memory_increase, "KB")
                            
                            # Calculate memory increase percentage
                            if initial_memory > 0:
                                increase_percent = (memory_increase / initial_memory) * 100
                                self.log_result("Memory Usage", "increase_percentage", increase_percent, "%")
            
        except Exception as e:
            print(f"❌ Memory usage test failed: {e}")
    
    def regression_test_comparison(self):
        """Compare performance against expected baselines"""
        print("\n📈 Regression Test - Performance Baselines...")
        
        # Expected performance baselines (adjust based on your system)
        baselines = {
            "REST API - Default Search": {"avg_response_time": 50.0},  # 50ms
            "REST API - Collection Filtered": {"avg_response_time": 45.0},  # Should be faster
            "JSON-RPC - Default Search": {"avg_response_time": 55.0},  # 55ms
            "JSON-RPC - Collection Filtered": {"avg_response_time": 50.0},  # Should be faster
            "CLI - Default Search": {"avg_response_time": 2000.0},  # 2s (includes process startup)
        }
        
        regression_issues = []
        
        for test_name, baseline in baselines.items():
            if test_name in self.results:
                for metric, expected in baseline.items():
                    if metric in self.results[test_name]:
                        actual = self.results[test_name][metric]["value"]
                        threshold = expected * 1.5  # Allow 50% performance degradation
                        
                        if actual > threshold:
                            regression_issues.append(f"{test_name} {metric}: {actual:.2f}ms > {threshold:.2f}ms")
                            print(f"⚠️  REGRESSION: {test_name} {metric} = {actual:.2f}ms (expected < {threshold:.2f}ms)")
                        else:
                            print(f"✅ OK: {test_name} {metric} = {actual:.2f}ms (within {threshold:.2f}ms)")
        
        if regression_issues:
            print(f"\n❌ Found {len(regression_issues)} regression issues:")
            for issue in regression_issues:
                print(f"   - {issue}")
            return False
        else:
            print(f"\n✅ No regression issues found!")
            return True
    
    def generate_performance_report(self):
        """Generate comprehensive performance report"""
        print("\n📋 Performance Test Report")
        print("=" * 50)
        
        for test_name, metrics in self.results.items():
            print(f"\n{test_name}:")
            for metric, data in metrics.items():
                print(f"  {metric}: {data['value']:.2f} {data['unit']}")
        
        # Performance summary
        print(f"\n📊 Performance Summary")
        print("-" * 30)
        
        # Collection filtering efficiency
        if "REST API - Default Search" in self.results and "REST API - Collection Filtered" in self.results:
            default_time = self.results["REST API - Default Search"]["avg_response_time"]["value"]
            filtered_time = self.results["REST API - Collection Filtered"]["avg_response_time"]["value"]
            efficiency = ((default_time - filtered_time) / default_time) * 100
            
            print(f"Collection Filtering Efficiency: {efficiency:.1f}% faster")
            if efficiency > 0:
                print("✅ Collection filtering improves performance")
            else:
                print("⚠️  Collection filtering may not improve performance")
        
        # Interface comparison
        rest_times = [v["avg_response_time"]["value"] for k, v in self.results.items() 
                     if k.startswith("REST API") and "avg_response_time" in v]
        jsonrpc_times = [v["avg_response_time"]["value"] for k, v in self.results.items() 
                        if k.startswith("JSON-RPC") and "avg_response_time" in v]
        
        if rest_times and jsonrpc_times:
            avg_rest = statistics.mean(rest_times)
            avg_jsonrpc = statistics.mean(jsonrpc_times)
            print(f"Average REST API time: {avg_rest:.2f}ms")
            print(f"Average JSON-RPC time: {avg_jsonrpc:.2f}ms")
            
            faster_interface = "REST API" if avg_rest < avg_jsonrpc else "JSON-RPC"
            print(f"Faster interface: {faster_interface}")
    
    def run_all_tests(self):
        """Run comprehensive performance and regression tests"""
        print("🚀 Starting Performance & Regression Testing")
        print("=" * 60)
        
        # Core performance tests
        self.benchmark_rest_api_search(iterations=10)
        self.benchmark_jsonrpc_search(iterations=10)
        
        # CLI testing (fewer iterations due to process overhead)
        if os.path.exists(CLI_BINARY):
            self.benchmark_cli_search(iterations=5)
        else:
            print("⚠️  CLI binary not found, skipping CLI performance tests")
        
        # Load testing
        self.concurrent_load_test(concurrent_users=3, requests_per_user=5)
        
        # Memory testing
        self.memory_usage_test()
        
        # Regression analysis
        no_regressions = self.regression_test_comparison()
        
        # Generate report
        self.generate_performance_report()
        
        return no_regressions

def main():
    """Main test runner"""
    if len(sys.argv) > 1:
        base_url = sys.argv[1]
    else:
        base_url = BASE_URL
    
    tester = PerformanceTester(base_url)
    
    # Check service availability
    try:
        response = requests.get(f"{base_url}/health", timeout=5)
        if response.status_code != 200:
            print(f"❌ Service not healthy: {response.status_code}")
            sys.exit(1)
    except Exception as e:
        print(f"❌ Cannot connect to service at {base_url}: {e}")
        sys.exit(1)
    
    # Run all tests
    no_regressions = tester.run_all_tests()
    
    print(f"\n🎯 Test Summary:")
    print(f"   Service URL: {base_url}")
    print(f"   Regression Status: {'✅ PASS' if no_regressions else '❌ FAIL'}")
    print(f"   Performance Tests: Complete")
    
    sys.exit(0 if no_regressions else 1)

if __name__ == "__main__":
    main()
