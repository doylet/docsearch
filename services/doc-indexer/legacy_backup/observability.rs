use anyhow::Result;
use opentelemetry::{
    global, 
    trace::{TraceId, SpanId, Tracer},
    KeyValue,
};
use opentelemetry_sdk::{
    trace::{self, TracerProvider},
    Resource,
};
use opentelemetry_semantic_conventions::resource;
use prometheus::{
    Counter, Histogram, IntCounter, IntGauge, Registry, 
    TextEncoder, Encoder,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Comprehensive observability configuration
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    pub service_name: String,
    pub service_version: String,
    pub environment: String,
    pub jaeger_endpoint: Option<String>,
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub log_level: String,
    pub structured_logs: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            service_name: "zero-latency-doc-indexer".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            metrics_enabled: true,
            tracing_enabled: true,
            log_level: "info".to_string(),
            structured_logs: true,
        }
    }
}

/// Production-grade observability manager
pub struct ObservabilityManager {
    config: ObservabilityConfig,
    metrics: Arc<SearchMetrics>,
    tracer: Option<Box<dyn Tracer + Send + Sync>>,
    registry: Arc<Registry>,
}

/// Comprehensive search metrics collection
pub struct SearchMetrics {
    // Request metrics
    pub search_requests_total: IntCounter,
    pub search_request_duration: Histogram,
    pub search_errors_total: IntCounter,
    
    // Performance metrics
    pub embedding_generation_duration: Histogram,
    pub vector_search_duration: Histogram,
    pub result_ranking_duration: Histogram,
    
    // Quality metrics
    pub query_enhancement_terms_added: Histogram,
    pub result_relevance_scores: Histogram,
    pub search_result_count: Histogram,
    
    // System metrics
    pub active_connections: IntGauge,
    pub memory_usage_bytes: IntGauge,
    pub documents_indexed_total: IntCounter,
    pub index_size_bytes: IntGauge,
    
    // Error tracking
    pub embedding_errors_total: IntCounter,
    pub vector_db_errors_total: IntCounter,
    pub indexing_errors_total: IntCounter,
}

/// Search operation context for tracing
#[derive(Debug, Clone)]
pub struct SearchContext {
    pub trace_id: String,
    pub span_id: String,
    pub query: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_start: std::time::Instant,
}

impl ObservabilityManager {
    /// Initialize comprehensive observability stack
    pub async fn new(config: ObservabilityConfig) -> Result<Self> {
        info!("🔬 Initializing OpenTelemetry observability stack");
        
        // Create Prometheus registry
        let registry = Arc::new(Registry::new());
        
        // Initialize metrics
        let metrics = Arc::new(SearchMetrics::new(&registry)?);
        
        // Initialize tracing if enabled
        let tracer = if config.tracing_enabled {
            Some(Self::setup_tracing(&config).await?)
        } else {
            None
        };
        
        // Setup structured logging
        Self::setup_logging(&config)?;
        
        info!(
            service_name = %config.service_name,
            service_version = %config.service_version,
            environment = %config.environment,
            "✅ Observability stack initialized successfully"
        );
        
        Ok(Self {
            config,
            metrics,
            tracer,
            registry,
        })
    }
    
    /// Setup OpenTelemetry tracing with Jaeger export
    async fn setup_tracing(config: &ObservabilityConfig) -> Result<Box<dyn Tracer + Send + Sync>> {
        info!("🔍 Setting up OpenTelemetry tracing");
        
        let tracer = opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name(&config.service_name)
            .with_tags(vec![
                KeyValue::new(resource::SERVICE_VERSION, config.service_version.clone()),
                KeyValue::new(resource::DEPLOYMENT_ENVIRONMENT, config.environment.clone()),
                KeyValue::new("component", "search-engine"),
            ])
            .install_simple()?;
        
        // Setup tracing subscriber with OpenTelemetry layer
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer.clone());
        
        tracing_subscriber::registry()
            .with(telemetry)
            .with(EnvFilter::from_default_env())
            .init();
        
