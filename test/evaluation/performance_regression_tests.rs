//! Performance regression testing framework for hybrid search
//!
//! This module implements comprehensive performance regression testing to ensure
//! that hybrid search improvements don't negatively impact system performance.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::timeout;

use zero_latency_core::Result;
use zero_latency_search::models::SearchResult;

/// Performance benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmarkConfig {
    /// Number of warmup iterations before measurement
    pub warmup_iterations: usize,
    /// Number of measurement iterations
    pub measurement_iterations: usize,
    /// Timeout for individual search operations (ms)
    pub search_timeout_ms: u64,
    /// Concurrent request levels to test
    pub concurrency_levels: Vec<usize>,
    /// Test different query complexities
    pub query_complexity_levels: Vec<QueryComplexity>,
    /// Memory monitoring enabled
    pub enable_memory_monitoring: bool,
    /// CPU monitoring enabled  
    pub enable_cpu_monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryComplexity {
    Simple,      // Single keyword
    Medium,      // Multiple keywords
    Complex,     // Long phrases with filters
    VeryComplex, // Multiple expansions with advanced features
}

impl Default for PerformanceBenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 100,
            measurement_iterations: 1000,
            search_timeout_ms: 1000,
            concurrency_levels: vec![1, 5, 10, 20, 50],
            query_complexity_levels: vec![
                QueryComplexity::Simple,
                QueryComplexity::Medium,
                QueryComplexity::Complex,
            ],
            enable_memory_monitoring: true,
            enable_cpu_monitoring: true,
        }
    }
}

/// Individual performance measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasurement {
    /// Response time in milliseconds
    pub response_time_ms: f64,
    /// Memory usage in MB (if monitored)
    pub memory_usage_mb: Option<f64>,
    /// CPU usage percentage (if monitored)
    pub cpu_usage_percent: Option<f64>,
    /// Success indicator
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Timestamp of measurement
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Performance statistics for a test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// Scenario name
    pub scenario: String,
    /// Total samples collected
    pub sample_count: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Response time statistics
    pub response_time_stats: StatisticalSummary,
    /// Memory usage statistics (if available)
    pub memory_stats: Option<StatisticalSummary>,
    /// CPU usage statistics (if available)
    pub cpu_stats: Option<StatisticalSummary>,
    /// Throughput (requests per second)
    pub throughput_rps: f64,
}

/// Statistical summary for performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSummary {
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Mean (average)
    pub mean: f64,
    /// Median (50th percentile)
    pub median: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// 90th percentile
    pub p90: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
}

/// Comparison between baseline and test performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    /// Scenario being compared
    pub scenario: String,
    /// Baseline performance statistics
    pub baseline_stats: PerformanceStats,
    /// Test performance statistics
    pub test_stats: PerformanceStats,
    /// Performance regression analysis
    pub regression_analysis: RegressionAnalysis,
}

/// Regression analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    /// Is there a significant performance regression?
    pub has_regression: bool,
    /// Percentage change in P95 latency
    pub p95_latency_change_percent: f64,
    /// Percentage change in throughput
    pub throughput_change_percent: f64,
    /// Percentage change in success rate
    pub success_rate_change_percent: f64,
    /// Percentage change in memory usage
    pub memory_usage_change_percent: Option<f64>,
    /// Severity of regression (if any)
    pub regression_severity: RegressionSeverity,
    /// Detailed findings
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    None,
    Minor,      // < 5% degradation
    Moderate,   // 5-15% degradation
    Major,      // 15-30% degradation
    Critical,   // > 30% degradation
}

/// Complete performance regression test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionTestReport {
    /// Test configuration
    pub config: PerformanceBenchmarkConfig,
    /// Test execution timestamp
    pub timestamp: String,
    /// System information
    pub system_info: SystemInfo,
    /// Per-scenario performance comparisons
    pub scenario_comparisons: Vec<PerformanceComparison>,
    /// Overall regression summary
    pub overall_summary: OverallRegressionSummary,
    /// Pass/fail result
    pub test_result: RegressionTestResult,
}

/// System information for test context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,
    /// CPU information
    pub cpu_info: String,
    /// Total memory (GB)
    pub total_memory_gb: f64,
    /// Available memory at test start (GB)
    pub available_memory_gb: f64,
    /// Rust version
    pub rust_version: String,
    /// Test environment
    pub environment: String,
}

