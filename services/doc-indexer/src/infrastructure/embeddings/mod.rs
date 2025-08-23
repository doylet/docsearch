/// Embedding generation infrastructure adapters
/// 
/// This module contains concrete implementations of the EmbeddingGenerator trait
/// for different embedding services and local implementations.

#[cfg(feature = "cloud")]
pub mod openai_adapter;

#[cfg(feature = "embedded")]
pub mod local_adapter;

// Re-export commonly used types
#[cfg(feature = "cloud")]
pub use openai_adapter::{OpenAIAdapter, OpenAIConfig};

#[cfg(feature = "embedded")]
pub use local_adapter::{LocalEmbeddingAdapter, LocalEmbeddingConfig};
