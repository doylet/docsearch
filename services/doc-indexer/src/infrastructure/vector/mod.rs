/// Vector store infrastructure adapters
/// 
/// This module contains concrete implementations of the VectorRepository trait
/// for different vector storage backends.

pub mod qdrant_adapter;
pub mod memory_adapter;

// Re-export commonly used types
pub use qdrant_adapter::{QdrantAdapter, QdrantConfig};
pub use memory_adapter::InMemoryVectorStore;
