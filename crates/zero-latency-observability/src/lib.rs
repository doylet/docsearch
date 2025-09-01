//! Observability patterns for Zero-Latency
//!
//! This crate provides reusable observability components including:
//! - Metrics collection interfaces
//! - Tracing and logging patterns  
//! - Health checking frameworks
//! - Performance monitoring

pub mod health;
pub mod metrics;
pub mod tracing;

// Re-export commonly used types
pub use health::{HealthChecker, HealthCheck, HealthStatus, HealthReport, HealthCheckResult};
pub use metrics::{MetricsRegistry, MetricType, Metric, Timer};
pub use tracing::{TraceContext, Span, StructuredLogger, Tracer, LogLevel};
