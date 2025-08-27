pub mod embeddings;
/// Infrastructure Layer
///
/// This module contains all infrastructure-level concerns including
/// external service adapters, configuration management, and system utilities.
pub mod analytics;
pub mod http;
pub mod jsonrpc;
pub mod load_testing;
pub mod memory;
pub mod production;
pub mod search_enhancement;
pub mod stdio;
pub mod streaming;
pub mod vector;

// Phase 4D: Enhanced API Features
pub mod batch_operations;
pub mod collection_management;
pub mod enhanced_api;
pub mod enhanced_search;

// Re-export commonly used types
pub use http::{HttpServer, ServerConfig};
pub use vector::InMemoryVectorStore;

// Phase 4D: Enhanced API Features
pub use batch_operations::{
    BatchOperationRequest, BatchOperationResult, BatchOperationType, BatchProcessor,
};
pub use collection_management::{
    CollectionConfig, CollectionManager, CreateCollectionRequest, CrossCollectionSearchRequest,
};
pub use enhanced_search::{EnhancedSearchRequest, EnhancedSearchResult, EnhancedSearchService};

// Cloud-dependent exports
#[cfg(feature = "cloud")]
pub use vector::{QdrantAdapter, QdrantConfig};

#[cfg(feature = "cloud")]
pub use embeddings::{OpenAIAdapter, OpenAIConfig};

// Embedded-dependent exports
#[cfg(feature = "embedded")]
pub use vector::{EmbeddedConfig, EmbeddedVectorStore};

#[cfg(feature = "embedded")]
pub use embeddings::{LocalEmbeddingAdapter, LocalEmbeddingConfig};
