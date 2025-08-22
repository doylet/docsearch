/// Infrastructure layer modules
/// 
/// This module contains all infrastructure concerns including HTTP servers,
/// JSON-RPC servers, streaming support, stdio transport, vector storage adapters, 
/// embedding service adapters, and other external service integrations.

pub mod http;
pub mod jsonrpc;
pub mod streaming;
pub mod stdio;
pub mod vector;
pub mod embeddings;

// Re-export commonly used types
pub use http::{HttpServer, ServerConfig};
pub use vector::{QdrantAdapter, QdrantConfig, InMemoryVectorStore, EmbeddedVectorStore, EmbeddedConfig};
pub use embeddings::{OpenAIAdapter, OpenAIConfig, LocalEmbeddingAdapter, LocalEmbeddingConfig};