/// Overall regression test summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallRegressionSummary {
    /// Total scenarios tested
    pub total_scenarios: usize,
    /// Scenarios with regressions
    pub scenarios_with_regression: usize,
    /// Worst regression severity observed
    pub worst_regression_severity: RegressionSeverity,
    /// Key performance impacts
    pub key_impacts: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionTestResult {
    Pass,
    FailMinor(String),
    FailMajor(String),
    FailCritical(String),
}

/// Performance regression test runner
pub struct PerformanceRegressionTester {
    config: PerformanceBenchmarkConfig,
}

impl PerformanceRegressionTester {
    pub fn new(config: PerformanceBenchmarkConfig) -> Self {
        Self { config }
    }
    
    /// Run comprehensive performance regression tests
    pub async fn run_regression_tests(&self) -> Result<RegressionTestReport> {
        println!("🚀 Starting performance regression testing...");
        
        let system_info = self.collect_system_info().await?;
        println!("💻 System: {} with {:.1}GB memory", system_info.os, system_info.total_memory_gb);
        
        let mut scenario_comparisons = Vec::new();
        
        // Test different query complexity levels
        for complexity in &self.config.query_complexity_levels {
            let scenario_name = format!("query_complexity_{:?}", complexity);
            println!("🔍 Testing scenario: {}", scenario_name);
            
            let baseline_stats = self.benchmark_baseline_performance(&scenario_name, complexity).await?;
            let test_stats = self.benchmark_hybrid_performance(&scenario_name, complexity).await?;
            
            let regression_analysis = self.analyze_regression(&baseline_stats, &test_stats);
            
            scenario_comparisons.push(PerformanceComparison {
                scenario: scenario_name,
                baseline_stats,
                test_stats,
                regression_analysis,
            });
        }
        
        // Test different concurrency levels
        for &concurrency in &self.config.concurrency_levels {
            let scenario_name = format!("concurrency_{}", concurrency);
            println!("⚡ Testing concurrency: {} concurrent requests", concurrency);
            
            let baseline_stats = self.benchmark_baseline_concurrency(concurrency).await?;
            let test_stats = self.benchmark_hybrid_concurrency(concurrency).await?;
            
            let regression_analysis = self.analyze_regression(&baseline_stats, &test_stats);
            
            scenario_comparisons.push(PerformanceComparison {
                scenario: scenario_name,
                baseline_stats,
                test_stats,
                regression_analysis,
            });
        }
        
        let overall_summary = self.generate_overall_summary(&scenario_comparisons);
        let test_result = self.determine_test_result(&overall_summary);
        
        let report = RegressionTestReport {
            config: self.config.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            system_info,
            scenario_comparisons,
            overall_summary,
            test_result,
        };
        
        println!("✅ Performance regression testing completed!");
        Ok(report)
    }
    
    /// Collect system information
    async fn collect_system_info(&self) -> Result<SystemInfo> {
        Ok(SystemInfo {
            os: std::env::consts::OS.to_string(),
            cpu_info: "Unknown CPU".to_string(), // TODO: Implement CPU detection
            total_memory_gb: 16.0, // TODO: Implement memory detection
            available_memory_gb: 12.0, // TODO: Implement available memory detection
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "test".to_string(),
        })
    }
    
    /// Benchmark baseline (vector-only) performance
    async fn benchmark_baseline_performance(
        &self,
        scenario: &str,
        complexity: &QueryComplexity,
    ) -> Result<PerformanceStats> {
        println!("📊 Benchmarking baseline performance for {}", scenario);
        
        // Generate test queries based on complexity
        let test_queries = self.generate_test_queries(complexity);
        
        // Warmup phase
        for _ in 0..self.config.warmup_iterations {
            let query = &test_queries[fastrand::usize(..test_queries.len())];
            let _ = self.execute_baseline_search(query).await;
        }
        
        // Measurement phase
        let mut measurements = Vec::new();
        let start_time = Instant::now();
        
        for _ in 0..self.config.measurement_iterations {
            let query = &test_queries[fastrand::usize(..test_queries.len())];
            let measurement = self.measure_baseline_search(query).await;
            measurements.push(measurement);
        }
        
        let total_duration = start_time.elapsed();
        let stats = self.calculate_performance_stats(scenario, &measurements, total_duration);
        
        println!("  ✅ Baseline P95: {:.1}ms, Throughput: {:.1} RPS", 
            stats.response_time_stats.p95, stats.throughput_rps);
        
        Ok(stats)
    }
    
