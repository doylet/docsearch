// Test binary for model downloading functionality
#[path = "../model_manager.rs"]
mod model_manager;

#[path = "../embedding_provider.rs"]
mod embedding_provider;

use crate::model_manager::ModelManager;
use anyhow::Result;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Testing model download functionality...");

    let model_manager = ModelManager::new()?;
    let model_info = ModelManager::get_gte_small_info();
    
    match model_manager.ensure_model_available(&model_info).await {
        Ok(model_paths) => {
            info!("✅ Model successfully available!");
            info!("ONNX path: {}", model_paths.onnx_path.display());
            info!("Tokenizer path: {}", model_paths.tokenizer_path.display());
            info!("Config path: {}", model_paths.config_path.display());
            
            // Verify files exist and show sizes
            if model_paths.onnx_path.exists() {
                let size = std::fs::metadata(&model_paths.onnx_path)?.len();
                info!("ONNX model file size: {} MB", size / 1024 / 1024);
            }
            
            if model_paths.tokenizer_path.exists() {
                let size = std::fs::metadata(&model_paths.tokenizer_path)?.len();
                info!("Tokenizer file size: {} KB", size / 1024);
            }
            
            if model_paths.config_path.exists() {
                let size = std::fs::metadata(&model_paths.config_path)?.len();
                info!("Config file size: {} bytes", size);
            }
        },
        Err(e) => {
            warn!("❌ Model download failed: {}", e);
            return Err(e);
        }
    }

    info!("✨ Model download test completed successfully!");
    Ok(())
}
