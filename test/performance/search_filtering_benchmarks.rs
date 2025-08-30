//! Sprint 005: Performance & Regression Testing for Search Filtering
//! ZL-005-008: Rust-based performance benchmarks and regression validation

use std::time::{Duration, Instant};
use tokio::time::timeout;
use serde_json::{json, Value};
use reqwest::Client;
use std::collections::HashMap;

/// Test configuration
const BASE_URL: &str = "http://localhost:8081";
const TIMEOUT_DURATION: Duration = Duration::from_secs(30);

/// Performance metrics collection
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub avg_time_ms: f64,
    pub median_time_ms: f64,
    pub min_time_ms: f64,
    pub max_time_ms: f64,
    pub samples: usize,
}

impl PerformanceMetrics {
    pub fn from_measurements(times: &[f64]) -> Self {
        if times.is_empty() {
            return Self {
                avg_time_ms: 0.0,
                median_time_ms: 0.0,
                min_time_ms: 0.0,
                max_time_ms: 0.0,
                samples: 0,
            };
        }

        let mut sorted_times = times.to_vec();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        Self {
            avg_time_ms: times.iter().sum::<f64>() / times.len() as f64,
            median_time_ms: sorted_times[sorted_times.len() / 2],
            min_time_ms: sorted_times[0],
            max_time_ms: sorted_times[sorted_times.len() - 1],
            samples: times.len(),
        }
    }
}

/// Benchmark REST API search performance
#[tokio::test]
async fn benchmark_rest_api_search() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Test scenarios
    let scenarios = vec![
        ("default_search", json!({
            "query": "test",
            "limit": 10
        })),
        ("collection_filtered", json!({
            "query": "test",
            "filters": {"collection_name": "zero_latency_docs"},
            "limit": 10
        })),
        ("high_threshold", json!({
            "query": "test",
            "threshold": 0.8,
            "limit": 10
        })),
        ("large_limit", json!({
            "query": "test",
            "limit": 50
        })),
    ];

    let mut results = HashMap::new();
    let iterations = 10;

    for (scenario_name, payload) in scenarios {
        let mut times = Vec::new();

        for _i in 0..iterations {
            let start = Instant::now();
            
            let response = timeout(
                TIMEOUT_DURATION,
                client.post(&format!("{}/api/search", BASE_URL))
                    .json(&payload)
                    .send()
            ).await??;

            let duration = start.elapsed();
            
            if response.status().is_success() {
                times.push(duration.as_secs_f64() * 1000.0); // Convert to ms
            }
        }

        let metrics = PerformanceMetrics::from_measurements(&times);
        println!("📊 REST API {} - Avg: {:.2}ms, Median: {:.2}ms, Samples: {}", 
                 scenario_name, metrics.avg_time_ms, metrics.median_time_ms, metrics.samples);
        
        results.insert(scenario_name.to_string(), metrics);
    }

    // Validate performance expectations
    if let Some(default) = results.get("default_search") {
        if let Some(filtered) = results.get("collection_filtered") {
            let efficiency = ((default.avg_time_ms - filtered.avg_time_ms) / default.avg_time_ms) * 100.0;
            println!("✅ Collection filtering efficiency: {:.1}% faster", efficiency);
            
            // Collection filtering should be at least as fast, or not significantly slower
            assert!(filtered.avg_time_ms <= default.avg_time_ms * 1.2, 
                    "Collection filtering is significantly slower than default search");
        }
    }

    Ok(())
}

