mod test_utils;

use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::runtime::Runtime;
use reqwest::Client;
use serde_json::json;
use test_utils::{TestUtils, TestAssertions};

#[test]
fn smoke_test_advanced_query_enhancement_and_ranking() {
    let test_utils = TestUtils::new();
    let config = test_utils.create_test_config();
    
    let mut child = Command::new(&config.binary_path)
        .args([
            "--docs-path",
            &config.docs_path,
            "--port",
            &config.port.to_string(),
            "--log-level",
            "info",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start doc-indexer CLI for advanced search test");

    let rt = Runtime::new().unwrap();
    let client = Client::new();

    // Wait for the server to be ready using test utilities
    test_utils.wait_for_health_blocking(config.port, 30)
        .expect("doc-indexer did not become healthy in time");

    // Index the test document using unique collection name
    let index_body = json!({
        "path": config.docs_path,
        "collection": config.collection_name
    });
    let resp = rt
        .block_on(client.post(&config.index_url()).json(&index_body).send())
        .expect("Failed to POST to /api/index");
    TestAssertions::assert_success_response(&resp, "Indexing");

    std::thread::sleep(Duration::from_secs(2));

    // Search using a synonym or technical term (e.g., 'find' instead of 'search', 'api' for technical expansion)
    let search_body = json!({
        "query": "find Zero-Latency",
        "collection": config.collection_name,
        "limit": 5
    });
    let resp = rt
        .block_on(client.post(&config.search_url()).json(&search_body).send())
        .expect("Failed to POST to /api/search");
    TestAssertions::assert_success_response(&resp, "Search");
    
    let json: serde_json::Value = rt
        .block_on(resp.json())
        .expect("Failed to parse search response");
    TestAssertions::assert_search_results_not_empty(&json, "Advanced query search");

    let results = json
        .get("results")
        .and_then(|r| r.as_array())
        .expect("No results array in search response");
    let found = results.iter().any(|res| {
        res.get("content").map_or(false, |c| {
            c.as_str()
                .map_or(false, |s| s.contains("Zero-Latency doc-indexer smoke test"))
        })
    });
    assert!(
        found,
        "Should find test document in search results for 'find Zero-Latency'"
    );

    // Clean up
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn smoke_test_end_to_end_index_and_search() {
    let test_utils = TestUtils::new();
    let config = test_utils.create_test_config();
    
    let mut child = Command::new(&config.binary_path)
        .args([
            "--docs-path",
            &config.docs_path,
            "--port",
            &config.port.to_string(),
            "--log-level",
            "info",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start doc-indexer CLI for end-to-end test");

    // Use a tokio runtime for async HTTP
    let rt = Runtime::new().unwrap();
    let client = Client::new();

    // Wait for the server to be ready using test utilities
    test_utils.wait_for_health_blocking(config.port, 30)
        .expect("doc-indexer did not become healthy in time");

    // Index documents using unique collection name
    let index_body = json!({
        "path": config.docs_path,
        "collection": config.collection_name
    });
    let resp = rt
        .block_on(client.post(&config.index_url()).json(&index_body).send())
        .expect("Failed to POST to /api/index");
    TestAssertions::assert_success_response(&resp, "Indexing");

    // Wait for indexing to complete
    std::thread::sleep(Duration::from_secs(3));

    // Search for indexed content
    let search_body = json!({
        "query": "Zero-Latency doc-indexer smoke test",
        "collection": config.collection_name,
        "limit": 10
    });
    let resp = rt
        .block_on(client.post(&config.search_url()).json(&search_body).send())
        .expect("Failed to POST to /api/search");
    TestAssertions::assert_success_response(&resp, "Search");

    let json: serde_json::Value = rt
        .block_on(resp.json())
        .expect("Failed to parse search response");
    TestAssertions::assert_search_results_not_empty(&json, "End-to-end search");

    // Test semantic search capability
    let semantic_search_body = json!({
        "query": "document indexing service",
        "collection": config.collection_name,
        "limit": 5
    });
    let resp = rt
        .block_on(client.post(&config.search_url()).json(&semantic_search_body).send())
        .expect("Failed to POST semantic search");
    TestAssertions::assert_success_response(&resp, "Semantic search");

    let semantic_json: serde_json::Value = rt
        .block_on(resp.json())
        .expect("Failed to parse semantic search response");
    TestAssertions::assert_search_results_not_empty(&semantic_json, "Semantic search");

    // Clean up
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn smoke_test_cli_runs_with_docs_path() {
    let test_utils = TestUtils::new();
    let config = test_utils.create_test_config();

    // Test that the CLI can start with custom docs path
    let mut child = Command::new(&config.binary_path)
        .args([
            "--docs-path",
            &config.docs_path,
            "--port",
            &config.port.to_string(),
            "--log-level",
            "debug",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start doc-indexer CLI with custom docs path");

    // Wait for server to start
    test_utils.wait_for_health_blocking(config.port, 30)
        .expect("doc-indexer with custom docs path did not become healthy");

    let rt = Runtime::new().unwrap();
    let client = Client::new();

    // Verify health endpoint responds
    let resp = rt
        .block_on(client.get(&config.health_url()).send())
        .expect("Failed to GET health endpoint");
    TestAssertions::assert_success_response(&resp, "Health check");

    // Verify we can hit the index endpoint (even if no indexing occurs)
    let index_body = json!({
        "path": config.docs_path,
        "collection": config.collection_name
    });
    let resp = rt
        .block_on(client.post(&config.index_url()).json(&index_body).send())
        .expect("Failed to POST to index endpoint");
    TestAssertions::assert_success_response(&resp, "Index endpoint");

    // Clean up
    let _ = child.kill();
    let _ = child.wait();
}
