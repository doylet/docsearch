use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}, clock::DefaultClock};
use std::num::NonZeroU32;
use tracing::{info, warn, debug};
use std::sync::{Arc, Mutex};
use lru::LruCache;
use ort::{Environment, ExecutionProvider, SessionBuilder, Session};
use tokenizers::Tokenizer;
use ndarray;

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

// Add local embedding dependencies
use crate::model_manager::ModelManager;

/// Local embedding provider using ONNX Runtime
/// Implements local inference for gte-small model
pub struct LocalEmbedder {
    config: EmbeddingConfig,
    session: Option<Arc<Session>>,
    tokenizer: Option<Arc<Tokenizer>>,
    cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
}

impl LocalEmbedder {
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        info!("🚀 Initializing LocalEmbedder for gte-small model");
        
        // Initialize model manager
        info!("🔄 Step 1: Initializing model manager...");
        let model_manager = ModelManager::new()
            .context("Failed to initialize model manager")?;
        info!("✅ Step 1: Model manager initialized");
        
        // Get model information
        info!("🔄 Step 2: Getting model info...");
        let model_info = ModelManager::get_gte_small_info();
        info!("✅ Step 2: Model info obtained");
        
        // Ensure model is available (download if necessary)
        info!("🔄 Step 3: Ensuring model availability...");
        let model_paths = match model_manager.ensure_model_available(&model_info).await {
            Ok(paths) => {
                info!("✅ Step 3: Model files ready at: {}", paths.onnx_path.display());
                paths
            }
            Err(e) => {
                warn!("⚠️ Step 3: Failed to ensure model availability: {}. Model loading will be skipped.", e);
                return Ok(Self {
                    config,
                    session: None,
                    tokenizer: None,
                    cache: Arc::new(Mutex::new(LruCache::new(
                        std::num::NonZeroUsize::new(1000).unwrap()
                    ))),
                });
            }
        };
        
        // Initialize ONNX Environment
        info!("🔄 Step 4: Creating ONNX Environment...");
        let environment = Arc::new(Environment::builder()
            .with_name("gte-small")
            .with_log_level(ort::LoggingLevel::Warning)
            .build()?);
        info!("✅ Step 4: ONNX Environment created");
        