    /// Benchmark hybrid search performance
    async fn benchmark_hybrid_performance(
        &self,
        scenario: &str,
        complexity: &QueryComplexity,
    ) -> Result<PerformanceStats> {
        println!("📊 Benchmarking hybrid performance for {}", scenario);
        
        // Generate test queries based on complexity
        let test_queries = self.generate_test_queries(complexity);
        
        // Warmup phase
        for _ in 0..self.config.warmup_iterations {
            let query = &test_queries[fastrand::usize(..test_queries.len())];
            let _ = self.execute_hybrid_search(query).await;
        }
        
        // Measurement phase
        let mut measurements = Vec::new();
        let start_time = Instant::now();
        
        for _ in 0..self.config.measurement_iterations {
            let query = &test_queries[fastrand::usize(..test_queries.len())];
            let measurement = self.measure_hybrid_search(query).await;
            measurements.push(measurement);
        }
        
        let total_duration = start_time.elapsed();
        let stats = self.calculate_performance_stats(scenario, &measurements, total_duration);
        
        println!("  ✅ Hybrid P95: {:.1}ms, Throughput: {:.1} RPS", 
            stats.response_time_stats.p95, stats.throughput_rps);
        
        Ok(stats)
    }
    
    /// Benchmark baseline performance under concurrency
    async fn benchmark_baseline_concurrency(&self, _concurrency: usize) -> Result<PerformanceStats> {
        // TODO: Implement concurrent baseline benchmarking
        todo!("Implement concurrent baseline benchmarking")
    }
    
    /// Benchmark hybrid performance under concurrency
    async fn benchmark_hybrid_concurrency(&self, _concurrency: usize) -> Result<PerformanceStats> {
        // TODO: Implement concurrent hybrid benchmarking
        todo!("Implement concurrent hybrid benchmarking")
    }
    
    /// Generate test queries based on complexity level
    fn generate_test_queries(&self, complexity: &QueryComplexity) -> Vec<String> {
        match complexity {
            QueryComplexity::Simple => vec![
                "rust".to_string(),
                "vector".to_string(),
                "search".to_string(),
                "database".to_string(),
                "api".to_string(),
            ],
            QueryComplexity::Medium => vec![
                "rust programming language".to_string(),
                "vector database search".to_string(),
                "api endpoint documentation".to_string(),
                "search query optimization".to_string(),
                "hybrid retrieval system".to_string(),
            ],
            QueryComplexity::Complex => vec![
                "rust programming language syntax and best practices".to_string(),
                "vector database search with semantic similarity matching".to_string(),
                "hybrid search combining BM25 and vector embeddings".to_string(),
                "query expansion techniques for improved recall".to_string(),
                "performance optimization for large scale search systems".to_string(),
            ],
            QueryComplexity::VeryComplex => vec![
                "advanced rust programming patterns with async concurrent processing for search systems".to_string(),
                "implementing hybrid retrieval systems with BM25 full text search and vector embeddings".to_string(),
                "query expansion and result deduplication strategies for multi-modal search architectures".to_string(),
            ],
        }
    }
    
    /// Execute baseline search (vector-only)
    async fn execute_baseline_search(&self, _query: &str) -> Result<Vec<SearchResult>> {
        // TODO: Implement actual baseline search execution
        tokio::time::sleep(Duration::from_millis(50)).await; // Simulate search
        Ok(Vec::new())
    }
    
    /// Execute hybrid search
    async fn execute_hybrid_search(&self, _query: &str) -> Result<Vec<SearchResult>> {
        // TODO: Implement actual hybrid search execution
        tokio::time::sleep(Duration::from_millis(75)).await; // Simulate search
        Ok(Vec::new())
    }
    
