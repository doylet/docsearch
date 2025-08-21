/// Embedding generation infrastructure adapters
/// 
/// This module contains concrete implementations of the EmbeddingGenerator trait
/// for different embedding services and local implementations.

pub mod openai_adapter;
pub mod local_adapter;

// Re-export commonly used types
pub use openai_adapter::{OpenAIAdapter, OpenAIConfig};
pub use local_adapter::{LocalEmbeddingAdapter, LocalEmbeddingConfig};
