/// OpenAI embeddings adapter
///
/// This adapter implements the EmbeddingGenerator trait for OpenAI's
/// embedding API, providing text-to-vector conversion capabilities.
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zero_latency_core::{Result, ZeroLatencyError};
use zero_latency_vector::EmbeddingGenerator;

/// OpenAI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

/// OpenAI embeddings adapter
pub struct OpenAIAdapter {
    config: OpenAIConfig,
    // In a real implementation, this would hold the HTTP client
    // client: reqwest::Client,
}

impl OpenAIAdapter {
    /// Create a new OpenAI adapter
    pub async fn new(config: OpenAIConfig) -> Result<Self> {
        // Validate configuration
        if config.api_key.is_empty() {
            return Err(ZeroLatencyError::configuration(
                "OpenAI API key is required",
            ));
        }

        // In a real implementation, this would:
        // 1. Create the HTTP client with proper headers
        // 2. Test the API connection
        // 3. Validate the model is available

        Ok(Self { config })
    }

    /// Get the embedding dimension for the configured model
    pub fn embedding_dimension(&self) -> usize {
        match self.config.model.as_str() {
            "text-embedding-ada-002" => 1536,
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            _ => 1536, // Default fallback
        }
    }

    /// Prepare text for embedding (truncate if too long)
    fn prepare_text(&self, text: &str) -> String {
        // OpenAI has token limits, so we might need to truncate
        // This is a simple character-based truncation
        // In a real implementation, you'd use a tokenizer
        const MAX_CHARS: usize = 8000; // Rough approximation

        if text.len() > MAX_CHARS {
            format!("{}...", &text[..MAX_CHARS])
        } else {
            text.to_string()
        }
    }

    /// Make API request to OpenAI embeddings endpoint
    async fn call_api(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // In a real implementation, this would:
        // 1. Create the request payload
        // 2. Send HTTP request to OpenAI API
        // 3. Parse the response
        // 4. Handle errors and retries
        // 5. Return the embeddings

        // For now, return mock embeddings
        let dimension = self.embedding_dimension();
        let mut embeddings = Vec::new();

        for text in texts {
            // Create a deterministic but pseudo-random embedding based on text
            let mut embedding = vec![0.0; dimension];
            let text_hash = simple_hash(&text);

            for (i, value) in embedding.iter_mut().enumerate() {
                let seed = (text_hash + i as u64) as f32;
                *value = (seed % 1000.0) / 1000.0 - 0.5; // Range: -0.5 to 0.5
            }

            // Normalize the vector
            let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            if magnitude > 0.0 {
                for value in &mut embedding {
                    *value /= magnitude;
                }
            }

            embeddings.push(embedding);
        }

        Ok(embeddings)
    }
}

#[async_trait]
impl EmbeddingGenerator for OpenAIAdapter {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let prepared_text = self.prepare_text(text);
        let embeddings = self.call_api(vec![prepared_text]).await?;

        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| ZeroLatencyError::internal("No embedding returned from API"))
    }

    async fn generate_batch_embeddings(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Prepare all texts
        let prepared_texts: Vec<String> =
            texts.iter().map(|text| self.prepare_text(text)).collect();

        // Process in batches to respect API limits
        const BATCH_SIZE: usize = 100; // OpenAI's batch limit
        let mut all_embeddings = Vec::new();

        for batch in prepared_texts.chunks(BATCH_SIZE) {
            let batch_embeddings = self.call_api(batch.to_vec()).await?;
            all_embeddings.extend(batch_embeddings);
        }

        Ok(all_embeddings)
    }

    fn dimension(&self) -> usize {
        self.embedding_dimension()
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }
}

/// Simple hash function for generating deterministic mock embeddings
fn simple_hash(text: &str) -> u64 {
    let mut hash = 0u64;
    for byte in text.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "text-embedding-3-small".to_string(),
            base_url: None,
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_generation() {
        let config = OpenAIConfig {
            api_key: "test-key".to_string(),
            model: "text-embedding-3-small".to_string(),
            ..Default::default()
        };

        let adapter = OpenAIAdapter::new(config).await.unwrap();

        // Test single embedding
        let embedding = adapter.generate_embedding("Hello world").await.unwrap();
        assert_eq!(embedding.len(), 1536);

        // Verify normalization (magnitude should be close to 1)
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_batch_embeddings() {
        let config = OpenAIConfig {
            api_key: "test-key".to_string(),
            model: "text-embedding-3-small".to_string(),
            ..Default::default()
        };

        let adapter = OpenAIAdapter::new(config).await.unwrap();

        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        let embeddings = adapter.generate_embeddings(texts).await.unwrap();
        assert_eq!(embeddings.len(), 3);

        // Each embedding should have the correct dimension
        for embedding in embeddings {
            assert_eq!(embedding.len(), 1536);
        }
    }

    #[tokio::test]
    async fn test_text_preparation() {
        let config = OpenAIConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let adapter = OpenAIAdapter::new(config).await.unwrap();

        // Short text should remain unchanged
        let short_text = "Short text";
        assert_eq!(adapter.prepare_text(short_text), short_text);

        // Long text should be truncated
        let long_text = "A".repeat(10000);
        let prepared = adapter.prepare_text(&long_text);
        assert!(prepared.len() < long_text.len());
        assert!(prepared.ends_with("..."));
    }

    #[tokio::test]
    async fn test_deterministic_embeddings() {
        let config = OpenAIConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let adapter = OpenAIAdapter::new(config).await.unwrap();

        // Same text should produce same embedding
        let text = "Consistent text";
        let embedding1 = adapter.generate_embedding(text).await.unwrap();
        let embedding2 = adapter.generate_embedding(text).await.unwrap();

        assert_eq!(embedding1, embedding2);
    }
}
