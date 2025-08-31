/// Vector store infrastructure adapters
///
/// This module contains concrete implementations of the VectorRepository trait
/// for different vector storage backends.

#[cfg(feature = "cloud")]
pub mod qdrant_adapter;

pub mod memory_adapter;

#[cfg(feature = "embedded")]
pub mod embedded_adapter;

// Re-export commonly used types
#[cfg(feature = "cloud")]
pub use qdrant_adapter::{QdrantAdapter, QdrantConfig};

pub use memory_adapter::InMemoryVectorStore;

#[cfg(feature = "embedded")]
pub use embedded_adapter::{EmbeddedConfig, EmbeddedVectorStore};
