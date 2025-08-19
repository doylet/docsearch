// Test ONNX SessionBuilder creation without model loading

use anyhow::Result;
use ort::{Environment, SessionBuilder, ExecutionProvider, LoggingLevel};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Testing ONNX SessionBuilder creation...");
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("✅ Logging initialized");
    
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
    
    Ok(())
}
