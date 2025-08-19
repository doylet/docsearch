// Test combined model and tokenizer loading

#[path = "../model_manager.rs"]
mod model_manager;

use anyhow::Result;
use ort::{Environment, SessionBuilder, ExecutionProvider, LoggingLevel};
use tokenizers::Tokenizer;
use std::sync::Arc;
use crate::model_manager::ModelManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Testing combined model and tokenizer loading...");
    
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
    
    println!("🔄 Creating ONNX Environment...");
    let environment = Arc::new(Environment::builder()
        .with_name("test-env")
        .with_log_level(LoggingLevel::Warning)
        .build()?);
    println!("✅ ONNX Environment created successfully!");
    
    println!("🔄 Creating and loading ONNX session...");
    let mut builder = SessionBuilder::new(&environment)?;
    builder = builder.with_execution_providers([ExecutionProvider::CPU(Default::default())])?;
    let _session = Arc::new(builder.with_model_from_file(&model_paths.onnx_path)?);
    println!("✅ ONNX session loaded successfully!");
    
    println!("🔄 Loading tokenizer...");
    let _tokenizer = Arc::new(Tokenizer::from_file(&model_paths.tokenizer_path)
        .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?);
    println!("✅ Tokenizer loaded successfully!");
    
    println!("🔄 Creating LRU cache...");
    use std::sync::Mutex;
    use lru::LruCache;
    use std::num::NonZeroUsize;
    
    let _cache: Arc<Mutex<LruCache<String, Vec<f32>>>> = Arc::new(Mutex::new(LruCache::new(
        NonZeroUsize::new(1000).unwrap()
    )));
    println!("✅ LRU cache created successfully!");
    
    println!("✅ All components loaded successfully!");
    Ok(())
}
