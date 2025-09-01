//! Distributed tracing and logging functionality
//!
//! Provides structured logging, distributed tracing, and observability
//! patterns for production services.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Trace context for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// Unique trace ID for the entire request
    pub trace_id: String,
    /// Span ID for this specific operation
    pub span_id: String,
    /// Parent span ID (if this is a child span)
    pub parent_span_id: Option<String>,
    /// Sampling decision
    pub sampled: bool,
    /// Baggage for cross-service context
    pub baggage: HashMap<String, String>,
}

impl TraceContext {
    /// Create a new root trace context
    pub fn new() -> Self {
        Self {
            trace_id: generate_trace_id(),
            span_id: generate_span_id(),
            parent_span_id: None,
            sampled: true,
            baggage: HashMap::new(),
        }
    }
    
    /// Create a child span from this context
    pub fn child_span(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: generate_span_id(),
            parent_span_id: Some(self.span_id.clone()),
            sampled: self.sampled,
            baggage: self.baggage.clone(),
        }
    }
    
    /// Add baggage item
    pub fn with_baggage(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.baggage.insert(key.into(), value.into());
        self
    }
    
    /// Set sampling decision
    pub fn with_sampling(mut self, sampled: bool) -> Self {
        self.sampled = sampled;
        self
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Log level for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp in RFC3339 format
    pub timestamp: String,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Structured fields
    pub fields: HashMap<String, serde_json::Value>,
    /// Optional trace context
    pub trace_context: Option<TraceContext>,
    /// Source location (file, line)
    pub source: Option<String>,
    /// Service name
    pub service: String,
}

/// Span for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Span context
    pub context: TraceContext,
    /// Operation name
    pub operation_name: String,
    /// Start timestamp (Unix microseconds)
    pub start_time_us: u64,
    /// End timestamp (Unix microseconds)
    pub end_time_us: Option<u64>,
    /// Duration in microseconds
    pub duration_us: Option<u64>,
    /// Span tags/attributes
    pub tags: HashMap<String, serde_json::Value>,
    /// Span logs/events
    pub logs: Vec<SpanLog>,
    /// Service name
    pub service: String,
}

/// Event logged within a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLog {
    /// Timestamp (Unix microseconds)
    pub timestamp_us: u64,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Additional fields
    pub fields: HashMap<String, serde_json::Value>,
}

impl Span {
    /// Create a new span
    pub fn new(context: TraceContext, operation_name: impl Into<String>, service: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
            
        Self {
            context,
            operation_name: operation_name.into(),
            start_time_us: now,
            end_time_us: None,
            duration_us: None,
            tags: HashMap::new(),
            logs: Vec::new(),
            service: service.into(),
        }
    }
    
    /// Add a tag to the span
    pub fn set_tag(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.tags.insert(key.into(), value.into());
    }
    
    /// Log an event in the span
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        self.log_with_fields(level, message, HashMap::new());
    }
    
    /// Log an event with additional fields
    pub fn log_with_fields(
        &mut self, 
        level: LogLevel, 
        message: impl Into<String>, 
        fields: HashMap<String, serde_json::Value>
    ) {
        let timestamp_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
            
        self.logs.push(SpanLog {
            timestamp_us,
            level,
            message: message.into(),
            fields,
        });
    }
    
    /// Finish the span
    pub fn finish(mut self) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
            
        self.end_time_us = Some(now);
        self.duration_us = Some(now - self.start_time_us);
        self
    }
    
    /// Mark span as failed with error
    pub fn set_error(&mut self, error: impl fmt::Display) {
        self.set_tag("error", true);
        self.set_tag("error.message", error.to_string());
        self.log(LogLevel::Error, format!("Span failed: {}", error));
    }
}

/// Structured logger with tracing support
pub struct StructuredLogger {
    service_name: String,
    min_level: LogLevel,
    trace_context: Option<TraceContext>,
}