/// Benchmark JSON-RPC search performance  
#[tokio::test]
async fn benchmark_jsonrpc_search() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let scenarios = vec![
        ("default_search", json!({
            "jsonrpc": "2.0",
            "method": "document.search",
            "params": {"query": "test"},
            "id": 1
        })),
        ("collection_filtered", json!({
            "jsonrpc": "2.0",
            "method": "document.search",
            "params": {
                "query": "test",
                "filters": {"collection": "zero_latency_docs"}
            },
            "id": 2
        })),
        ("high_threshold", json!({
            "jsonrpc": "2.0",
            "method": "document.search",
            "params": {
                "query": "test",
                "threshold": 0.8
            },
            "id": 3
        })),
    ];

    let mut results = HashMap::new();
    let iterations = 10;

    for (scenario_name, payload) in scenarios {
        let mut times = Vec::new();

        for _i in 0..iterations {
            let start = Instant::now();
            
            let response = timeout(
                TIMEOUT_DURATION,
                client.post(&format!("{}/jsonrpc", BASE_URL))
                    .json(&payload)
                    .send()
            ).await??;

            let duration = start.elapsed();
            
            if response.status().is_success() {
                let data: Value = response.json().await?;
                // Ensure no JSON-RPC error
                if !data.get("error").is_some() {
                    times.push(duration.as_secs_f64() * 1000.0);
                }
            }
        }

        let metrics = PerformanceMetrics::from_measurements(&times);
        println!("📊 JSON-RPC {} - Avg: {:.2}ms, Median: {:.2}ms, Samples: {}", 
                 scenario_name, metrics.avg_time_ms, metrics.median_time_ms, metrics.samples);
        
        results.insert(scenario_name.to_string(), metrics);
    }

    // Validate JSON-RPC performance
    if let Some(default) = results.get("default_search") {
        // JSON-RPC should have reasonable performance (under 200ms avg)
        assert!(default.avg_time_ms < 200.0, 
                "JSON-RPC search is too slow: {:.2}ms", default.avg_time_ms);
        
        if let Some(filtered) = results.get("collection_filtered") {
            // Collection filtering should not be significantly slower
            assert!(filtered.avg_time_ms <= default.avg_time_ms * 1.5, 
                    "JSON-RPC collection filtering is significantly slower");
        }
    }

    Ok(())
}

/// Test concurrent load performance
#[tokio::test]
async fn test_concurrent_load() -> Result<(), Box<dyn std::error::Error>> {
    let concurrent_users = 5;
    let requests_per_user = 10;
    
    println!("🚀 Testing concurrent load: {} users, {} requests each", 
             concurrent_users, requests_per_user);

    let client = Client::new();
    let payload = json!({
        "query": "test",
        "filters": {"collection_name": "zero_latency_docs"},
        "limit": 10
    });

    let mut tasks = Vec::new();
    
    for user_id in 0..concurrent_users {
        let client = client.clone();
        let payload = payload.clone();
        
        let task = tokio::spawn(async move {
            let mut user_times = Vec::new();
            
            for _request in 0..requests_per_user {
                let start = Instant::now();
                
                match timeout(
                    TIMEOUT_DURATION,
                    client.post(&format!("{}/api/search", BASE_URL))
                        .json(&payload)
                        .send()
                ).await {
                    Ok(Ok(response)) if response.status().is_success() => {
                        let duration = start.elapsed();
                        user_times.push(duration.as_secs_f64() * 1000.0);
                    }
                    _ => {
                        println!("❌ Request failed for user {}", user_id);
                    }
                }
            }
            
            user_times
        });
        
        tasks.push(task);
    }

    // Collect all response times
    let mut all_times = Vec::new();
    for task in tasks {
        let user_times = task.await?;
        all_times.extend(user_times);
    }

    let metrics = PerformanceMetrics::from_measurements(&all_times);
    println!("📊 Concurrent Load - Avg: {:.2}ms, Median: {:.2}ms, Total Requests: {}", 
             metrics.avg_time_ms, metrics.median_time_ms, metrics.samples);

    // Validate concurrent performance
    assert!(metrics.avg_time_ms < 500.0, 
            "Concurrent load performance is too slow: {:.2}ms", metrics.avg_time_ms);
    assert!(metrics.samples >= (concurrent_users * requests_per_user) / 2, 
            "Too many failed requests during load test");

    // Calculate 95th percentile
    let mut sorted_times = all_times.clone();
    sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
    let p95_time = sorted_times.get(p95_index).unwrap_or(&0.0);
    
    println!("📊 95th percentile response time: {:.2}ms", p95_time);
    assert!(*p95_time < 1000.0, "95th percentile response time too high: {:.2}ms", p95_time);

    Ok(())
}

