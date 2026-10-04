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
pub use health::{HealthCheck, HealthCheckResult, HealthChecker, HealthReport, HealthStatus};
pub use metrics::{Metric, MetricType, MetricsRegistry, Timer};
pub use tracing::{LogLevel, Span, StructuredLogger, TraceContext, Tracer};
