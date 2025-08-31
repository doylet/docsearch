use async_trait::async_trait;
/// In-memory vector store adapter
///
/// This adapter provides an in-memory implementation of VectorRepository
/// for testing and development purposes. It's not suitable for production
/// but useful for integration tests and local development.
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use zero_latency_core::{models::HealthStatus, values::Score, Result};
use zero_latency_vector::{
    SimilarityCalculator, SimilarityResult, VectorDocument, VectorRepository,
};

/// In-memory vector store
pub struct InMemoryVectorStore {
    documents: Arc<RwLock<HashMap<String, VectorDocument>>>,
    similarity_calculator: Arc<dyn SimilarityCalculator>,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            similarity_calculator: Arc::new(CosineCalculator),
        }
    }

    /// Create with a custom similarity calculator
    pub fn with_similarity_calculator(calculator: Arc<dyn SimilarityCalculator>) -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            similarity_calculator: calculator,
        }
    }

    /// Get the number of stored documents (for testing)
    pub async fn len(&self) -> usize {
        self.documents.read().await.len()
    }

    /// Check if the store is empty (for testing)
    pub async fn is_empty(&self) -> bool {
        self.documents.read().await.is_empty()
    }

    /// Clear all documents (for testing)
    pub async fn clear(&self) {
        self.documents.write().await.clear();
    }
}

#[async_trait]
impl VectorRepository for InMemoryVectorStore {
    async fn insert(&self, vectors: Vec<VectorDocument>) -> Result<()> {
        let mut docs = self.documents.write().await;
        for document in vectors {
            docs.insert(document.id.to_string(), document);
        }
        Ok(())
    }

    async fn search(&self, query_vector: Vec<f32>, k: usize) -> Result<Vec<SimilarityResult>> {
        let docs = self.documents.read().await;
        let mut results = Vec::new();

        // Calculate similarity for each document
        for document in docs.values() {
            let similarity = self
                .similarity_calculator
                .calculate_similarity(&query_vector, &document.embedding);

            results.push(SimilarityResult {
                document_id: document.id,
                similarity: Score::new(similarity).unwrap_or_else(|_| Score::new(0.0).unwrap()),
                metadata: document.metadata.clone(),
            });
        }

        // Sort by similarity score (descending)
        results.sort_by(|a, b| {
            b.similarity
                .value()
                .partial_cmp(&a.similarity.value())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(k);

        Ok(results)
    }

    async fn search_in_collection(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        k: usize,
    ) -> Result<Vec<SimilarityResult>> {
        // For memory adapter, we'll filter by collection name in metadata
        let docs = self.documents.read().await;
        let mut results = Vec::new();

        // Calculate similarity for each document in the specified collection
        for document in docs.values() {
            // Check if document belongs to the specified collection
            if let Some(doc_collection) = document.metadata.collection.as_ref() {
                if doc_collection != collection_name {
                    continue; // Skip documents not in this collection
                }
            } else if collection_name != "default" {
                continue; // Skip documents without collection if not searching default
            }

            let similarity = self
                .similarity_calculator
                .calculate_similarity(&query_vector, &document.embedding);

            results.push(SimilarityResult {
                document_id: document.id,
                similarity: Score::new(similarity).unwrap_or_else(|_| Score::new(0.0).unwrap()),
                metadata: document.metadata.clone(),
            });
        }

        // Sort by similarity score (descending)
        results.sort_by(|a, b| {
            b.similarity
                .value()
                .partial_cmp(&a.similarity.value())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(k);

        tracing::debug!(
            "MemoryAdapter: Collection-specific search in '{}' returned {} results",
            collection_name,
            results.len()
        );
        Ok(results)
    }

    async fn delete(&self, document_id: &str) -> Result<bool> {
        let mut docs = self.documents.write().await;
        Ok(docs.remove(document_id).is_some())
    }

    async fn update(&self, document_id: &str, vector: Vec<f32>) -> Result<bool> {
        let mut docs = self.documents.write().await;
        if let Some(doc) = docs.get_mut(document_id) {
            doc.embedding = vector;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }

    async fn count(&self) -> Result<usize> {
        Ok(self.documents.read().await.len())
    }
}

/// Simple cosine similarity calculator
struct CosineCalculator;

impl SimilarityCalculator for CosineCalculator {
    fn calculate_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        // Calculate dot product
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

        // Calculate magnitudes
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        // Avoid division by zero
        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        // Calculate cosine similarity
        let similarity = dot_product / (magnitude_a * magnitude_b);

        // Clamp to [0, 1] range (cosine can be negative)
        similarity.max(0.0).min(1.0)
    }

    fn batch_similarities(&self, query: &[f32], candidates: &[Vec<f32>]) -> Vec<f32> {
        candidates
            .iter()
            .map(|candidate| self.calculate_similarity(query, candidate))
            .collect()
    }

    fn metric(&self) -> zero_latency_vector::SimilarityMetric {
        zero_latency_vector::SimilarityMetric::Cosine
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}