    /// Measure baseline search performance
    async fn measure_baseline_search(&self, query: &str) -> PerformanceMeasurement {
        let start = Instant::now();
        let start_memory = self.get_memory_usage();
        let start_cpu = self.get_cpu_usage();
        
        let result = timeout(
            Duration::from_millis(self.config.search_timeout_ms),
            self.execute_baseline_search(query),
        ).await;
        
        let duration = start.elapsed();
        let end_memory = self.get_memory_usage();
        let end_cpu = self.get_cpu_usage();
        
        match result {
            Ok(Ok(_)) => PerformanceMeasurement {
                response_time_ms: duration.as_secs_f64() * 1000.0,
                memory_usage_mb: if self.config.enable_memory_monitoring {
                    Some(end_memory - start_memory)
                } else {
                    None
                },
                cpu_usage_percent: if self.config.enable_cpu_monitoring {
                    Some(end_cpu - start_cpu)
                } else {
                    None
                },
                success: true,
                error_message: None,
                timestamp: chrono::Utc::now(),
            },
            Ok(Err(e)) => PerformanceMeasurement {
                response_time_ms: duration.as_secs_f64() * 1000.0,
                memory_usage_mb: None,
                cpu_usage_percent: None,
                success: false,
                error_message: Some(e.to_string()),
                timestamp: chrono::Utc::now(),
            },
            Err(_) => PerformanceMeasurement {
                response_time_ms: self.config.search_timeout_ms as f64,
                memory_usage_mb: None,
                cpu_usage_percent: None,
                success: false,
                error_message: Some("Timeout".to_string()),
                timestamp: chrono::Utc::now(),
            },
        }
    }
    
    /// Measure hybrid search performance
    async fn measure_hybrid_search(&self, query: &str) -> PerformanceMeasurement {
        let start = Instant::now();
        let start_memory = self.get_memory_usage();
        let start_cpu = self.get_cpu_usage();
        
        let result = timeout(
            Duration::from_millis(self.config.search_timeout_ms),
            self.execute_hybrid_search(query),
        ).await;
        
        let duration = start.elapsed();
        let end_memory = self.get_memory_usage();
        let end_cpu = self.get_cpu_usage();
        
        match result {
            Ok(Ok(_)) => PerformanceMeasurement {
                response_time_ms: duration.as_secs_f64() * 1000.0,
                memory_usage_mb: if self.config.enable_memory_monitoring {
                    Some(end_memory - start_memory)
                } else {
                    None
                },
                cpu_usage_percent: if self.config.enable_cpu_monitoring {
                    Some(end_cpu - start_cpu)
                } else {
                    None
                },
                success: true,
                error_message: None,
                timestamp: chrono::Utc::now(),
            },
            Ok(Err(e)) => PerformanceMeasurement {
                response_time_ms: duration.as_secs_f64() * 1000.0,
                memory_usage_mb: None,
                cpu_usage_percent: None,
                success: false,
                error_message: Some(e.to_string()),
                timestamp: chrono::Utc::now(),
            },
            Err(_) => PerformanceMeasurement {
                response_time_ms: self.config.search_timeout_ms as f64,
                memory_usage_mb: None,
                cpu_usage_percent: None,
                success: false,
                error_message: Some("Timeout".to_string()),
                timestamp: chrono::Utc::now(),
            },
        }
    }
    
    /// Get current memory usage (stub implementation)
    fn get_memory_usage(&self) -> f64 {
        // TODO: Implement actual memory usage monitoring
        0.0
    }
    
    /// Get current CPU usage (stub implementation)
    fn get_cpu_usage(&self) -> f64 {
        // TODO: Implement actual CPU usage monitoring
        0.0
    }
    
    /// Calculate performance statistics from measurements
    fn calculate_performance_stats(
        &self,
        scenario: &str,
        measurements: &[PerformanceMeasurement],
        total_duration: Duration,
    ) -> PerformanceStats {
        let successful_measurements: Vec<_> = measurements
            .iter()
            .filter(|m| m.success)
            .collect();
        
        let success_rate = successful_measurements.len() as f64 / measurements.len() as f64;
        
        let response_times: Vec<f64> = successful_measurements
            .iter()
            .map(|m| m.response_time_ms)
            .collect();
        
        let response_time_stats = self.calculate_statistical_summary(&response_times);
        
        let memory_usage: Vec<f64> = successful_measurements
            .iter()
            .filter_map(|m| m.memory_usage_mb)
            .collect();
        
        let memory_stats = if !memory_usage.is_empty() {
            Some(self.calculate_statistical_summary(&memory_usage))
        } else {
            None
        };
        
        let cpu_usage: Vec<f64> = successful_measurements
            .iter()
            .filter_map(|m| m.cpu_usage_percent)
            .collect();
        
        let cpu_stats = if !cpu_usage.is_empty() {
            Some(self.calculate_statistical_summary(&cpu_usage))
        } else {
            None
        };
        
        let throughput_rps = successful_measurements.len() as f64 / total_duration.as_secs_f64();
        
        PerformanceStats {
            scenario: scenario.to_string(),
            sample_count: measurements.len(),
            success_rate,
            response_time_stats,
            memory_stats,
            cpu_stats,
            throughput_rps,
        }
    }
    
