use crate::infrastructure::memory::{PooledVector, VectorPool, VectorPoolConfig};
/// Local embeddings adapter
///
/// This adapter provides a simple local implementation of EmbeddingGenerator
/// for testing and development purposes. It creates deterministic embeddings
/// based on text content without requiring external API calls.
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use zero_latency_core::{Result, ZeroLatencyError};
use zero_latency_vector::EmbeddingGenerator;

/// Configuration for local embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalEmbeddingConfig {
    pub dimension: usize,
    pub seed: u64,
    pub enable_vector_pooling: bool,
}

/// Local embeddings adapter that generates deterministic embeddings
/// based on text content using a simple algorithm
pub struct LocalEmbeddingAdapter {
    config: LocalEmbeddingConfig,
    vector_pool: Option<Arc<VectorPool>>,
}

impl LocalEmbeddingAdapter {
    /// Create a new local embedding adapter
    pub fn new(config: LocalEmbeddingConfig) -> Result<Self> {
        if config.dimension == 0 {
            return Err(ZeroLatencyError::configuration(
                "Embedding dimension must be greater than 0",
            ));
        }

        let vector_pool = if config.enable_vector_pooling {
            let pool_config = VectorPoolConfig {
                max_pool_size: 50,
                dimension: config.dimension,
                dimension_tolerance: 16,
            };
            Some(Arc::new(VectorPool::new(pool_config)))
        } else {
            None
        };

        Ok(Self {
            config,
            vector_pool,
        })
    }

    /// Generate embedding for text using deterministic algorithm with optional pooling
    fn generate_deterministic_embedding(&self, text: &str) -> Vec<f32> {
        let mut embedding = if let Some(ref pool) = self.vector_pool {
            // Use pooled vector for better memory efficiency
            let mut pooled = PooledVector::new(pool.clone(), self.config.dimension);
            pooled.resize(self.config.dimension, 0.0);

            // Process the embedding in-place
            self.fill_embedding(&mut pooled, text);

            // Take ownership to return
            pooled.into_inner()
        } else {
            // Traditional allocation
            let mut embedding = Vec::with_capacity(self.config.dimension);
            embedding.resize(self.config.dimension, 0.0);
            self.fill_embedding(&mut embedding, text);
            embedding
        };

        // Normalize the vector to unit length
        self.normalize_vector(&mut embedding);

        embedding
    }

    /// Fill embedding vector with values based on text content
    fn fill_embedding(&self, embedding: &mut [f32], text: &str) {
        // Simple but more predictable embedding based on character frequencies
        // This will make similar texts have more similar embeddings
        let chars: Vec<char> = text.to_lowercase().chars().collect();
        let char_count = chars.len();

        for (i, value) in embedding.iter_mut().enumerate() {
            let mut char_sum = 0.0;

            // Sum character values at positions that map to this dimension
            for (j, &ch) in chars.iter().enumerate() {
                if (j + self.config.seed as usize) % self.config.dimension == i {
                    char_sum += (ch as u32) as f32;
                }
            }

            // Normalize by text length and add some dimension-specific variation
            *value = (char_sum / (char_count.max(1) as f32)) / 1000.0;

            // Add dimension-specific bias based on text hash
            let text_hash = self.hash_text(text);
            let dimension_bias = ((text_hash.wrapping_add(i as u64)) % 1000) as f32 / 10000.0;
            *value += dimension_bias;
        }
    }

    /// Hash text content
    fn hash_text(&self, text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash the full text
        text.hash(&mut hasher);

        // Also hash character bigrams for better sensitivity to text differences
        let chars: Vec<char> = text.chars().collect();
        for window in chars.windows(2) {
            window.hash(&mut hasher);
        }

        // Include word count to differentiate lengths
        text.split_whitespace().count().hash(&mut hasher);

        self.config.seed.hash(&mut hasher);
        hasher.finish()
    }

    /// Create hash with additional seed
    fn hash_with_seed(&self, base_hash: u64, seed: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        base_hash.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish()
    }

    /// Get vector pool statistics if pooling is enabled
    pub fn pool_stats(&self) -> Option<String> {
        self.vector_pool
            .as_ref()
            .map(|pool| pool.stats().to_string())
    }

    /// Normalize vector to unit length
    fn normalize_vector(&self, vector: &mut [f32]) {
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude > 0.0 {
            for value in vector.iter_mut() {
                *value /= magnitude;
            }
        }
    }

