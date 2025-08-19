use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}, clock::DefaultClock};
use std::num::NonZeroU32;

/// Response from an embedding API call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
    pub index: usize,
}

/// Batch embedding response
#[derive(Debug, Clone)]
pub struct BatchEmbeddingResponse {
    pub embeddings: Vec<EmbeddingResponse>,
    pub model: String,
    pub dimension: usize,
    pub usage: EmbeddingUsage,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    pub input_tokens: usize,
}

/// Configuration for embedding generation
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub model: String,
    pub dimensions: Option<usize>,
    pub batch_size: usize,
    pub max_retries: usize,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub requests_per_minute: u32,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model: "text-embedding-3-small".to_string(),
            dimensions: Some(384),
            batch_size: 32,
            max_retries: 3,
            base_delay_ms: 200,
            max_delay_ms: 30000,
            requests_per_minute: 60,
        }
    }
}

/// Trait for embedding providers
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings for a batch of texts
    async fn generate_embeddings(&self, texts: &[String]) -> Result<BatchEmbeddingResponse>;
    
    /// Get the configuration used by this provider
    fn config(&self) -> &EmbeddingConfig;
    
    /// Check if the provider is healthy
    async fn health_check(&self) -> Result<()>;
}

/// OpenAI embeddings API client
pub struct OpenAIEmbedder {
    client: Client,
    api_key: String,
    config: EmbeddingConfig,
    rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl OpenAIEmbedder {
    pub fn new(api_key: String, config: EmbeddingConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        // Create rate limiter: requests_per_minute / 60 = requests per second
        let quota = Quota::per_minute(NonZeroU32::new(config.requests_per_minute).unwrap());
        let rate_limiter = RateLimiter::direct(quota);

        Ok(Self {
            client,
            api_key,
            config,
            rate_limiter,
        })
    }

    async fn call_openai_api(&self, texts: &[String]) -> Result<OpenAIResponse> {
        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            input: texts.to_vec(),
            dimensions: self.config.dimensions,
            encoding_format: "float".to_string(),
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "OpenAI API error {}: {}",
                status,
                error_text
            ));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI API response")?;

        Ok(openai_response)
    }

    async fn generate_embeddings_with_retry(&self, texts: &[String]) -> Result<BatchEmbeddingResponse> {
        let mut last_error = None;
        let mut delay_ms = self.config.base_delay_ms;

        for attempt in 0..=self.config.max_retries {
            match self.call_openai_api(texts).await {
                Ok(response) => {
                    let embeddings: Vec<EmbeddingResponse> = response
                        .data
                        .into_iter()
                        .map(|data| EmbeddingResponse {
                            embedding: data.embedding,
                            index: data.index,
                        })
                        .collect();

                    let dimension = embeddings.first().map(|e| e.embedding.len()).unwrap_or(0);

                    return Ok(BatchEmbeddingResponse {
                        embeddings,
                        model: response.model,
                        dimension,
                        usage: EmbeddingUsage {
                            input_tokens: response.usage.total_tokens,
                        },
                    });
                }
                Err(err) => {
                    last_error = Some(err);
                    
                    if attempt < self.config.max_retries {
                        // Add jitter to delay (±25%)
                        let jitter = (delay_ms as f64 * 0.25) as u64;
                        let jittered_delay = delay_ms + (rand::random::<u64>() % (2 * jitter + 1)) - jitter;
                        
                        tokio::time::sleep(Duration::from_millis(jittered_delay)).await;
                        
                        // Exponential backoff with cap
                        delay_ms = std::cmp::min(delay_ms * 2, self.config.max_delay_ms);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error during embedding generation")))
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbedder {
    async fn generate_embeddings(&self, texts: &[String]) -> Result<BatchEmbeddingResponse> {
        if texts.is_empty() {
            return Err(anyhow::anyhow!("Cannot generate embeddings for empty text list"));
        }

        if texts.len() > self.config.batch_size {
            return Err(anyhow::anyhow!(
                "Batch size {} exceeds maximum {}",
                texts.len(),
                self.config.batch_size
            ));
        }

        self.generate_embeddings_with_retry(texts).await
    }

    fn config(&self) -> &EmbeddingConfig {
        &self.config
    }

    async fn health_check(&self) -> Result<()> {
        // Simple health check with a minimal request
        let test_texts = vec!["health check".to_string()];
        self.generate_embeddings(&test_texts).await?;
        Ok(())
    }
}

/// OpenAI API request structure
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<usize>,
    encoding_format: String,
}

/// OpenAI API response structure
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    data: Vec<OpenAIEmbeddingData>,
    model: String,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    total_tokens: usize,
}

/// Mock embedding provider for testing
pub struct MockEmbedder {
    config: EmbeddingConfig,
}

impl MockEmbedder {
    pub fn new(config: EmbeddingConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbedder {
    async fn generate_embeddings(&self, texts: &[String]) -> Result<BatchEmbeddingResponse> {
        let embeddings: Vec<EmbeddingResponse> = texts
            .iter()
            .enumerate()
            .map(|(index, text)| {
                // Generate deterministic mock embeddings based on text hash
                let hash = xxhash_rust::xxh3::xxh3_64(text.as_bytes());
                let mut embedding = vec![0.0f32; self.config.dimensions.unwrap_or(384)];
                
                // Fill with pseudo-random values based on hash
                for (i, val) in embedding.iter_mut().enumerate() {
                    let seed = hash.wrapping_add(i as u64);
                    *val = ((seed % 1000) as f32 - 500.0) / 1000.0; // Range: -0.5 to 0.5
                }
                
                // Normalize to unit vector
                let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                if magnitude > 0.0 {
                    for val in &mut embedding {
                        *val /= magnitude;
                    }
                }

                EmbeddingResponse { embedding, index }
            })
            .collect();

        Ok(BatchEmbeddingResponse {
            embeddings,
            model: self.config.model.clone(),
            dimension: self.config.dimensions.unwrap_or(384),
            usage: EmbeddingUsage {
                input_tokens: texts.iter().map(|t| t.len()).sum::<usize>() / 4, // Rough token estimate
            },
        })
    }

    fn config(&self) -> &EmbeddingConfig {
        &self.config
    }

    async fn health_check(&self) -> Result<()> {
        Ok(()) // Mock always healthy
    }
}

// Add rand dependency for jitter
use rand;
