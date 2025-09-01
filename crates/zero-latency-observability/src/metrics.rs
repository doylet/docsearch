//! Metrics collection and reporting functionality
//!
//! Provides a centralized metrics system for collecting and exposing
//! performance metrics, counters, histograms, and gauges.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Metric types supported by the observability system
#[derive(Debug, Clone)]
pub enum MetricType {
    /// Counter - monotonically increasing value
    Counter(u64),
    /// Gauge - value that can go up or down
    Gauge(f64),
    /// Histogram - distribution of values
    Histogram {
        count: u64,
        sum: f64,
        buckets: Vec<(f64, u64)>, // (upper_bound, count)
    },
    /// Timer - special histogram for duration measurements
    Timer {
        count: u64,
        sum_ms: f64,
        p50_ms: f64,
        p95_ms: f64,
        p99_ms: f64,
    },
}

/// Individual metric with metadata
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub description: String,
    pub labels: HashMap<String, String>,
    pub metric_type: MetricType,
    pub timestamp: Instant,
}

/// Central metrics registry for collecting and exposing metrics
#[derive(Debug, Clone)]
pub struct MetricsRegistry {
    metrics: Arc<Mutex<HashMap<String, Metric>>>,
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Increment a counter metric
    pub fn increment_counter(&self, name: &str, description: &str, labels: HashMap<String, String>) {
        let mut metrics = self.metrics.lock().unwrap();
        let key = self.build_key(name, &labels);
        
        match metrics.get_mut(&key) {
            Some(metric) => {
                if let MetricType::Counter(ref mut count) = metric.metric_type {
                    *count += 1;
                    metric.timestamp = Instant::now();
                }
            }
            None => {
                let metric = Metric {
                    name: name.to_string(),
                    description: description.to_string(),
                    labels,
                    metric_type: MetricType::Counter(1),
                    timestamp: Instant::now(),
                };
                metrics.insert(key, metric);
            }
        }
    }
    
    /// Add a value to a counter metric
    pub fn add_to_counter(&self, name: &str, description: &str, value: u64, labels: HashMap<String, String>) {
        let mut metrics = self.metrics.lock().unwrap();
        let key = self.build_key(name, &labels);
        
        match metrics.get_mut(&key) {
            Some(metric) => {
                if let MetricType::Counter(ref mut count) = metric.metric_type {
                    *count += value;
                    metric.timestamp = Instant::now();
                }
            }
            None => {
                let metric = Metric {
                    name: name.to_string(),
                    description: description.to_string(),
                    labels,
                    metric_type: MetricType::Counter(value),
                    timestamp: Instant::now(),
                };
                metrics.insert(key, metric);
            }
        }
    }
    
    /// Set a gauge metric value
    pub fn set_gauge(&self, name: &str, description: &str, value: f64, labels: HashMap<String, String>) {
        let mut metrics = self.metrics.lock().unwrap();
        let key = self.build_key(name, &labels);
        
        let metric = Metric {
            name: name.to_string(),
            description: description.to_string(),
            labels,
            metric_type: MetricType::Gauge(value),
            timestamp: Instant::now(),
        };
        metrics.insert(key, metric);
    }
    
    /// Record a timing measurement
    pub fn record_timing(&self, name: &str, description: &str, duration: Duration, labels: HashMap<String, String>) {
        let duration_ms = duration.as_secs_f64() * 1000.0;
        let mut metrics = self.metrics.lock().unwrap();
        let key = self.build_key(name, &labels);
        
        match metrics.get_mut(&key) {
            Some(metric) => {
                if let MetricType::Timer { ref mut count, ref mut sum_ms, .. } = metric.metric_type {
                    *count += 1;
                    *sum_ms += duration_ms;
                    metric.timestamp = Instant::now();
                    // TODO: Update percentiles with reservoir sampling
                }
            }
            None => {
                let metric = Metric {
                    name: name.to_string(),
                    description: description.to_string(),
                    labels,
                    metric_type: MetricType::Timer {
                        count: 1,
                        sum_ms: duration_ms,
                        p50_ms: duration_ms,
                        p95_ms: duration_ms,
                        p99_ms: duration_ms,
                    },
                    timestamp: Instant::now(),
                };
                metrics.insert(key, metric);
            }
        }
    }
    
