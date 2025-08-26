/// Infrastructure Layer
/// 
/// This module contains all infrastructure-level concerns including
/// external service adapters, configuration management, and system utilities.

pub mod http;
pub mod jsonrpc;
pub mod streaming;
pub mod stdio;
pub mod search_enhancement;
pub mod vector;
pub mod embeddings;
pub mod memory;
pub mod load_testing;
pub mod production;

// Phase 4D: Enhanced API Features
pub mod enhanced_search;
pub mod collection_management;
pub mod batch_operations;
pub mod enhanced_api;

// Re-export commonly used types
pub use http::{HttpServer, ServerConfig};
pub use vector::InMemoryVectorStore;
pub use memory::{VectorPool, VectorPoolConfig, StringInterner, InternedString, MemoryEfficientCache, CacheConfig};

// Phase 4D: Enhanced API Features
pub use enhanced_search::{EnhancedSearchService, EnhancedSearchRequest, EnhancedSearchResult};
pub use collection_management::{CollectionManager, CollectionConfig, CreateCollectionRequest, CrossCollectionSearchRequest};
pub use batch_operations::{BatchProcessor, BatchOperationRequest, BatchOperationType, BatchOperationResult};
pub use enhanced_api::{EnhancedApiService, ApiServiceConfig, ApiEnhancedSearchRequest, ApiEnhancedSearchResponse};

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
