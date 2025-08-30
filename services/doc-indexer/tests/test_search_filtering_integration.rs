//! Sprint 005: Comprehensive Search Filtering Rust Integration Tests
//! Tests for collection filtering functionality across all interfaces

use std::time::Duration;
use tokio::time::timeout;
use serde_json::{json, Value};
use reqwest::Client;

/// Test configuration
const BASE_URL: &str = "http://localhost:8081";
const TIMEOUT_DURATION: Duration = Duration::from_secs(30);

#[tokio::test]
async fn test_rest_api_collection_filtering() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Test with collection filtering
    let payload = json!({
        "query": "test",
        "limit": 10,
        "filters": {
            "collection_name": "zero_latency_docs"
        }
    });
    
    let response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/api/search", BASE_URL))
            .json(&payload)
            .send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    
    let data: Value = response.json().await?;
    let results = data["results"].as_array().unwrap_or(&vec![]);
    
    // Verify all results are from the correct collection
    for result in results {
        if let Some(collection) = result.get("collection") {
            assert_eq!(collection.as_str().unwrap(), "zero_latency_docs");
        }
    }
    
    println!("✅ REST API collection filtering test passed: {} results", results.len());
    Ok(())
}

#[tokio::test]
async fn test_rest_api_default_search() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let payload = json!({
        "query": "test",
        "limit": 10
    });
    
    let response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/api/search", BASE_URL))
            .json(&payload)
            .send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    
    let data: Value = response.json().await?;
    let results = data["results"].as_array().unwrap_or(&vec![]);
    
    println!("✅ REST API default search test passed: {} results", results.len());
    Ok(())
}

#[tokio::test]
async fn test_jsonrpc_collection_filtering() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "document.search",
        "params": {
            "query": "test",
            "filters": {
                "collection": "zero_latency_docs"
            }
        },
        "id": 1
    });
    
    let response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/jsonrpc", BASE_URL))
            .json(&payload)
            .send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    
    let data: Value = response.json().await?;
    
    // Check for JSON-RPC compliance
    assert!(data.get("jsonrpc").is_some());
    assert!(data.get("id").is_some());
    assert!(data.get("error").is_none());
    
    if let Some(result) = data.get("result") {
        let results = result["results"].as_array().unwrap_or(&vec![]);
        
        // Verify collection filtering
        for result in results {
            if let Some(metadata) = result.get("metadata") {
                if let Some(collection) = metadata.get("collection") {
                    assert_eq!(collection.as_str().unwrap(), "zero_latency_docs");
                }
            }
        }
        
        println!("✅ JSON-RPC collection filtering test passed: {} results", results.len());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_jsonrpc_default_search() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "document.search",
        "params": {
            "query": "test"
        },
        "id": 2
    });
    
    let response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/jsonrpc", BASE_URL))
            .json(&payload)
            .send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    
    let data: Value = response.json().await?;
    
    // JSON-RPC compliance checks
    assert!(data.get("jsonrpc").is_some());
    assert!(data.get("id").is_some());
    assert!(data.get("error").is_none());
    
    if let Some(result) = data.get("result") {
        let results = result["results"].as_array().unwrap_or(&vec![]);
        println!("✅ JSON-RPC default search test passed: {} results", results.len());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_invalid_collection_handling() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Test REST API with invalid collection
    let payload = json!({
        "query": "test",
        "filters": {
            "collection_name": "invalid_collection_name"
        }
    });
    
    let response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/api/search", BASE_URL))
            .json(&payload)
            .send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    
    let data: Value = response.json().await?;
    let results = data["results"].as_array().unwrap_or(&vec![]);
    
    // Should return empty results for invalid collection
    assert_eq!(results.len(), 0);
    
    println!("✅ Invalid collection handling test passed: no results for invalid collection");
    Ok(())
}

#[tokio::test]
async fn test_empty_query_handling() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let payload = json!({
        "query": "",
        "filters": {
            "collection_name": "zero_latency_docs"
        }
    });
    
    let response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/api/search", BASE_URL))
            .json(&payload)
            .send()
    ).await??;
    
    // Should handle gracefully (either 200 with empty results or 400)
    assert!(response.status() == 200 || response.status() == 400);
    
    println!("✅ Empty query handling test passed: status {}", response.status());
    Ok(())
}

