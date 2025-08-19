// Simple test for LocalEmbedder initialization only
#[path = "../model_manager.rs"]
mod model_manager;

#[path = "../embedding_provider.rs"]
mod embedding_provider;

use anyhow::Result;
use tracing::info;
use crate::embedding_provider::{LocalEmbedder, EmbeddingConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Testing simple LocalEmbedder initialization...");

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

    info!("🔄 Attempting to create LocalEmbedder...");
    
    match LocalEmbedder::new(config).await {
        Ok(embedder) => {
            info!("✅ LocalEmbedder initialized successfully!");
            
            if embedder.is_model_loaded() {
                info!("✅ Model and tokenizer loaded successfully!");
            } else {
                info!("⚠️ Model not loaded - LocalEmbedder in fallback mode");
            }
            
            info!("✨ Simple initialization test completed successfully!");
        },
        Err(e) => {
            info!("❌ LocalEmbedder initialization failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
