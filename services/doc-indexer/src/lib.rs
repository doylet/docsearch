// lib.rs - Library interface for doc-indexer
// Exposes the SOLID service layer for testing and external use

pub mod config;
/// Doc Indexer Library
/// 
/// The doc-indexer service provides semantic search capabilities over documents
/// using vector embeddings and various storage backends.

pub mod application;
pub mod infrastructure;

// Phase 4D: Core placeholder for missing dependencies
pub mod core_placeholder;

// Re-export key types for easier access
pub use application::services::indexing_service::{IndexingService, IndexingServiceBuilder};
pub use application::interfaces::*;
pub use application::adapters::*;
pub use application::indexing_strategies::*;

// Re-export domain types from zero-latency-core
pub use zero_latency_core::models::Document;
