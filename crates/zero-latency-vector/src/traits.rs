use crate::models::*;
use async_trait::async_trait;
use zero_latency_core::{models::HealthStatus, Result};

/// Vector storage operations
#[async_trait]
pub trait VectorRepository: Send + Sync {
    async fn insert(&self, vectors: Vec<VectorDocument>) -> Result<()>;
    async fn search(&self, query_vector: Vec<f32>, k: usize) -> Result<Vec<SimilarityResult>>;
    async fn search_in_collection(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        k: usize,
    ) -> Result<Vec<SimilarityResult>>;
    async fn delete(&self, document_id: &str) -> Result<bool>;
    /// Delete every vector tagged with this collection, returning how many were
    /// removed. Vectors without a collection tag are left alone.
    async fn delete_collection(&self, collection_name: &str) -> Result<usize>;
    async fn update(&self, document_id: &str, vector: Vec<f32>) -> Result<bool>;
    async fn health_check(&self) -> Result<HealthStatus>;
    async fn count(&self) -> Result<usize>;
}

/// Embedding generation
#[async_trait]
pub trait EmbeddingGenerator: Send + Sync {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
    async fn generate_batch_embeddings(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>>;

    /// Embed a search query. Models trained with a query instruction override this;
    /// by default queries and documents are embedded the same way.
    async fn generate_query_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.generate_embedding(text).await
    }

    fn dimension(&self) -> usize;
    fn model_name(&self) -> &str;

    /// Identifies which model wrote a set of vectors, so a store can detect
    /// vectors from a different model. Defaults to `<model_name>@<dimension>`.
    fn model_id(&self) -> String {
        format!("{}@{}", self.model_name(), self.dimension())
    }
}

/// Similarity calculations
pub trait SimilarityCalculator: Send + Sync {
    fn calculate_similarity(&self, a: &[f32], b: &[f32]) -> f32;
    fn batch_similarities(&self, query: &[f32], candidates: &[Vec<f32>]) -> Vec<f32>;
    fn metric(&self) -> SimilarityMetric;
}