impl StructuredLogger {
    /// Create a new structured logger
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            min_level: LogLevel::Info,
            trace_context: None,
        }
    }
    
    /// Set minimum log level
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }
    
    /// Set trace context
    pub fn with_trace_context(mut self, context: TraceContext) -> Self {
        self.trace_context = Some(context);
        self
    }
    
    /// Log at trace level
    pub fn trace(&self, message: impl Into<String>) {
        self.log_with_fields(LogLevel::Trace, message, HashMap::new());
    }
    
    /// Log at debug level
    pub fn debug(&self, message: impl Into<String>) {
        self.log_with_fields(LogLevel::Debug, message, HashMap::new());
    }
    
    /// Log at info level
    pub fn info(&self, message: impl Into<String>) {
        self.log_with_fields(LogLevel::Info, message, HashMap::new());
    }
    
    /// Log at warn level
    pub fn warn(&self, message: impl Into<String>) {
        self.log_with_fields(LogLevel::Warn, message, HashMap::new());
    }
    
    /// Log at error level
    pub fn error(&self, message: impl Into<String>) {
        self.log_with_fields(LogLevel::Error, message, HashMap::new());
    }
    
    /// Log with structured fields
    pub fn log_with_fields(
        &self,
        level: LogLevel,
        message: impl Into<String>,
        fields: HashMap<String, serde_json::Value>,
    ) {
        if level < self.min_level {
            return;
        }
        
        let entry = LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level,
            message: message.into(),
            fields,
            trace_context: self.trace_context.clone(),
            source: None, // TODO: Capture caller location
            service: self.service_name.clone(),
        };
        
        // Output as JSON for structured logging
        if let Ok(json) = serde_json::to_string(&entry) {
            println!("{}", json);
        }
    }
}

/// Tracer for creating and managing spans
pub struct Tracer {
    service_name: String,
    spans: Arc<std::sync::Mutex<Vec<Span>>>,
}

impl Tracer {
    /// Create a new tracer
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            spans: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
    
    /// Start a new root span
    pub fn start_span(&self, operation_name: impl Into<String>) -> Span {
        let context = TraceContext::new();
        Span::new(context, operation_name, &self.service_name)
    }
    
    /// Start a child span
    pub fn start_child_span(&self, parent: &TraceContext, operation_name: impl Into<String>) -> Span {
        let context = parent.child_span();
        Span::new(context, operation_name, &self.service_name)
    }
    
    /// Finish and collect a span
    pub fn finish_span(&self, span: Span) {
        let finished_span = span.finish();
        
        if finished_span.context.sampled {
            let mut spans = self.spans.lock().unwrap();
            spans.push(finished_span);
        }
    }
    
    /// Get all collected spans
    pub fn get_spans(&self) -> Vec<Span> {
        let spans = self.spans.lock().unwrap();
        spans.clone()
    }
    
    /// Clear collected spans
    pub fn clear_spans(&self) {
        let mut spans = self.spans.lock().unwrap();
        spans.clear();
    }
}

/// Helper macros for logging
#[macro_export]
macro_rules! log_info {
    ($logger:expr, $message:expr) => {
        $logger.info($message)
    };
    ($logger:expr, $message:expr, $($key:expr => $value:expr),*) => {
        {
            let mut fields = std::collections::HashMap::new();
            $(fields.insert($key.to_string(), serde_json::json!($value));)*
            $logger.log_with_fields(crate::tracing::LogLevel::Info, $message, fields)
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $message:expr) => {
        $logger.error($message)
    };
    ($logger:expr, $message:expr, $($key:expr => $value:expr),*) => {
        {
            let mut fields = std::collections::HashMap::new();
            $(fields.insert($key.to_string(), serde_json::json!($value));)*
            $logger.log_with_fields(crate::tracing::LogLevel::Error, $message, fields)
        }
    };
}

// Helper functions

fn generate_trace_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:016x}{:016x}", rng.gen::<u64>(), rng.gen::<u64>())
}

fn generate_span_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:016x}", rng.gen::<u64>())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trace_context() {
        let parent = TraceContext::new();
        let child = parent.child_span();
        
        assert_eq!(parent.trace_id, child.trace_id);
        assert_ne!(parent.span_id, child.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id));
    }
    
    #[test]
    fn test_span_lifecycle() {
        let context = TraceContext::new();
        let mut span = Span::new(context, "test_operation", "test_service");
        
        span.set_tag("user_id", "12345");
        span.log(LogLevel::Info, "Processing request");
        
        let finished_span = span.finish();
        
        assert!(finished_span.end_time_us.is_some());
        assert!(finished_span.duration_us.is_some());
        assert_eq!(finished_span.tags.get("user_id").unwrap(), "12345");
        assert_eq!(finished_span.logs.len(), 1);
    }
    
    #[test]
    fn test_structured_logger() {
        let logger = StructuredLogger::new("test_service")
            .with_level(LogLevel::Debug);
            
        logger.info("Test message");
        logger.log_with_fields(
            LogLevel::Warn,
            "Warning message",
            [("key".to_string(), serde_json::json!("value"))]
                .iter().cloned().collect()
        );
    }
    
    #[test]
    fn test_tracer() {
        let tracer = Tracer::new("test_service");
        
        let span = tracer.start_span("test_operation");
        tracer.finish_span(span);
        
        let spans = tracer.get_spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].operation_name, "test_operation");
    }
}
