/// Infrastructure layer modules
/// 
/// This module contains all infrastructure concerns including HTTP servers,
/// JSON-RPC servers, vector storage adapters, embedding service adapters, 
/// and other external service integrations.

pub mod http;
pub mod jsonrpc;
pub mod vector;
pub mod embeddings;

// Re-export commonly used types
pub use http::{HttpServer, ServerConfig};
pub use vector::{QdrantAdapter, QdrantConfig, InMemoryVectorStore};
pub use embeddings::{OpenAIAdapter, OpenAIConfig, LocalEmbeddingAdapter, LocalEmbeddingConfig};
