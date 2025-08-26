/// Load Testing Metrics Collection
///
/// Comprehensive metrics collection and analysis for load testing scenarios,
/// enabling detailed performance validation and optimization regression detection.
use std::collections::HashMap;
use std::time::{Duration, Instant};
use sysinfo::System;

/// Comprehensive metrics collected during load testing
#[derive(Debug, Clone)]
pub struct LoadTestMetrics {
    /// Test execution metadata
    pub test_name: String,
    pub start_time: Instant,
    pub duration: Duration,
    pub total_requests: u64,

    /// Performance metrics
    pub performance: PerformanceMetrics,

    /// Memory optimization validation
    pub memory: MemoryMetrics,

    /// Error analysis
    pub errors: ErrorMetrics,

    /// Per-scenario breakdown
    pub scenarios: HashMap<String, ScenarioMetrics>,
}

/// Core performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Response time statistics
    pub response_times: ResponseTimeStats,

    /// Throughput measurements
    pub throughput: ThroughputStats,

    /// Concurrency handling
    pub concurrency: ConcurrencyStats,

    /// Rate limiting effectiveness
    pub rate_limiting: Option<RateLimitStats>,
}

/// Response time statistical analysis
#[derive(Debug, Clone)]
pub struct ResponseTimeStats {
    pub min: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub median: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub std_dev: Duration,
}

/// Throughput analysis
#[derive(Debug, Clone)]
pub struct ThroughputStats {
    pub requests_per_second: f64,
    pub peak_rps: f64,
    pub sustained_rps: f64,
    pub rps_std_dev: f64,
}

/// Concurrency handling metrics
#[derive(Debug, Clone)]
pub struct ConcurrencyStats {
    pub max_concurrent: usize,
    pub avg_concurrent: f64,
    pub queue_depth_max: usize,
    pub queue_depth_avg: f64,
    pub connection_reuse_rate: f64,
}

/// Rate limiting validation
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    pub target_rate: u64,
    pub actual_rate: f64,
    pub rate_compliance: f64, // Percentage within target
    pub rejected_requests: u64,
}

/// Memory optimization validation metrics
#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    /// Process memory usage
    pub process_memory: ProcessMemoryStats,

    /// Vector pool effectiveness
    pub vector_pool: Option<VectorPoolStats>,

    /// String interning effectiveness
    pub string_intern: Option<StringInternStats>,

    /// Cache performance
    pub cache_performance: Option<CacheStats>,

    /// Memory leak detection
    pub leak_detection: MemoryLeakStats,
}

/// Process-level memory statistics
#[derive(Debug, Clone)]
pub struct ProcessMemoryStats {
    pub baseline_mb: f64,
    pub peak_mb: f64,
    pub final_mb: f64,
    pub max_growth_mb: f64,
    pub avg_mb: f64,
    pub memory_timeline: Vec<(Duration, f64)>,
}

/// Vector pool performance metrics
#[derive(Debug, Clone)]
pub struct VectorPoolStats {
    pub pool_hits: u64,
    pub pool_misses: u64,
    pub hit_rate: f64,
    pub avg_pool_size: f64,
    pub max_pool_size: usize,
    pub memory_saved_mb: f64,
}

/// String interning effectiveness
#[derive(Debug, Clone)]
pub struct StringInternStats {
    pub total_strings: u64,
    pub unique_strings: u64,
    pub deduplication_rate: f64,
    pub memory_saved_mb: f64,
    pub lookup_avg_time: Duration,
}

/// Cache performance analysis
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_rate: f64,
    pub evictions: u64,
    pub memory_usage_mb: f64,
    pub avg_lookup_time: Duration,
}

/// Memory leak detection results
#[derive(Debug, Clone)]
pub struct MemoryLeakStats {
    pub suspected_leak: bool,
    pub growth_rate_mb_per_sec: f64,
    pub stability_score: f64, // 0.0 = unstable, 1.0 = stable
    pub final_vs_baseline_ratio: f64,
}

