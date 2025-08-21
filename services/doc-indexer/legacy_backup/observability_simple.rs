//! Simple observability module with Prometheus metrics
//! 
//! Provides production-ready observability for the search service with:
//! - Prometheus metrics collection
//! - Request tracking and timing
//! - Error counting
//! - Performance monitoring

use anyhow::{Context, Result};
use prometheus::{
    Encoder, Histogram, HistogramOpts, IntCounter,
    Registry, TextEncoder, Opts,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

/// Configuration for observability
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    pub service_name: String,
    pub metrics_enabled: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            service_name: "doc-indexer".to_string(),
            metrics_enabled: true,
        }
    }
}

/// Collection of Prometheus metrics for search operations
#[derive(Debug, Clone)]
pub struct SearchMetrics {
    pub search_requests_total: IntCounter,
    pub search_errors_total: IntCounter,
    pub search_request_duration: Histogram,
    pub embedding_generation_duration: Histogram,
    pub vector_search_duration: Histogram,
    pub result_ranking_duration: Histogram,
    pub query_enhancement_total: IntCounter,
    pub query_expansions_total: IntCounter,
    pub search_results_returned: Histogram,
    pub search_latency_p99: Histogram,
}

impl SearchMetrics {
    /// Create new search metrics and register them
    pub fn new(registry: &Registry) -> Result<Self> {
        let search_requests_total = IntCounter::with_opts(
            Opts::new(
                "search_requests_total",
                "Total number of search requests"
            )
        )?;

        let search_errors_total = IntCounter::with_opts(
            Opts::new(
                "search_errors_total",
                "Total number of search errors"
            )
        )?;

        let search_request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "search_request_duration_seconds",
                "Duration of search requests in seconds"
            ).buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0])
        )?;

        let embedding_generation_duration = Histogram::with_opts(
            HistogramOpts::new(
                "embedding_generation_duration_seconds",
                "Duration of embedding generation in seconds"
            ).buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0])
        )?;

        let vector_search_duration = Histogram::with_opts(
            HistogramOpts::new(
                "vector_search_duration_seconds",
                "Duration of vector search in seconds"
            ).buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0])
        )?;

        let result_ranking_duration = Histogram::with_opts(
            HistogramOpts::new(
                "result_ranking_duration_seconds",
                "Duration of result ranking in seconds"
            ).buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1])
        )?;

        let query_enhancement_total = IntCounter::with_opts(
            Opts::new(
                "query_enhancement_total",
                "Total number of query enhancements performed"
            )
        )?;

        let query_expansions_total = IntCounter::with_opts(
            Opts::new(
                "query_expansions_total",
                "Total number of query expansions performed"
            )
        )?;

        let search_results_returned = Histogram::with_opts(
            HistogramOpts::new(
                "search_results_returned",
                "Number of search results returned"
            ).buckets(vec![0.0, 1.0, 5.0, 10.0, 25.0, 50.0, 100.0])
        )?;

        let search_latency_p99 = Histogram::with_opts(
            HistogramOpts::new(
                "search_latency_p99_seconds",
                "99th percentile search latency"
            ).buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0])
        )?;

        // Register all metrics
        registry.register(Box::new(search_requests_total.clone()))?;
        registry.register(Box::new(search_errors_total.clone()))?;
        registry.register(Box::new(search_request_duration.clone()))?;
        registry.register(Box::new(embedding_generation_duration.clone()))?;
        registry.register(Box::new(vector_search_duration.clone()))?;
        registry.register(Box::new(result_ranking_duration.clone()))?;
        registry.register(Box::new(query_enhancement_total.clone()))?;
        registry.register(Box::new(query_expansions_total.clone()))?;
        registry.register(Box::new(search_results_returned.clone()))?;
        registry.register(Box::new(search_latency_p99.clone()))?;

        info!("✅ Search metrics registered successfully");

        Ok(SearchMetrics {
            search_requests_total,
            search_errors_total,
            search_request_duration,
            embedding_generation_duration,
            vector_search_duration,
            result_ranking_duration,
            query_enhancement_total,
            query_expansions_total,
            search_results_returned,
            search_latency_p99,
        })
    }
}

/// Main observability manager
#[derive(Debug)]
pub struct ObservabilityManager {
    pub metrics: SearchMetrics,
    pub registry: Arc<Registry>,
    config: ObservabilityConfig,
}

impl ObservabilityManager {
    /// Initialize observability manager
    pub async fn new(config: ObservabilityConfig) -> Result<Self> {
        info!("🔬 Initializing observability stack");
        
        let registry = Arc::new(Registry::new());
        let metrics = SearchMetrics::new(&registry)?;
        
        info!("✅ Observability initialization completed");
        
        Ok(Self {
            metrics,
            registry,
            config,
        })
    }

    /// Record a search request
    pub fn record_search_request(&self) {
        self.metrics.search_requests_total.inc();
    }

    /// Record search error
    pub fn record_search_error(&self) {
        self.metrics.search_errors_total.inc();
    }

    /// Record search completion with timing
    pub fn record_search_completion(&self, duration: Duration, result_count: usize) {
        self.metrics.search_request_duration.observe(duration.as_secs_f64());
        self.metrics.search_results_returned.observe(result_count as f64);
        self.metrics.search_latency_p99.observe(duration.as_secs_f64());
    }

    /// Record embedding generation timing
    pub fn record_embedding_generation(&self, duration: Duration) {
        self.metrics.embedding_generation_duration.observe(duration.as_secs_f64());
    }

    /// Record vector search timing
    pub fn record_vector_search(&self, duration: Duration, result_count: usize) {
        self.metrics.vector_search_duration.observe(duration.as_secs_f64());
    }

    /// Record result ranking timing
    pub fn record_result_ranking(&self, duration: Duration) {
        self.metrics.result_ranking_duration.observe(duration.as_secs_f64());
    }

    /// Record query enhancement
    pub fn record_query_enhancement(&self, expanded: bool) {
        self.metrics.query_enhancement_total.inc();
        if expanded {
            self.metrics.query_expansions_total.inc();
        }
    }

    /// Get metrics in Prometheus text format
    pub fn get_metrics_text(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)
            .context("Failed to encode metrics")?;
        
        String::from_utf8(buffer)
            .context("Failed to convert metrics to UTF-8")
    }

    /// Get service info
    pub fn get_service_info(&self) -> &ObservabilityConfig {
        &self.config
    }
}

/// Helper function to create default observability manager
pub async fn create_default_observability() -> Result<Arc<ObservabilityManager>> {
    let config = ObservabilityConfig::default();
    let manager = ObservabilityManager::new(config).await?;
    Ok(Arc::new(manager))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_observability_creation() {
        let config = ObservabilityConfig::default();
        let observability = ObservabilityManager::new(config).await.unwrap();
        
        assert!(observability.get_metrics_text().is_ok());
    }

    #[tokio::test]
    async fn test_metrics_recording() {
        let observability = create_default_observability().await.unwrap();
        
        // Record some metrics
        observability.record_search_request();
        observability.record_embedding_generation(Duration::from_millis(50));
        observability.record_vector_search(Duration::from_millis(30), 10);
        observability.record_result_ranking(Duration::from_millis(5));
        observability.record_search_completion(Duration::from_millis(100), 10);
        
        let metrics_text = observability.get_metrics_text().unwrap();
        assert!(metrics_text.contains("search_requests_total"));
        assert!(metrics_text.contains("embedding_generation_duration"));
    }
}
