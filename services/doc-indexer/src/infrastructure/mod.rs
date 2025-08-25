/// Infrastructure layer modules
/// 
/// This module contains all infrastructure concerns including HTTP servers,
/// JSON-RPC servers, streaming support, stdio transport, vector storage adapters, 
/// embedding service adapters, memory optimization utilities, and other external service integrations.

pub mod http;
pub mod jsonrpc;
pub mod streaming;
pub mod stdio;
pub mod search_enhancement;
pub mod vector;
pub mod embeddings;
pub mod memory;

// Re-export commonly used types
pub use http::{HttpServer, ServerConfig};
pub use vector::InMemoryVectorStore;
pub use memory::{VectorPool, VectorPoolConfig, StringInterner, InternedString, MemoryEfficientCache, CacheConfig};

// Cloud-dependent exports
#[cfg(feature = "cloud")]
pub use vector::{QdrantAdapter, QdrantConfig};

#[cfg(feature = "cloud")]
pub use embeddings::{OpenAIAdapter, OpenAIConfig};

// Embedded-dependent exports
#[cfg(feature = "embedded")]
pub use vector::{EmbeddedVectorStore, EmbeddedConfig};

#[cfg(feature = "embedded")]
pub use embeddings::{LocalEmbeddingAdapter, LocalEmbeddingConfig};