/// Error analysis and categorization
#[derive(Debug, Clone)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub error_rate: f64,
    pub errors_by_type: HashMap<String, u64>,
    pub timeout_errors: u64,
    pub memory_errors: u64,
    pub connection_errors: u64,
    pub first_error_time: Option<Duration>,
}

/// Per-scenario performance breakdown
#[derive(Debug, Clone)]
pub struct ScenarioMetrics {
    pub scenario_name: String,
    pub requests: u64,
    pub success_rate: f64,
    pub avg_response_time: Duration,
    pub memory_efficiency: f64,
    pub specific_metrics: HashMap<String, f64>,
}

/// Real-time metrics collector during load testing
pub struct MetricsCollector {
    start_time: Instant,
    response_times: Vec<Duration>,
    error_counts: HashMap<String, u64>,
    memory_samples: Vec<(Duration, f64)>,
    scenario_data: HashMap<String, ScenarioData>,
    system: System,
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        MetricsCollector {
            start_time: self.start_time,
            response_times: self.response_times.clone(),
            error_counts: self.error_counts.clone(),
            memory_samples: self.memory_samples.clone(),
            scenario_data: self.scenario_data.clone(),
            system: System::new_all(),
        }
    }
    // Removed extra closing brace
}

/// Internal scenario tracking data
#[derive(Debug, Default, Clone)]
struct ScenarioData {
    requests: u64,
    successes: u64,
    response_times: Vec<Duration>,
    memory_usage: Vec<f64>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            response_times: Vec::new(),
            error_counts: HashMap::new(),
            memory_samples: Vec::new(),
            scenario_data: HashMap::new(),
            system: System::new_all(),
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self, scenario: &str, response_time: Duration) {
        self.response_times.push(response_time);

        let data = self.scenario_data.entry(scenario.to_string()).or_default();
        data.requests += 1;
        data.successes += 1;
        data.response_times.push(response_time);
    }

    /// Record an error
    pub fn record_error(&mut self, scenario: &str, error_type: &str) {
        *self.error_counts.entry(error_type.to_string()).or_insert(0) += 1;

        let data = self.scenario_data.entry(scenario.to_string()).or_default();
        data.requests += 1;
    }

    /// Sample current memory usage
    pub fn sample_memory(&mut self) {
        self.system.refresh_memory();
        self.system.refresh_processes();

        let elapsed = self.start_time.elapsed();

        if let Some(process) = self.system.processes().values().next() {
            let memory_mb = process.memory() as f64 / 1024.0 / 1024.0;
            self.memory_samples.push((elapsed, memory_mb));

            // Update scenario memory tracking
            for data in self.scenario_data.values_mut() {
                data.memory_usage.push(memory_mb);
            }
        }
    }

    /// Generate final metrics report
    pub fn finalize(self, test_name: String) -> LoadTestMetrics {
        let duration = self.start_time.elapsed();
        let total_requests = self.response_times.len() as u64;

        LoadTestMetrics {
            test_name,
            start_time: self.start_time,
            duration,
            total_requests,
            performance: self.calculate_performance_metrics(),
            memory: self.calculate_memory_metrics(),
            errors: self.calculate_error_metrics(total_requests),
            scenarios: self.calculate_scenario_metrics(),
        }
    }

    fn calculate_performance_metrics(&self) -> PerformanceMetrics {
        let response_times = self.calculate_response_time_stats();
        let throughput = self.calculate_throughput_stats();
        let concurrency = self.calculate_concurrency_stats();

        PerformanceMetrics {
            response_times,
            throughput,
            concurrency,
            rate_limiting: None, // TODO: Implement if rate limiting is enabled
        }
    }

    fn calculate_response_time_stats(&self) -> ResponseTimeStats {
        if self.response_times.is_empty() {
            return ResponseTimeStats {
                min: Duration::ZERO,
                max: Duration::ZERO,
                mean: Duration::ZERO,
                median: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                std_dev: Duration::ZERO,
            };
        }

        let mut sorted_times = self.response_times.clone();
        sorted_times.sort();

        let min = sorted_times[0];
        let max = sorted_times[sorted_times.len() - 1];

        let sum: Duration = sorted_times.iter().sum();
        let mean = sum / sorted_times.len() as u32;

        let median_idx = sorted_times.len() / 2;
        let median = sorted_times[median_idx];

        let p95_idx = (sorted_times.len() as f64 * 0.95) as usize;
        let p95 = sorted_times[p95_idx.min(sorted_times.len() - 1)];

        let p99_idx = (sorted_times.len() as f64 * 0.99) as usize;
        let p99 = sorted_times[p99_idx.min(sorted_times.len() - 1)];

        // Calculate standard deviation
        let mean_secs = mean.as_secs_f64();
        let variance: f64 = sorted_times
            .iter()
            .map(|&time| {
                let diff = time.as_secs_f64() - mean_secs;
                diff * diff
            })
            .sum::<f64>()
            / sorted_times.len() as f64;
        let std_dev = Duration::from_secs_f64(variance.sqrt());

        ResponseTimeStats {
            min,
            max,
            mean,
            median,
            p95,
            p99,
            std_dev,
        }
    }

    fn calculate_throughput_stats(&self) -> ThroughputStats {
        let duration_secs = self.start_time.elapsed().as_secs_f64();
        let requests_per_second = self.response_times.len() as f64 / duration_secs;

        ThroughputStats {
            requests_per_second,
            peak_rps: requests_per_second, // TODO: Calculate actual peak
            sustained_rps: requests_per_second,
            rps_std_dev: 0.0, // TODO: Calculate RPS variance
        }
    }

    fn calculate_concurrency_stats(&self) -> ConcurrencyStats {
        // Simplified concurrency stats - would need more detailed tracking in practice
        ConcurrencyStats {
            max_concurrent: 1, // TODO: Track actual concurrency
            avg_concurrent: 1.0,
            queue_depth_max: 0,
            queue_depth_avg: 0.0,
            connection_reuse_rate: 1.0,
        }
    }

    fn calculate_memory_metrics(&self) -> MemoryMetrics {
        let process_memory = self.calculate_process_memory_stats();
        let leak_detection = self.calculate_memory_leak_stats(&process_memory);

        MemoryMetrics {
            process_memory,
            vector_pool: None,       // TODO: Integrate with vector pool metrics
            string_intern: None,     // TODO: Integrate with string intern metrics
            cache_performance: None, // TODO: Integrate with cache metrics
            leak_detection,
        }
    }

    fn calculate_process_memory_stats(&self) -> ProcessMemoryStats {
        if self.memory_samples.is_empty() {
            return ProcessMemoryStats {
                baseline_mb: 0.0,
                peak_mb: 0.0,
                final_mb: 0.0,
                max_growth_mb: 0.0,
                avg_mb: 0.0,
                memory_timeline: Vec::new(),
            };
        }

        let baseline_mb = self.memory_samples[0].1;
        let peak_mb = self
            .memory_samples
            .iter()
            .map(|(_, mem)| *mem)
            .fold(0.0, f64::max);
        let final_mb = self.memory_samples.last().unwrap().1;
        let max_growth_mb = peak_mb - baseline_mb;
        let avg_mb = self.memory_samples.iter().map(|(_, mem)| *mem).sum::<f64>()
            / self.memory_samples.len() as f64;

        ProcessMemoryStats {
            baseline_mb,
            peak_mb,
            final_mb,
            max_growth_mb,
            avg_mb,
            memory_timeline: self.memory_samples.clone(),
        }
    }

    fn calculate_memory_leak_stats(&self, process_memory: &ProcessMemoryStats) -> MemoryLeakStats {
        let growth_rate = if self.start_time.elapsed().as_secs_f64() > 0.0 {
            (process_memory.final_mb - process_memory.baseline_mb)
                / self.start_time.elapsed().as_secs_f64()
        } else {
            0.0
        };

        let suspected_leak = growth_rate > 1.0; // More than 1MB/sec growth

        let stability_score = if process_memory.max_growth_mb > 0.0 {
            1.0 - (growth_rate.abs() / process_memory.max_growth_mb).min(1.0)
        } else {
            1.0
        };

        let final_vs_baseline_ratio = if process_memory.baseline_mb > 0.0 {
            process_memory.final_mb / process_memory.baseline_mb
        } else {
            1.0
        };

        MemoryLeakStats {
            suspected_leak,
            growth_rate_mb_per_sec: growth_rate,
            stability_score,
            final_vs_baseline_ratio,
        }
    }

    fn calculate_error_metrics(&self, total_requests: u64) -> ErrorMetrics {
        let total_errors: u64 = self.error_counts.values().sum();
        let error_rate = if total_requests > 0 {
            total_errors as f64 / total_requests as f64
        } else {
            0.0
        };

        ErrorMetrics {
            total_errors,
            error_rate,
            errors_by_type: self.error_counts.clone(),
            timeout_errors: *self.error_counts.get("timeout").unwrap_or(&0),
            memory_errors: *self.error_counts.get("memory").unwrap_or(&0),
            connection_errors: *self.error_counts.get("connection").unwrap_or(&0),
            first_error_time: None, // TODO: Track first error timing
        }
    }

    fn calculate_scenario_metrics(&self) -> HashMap<String, ScenarioMetrics> {
        self.scenario_data
            .iter()
            .map(|(name, data)| {
                let success_rate = if data.requests > 0 {
                    data.successes as f64 / data.requests as f64
                } else {
                    0.0
                };

                let avg_response_time = if !data.response_times.is_empty() {
                    data.response_times.iter().sum::<Duration>() / data.response_times.len() as u32
                } else {
                    Duration::ZERO
                };

                let memory_efficiency = if !data.memory_usage.is_empty() {
                    let avg_memory =
                        data.memory_usage.iter().sum::<f64>() / data.memory_usage.len() as f64;
                    1.0 / (1.0 + avg_memory / 100.0) // Normalize memory efficiency
                } else {
                    1.0
                };

                (
                    name.clone(),
                    ScenarioMetrics {
                        scenario_name: name.clone(),
                        requests: data.requests,
                        success_rate,
                        avg_response_time,
                        memory_efficiency,
                        specific_metrics: HashMap::new(),
                    },
                )
            })
            .collect()
    }
}

