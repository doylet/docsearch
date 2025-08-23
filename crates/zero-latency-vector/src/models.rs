use zero_latency_core::{Uuid, values::Score};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vector document for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: Uuid,
    pub embedding: Vec<f32>,
    pub metadata: VectorMetadata,
}

/// Metadata associated with vectors
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorMetadata {
    pub document_id: Uuid,
    pub chunk_index: usize,
    pub content: String,
    pub title: String,
    pub heading_path: Vec<String>,
    pub url: Option<String>,
    pub custom: HashMap<String, String>,
}

/// Similarity search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    pub document_id: Uuid,
    pub similarity: Score,
    pub metadata: VectorMetadata,
}

/// Embedding generation request
#[derive(Debug, Clone)]
pub struct EmbeddingRequest {
    pub text: String,
    pub model: Option<String>,
    pub options: EmbeddingOptions,
}

/// Options for embedding generation
#[derive(Debug, Clone, Default)]
pub struct EmbeddingOptions {
    pub normalize: bool,
    pub max_tokens: Option<usize>,
    pub batch_size: Option<usize>,
}

/// Vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub store_type: VectorStoreType,
    pub connection_string: String,
    pub collection_name: String,
    pub dimension: usize,
    pub metric: SimilarityMetric,
}

/// Supported vector store types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorStoreType {
    InMemory,
    Chroma,
    Qdrant,
    Pinecone,
}

/// Similarity calculation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimilarityMetric {
    Cosine,
    Euclidean,
    DotProduct,
}