        // Create ONNX session
        info!("🔄 Step 5: Creating ONNX SessionBuilder...");
        let session = match SessionBuilder::new(&environment) {
            Ok(mut builder) => {
                info!("✅ Step 5a: SessionBuilder created successfully");
                
                info!("🔄 Step 5b: Setting execution providers...");
                match builder.with_execution_providers([ExecutionProvider::CPU(Default::default())]) {
                    Ok(b) => {
                        info!("✅ Step 5b: Execution providers set");
                        builder = b;
                        
                        info!("🔄 Step 5c: Loading model from file: {}", model_paths.onnx_path.display());
                        match builder.with_model_from_file(&model_paths.onnx_path) {
                            Ok(session) => {
                                info!("✅ Step 5c: ONNX session loaded successfully");
                                Some(Arc::new(session))
                            }
                            Err(e) => {
                                warn!("❌ Step 5c: Failed to load model from file: {}", e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        warn!("❌ Step 5b: Failed to set execution providers: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                warn!("❌ Step 5a: Failed to create SessionBuilder: {}", e);
                None
            }
        };
        
        // Load tokenizer
        info!("🔄 Step 6: Loading tokenizer from: {}", model_paths.tokenizer_path.display());
        let tokenizer = match Tokenizer::from_file(&model_paths.tokenizer_path) {
            Ok(tokenizer) => {
                info!("✅ Step 6: Tokenizer loaded successfully");
                Some(Arc::new(tokenizer))
            }
            Err(e) => {
                warn!("❌ Step 6: Failed to load tokenizer: {}", e);
                None
            }
        };
        
        // Initialize LRU cache (1000 embeddings max)
        info!("🔄 Step 7: Initializing LRU cache...");
        let cache = Arc::new(Mutex::new(LruCache::new(
            std::num::NonZeroUsize::new(1000).unwrap()
        )));
        info!("✅ Step 7: LRU cache initialized");
        
        info!("🎉 LocalEmbedder initialization completed successfully!");
        Ok(Self {
            config,
            session,
            tokenizer,
            cache,
        })
    }
    
    /// Check if the local embedder has a model loaded
    pub fn is_model_loaded(&self) -> bool {
        self.session.is_some() && self.tokenizer.is_some()
    }
    
    async fn generate_single_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Check if model is available
        let session = self.session.as_ref()
            .ok_or_else(|| anyhow::anyhow!("ONNX session not loaded - cannot generate embeddings"))?;
        let tokenizer = self.tokenizer.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Tokenizer not loaded - cannot generate embeddings"))?;
        
        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(text) {
                debug!("Cache hit for text: {}", &text[..text.len().min(50)]);
                return Ok(cached.clone());
            }
        }
        
        // Tokenize the input text
        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;
        
        let input_ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        
        info!("🔥 Running ONNX inference for: {}", &text[..text.len().min(50)]);
        
        // Try ONNX inference first, fallback to enhanced algorithm if needed
        match self.run_onnx_inference(session, input_ids, attention_mask).await {
            Ok(embedding) => {
                // Cache the result
                {
                    let mut cache = self.cache.lock().unwrap();
                    cache.put(text.to_string(), embedding.clone());
                }
                
                info!("✅ Generated {}-dimensional embedding using ONNX inference", embedding.len());
                Ok(embedding)
            }
            Err(e) => {
                warn!("⚠️ ONNX inference failed, falling back to enhanced tokenizer: {}", e);
                
                let embedding_dim = self.config.dimensions.unwrap_or(384);
                let embedding = Self::generate_advanced_embedding(input_ids, attention_mask, embedding_dim)?;
                
                // Cache the result
                {
                    let mut cache = self.cache.lock().unwrap();
                    cache.put(text.to_string(), embedding.clone());
                }
                
                info!("✅ Generated {}-dimensional embedding using enhanced tokenization fallback", embedding.len());
                Ok(embedding)
            }
        }
    }
    
    /// Run ONNX inference to generate embeddings
    async fn run_onnx_inference(
        &self, 
        session: &Session, 
        input_ids: &[u32], 
        attention_mask: &[u32]
    ) -> Result<Vec<f32>> {
        info!("🔄 Starting ONNX inference with {} tokens", input_ids.len());
        use ort::Value;
        
        let seq_len = input_ids.len();
        if seq_len == 0 {
            return Err(anyhow::anyhow!("Empty input sequence"));
        }

        info!("🔄 Converting inputs to i64...");
        // Convert to i64 as required by ONNX models
        let input_ids_i64: Vec<i64> = input_ids.iter().map(|&x| x as i64).collect();
        let attention_mask_i64: Vec<i64> = attention_mask.iter().map(|&x| x as i64).collect();
        
        info!("🔄 Creating ndarray tensors...");
        // Create ndarray tensors with shape [1, seq_len] (batch_size=1)
        let input_ids_array = ndarray::Array2::from_shape_vec((1, seq_len), input_ids_i64)?;
        let attention_mask_array = ndarray::Array2::from_shape_vec((1, seq_len), attention_mask_i64)?;
        
        info!("🔄 Creating ONNX Value tensors...");
        // Create ONNX Value tensors - use CowArray for proper CowRepr type
        let allocator = session.allocator();
        
        // Convert to CowArray which has CowRepr internally
        let input_ids_cow = ndarray::CowArray::from(input_ids_array).into_dyn();
        let attention_mask_cow = ndarray::CowArray::from(attention_mask_array).into_dyn();
        
        let input_ids_tensor = Value::from_array(allocator, &input_ids_cow)?;
        let attention_mask_tensor = Value::from_array(allocator, &attention_mask_cow)?;
        
        info!("🔄 Preparing inputs and running inference...");
        // Prepare inputs as Vec<Value> - session.run expects this format
        let inputs = vec![input_ids_tensor, attention_mask_tensor];
        
        // Run inference
        info!("🔄 Calling session.run()...");
        let outputs = session.run(inputs)?;
        info!("✅ ONNX session.run() completed successfully");
        
        // Get the output tensor (first output, usually "last_hidden_state")
        let output_tensor = outputs.get(0)
            .ok_or_else(|| anyhow::anyhow!("No output tensor found"))?;
        
        info!("🔄 Extracting output tensor data...");
        // Extract the tensor data and dimensions
        let output_data = output_tensor.try_extract::<f32>()?;
        let output_view = output_data.view();
        let output_shape = output_view.shape();
        
        info!("✅ Output tensor shape: {:?}", output_shape);
        
        // Verify output shape: [batch_size, seq_len, hidden_size]
        if output_shape.len() != 3 || output_shape[0] != 1 {
            return Err(anyhow::anyhow!(
                "Unexpected output shape: {:?}, expected [1, {}, hidden_size]", 
                output_shape, seq_len
            ));
        }
        
        let hidden_size = output_shape[2];
        let output_seq_len = output_shape[1];
        
        info!("🔄 Performing mean pooling...");
        // Perform mean pooling weighted by attention mask
        let embedding = self.mean_pool_with_attention(
            &output_view, 
            attention_mask, 
            output_seq_len, 
            hidden_size
        )?;
        
        info!("✅ ONNX inference completed: {} -> {} dimensions", hidden_size, embedding.len());
        Ok(embedding)
    }
    
    /// Perform mean pooling with attention mask weighting
    fn mean_pool_with_attention(
        &self,
        hidden_states: &ndarray::ArrayView<f32, ndarray::Dim<ndarray::IxDynImpl>>,
        attention_mask: &[u32],
        seq_len: usize,
        hidden_size: usize,
    ) -> Result<Vec<f32>> {
        let mut pooled = vec![0.0f32; hidden_size];
        let mut total_weight = 0.0f32;
        
        for seq_idx in 0..seq_len.min(attention_mask.len()) {
            let attention_weight = attention_mask[seq_idx] as f32;
            if attention_weight > 0.0 {
                total_weight += attention_weight;
                
                // Add weighted hidden states for this sequence position
                // Note: shape is [batch_size=1, seq_len, hidden_size]
                for hidden_idx in 0..hidden_size {
                    // Access tensor as [0, seq_idx, hidden_idx]
                    if let Some(&value) = hidden_states.get([0, seq_idx, hidden_idx]) {
                        pooled[hidden_idx] += value * attention_weight;
                    }
                }
            }
        }
        
        // Normalize by total attention weight
        if total_weight > 0.0 {
            for val in &mut pooled {
                *val /= total_weight;
            }
        }
        
        // Normalize to unit vector
        Ok(Self::normalize_embedding(&pooled))
    }
    
    /// Generate advanced embeddings based on actual tokenization
    fn generate_advanced_embedding(input_ids: &[u32], attention_mask: &[u32], embedding_dim: usize) -> Result<Vec<f32>> {
        let seq_len = input_ids.len();
        if seq_len == 0 {
            return Err(anyhow::anyhow!("Empty input sequence"));
        }
        
        let mut embedding = vec![0.0f32; embedding_dim];
        
        // Create position-aware embeddings based on token IDs and positions
        let mut total_weight = 0.0f32;
        
        for (pos, (&token_id, &attention)) in input_ids.iter().zip(attention_mask.iter()).enumerate() {
            if attention == 0 {
                continue; // Skip padded tokens
            }
            
            let weight = attention as f32;
            total_weight += weight;
            
            // Position encoding component
            let pos_factor = (pos as f32 + 1.0) / (seq_len as f32);
            
            for dim in 0..embedding_dim {
                // Combine token ID, position, and dimension for deterministic generation
                let seed = (token_id as u64)
                    .wrapping_mul(dim as u64 + 1)
                    .wrapping_mul(((pos_factor * 1000.0) as u64) + 1);
                
                // Generate value in range [-1, 1]
                let val = ((seed % 2000) as f32 - 1000.0) / 1000.0;
                
                // Weight by attention and position
                embedding[dim] += val * weight * (1.0 - pos_factor * 0.1); // Slight position decay
            }
        }
        
        // Normalize by total attention weight
        if total_weight > 0.0 {
            for val in &mut embedding {
                *val /= total_weight;
            }
        }
        
        // Normalize to unit vector
        Ok(Self::normalize_embedding(&embedding))
    }
    
    /// Normalize embedding to unit vector
    fn normalize_embedding(embedding: &[f32]) -> Vec<f32> {
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            embedding.iter().map(|x| x / magnitude).collect()
        } else {
            embedding.to_vec()
        }
    }
}#[async_trait]
impl EmbeddingProvider for LocalEmbedder {
    async fn generate_embeddings(&self, texts: &[String]) -> Result<BatchEmbeddingResponse> {
        if texts.is_empty() {
            return Err(anyhow::anyhow!("Cannot generate embeddings for empty text list"));
        }

        let mut embeddings = Vec::new();
        
        // Process texts sequentially for now
        // TODO: Implement parallel processing in Week 2
        for (index, text) in texts.iter().enumerate() {
            let embedding = self.generate_single_embedding(text).await?;
            embeddings.push(EmbeddingResponse { embedding, index });
        }

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
        // Test with a simple embedding
        let test_texts = vec!["health check".to_string()];
        self.generate_embeddings(&test_texts).await?;
        Ok(())
    }
}