/// Test search response consistency
#[tokio::test]
async fn test_search_response_consistency() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Test same query multiple times to ensure consistent response times
    let query = "test";
    let collection = "zero_latency_docs";
    let iterations = 20;
    
    println!("🔍 Testing search response consistency over {} iterations", iterations);

    let mut rest_times = Vec::new();
    let mut jsonrpc_times = Vec::new();

    // REST API tests
    for _i in 0..iterations {
        let payload = json!({
            "query": query,
            "filters": {"collection_name": collection},
            "limit": 10
        });

        let start = Instant::now();
        let response = timeout(
            TIMEOUT_DURATION,
            client.post(&format!("{}/api/search", BASE_URL))
                .json(&payload)
                .send()
        ).await??;

        if response.status().is_success() {
            let duration = start.elapsed();
            rest_times.push(duration.as_secs_f64() * 1000.0);
        }
    }

    // JSON-RPC tests
    for i in 0..iterations {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "document.search",
            "params": {
                "query": query,
                "filters": {"collection": collection}
            },
            "id": i + 1
        });

        let start = Instant::now();
        let response = timeout(
            TIMEOUT_DURATION,
            client.post(&format!("{}/jsonrpc", BASE_URL))
                .json(&payload)
                .send()
        ).await??;

        if response.status().is_success() {
            let data: Value = response.json().await?;
            if !data.get("error").is_some() {
                let duration = start.elapsed();
                jsonrpc_times.push(duration.as_secs_f64() * 1000.0);
            }
        }
    }

    let rest_metrics = PerformanceMetrics::from_measurements(&rest_times);
    let jsonrpc_metrics = PerformanceMetrics::from_measurements(&jsonrpc_times);

    println!("📊 REST API consistency - Avg: {:.2}ms, StdDev: {:.2}ms", 
             rest_metrics.avg_time_ms, 
             (rest_metrics.max_time_ms - rest_metrics.min_time_ms) / 2.0);
    
    println!("📊 JSON-RPC consistency - Avg: {:.2}ms, StdDev: {:.2}ms", 
             jsonrpc_metrics.avg_time_ms,
             (jsonrpc_metrics.max_time_ms - jsonrpc_metrics.min_time_ms) / 2.0);

    // Validate consistency (max time should not be more than 5x min time)
    assert!(rest_metrics.max_time_ms <= rest_metrics.min_time_ms * 5.0,
            "REST API response times are inconsistent");
    assert!(jsonrpc_metrics.max_time_ms <= jsonrpc_metrics.min_time_ms * 5.0,
            "JSON-RPC response times are inconsistent");

    Ok(())
}