    /// Get all current metrics
    pub fn get_all_metrics(&self) -> Vec<Metric> {
        let metrics = self.metrics.lock().unwrap();
        metrics.values().cloned().collect()
    }
    
    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let metrics = self.metrics.lock().unwrap();
        let mut output = String::new();
        
        for metric in metrics.values() {
            // Add metric help text
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.description));
            
            // Add metric type
            let metric_type = match metric.metric_type {
                MetricType::Counter(_) => "counter",
                MetricType::Gauge(_) => "gauge",
                MetricType::Histogram { .. } => "histogram",
                MetricType::Timer { .. } => "histogram",
            };
            output.push_str(&format!("# TYPE {} {}\n", metric.name, metric_type));
            
            // Add metric value(s)
            let labels_str = self.format_labels(&metric.labels);
            match &metric.metric_type {
                MetricType::Counter(value) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, value));
                }
                MetricType::Gauge(value) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, value));
                }
                MetricType::Timer { count, sum_ms, p50_ms, p95_ms, p99_ms } => {
                    output.push_str(&format!("{}_count{} {}\n", metric.name, labels_str, count));
                    output.push_str(&format!("{}_sum{} {}\n", metric.name, labels_str, sum_ms / 1000.0));
                    
                    let p50_labels = self.append_to_labels(&labels_str, "quantile", "0.5");
                    output.push_str(&format!("{}{} {}\n", metric.name, p50_labels, p50_ms / 1000.0));
                    
                    let p95_labels = self.append_to_labels(&labels_str, "quantile", "0.95");
                    output.push_str(&format!("{}{} {}\n", metric.name, p95_labels, p95_ms / 1000.0));
                    
                    let p99_labels = self.append_to_labels(&labels_str, "quantile", "0.99");
                    output.push_str(&format!("{}{} {}\n", metric.name, p99_labels, p99_ms / 1000.0));
                }
                MetricType::Histogram { count, sum, buckets } => {
                    output.push_str(&format!("{}_count{} {}\n", metric.name, labels_str, count));
                    output.push_str(&format!("{}_sum{} {}\n", metric.name, labels_str, sum));
                    for (upper_bound, bucket_count) in buckets {
                        let bucket_labels = self.append_to_labels(&labels_str, "le", &upper_bound.to_string());
                        output.push_str(&format!("{}{} {}\n", metric.name, bucket_labels, bucket_count));
                    }
                }
            }
            output.push('\n');
        }
        
        output
    }
    
    /// Clear all metrics (useful for testing)
    pub fn clear(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.clear();
    }
    
    // Helper methods
    
    fn build_key(&self, name: &str, labels: &HashMap<String, String>) -> String {
        let mut key = name.to_string();
        let mut sorted_labels: Vec<_> = labels.iter().collect();
        sorted_labels.sort_by_key(|(k, _)| *k);
        
        for (k, v) in sorted_labels {
            key.push_str(&format!("{}={}", k, v));
        }
        key
    }
    
    fn format_labels(&self, labels: &HashMap<String, String>) -> String {
        if labels.is_empty() {
            return String::new();
        }
        
        let mut sorted_labels: Vec<_> = labels.iter().collect();
        sorted_labels.sort_by_key(|(k, _)| *k);
        
        let label_pairs: Vec<String> = sorted_labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect();
            
        format!("{{{}}}", label_pairs.join(","))
    }
    
    fn append_to_labels(&self, existing_labels: &str, key: &str, value: &str) -> String {
        if existing_labels.is_empty() {
            format!("{}=\"{}\"", key, value)
        } else {
            // Remove the closing brace and add the new label
            let without_closing = existing_labels.trim_end_matches('}');
            format!("{},{}=\"{}\"}}", without_closing, key, value)
        }
    }
}

/// Timer utility for measuring execution time
pub struct Timer {
    start: Instant,
    registry: MetricsRegistry,
    name: String,
    description: String,
    labels: HashMap<String, String>,
}

