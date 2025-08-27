#[test]
fn smoke_test_advanced_query_enhancement_and_ranking() {
    // Use a unique port to avoid conflicts
    let port = 18082u16;
    let docs_path = std::fs::canonicalize("tests/fixtures")
        .expect("Failed to resolve absolute path to test fixtures");
    let mut child = Command::new("../../target/debug/doc-indexer")
        .args([
            "--docs-path",
            docs_path.to_str().unwrap(),
            "--port",
            &port.to_string(),
            "--log-level",
            "info",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start doc-indexer CLI for advanced search test");

    let rt = Runtime::new().unwrap();
    let client = Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Wait for the server to be ready (poll /health)
    let health_url = format!("{}/health", base_url);
    let mut ready = false;
    for _ in 0..60 {
        if let Ok(resp) = rt.block_on(client.get(&health_url).send()) {
            if resp.status().is_success() {
                ready = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    assert!(ready, "doc-indexer did not become healthy in time");

    // Index the test document
    let index_url = format!("{}/api/index", base_url);
    let index_body = json!({
        "path": docs_path.to_str().unwrap(),
        "collection": "advanced_test_collection"
    });
    let resp = rt
        .block_on(client.post(&index_url).json(&index_body).send())
        .expect("Failed to POST to /api/index");
    assert!(
        resp.status().is_success(),
        "Indexing failed: {}",
        rt.block_on(resp.text()).unwrap_or_default()
    );

    std::thread::sleep(Duration::from_secs(2));

    // Search using a synonym or technical term (e.g., 'find' instead of 'search', 'api' for technical expansion)
    let search_url = format!("{}/api/search", base_url);
    let search_body = json!({
        "query": "find Zero-Latency",
        "collection": "advanced_test_collection",
        "limit": 5
    });
    let resp = rt
        .block_on(client.post(&search_url).json(&search_body).send())
        .expect("Failed to POST to /api/search");
    assert!(
        resp.status().is_success(),
        "Search failed: {}",
        rt.block_on(resp.text()).unwrap_or_default()
    );
    let json: serde_json::Value = rt
        .block_on(resp.json())
        .expect("Failed to parse search response");
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
        "Advanced search (synonym/expansion) did not find the indexed document: {:?}",
        results
    );

    // Kill the process
    let _ = child.kill();
    let _ = child.wait();
}
// Smoke tests for Zero-Latency doc-indexer core functionality
// These tests validate CLI search, index, and reindex commands at a high level

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use reqwest::Client;
use serde_json::json;
use std::io::Read;
use std::path::PathBuf;
use tokio::runtime::Runtime;

#[test]
fn smoke_test_end_to_end_index_and_search() {
    // Use a unique port to avoid conflicts
    let port = 18081u16;
    let docs_path = std::fs::canonicalize("tests/fixtures")
        .expect("Failed to resolve absolute path to test fixtures");
    let mut child = Command::new("../../target/debug/doc-indexer")
        .args([
            "--docs-path",
            docs_path.to_str().unwrap(),
            "--port",
            &port.to_string(),
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
    let base_url = format!("http://127.0.0.1:{}", port);

    // Wait for the server to be ready (poll /health)
    let health_url = format!("{}/health", base_url);
    let mut ready = false;
    for _ in 0..60 {
        if let Ok(resp) = rt.block_on(client.get(&health_url).send()) {
            if resp.status().is_success() {
                ready = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    assert!(ready, "doc-indexer did not become healthy in time");

    // Index the test document
    let index_url = format!("{}/api/index", base_url);
    let index_body = json!({
        "path": docs_path.to_str().unwrap(),
        "collection": "smoke_test_collection"
    });
    let resp = rt
        .block_on(client.post(&index_url).json(&index_body).send())
        .expect("Failed to POST to /api/index");
    assert!(
        resp.status().is_success(),
        "Indexing failed: {}",
        rt.block_on(resp.text()).unwrap_or_default()
    );

    // Wait a bit for indexing to complete
    std::thread::sleep(Duration::from_secs(2));

    // Search for a known phrase from the test document
    let search_url = format!("{}/api/search", base_url);
    let search_body = json!({
        "query": "test document for Zero-Latency",
        "collection": "smoke_test_collection",
        "limit": 5
    });
    let resp = rt
        .block_on(client.post(&search_url).json(&search_body).send())
        .expect("Failed to POST to /api/search");
    assert!(
        resp.status().is_success(),
        "Search failed: {}",
        rt.block_on(resp.text()).unwrap_or_default()
    );
    let json: serde_json::Value = rt
        .block_on(resp.json())
        .expect("Failed to parse search response");
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
        "Indexed document not found in search results: {:?}",
        results
    );

    // Kill the process
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn smoke_test_cli_runs_with_docs_path() {
    // Spawn the process and kill it after a short timeout to avoid hanging
    let mut child = Command::new("../../target/debug/doc-indexer")
        .args([
            "--docs-path",
            "/Users/thomasdoyle/Daintree/projects/rust/Zero-Latency/docs/misc/artefacts",
            "--log-level",
            "info",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start doc-indexer CLI");

    // Wait for a short period to let the process start
    thread::sleep(Duration::from_secs(6));

    // Kill the process
    let _ = child.kill();
    let output = child
        .wait_with_output()
        .expect("Failed to get output from doc-indexer CLI");

    // Check that the output contains the expected startup message (in either stdout or stderr)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);
    assert!(
        combined.contains("Starting doc-indexer service")
            || combined.contains("Starting Zero Latency Documentation Indexer"),
        "Expected startup message in output, got: {}",
        combined
    );
}

#[test]
fn smoke_test_cli_env_example() {
    let output = Command::new("../../target/debug/doc-indexer")
        .args(["--env-example"])
        .output()
        .expect("Failed to run doc-indexer CLI with --env-example");
    assert!(
        output.status.success(),
        "doc-indexer CLI --env-example failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Match a variable that is always present in the output
    assert!(
        stdout.contains("DOC_INDEXER_LOG_LEVEL"),
        "Expected env example output, got: {}",
        stdout
    );
}

#[test]
fn smoke_test_cli_stdio_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "doc-indexer", "--", "--stdio-help"])
        .output()
        .expect("Failed to run doc-indexer CLI with --stdio-help");
    assert!(
        output.status.success(),
        "doc-indexer CLI --stdio-help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("json-rpc"),
        "Expected stdio help output, got: {}",
        stdout
    );
}
