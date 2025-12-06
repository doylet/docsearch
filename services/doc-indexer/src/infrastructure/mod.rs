/// Infrastructure Layer
///
/// This module contains all infrastructure-level concerns including
/// external service adapters, configuration management, and system utilities.

// Organized infrastructure components
pub mod api;
pub mod operations;
pub mod persistence;
pub mod protocols;

// Core infrastructure components
pub mod concurrent_search;
pub mod load_testing;
pub mod memory;
pub mod search_enhancement;
pub mod stdio;

// Phase 4D: Enhanced API Features
pub mod batch_operations;
pub mod collection_management;
pub mod enhanced_api;
pub mod enhanced_search;

// Re-export commonly used types
pub use api::http::{HttpServer, ServerConfig};
pub use persistence::vector::InMemoryVectorStore;

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
pub use persistence::vector::{QdrantAdapter, QdrantConfig};

#[cfg(feature = "cloud")]
pub use persistence::embeddings::{OpenAIAdapter, OpenAIConfig};

// Embedded-dependent exports
#[cfg(feature = "embedded")]
pub use persistence::vector::{EmbeddedConfig, EmbeddedVectorStore};

#[cfg(feature = "embedded")]
pub use persistence::embeddings::{LocalEmbeddingAdapter, LocalEmbeddingConfig};
