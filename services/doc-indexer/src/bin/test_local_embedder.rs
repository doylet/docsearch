// Test binary for LocalEmbedder initialization and embedding generation
#[path = "../model_manager.rs"]
mod model_manager;

#[path = "../embedding_provider.rs"]
mod embedding_provider;

use anyhow::Result;
use tracing::{info, warn};
use crate::embedding_provider::{LocalEmbedder, EmbeddingConfig, EmbeddingProvider};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Testing LocalEmbedder initialization and embedding generation...");

    // Create embedding config
    let config = EmbeddingConfig {
        model: "gte-small".to_string(),
        dimensions: Some(384),
        batch_size: 32,
        max_retries: 3,
        base_delay_ms: 100,
        max_delay_ms: 5000,
        requests_per_minute: 60,
    };

    // Initialize LocalEmbedder
    match LocalEmbedder::new(config).await {
        Ok(embedder) => {
            info!("✅ LocalEmbedder initialized successfully!");
            
            if embedder.is_model_loaded() {
                info!("✅ Model and tokenizer loaded successfully!");
                
                // Test embedding generation
                let test_texts = vec![
                    "This is a test document about machine learning.".to_string(),
                    "Vector embeddings represent text as numerical vectors.".to_string(),
                    "ONNX Runtime enables cross-platform model inference.".to_string(),
                ];
                
                match embedder.generate_embeddings(&test_texts).await {
                    Ok(response) => {
                        info!("✅ Generated embeddings for {} texts", test_texts.len());
                        info!("Model: {}", response.model);
                        info!("Dimension: {}", response.dimension);
                        
                        for (i, embedding_response) in response.embeddings.iter().enumerate() {
                            let embedding = &embedding_response.embedding;
                            info!("Text {}: Generated {}-dimensional embedding", i + 1, embedding.len());
                            
                            // Verify embedding properties
                            let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                            info!("  Magnitude (should be ~1.0 for normalized): {:.4}", magnitude);
                            
                            // Show first few dimensions
                            let preview: Vec<f32> = embedding.iter().take(5).cloned().collect();
                            info!("  First 5 dimensions: {:?}", preview);
                        }
                        
                        info!("✨ LocalEmbedder test completed successfully!");
                    },
                    Err(e) => {
                        warn!("❌ Failed to generate embeddings: {}", e);
                        return Err(e);
                    }
                }
            } else {
                warn!("⚠️ Model not loaded - LocalEmbedder in fallback mode");
            }
        },
        Err(e) => {
            warn!("❌ LocalEmbedder initialization failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