        info!("✅ OpenTelemetry tracing configured");
        Ok(Box::new(tracer))
    }
    
    /// Setup structured logging with correlation IDs
    fn setup_logging(config: &ObservabilityConfig) -> Result<()> {
        info!("📝 Setting up structured logging");
        
        let env_filter = EnvFilter::try_new(&config.log_level)
            .unwrap_or_else(|_| EnvFilter::new("info"));
        
        if config.structured_logs {
            tracing_subscriber::fmt()
                .json()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .init();
        } else {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .init();
        }
        
        info!("✅ Structured logging configured");
        Ok(())
    }
    
    /// Create search operation context with tracing
    pub fn start_search_operation(&self, query: &str) -> SearchContext {
        let trace_id = format!("{:032x}", rand::random::<u128>());
        let span_id = format!("{:016x}", rand::random::<u64>());
        
        SearchContext {
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            query: query.to_string(),
            user_id: None,
            session_id: None,
            request_start: std::time::Instant::now(),
        }
    }
    
    /// Record search request metrics
    pub fn record_search_request(&self, context: &SearchContext, success: bool) {
        self.metrics.search_requests_total.inc();
        
        let duration = context.request_start.elapsed();
        self.metrics.search_request_duration.observe(duration.as_secs_f64());
        
        if !success {
            self.metrics.search_errors_total.inc();
        }
        
        info!(
            trace_id = %context.trace_id,
            query = %context.query,
            duration_ms = duration.as_millis(),
            success = success,
            "Search request completed"
        );
    }
    
    /// Record embedding generation metrics
    pub fn record_embedding_generation(&self, duration: std::time::Duration, success: bool) {
        self.metrics.embedding_generation_duration.observe(duration.as_secs_f64());
        
        if !success {
            self.metrics.embedding_errors_total.inc();
        }
    }
    
    /// Record vector search metrics
    pub fn record_vector_search(&self, duration: std::time::Duration, result_count: usize) {
        self.metrics.vector_search_duration.observe(duration.as_secs_f64());
        self.metrics.search_result_count.observe(result_count as f64);
    }
    
    /// Record result ranking metrics
    pub fn record_result_ranking(&self, duration: std::time::Duration, avg_score: f64) {
        self.metrics.result_ranking_duration.observe(duration.as_secs_f64());
        self.metrics.result_relevance_scores.observe(avg_score);
    }
    
    /// Record query enhancement metrics
    pub fn record_query_enhancement(&self, original_terms: usize, enhanced_terms: usize) {
        let terms_added = (enhanced_terms - original_terms) as f64;
        self.metrics.query_enhancement_terms_added.observe(terms_added);
    }
    
    /// Get Prometheus metrics for export
    pub fn get_metrics_registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }
    
    /// Health check for observability components
    pub async fn health_check(&self) -> HashMap<String, serde_json::Value> {
        let mut health = HashMap::new();
        
        // Check metrics collection
        health.insert(
            "metrics".to_string(),
            serde_json::json!({
                "status": "healthy",
                "total_searches": self.metrics.search_requests_total.get(),
                "total_errors": self.metrics.search_errors_total.get(),
            })
        );
        
        // Check tracing
        health.insert(
            "tracing".to_string(),
            serde_json::json!({
                "status": if self.tracer.is_some() { "enabled" } else { "disabled" },
                "endpoint": self.config.jaeger_endpoint.as_ref().unwrap_or(&"none".to_string()),
            })
        );
        
        // Check logging
        health.insert(
            "logging".to_string(),
            serde_json::json!({
                "status": "healthy",
                "level": self.config.log_level,
                "structured": self.config.structured_logs,
            })
        );
        
        health
    }
}

