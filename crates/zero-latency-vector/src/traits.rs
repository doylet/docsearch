use async_trait::async_trait;
use zero_latency_core::{Result, models::HealthStatus};
use crate::models::*;

/// Vector storage operations
#[async_trait]
pub trait VectorRepository: Send + Sync {
    async fn insert(&self, vectors: Vec<VectorDocument>) -> Result<()>;
    async fn search(&self, query_vector: Vec<f32>, k: usize) -> Result<Vec<SimilarityResult>>;
    async fn search_in_collection(&self, collection_name: &str, query_vector: Vec<f32>, k: usize) -> Result<Vec<SimilarityResult>>;
    async fn delete(&self, document_id: &str) -> Result<bool>;
    async fn update(&self, document_id: &str, vector: Vec<f32>) -> Result<bool>;
    async fn health_check(&self) -> Result<HealthStatus>;
    async fn count(&self) -> Result<usize>;
}

/// Embedding generation
#[async_trait]
pub trait EmbeddingGenerator: Send + Sync {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
    async fn generate_batch_embeddings(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
    fn model_name(&self) -> &str;
}

/// Similarity calculations
pub trait SimilarityCalculator: Send + Sync {
    fn calculate_similarity(&self, a: &[f32], b: &[f32]) -> f32;
    fn batch_similarities(&self, query: &[f32], candidates: &[Vec<f32>]) -> Vec<f32>;
    fn metric(&self) -> SimilarityMetric;
}
