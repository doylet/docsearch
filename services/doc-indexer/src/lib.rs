// lib.rs - Library interface for doc-indexer
// Exposes the SOLID service layer for testing and external use

pub mod config;
pub mod application;
pub mod infrastructure;

// Re-export key types for easier access
pub use application::services::indexing_service::{IndexingService, IndexingServiceBuilder};
pub use application::interfaces::*;
pub use application::adapters::*;
pub use application::indexing_strategies::*;

// Re-export domain types from zero-latency-core
pub use zero_latency_core::models::Document;