impl SearchMetrics {
    fn new(registry: &Registry) -> Result<Self> {
        let search_requests_total = IntCounter::new(
            "search_requests_total",
            "Total number of search requests"
        )?;
        
        let search_request_duration = Histogram::new(
            "search_request_duration_seconds",
            "Duration of search requests in seconds"
        )?;
        
        let search_errors_total = IntCounter::new(
            "search_errors_total",
            "Total number of search errors"
        )?;
        
        let embedding_generation_duration = Histogram::new(
            "embedding_generation_duration_seconds",
            "Duration of embedding generation in seconds"
        )?;
        
        let vector_search_duration = Histogram::new(
            "vector_search_duration_seconds",
            "Duration of vector search operations in seconds"
        )?;
        
        let result_ranking_duration = Histogram::new(
            "result_ranking_duration_seconds",
            "Duration of result ranking operations in seconds"
        )?;
        
        let query_enhancement_terms_added = Histogram::new(
            "query_enhancement_terms_added",
            "Number of terms added during query enhancement"
        )?;
        
        let result_relevance_scores = Histogram::new(
            "result_relevance_scores",
            "Distribution of result relevance scores"
        )?;
        
        let search_result_count = Histogram::new(
            "search_result_count",
            "Number of results returned per search"
        )?;
        
        let active_connections = IntGauge::new(
            "active_connections",
            "Number of active connections"
        )?;
        
        let memory_usage_bytes = IntGauge::new(
            "memory_usage_bytes",
            "Memory usage in bytes"
        )?;
        
        let documents_indexed_total = IntCounter::new(
            "documents_indexed_total",
            "Total number of documents indexed"
        )?;
        
        let index_size_bytes = IntGauge::new(
            "index_size_bytes",
            "Size of the search index in bytes"
        )?;
        
        let embedding_errors_total = IntCounter::new(
            "embedding_errors_total",
            "Total number of embedding generation errors"
        )?;
        
        let vector_db_errors_total = IntCounter::new(
            "vector_db_errors_total",
            "Total number of vector database errors"
        )?;
        
        let indexing_errors_total = IntCounter::new(
            "indexing_errors_total",
            "Total number of indexing errors"
        )?;
        
        // Register all metrics
        registry.register(Box::new(search_requests_total.clone()))?;
        registry.register(Box::new(search_request_duration.clone()))?;
        registry.register(Box::new(search_errors_total.clone()))?;
        registry.register(Box::new(embedding_generation_duration.clone()))?;
        registry.register(Box::new(vector_search_duration.clone()))?;
        registry.register(Box::new(result_ranking_duration.clone()))?;
        registry.register(Box::new(query_enhancement_terms_added.clone()))?;
        registry.register(Box::new(result_relevance_scores.clone()))?;
        registry.register(Box::new(search_result_count.clone()))?;
        registry.register(Box::new(active_connections.clone()))?;
        registry.register(Box::new(memory_usage_bytes.clone()))?;
        registry.register(Box::new(documents_indexed_total.clone()))?;
        registry.register(Box::new(index_size_bytes.clone()))?;
        registry.register(Box::new(embedding_errors_total.clone()))?;
        registry.register(Box::new(vector_db_errors_total.clone()))?;
        registry.register(Box::new(indexing_errors_total.clone()))?;
        
        Ok(Self {
            search_requests_total,
            search_request_duration,
            search_errors_total,
            embedding_generation_duration,
            vector_search_duration,
            result_ranking_duration,
            query_enhancement_terms_added,
            result_relevance_scores,
            search_result_count,
            active_connections,
            memory_usage_bytes,
            documents_indexed_total,
            index_size_bytes,
            embedding_errors_total,
            vector_db_errors_total,
            indexing_errors_total,
        })
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
}

/// Correlation ID utilities for request tracing
pub mod correlation {
    use std::collections::HashMap;
    use uuid::Uuid;
    
    thread_local! {
        static CORRELATION_CONTEXT: std::cell::RefCell<HashMap<String, String>> = 
            std::cell::RefCell::new(HashMap::new());
    }
    
    pub fn set_correlation_id(id: String) {
        CORRELATION_CONTEXT.with(|ctx| {
            ctx.borrow_mut().insert("correlation_id".to_string(), id);
        });
    }
    
    pub fn get_correlation_id() -> Option<String> {
        CORRELATION_CONTEXT.with(|ctx| {
            ctx.borrow().get("correlation_id").cloned()
        })
    }
    
    pub fn generate_correlation_id() -> String {
        Uuid::new_v4().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_observability_manager_creation() {
        let config = ObservabilityConfig::default();
        let manager = ObservabilityManager::new(config).await;
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_correlation_id_generation() {
        let id1 = correlation::generate_correlation_id();
        let id2 = correlation::generate_correlation_id();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID v4 length
    }
}