impl Timer {
    pub fn new(registry: MetricsRegistry, name: &str, description: &str, labels: HashMap<String, String>) -> Self {
        Self {
            start: Instant::now(),
            registry,
            name: name.to_string(),
            description: description.to_string(),
            labels,
        }
    }
    
    pub fn finish(self) {
        let duration = self.start.elapsed();
        self.registry.record_timing(&self.name, &self.description, duration, self.labels);
    }
}

/// Convenience macros for common metrics operations
#[macro_export]
macro_rules! increment_counter {
    ($registry:expr, $name:expr, $description:expr) => {
        $registry.increment_counter($name, $description, std::collections::HashMap::new())
    };
    ($registry:expr, $name:expr, $description:expr, $($key:expr => $value:expr),*) => {
        {
            let mut labels = std::collections::HashMap::new();
            $(labels.insert($key.to_string(), $value.to_string());)*
            $registry.increment_counter($name, $description, labels)
        }
    };
}

#[macro_export]
macro_rules! set_gauge {
    ($registry:expr, $name:expr, $description:expr, $value:expr) => {
        $registry.set_gauge($name, $description, $value, std::collections::HashMap::new())
    };
    ($registry:expr, $name:expr, $description:expr, $value:expr, $($key:expr => $val:expr),*) => {
        {
            let mut labels = std::collections::HashMap::new();
            $(labels.insert($key.to_string(), $val.to_string());)*
            $registry.set_gauge($name, $description, $value, labels)
        }
    };
}

#[macro_export]
macro_rules! time_operation {
    ($registry:expr, $name:expr, $description:expr, $block:block) => {
        {
            let timer = Timer::new($registry.clone(), $name, $description, std::collections::HashMap::new());
            let result = $block;
            timer.finish();
            result
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_counter_metrics() {
        let registry = MetricsRegistry::new();
        
        increment_counter!(registry, "test_counter", "Test counter");
        increment_counter!(registry, "test_counter", "Test counter");
        
        let metrics = registry.get_all_metrics();
        assert_eq!(metrics.len(), 1);
        
        if let MetricType::Counter(count) = metrics[0].metric_type {
            assert_eq!(count, 2);
        } else {
            panic!("Expected counter metric");
        }
    }
    
    #[test]
    fn test_gauge_metrics() {
        let registry = MetricsRegistry::new();
        
        set_gauge!(registry, "test_gauge", "Test gauge", 42.5);
        
        let metrics = registry.get_all_metrics();
        assert_eq!(metrics.len(), 1);
        
        if let MetricType::Gauge(value) = metrics[0].metric_type {
            assert_eq!(value, 42.5);
        } else {
            panic!("Expected gauge metric");
        }
    }
    
    #[test]
    fn test_timer_metrics() {
        let registry = MetricsRegistry::new();
        
        let result = time_operation!(registry, "test_timer", "Test timer", {
            thread::sleep(Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        
        let metrics = registry.get_all_metrics();
        assert_eq!(metrics.len(), 1);
        
        if let MetricType::Timer { count, sum_ms, .. } = metrics[0].metric_type {
            assert_eq!(count, 1);
            assert!(sum_ms >= 10.0); // Should be at least 10ms
        } else {
            panic!("Expected timer metric");
        }
    }
    
    #[test]
    fn test_prometheus_export() {
        let registry = MetricsRegistry::new();
        
        increment_counter!(registry, "http_requests_total", "Total HTTP requests", "method" => "GET", "status" => "200");
        set_gauge!(registry, "memory_usage_bytes", "Memory usage in bytes", 1048576.0);
        
        let prometheus_output = registry.export_prometheus();
        
        assert!(prometheus_output.contains("# HELP http_requests_total Total HTTP requests"));
        assert!(prometheus_output.contains("# TYPE http_requests_total counter"));
        assert!(prometheus_output.contains("http_requests_total{method=\"GET\",status=\"200\"} 1"));
        
        assert!(prometheus_output.contains("# HELP memory_usage_bytes Memory usage in bytes"));
        assert!(prometheus_output.contains("# TYPE memory_usage_bytes gauge"));
        assert!(prometheus_output.contains("memory_usage_bytes 1048576"));
    }
}
