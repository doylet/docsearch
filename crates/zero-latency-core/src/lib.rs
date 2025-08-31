/// Core domain models and traits for Zero-Latency
///
/// This crate provides the fundamental building blocks used across all
/// Zero-Latency services, including:
/// - Common domain entities
/// - Result types and error handling
/// - Service traits and interfaces
/// - Shared value objects
pub mod doc_id;
pub mod error;
pub mod models;
pub mod traits;
pub mod values;

pub use doc_id::DocId;
pub use error::{Result, ZeroLatencyError};

/// Re-export commonly used types
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;