    /// Calculate similarity between two texts (for testing)
    pub async fn text_similarity(&self, text1: &str, text2: &str) -> Result<f32> {
        let embedding1 = self.generate_embedding(text1).await?;
        let embedding2 = self.generate_embedding(text2).await?;

        // Calculate cosine similarity
        let dot_product: f32 = embedding1
            .iter()
            .zip(embedding2.iter())
            .map(|(a, b)| a * b)
            .sum();

        // Since vectors are normalized, dot product is the cosine similarity
        Ok(dot_product.max(0.0).min(1.0))
    }
}

#[async_trait]
impl EmbeddingGenerator for LocalEmbeddingAdapter {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        if text.is_empty() {
            return Err(ZeroLatencyError::validation("text", "Text cannot be empty"));
        }

        Ok(self.generate_deterministic_embedding(text))
    }

    async fn generate_batch_embeddings(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut embeddings = Vec::with_capacity(texts.len());

        for text in texts {
            if text.is_empty() {
                return Err(ZeroLatencyError::validation("text", "Text cannot be empty"));
            }

            embeddings.push(self.generate_deterministic_embedding(text));
        }

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }

    fn model_name(&self) -> &str {
        "local-deterministic"
    }
}

impl Default for LocalEmbeddingConfig {
    fn default() -> Self {
        Self {
            dimension: 384,              // Common dimension for smaller models
            seed: 42,                    // Default seed for reproducibility
            enable_vector_pooling: true, // Enable memory optimization by default
        }
    }
}

impl Default for LocalEmbeddingAdapter {
    fn default() -> Self {
        Self::new(LocalEmbeddingConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_generation() {
        let config = LocalEmbeddingConfig {
            dimension: 128,
            seed: 12345,
            enable_vector_pooling: true,
        };

        let adapter = LocalEmbeddingAdapter::new(config).unwrap();

        // Test single embedding
        let embedding = adapter.generate_embedding("Hello world").await.unwrap();
        assert_eq!(embedding.len(), 128);

        // Verify normalization (magnitude should be close to 1)
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_deterministic_embeddings() {
        let adapter = LocalEmbeddingAdapter::default();

        // Same text should produce same embedding
        let text = "Consistent text";
        let embedding1 = adapter.generate_embedding(text).await.unwrap();
        let embedding2 = adapter.generate_embedding(text).await.unwrap();

        assert_eq!(embedding1, embedding2);

        // Different text should produce different embeddings
        let embedding3 = adapter.generate_embedding("Different text").await.unwrap();
        assert_ne!(embedding1, embedding3);
    }

    #[tokio::test]
    async fn test_batch_embeddings() {
        let adapter = LocalEmbeddingAdapter::default();

        let _texts = [
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        // Test batch embedding generation
        let embeddings = adapter.generate_embedding("First text").await.unwrap();
        assert_eq!(embeddings.len(), 384);

        // Verify normalization
        let magnitude: f32 = embeddings.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_empty_text_error() {
        let adapter = LocalEmbeddingAdapter::default();

        // Empty text should return error
        let result = adapter.generate_embedding("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_text_similarity() {
        let adapter = LocalEmbeddingAdapter::default();

        // Similar texts should have higher similarity
        let similarity1 = adapter
            .text_similarity("hello world", "hello world")
            .await
            .unwrap();
        let similarity2 = adapter
            .text_similarity("hello world", "goodbye world")
            .await
            .unwrap();
        let similarity3 = adapter
            .text_similarity("hello world", "completely different")
            .await
            .unwrap();

        // Same text should have similarity of 1.0
        assert!((similarity1 - 1.0).abs() < 0.001);

        // Different texts should have lower similarity
        assert!(similarity2 < similarity1);
        assert!(similarity3 < similarity2);

        // All similarities should be in [0, 1] range
        assert!((0.0..=1.0).contains(&similarity1));
        assert!((0.0..=1.0).contains(&similarity2));
        assert!((0.0..=1.0).contains(&similarity3));
    }

    #[tokio::test]
    async fn test_different_seeds_produce_different_embeddings() {
        let config1 = LocalEmbeddingConfig {
            dimension: 128,
            seed: 1,
            enable_vector_pooling: false,
        };
        let config2 = LocalEmbeddingConfig {
            dimension: 128,
            seed: 2,
            enable_vector_pooling: false,
        };

        let adapter1 = LocalEmbeddingAdapter::new(config1).unwrap();
        let adapter2 = LocalEmbeddingAdapter::new(config2).unwrap();

        let text = "Same text";
        let embedding1 = adapter1.generate_embedding(text).await.unwrap();
        let embedding2 = adapter2.generate_embedding(text).await.unwrap();

        // Different seeds should produce different embeddings
        assert_ne!(embedding1, embedding2);
    }
}