    /// Calculate statistical summary for a set of values
    fn calculate_statistical_summary(&self, values: &[f64]) -> StatisticalSummary {
        if values.is_empty() {
            return StatisticalSummary {
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                p90: 0.0,
                p95: 0.0,
                p99: 0.0,
            };
        }
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min = sorted_values[0];
        let max = sorted_values[sorted_values.len() - 1];
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        let median_idx = sorted_values.len() / 2;
        let median = if sorted_values.len() % 2 == 0 {
            (sorted_values[median_idx - 1] + sorted_values[median_idx]) / 2.0
        } else {
            sorted_values[median_idx]
        };
        
        let variance = values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        let percentile = |p: f64| {
            let idx = ((p / 100.0) * (sorted_values.len() - 1) as f64) as usize;
            sorted_values[idx.min(sorted_values.len() - 1)]
        };
        
        StatisticalSummary {
            min,
            max,
            mean,
            median,
            std_dev,
            p90: percentile(90.0),
            p95: percentile(95.0),
            p99: percentile(99.0),
        }
    }
    
    /// Analyze performance regression between baseline and test
    fn analyze_regression(
        &self,
        baseline: &PerformanceStats,
        test: &PerformanceStats,
    ) -> RegressionAnalysis {
        let p95_change = ((test.response_time_stats.p95 - baseline.response_time_stats.p95) 
            / baseline.response_time_stats.p95) * 100.0;
        
        let throughput_change = ((test.throughput_rps - baseline.throughput_rps) 
            / baseline.throughput_rps) * 100.0;
        
        let success_rate_change = ((test.success_rate - baseline.success_rate) 
            / baseline.success_rate) * 100.0;
        
        let memory_change = match (&baseline.memory_stats, &test.memory_stats) {
            (Some(baseline_mem), Some(test_mem)) => {
                Some(((test_mem.mean - baseline_mem.mean) / baseline_mem.mean) * 100.0)
            }
            _ => None,
        };
        
        // Determine if there's a regression
        let has_regression = p95_change > 5.0 || throughput_change < -5.0 || success_rate_change < -1.0;
        
        let regression_severity = if !has_regression {
            RegressionSeverity::None
        } else if p95_change > 30.0 || throughput_change < -30.0 {
            RegressionSeverity::Critical
        } else if p95_change > 15.0 || throughput_change < -15.0 {
            RegressionSeverity::Major
        } else if p95_change > 5.0 || throughput_change < -5.0 {
            RegressionSeverity::Moderate
        } else {
            RegressionSeverity::Minor
        };
        
        let mut findings = Vec::new();
        
        if p95_change > 0.0 {
            findings.push(format!("P95 latency increased by {:.1}%", p95_change));
        } else if p95_change < 0.0 {
            findings.push(format!("P95 latency improved by {:.1}%", -p95_change));
        }
        
        if throughput_change > 0.0 {
            findings.push(format!("Throughput improved by {:.1}%", throughput_change));
        } else if throughput_change < 0.0 {
            findings.push(format!("Throughput decreased by {:.1}%", -throughput_change));
        }
        
        if let Some(mem_change) = memory_change {
            if mem_change > 0.0 {
                findings.push(format!("Memory usage increased by {:.1}%", mem_change));
            } else if mem_change < 0.0 {
                findings.push(format!("Memory usage decreased by {:.1}%", -mem_change));
            }
        }
        
        RegressionAnalysis {
            has_regression,
            p95_latency_change_percent: p95_change,
            throughput_change_percent: throughput_change,
            success_rate_change_percent: success_rate_change,
            memory_usage_change_percent: memory_change,
            regression_severity,
            findings,
        }
    }
    
