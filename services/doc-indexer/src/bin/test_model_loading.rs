// Test model loading with ONNX SessionBuilder

#[path = "../model_manager.rs"]
mod model_manager;

use anyhow::Result;
use ort::{Environment, SessionBuilder, ExecutionProvider, LoggingLevel};
use std::sync::Arc;
use crate::model_manager::ModelManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Testing ONNX model loading...");
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("✅ Logging initialized");
    
    println!("🔄 Getting model info...");
    let model_manager = ModelManager::new()?;
    let model_info = ModelManager::get_gte_small_info();
    println!("✅ Model info obtained");
    
    println!("🔄 Ensuring model availability...");
    let model_paths = model_manager.ensure_model_available(&model_info).await?;
    println!("✅ Model files ready at: {}", model_paths.onnx_path.display());
    
    println!("🔄 Creating ONNX Environment...");
    let environment = Arc::new(Environment::builder()
        .with_name("test-env")
        .with_log_level(LoggingLevel::Warning)
        .build()?);
    println!("✅ ONNX Environment created successfully!");
    
    println!("🔄 Creating SessionBuilder...");
    let mut builder = SessionBuilder::new(&environment)?;
    println!("✅ SessionBuilder created successfully!");
    
    println!("🔄 Setting execution providers...");
    builder = builder.with_execution_providers([ExecutionProvider::CPU(Default::default())])?;
    println!("✅ Execution providers set successfully!");
    
    println!("🔄 Loading model from file: {}", model_paths.onnx_path.display());
    let _session = builder.with_model_from_file(&model_paths.onnx_path)?;
    println!("✅ Model loaded successfully!");
    
    Ok(())
}
