// Test tokenizer loading in isolation

#[path = "../model_manager.rs"]
mod model_manager;

use anyhow::Result;
use tokenizers::Tokenizer;
use crate::model_manager::ModelManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Testing tokenizer loading...");
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("✅ Logging initialized");
    
    println!("🔄 Getting model info...");
    let model_manager = ModelManager::new()?;
    let model_info = ModelManager::get_gte_small_info();
    println!("✅ Model info obtained");
    
    println!("🔄 Ensuring model availability...");
    let model_paths = model_manager.ensure_model_available(&model_info).await?;
    println!("✅ Model files ready");
    
    println!("🔄 Loading tokenizer from: {}", model_paths.tokenizer_path.display());
    let _tokenizer = Tokenizer::from_file(&model_paths.tokenizer_path)
        .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;
    println!("✅ Tokenizer loaded successfully!");
    
    Ok(())
}