/// Generate a comprehensive metrics summary report
pub fn generate_summary_report(metrics: &LoadTestMetrics) -> String {
    format!(
        r#"
=== Load Test Results: {} ===

Duration: {:.2}s
Total Requests: {}
Success Rate: {:.2}%

Performance:
  Mean Response Time: {:.2}ms
  95th Percentile: {:.2}ms
  99th Percentile: {:.2}ms
  Throughput: {:.2} req/s

Memory:
  Baseline: {:.1} MB
  Peak: {:.1} MB
  Final: {:.1} MB
  Memory Leak Suspected: {}
  Stability Score: {:.2}

Errors:
  Total: {} ({:.2}%)
  Timeouts: {}
  Memory Errors: {}
  Connection Errors: {}

Scenarios:
{}
        "#,
        metrics.test_name,
        metrics.duration.as_secs_f64(),
        metrics.total_requests,
        (1.0 - metrics.errors.error_rate) * 100.0,
        metrics.performance.response_times.mean.as_millis(),
        metrics.performance.response_times.p95.as_millis(),
        metrics.performance.response_times.p99.as_millis(),
        metrics.performance.throughput.requests_per_second,
        metrics.memory.process_memory.baseline_mb,
        metrics.memory.process_memory.peak_mb,
        metrics.memory.process_memory.final_mb,
        if metrics.memory.leak_detection.suspected_leak {
            "YES"
        } else {
            "NO"
        },
        metrics.memory.leak_detection.stability_score,
        metrics.errors.total_errors,
        metrics.errors.error_rate * 100.0,
        metrics.errors.timeout_errors,
        metrics.errors.memory_errors,
        metrics.errors.connection_errors,
        metrics
            .scenarios
            .iter()
            .map(|(name, scenario)| format!(
                "  {}: {} requests, {:.1}% success, {:.2}ms avg",
                name,
                scenario.requests,
                scenario.success_rate * 100.0,
                scenario.avg_response_time.as_millis()
            ))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