#[tokio::test]
async fn test_cross_interface_consistency() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let query = "test";
    let collection = "zero_latency_docs";
    
    // Get REST API results
    let rest_payload = json!({
        "query": query,
        "filters": {
            "collection_name": collection
        }
    });
    
    let rest_response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/api/search", BASE_URL))
            .json(&rest_payload)
            .send()
    ).await??;
    
    let rest_data: Value = rest_response.json().await?;
    let rest_results = rest_data["results"].as_array().unwrap_or(&vec![]);
    
    // Get JSON-RPC results
    let jsonrpc_payload = json!({
        "jsonrpc": "2.0",
        "method": "document.search",
        "params": {
            "query": query,
            "filters": {
                "collection": collection
            }
        },
        "id": 1
    });
    
    let jsonrpc_response = timeout(
        TIMEOUT_DURATION,
        client.post(&format!("{}/jsonrpc", BASE_URL))
            .json(&jsonrpc_payload)
            .send()
    ).await??;
    
    let jsonrpc_data: Value = jsonrpc_response.json().await?;
    let jsonrpc_results = if let Some(result) = jsonrpc_data.get("result") {
        result["results"].as_array().unwrap_or(&vec![])
    } else {
        &vec![]
    };
    
    // Compare result counts (allow small differences due to timing)
    let rest_count = rest_results.len();
    let jsonrpc_count = jsonrpc_results.len();
    
    let diff = if rest_count > jsonrpc_count {
        rest_count - jsonrpc_count
    } else {
        jsonrpc_count - rest_count
    };
    
    assert!(diff <= 2, "Large difference in result counts: REST={}, JSON-RPC={}", rest_count, jsonrpc_count);
    
    println!("✅ Cross-interface consistency test passed: REST={}, JSON-RPC={}", rest_count, jsonrpc_count);
    Ok(())
}

#[tokio::test]
async fn test_collection_parameter_validation() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Test various collection parameter scenarios
    let test_cases = vec![
        ("zero_latency_docs", true),
        ("copilot-chat-dist", true),
        ("nonexistent_collection", false),
        ("", false),
    ];
    
    for (collection, should_have_results) in test_cases {
        let payload = json!({
            "query": "test",
            "filters": {
                "collection_name": collection
            }
        });
        
        let response = timeout(
            TIMEOUT_DURATION,
            client.post(&format!("{}/api/search", BASE_URL))
                .json(&payload)
                .send()
        ).await??;
        
        assert_eq!(response.status(), 200);
        
        let data: Value = response.json().await?;
        let results = data["results"].as_array().unwrap_or(&vec![]);
        
        if should_have_results {
            // For valid collections, we might or might not have results depending on content
            println!("Collection '{}': {} results", collection, results.len());
        } else {
            // For invalid collections, should have no results
            assert_eq!(results.len(), 0, "Expected no results for collection '{}'", collection);
            println!("Collection '{}': correctly returned no results", collection);
        }
    }
    
    println!("✅ Collection parameter validation test passed");
    Ok(())
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    
    /// Helper function to check if the doc-indexer service is running
    pub async fn is_service_running() -> bool {
        let client = Client::new();
        
        match timeout(
            Duration::from_secs(5),
            client.get(&format!("{}/health", BASE_URL)).send()
        ).await {
            Ok(Ok(response)) => response.status().is_success(),
            _ => false,
        }
    }
    
    /// Helper function to wait for service to be ready
    pub async fn wait_for_service(max_attempts: u32) -> Result<(), Box<dyn std::error::Error>> {
        for attempt in 1..=max_attempts {
            if is_service_running().await {
                println!("Service is ready (attempt {})", attempt);
                return Ok(());
            }
            
            if attempt < max_attempts {
                println!("Service not ready, waiting... (attempt {})", attempt);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
        
        Err("Service did not become ready within timeout".into())
    }
}

/// Integration test runner that ensures service is running first
#[tokio::test]
async fn test_service_availability() -> Result<(), Box<dyn std::error::Error>> {
    use test_helpers::*;
    
    // Wait for service to be ready
    wait_for_service(10).await?;
    
    println!("✅ Service availability test passed");
    Ok(())
}
