//! Embedding generation infrastructure adapters
//!
//! This module contains concrete implementations of the EmbeddingGenerator trait
//! for different embedding services and local implementations.

#[cfg(feature = "cloud")]
pub mod openai_adapter;

#[cfg(feature = "embedded")]
pub mod hash_adapter;

#[cfg(feature = "embedded")]
pub mod model_files;

#[cfg(feature = "embedded")]
pub mod onnx_adapter;

// Re-export commonly used types
#[cfg(feature = "cloud")]
pub use openai_adapter::{OpenAIAdapter, OpenAIConfig};

#[cfg(feature = "embedded")]
pub use hash_adapter::{HashEmbeddingAdapter, HashEmbeddingConfig};

#[cfg(feature = "embedded")]
pub use model_files::{ModelSpec, ModelStore};

#[cfg(feature = "embedded")]
pub use onnx_adapter::OnnxEmbeddingAdapter;
