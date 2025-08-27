use std::process::{Command, Stdio, Child};
use std::time::Duration;
use tokio::runtime::Runtime;
use reqwest::Client;
use zero_latency_config::TestConfigHelper;

/// Test utilities for doc-indexer integration tests
pub struct TestUtils {
    config_helper: TestConfigHelper,
}

impl TestUtils {
    /// Create new test utilities
    pub fn new() -> Self {
        Self {
            config_helper: TestConfigHelper::new(),
        }
    }
    
    /// Get a unique port for testing
    pub fn get_unique_port(&self) -> u16 {
        self.config_helper.get_unique_port()
    }
    
    /// Get a unique collection name for testing
    pub fn get_unique_collection_name(&self) -> String {
        self.config_helper.get_unique_collection_name()
    }
    
    /// Resolve the doc-indexer binary path
    pub fn resolve_binary_path(&self) -> String {
        self.config_helper
            .resolve_binary_path()
            .unwrap_or_else(|| "../../target/debug/doc-indexer".to_string())
    }
    
    /// Start a doc-indexer server process for testing
    pub fn start_test_server(&self, port: u16, docs_path: &str) -> Result<Child, Box<dyn std::error::Error>> {
        let binary_path = self.resolve_binary_path();
        
        let child = Command::new(&binary_path)
            .args([
                "--docs-path",
                docs_path,
                "--port",
                &port.to_string(),
                "--log-level",
                "info",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
            
        Ok(child)
    }
    
    /// Wait for a server to become healthy
    pub async fn wait_for_health(&self, port: u16, timeout_seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new();
        let health_url = format!("http://localhost:{}/health", port);
        
        for _ in 0..(timeout_seconds * 2) {
            if let Ok(resp) = client.get(&health_url).send().await {
                if resp.status().is_success() {
                    return Ok(());
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        Err(format!("Server at port {} did not become healthy within {} seconds", port, timeout_seconds).into())
    }
    
    /// Wait for a server to become healthy (blocking version for non-async tests)
    pub fn wait_for_health_blocking(&self, port: u16, timeout_seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
        let rt = Runtime::new()?;
        rt.block_on(self.wait_for_health(port, timeout_seconds))
    }
    
    /// Create a test configuration for a specific test
    pub fn create_test_config(&self) -> TestConfig {
        TestConfig {
            port: self.get_unique_port(),
            collection_name: self.get_unique_collection_name(),
            binary_path: self.resolve_binary_path(),
            docs_path: self.resolve_fixtures_path(),
        }
    }
    
    /// Resolve the test fixtures path
    fn resolve_fixtures_path(&self) -> String {
        std::fs::canonicalize("tests/fixtures")
            .expect("Failed to resolve absolute path to test fixtures")
            .to_str()
            .unwrap()
            .to_string()
    }
}

impl Default for TestUtils {
    fn default() -> Self {
        Self::new()
    }
}

/// Test configuration for a single test
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub port: u16,
    pub collection_name: String,
    pub binary_path: String,
    pub docs_path: String,
}

impl TestConfig {
    /// Get the base URL for this test server
    pub fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
    
    /// Get the health check URL
    pub fn health_url(&self) -> String {
        format!("{}/health", self.base_url())
    }
    
    /// Get the index API URL
    pub fn index_url(&self) -> String {
        format!("{}/api/index", self.base_url())
    }
    
    /// Get the search API URL
    pub fn search_url(&self) -> String {
        format!("{}/api/search", self.base_url())
    }
}

/// Common test assertions and helpers
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a server response is successful
    pub fn assert_success_response(response: &reqwest::Response, context: &str) {
        assert!(
            response.status().is_success(),
            "{} failed with status: {}",
            context,
            response.status()
        );
    }
    
    /// Assert that a search response contains expected results
    pub fn assert_search_results_not_empty(response_body: &serde_json::Value, context: &str) {
        let results = response_body
            .get("results")
            .expect(&format!("{}: Response should have 'results' field", context))
            .as_array()
            .expect(&format!("{}: 'results' should be an array", context));
            
        assert!(
            !results.is_empty(),
            "{}: Search should return at least one result",
            context
        );
    }
    
    /// Assert that search results meet quality thresholds
    pub fn assert_search_quality(response_body: &serde_json::Value, min_score: f64, context: &str) {
        let results = response_body
            .get("results")
            .expect(&format!("{}: Response should have 'results' field", context))
            .as_array()
            .expect(&format!("{}: 'results' should be an array", context));
            
        for (i, result) in results.iter().enumerate() {
            let score = result
                .get("score")
                .expect(&format!("{}: Result {} should have 'score' field", context, i))
                .as_f64()
                .expect(&format!("{}: Score should be a number", context));
                
            assert!(
                score >= min_score,
                "{}: Result {} score {} is below minimum threshold {}",
                context,
                i,
                score,
                min_score
            );
        }
    }
}

/// Lifecycle management for test servers
pub struct TestServerManager {
    servers: Vec<Child>,
}

impl TestServerManager {
    /// Create a new server manager
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
        }
    }
    
    /// Start a server and register it for cleanup
    pub fn start_managed_server(&mut self, test_utils: &TestUtils, config: &TestConfig) -> Result<(), Box<dyn std::error::Error>> {
        let child = test_utils.start_test_server(config.port, &config.docs_path)?;
        self.servers.push(child);
        Ok(())
    }
    
    /// Stop all managed servers
    pub fn stop_all(&mut self) {
        for mut server in self.servers.drain(..) {
            let _ = server.kill();
            let _ = server.wait();
        }
    }
}

impl Drop for TestServerManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_utils_creates_unique_values() {
        let utils = TestUtils::new();
        
        let port1 = utils.get_unique_port();
        let port2 = utils.get_unique_port();
        assert_ne!(port1, port2);
        
        let collection1 = utils.get_unique_collection_name();
        let collection2 = utils.get_unique_collection_name();
        assert_ne!(collection1, collection2);
    }
    
    #[test]
    fn test_config_urls() {
        let config = TestConfig {
            port: 9999,
            collection_name: "test_collection".to_string(),
            binary_path: "test_binary".to_string(),
            docs_path: "test_docs".to_string(),
        };
        
        assert_eq!(config.base_url(), "http://localhost:9999");
        assert_eq!(config.health_url(), "http://localhost:9999/health");
        assert_eq!(config.index_url(), "http://localhost:9999/api/index");
        assert_eq!(config.search_url(), "http://localhost:9999/api/search");
    }
}