    /// Generate overall regression summary
    fn generate_overall_summary(&self, comparisons: &[PerformanceComparison]) -> OverallRegressionSummary {
        let total_scenarios = comparisons.len();
        let scenarios_with_regression = comparisons
            .iter()
            .filter(|comp| comp.regression_analysis.has_regression)
            .count();
        
        let worst_regression_severity = comparisons
            .iter()
            .map(|comp| &comp.regression_analysis.regression_severity)
            .max_by_key(|severity| match severity {
                RegressionSeverity::None => 0,
                RegressionSeverity::Minor => 1,
                RegressionSeverity::Moderate => 2,
                RegressionSeverity::Major => 3,
                RegressionSeverity::Critical => 4,
            })
            .cloned()
            .unwrap_or(RegressionSeverity::None);
        
        let mut key_impacts = Vec::new();
        let mut recommendations = Vec::new();
        
        if scenarios_with_regression == 0 {
            key_impacts.push("No performance regressions detected".to_string());
            recommendations.push("Proceed with deployment".to_string());
        } else {
            key_impacts.push(format!(
                "{}/{} scenarios show performance regression", 
                scenarios_with_regression, total_scenarios
            ));
            
            match worst_regression_severity {
                RegressionSeverity::Critical => {
                    recommendations.push("❌ Do not deploy - critical performance issues".to_string());
                }
                RegressionSeverity::Major => {
                    recommendations.push("⚠️ Address major performance issues before deployment".to_string());
                }
                RegressionSeverity::Moderate => {
                    recommendations.push("⚠️ Consider performance optimization before deployment".to_string());
                }
                RegressionSeverity::Minor => {
                    recommendations.push("✅ Minor regressions acceptable for deployment".to_string());
                }
                RegressionSeverity::None => {}
            }
        }
        
        OverallRegressionSummary {
            total_scenarios,
            scenarios_with_regression,
            worst_regression_severity,
            key_impacts,
            recommendations,
        }
    }
    
    /// Determine overall test result
    fn determine_test_result(&self, summary: &OverallRegressionSummary) -> RegressionTestResult {
        match summary.worst_regression_severity {
            RegressionSeverity::None | RegressionSeverity::Minor => RegressionTestResult::Pass,
            RegressionSeverity::Moderate => {
                RegressionTestResult::FailMinor("Moderate performance regression detected".to_string())
            }
            RegressionSeverity::Major => {
                RegressionTestResult::FailMajor("Major performance regression detected".to_string())
            }
            RegressionSeverity::Critical => {
                RegressionTestResult::FailCritical("Critical performance regression detected".to_string())
            }
        }
    }
}

/// Convenience function to run performance regression tests
pub async fn run_performance_regression_tests(
    config: Option<PerformanceBenchmarkConfig>,
) -> Result<RegressionTestReport> {
    let config = config.unwrap_or_default();
    let tester = PerformanceRegressionTester::new(config);
    tester.run_regression_tests().await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_regression_tester_initialization() {
        let config = PerformanceBenchmarkConfig::default();
        let tester = PerformanceRegressionTester::new(config);
        
        assert_eq!(tester.config.warmup_iterations, 100);
        assert_eq!(tester.config.measurement_iterations, 1000);
    }
    
    #[test]
    fn test_statistical_summary_calculation() {
        let config = PerformanceBenchmarkConfig::default();
        let tester = PerformanceRegressionTester::new(config);
        
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];
        let summary = tester.calculate_statistical_summary(&values);
        
        assert_eq!(summary.min, 10.0);
        assert_eq!(summary.max, 100.0);
        assert_eq!(summary.mean, 55.0);
        assert_eq!(summary.median, 55.0);
        assert!(summary.p95 >= 90.0);
    }
    
    #[test]
    fn test_regression_analysis() {
        let config = PerformanceBenchmarkConfig::default();
        let tester = PerformanceRegressionTester::new(config);
        
        let baseline_stats = PerformanceStats {
            scenario: "test".to_string(),
            sample_count: 100,
            success_rate: 1.0,
            response_time_stats: StatisticalSummary {
                min: 50.0,
                max: 200.0,
                mean: 100.0,
                median: 95.0,
                std_dev: 25.0,
                p90: 150.0,
                p95: 175.0,
                p99: 190.0,
            },
            memory_stats: None,
            cpu_stats: None,
            throughput_rps: 100.0,
        };
        
        let test_stats = PerformanceStats {
            scenario: "test".to_string(),
            sample_count: 100,
            success_rate: 1.0,
            response_time_stats: StatisticalSummary {
                min: 60.0,
                max: 250.0,
                mean: 120.0,
                median: 115.0,
                std_dev: 30.0,
                p90: 180.0,
                p95: 210.0, // 20% increase
                p99: 230.0,
            },
            memory_stats: None,
            cpu_stats: None,
            throughput_rps: 85.0, // 15% decrease
        };
        
        let analysis = tester.analyze_regression(&baseline_stats, &test_stats);
        
        assert!(analysis.has_regression);
        assert!(analysis.p95_latency_change_percent > 15.0);
        assert!(analysis.throughput_change_percent < -10.0);
        assert!(matches!(analysis.regression_severity, RegressionSeverity::Major));
    }
}
