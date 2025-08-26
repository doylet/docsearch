/// Load Testing Infrastructure
/// 
/// Provides comprehensive load testing capabilities for the doc-indexer service,
/// enabling performance validation under realistic production workloads.
/// 
/// Key Features:
/// - Concurrent request simulation
/// - Memory optimization validation
/// - Performance regression detection
/// - Production workload scenarios

pub mod scenario;
pub mod metrics;
pub mod runner;

pub use scenario::{LoadTestScenario, ScenarioConfig};
pub use metrics::{LoadTestMetrics, PerformanceMetrics};
pub use runner::{LoadTestRunner, LoadTestResult};

use std::time::Duration;

/// Configuration for load testing execution
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// Number of concurrent connections
    pub concurrency: usize,
    
    /// Duration of the test
    pub duration: Duration,
    
    /// Rate limiting (requests per second)
    pub rate_limit: Option<u64>,
    
    /// Memory optimization validation enabled
    pub validate_optimizations: bool,
    
    /// Collect detailed performance metrics
    pub detailed_metrics: bool,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrency: std::env::var("LOAD_TEST_CONCURRENCY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            duration: Duration::from_secs(
                std::env::var("LOAD_TEST_DURATION")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60)
            ),
            rate_limit: std::env::var("LOAD_TEST_RATE_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok()),
            validate_optimizations: std::env::var("LOAD_TEST_VALIDATE_OPTIMIZATIONS")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            detailed_metrics: std::env::var("LOAD_TEST_DETAILED_METRICS")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }
}