/// Regression test against baseline performance
#[tokio::test]
async fn test_performance_regression() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Performance baselines (adjust based on your system)
    let baselines = HashMap::from([
        ("rest_default", 100.0),      // 100ms max
        ("rest_filtered", 100.0),     // 100ms max  
        ("jsonrpc_default", 120.0),   // 120ms max
        ("jsonrpc_filtered", 120.0),  // 120ms max
    ]);

    println!("📈 Running regression tests against baselines");

    // Test REST API default search
    let start = Instant::now();
    let response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/api/search", BASE_URL))
            .json(&json!({"query": "test", "limit": 10}))
            .send()
    ).await??;
    let rest_default_time = start.elapsed().as_secs_f64() * 1000.0;

    // Test REST API filtered search  
    let start = Instant::now();
    let response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/api/search", BASE_URL))
            .json(&json!({
                "query": "test",
                "filters": {"collection_name": "zero_latency_docs"},
                "limit": 10
            }))
            .send()
    ).await??;
    let rest_filtered_time = start.elapsed().as_secs_f64() * 1000.0;

    // Test JSON-RPC default search
    let start = Instant::now();
    let response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/jsonrpc", BASE_URL))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "document.search",
                "params": {"query": "test"},
                "id": 1
            }))
            .send()
    ).await??;
    let jsonrpc_default_time = start.elapsed().as_secs_f64() * 1000.0;

    // Test JSON-RPC filtered search
    let start = Instant::now();
    let response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/jsonrpc", BASE_URL))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "document.search",
                "params": {
                    "query": "test",
                    "filters": {"collection": "zero_latency_docs"}
                },
                "id": 2
            }))
            .send()
    ).await??;
    let jsonrpc_filtered_time = start.elapsed().as_secs_f64() * 1000.0;

    // Check against baselines
    let actual_times = HashMap::from([
        ("rest_default", rest_default_time),
        ("rest_filtered", rest_filtered_time), 
        ("jsonrpc_default", jsonrpc_default_time),
        ("jsonrpc_filtered", jsonrpc_filtered_time),
    ]);

    let mut regressions = Vec::new();

    for (test_name, baseline) in &baselines {
        if let Some(&actual) = actual_times.get(test_name) {
            println!("📊 {}: {:.2}ms (baseline: {:.2}ms)", test_name, actual, baseline);
            
            if actual > *baseline {
                regressions.push(format!("{}: {:.2}ms > {:.2}ms", test_name, actual, baseline));
            }
        }
    }

    if !regressions.is_empty() {
        println!("❌ Performance regressions detected:");
        for regression in &regressions {
            println!("   - {}", regression);
        }
        panic!("Performance regression test failed");
    } else {
        println!("✅ All performance tests within baseline expectations");
    }

    Ok(())
}

/// Benchmark collection filtering efficiency
#[tokio::test]
async fn benchmark_collection_filtering_efficiency() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let iterations = 15;
    
    println!("🔍 Benchmarking collection filtering efficiency over {} iterations", iterations);

    // Benchmark unfiltered search
    let mut unfiltered_times = Vec::new();
    for _i in 0..iterations {
        let start = Instant::now();
        let response = timeout(
            TIMEOUT_DURATION,
            client.post(&format!("{}/api/search", BASE_URL))
                .json(&json!({"query": "test", "limit": 10}))
                .send()
        ).await??;
        
        if response.status().is_success() {
            let duration = start.elapsed();
            unfiltered_times.push(duration.as_secs_f64() * 1000.0);
        }
    }

    // Benchmark filtered search
    let mut filtered_times = Vec::new();
    for _i in 0..iterations {
        let start = Instant::now();
        let response = timeout(
            TIMEOUT_DURATION,
            client.post(&format!("{}/api/search", BASE_URL))
                .json(&json!({
                    "query": "test",
                    "filters": {"collection_name": "zero_latency_docs"},
                    "limit": 10
                }))
                .send()
        ).await??;
        
        if response.status().is_success() {
            let duration = start.elapsed();
            filtered_times.push(duration.as_secs_f64() * 1000.0);
        }
    }

    let unfiltered_metrics = PerformanceMetrics::from_measurements(&unfiltered_times);
    let filtered_metrics = PerformanceMetrics::from_measurements(&filtered_times);

    println!("📊 Unfiltered search - Avg: {:.2}ms, Median: {:.2}ms", 
             unfiltered_metrics.avg_time_ms, unfiltered_metrics.median_time_ms);
    println!("📊 Collection filtered - Avg: {:.2}ms, Median: {:.2}ms", 
             filtered_metrics.avg_time_ms, filtered_metrics.median_time_ms);

    // Calculate efficiency
    let efficiency = if unfiltered_metrics.avg_time_ms > 0.0 {
        ((unfiltered_metrics.avg_time_ms - filtered_metrics.avg_time_ms) / unfiltered_metrics.avg_time_ms) * 100.0
    } else {
        0.0
    };

    println!("📊 Collection filtering efficiency: {:.1}%", efficiency);

    // Collection filtering should not be significantly slower
    assert!(filtered_metrics.avg_time_ms <= unfiltered_metrics.avg_time_ms * 1.3,
            "Collection filtering is significantly slower than unfiltered search");

    if efficiency > 0.0 {
        println!("✅ Collection filtering improves performance");
    } else {
        println!("⚠️  Collection filtering does not improve performance significantly");
    }

    Ok(())
}
