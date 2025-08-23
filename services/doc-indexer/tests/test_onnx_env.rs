// Test just ONNX runtime initialization without model loading

use anyhow::Result;
use ort::{Environment, LoggingLevel};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Testing ONNX Environment creation...");
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("✅ Logging initialized");
    
    println!("🔄 Creating ONNX Environment...");
    let environment = Environment::builder()
        .with_name("test-env")
        .with_log_level(LoggingLevel::Warning)
        .build()?;
    println!("✅ ONNX Environment created successfully!");
    
    Ok(())
}
