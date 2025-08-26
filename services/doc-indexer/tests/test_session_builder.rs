// Test ONNX SessionBuilder creation without model loading

use anyhow::Result;
use ort::{Environment, ExecutionProvider, LoggingLevel, SessionBuilder};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Testing ONNX SessionBuilder creation...");

    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("✅ Logging initialized");

    println!("🔄 Creating ONNX Environment...");
    let environment = Arc::new(
        Environment::builder()
            .with_name("test-env")
            .with_log_level(LoggingLevel::Warning)
            .build()?,
    );
    println!("Success: ONNX Environment created successfully!");

    println!("🔄 Creating SessionBuilder...");
    let builder = SessionBuilder::new(&environment)?;
    println!("Success: SessionBuilder created successfully!");

    println!("🔄 Setting execution providers...");
    let _builder =
        builder.with_execution_providers([ExecutionProvider::CPU(Default::default())])?;
    println!("Success: Execution providers set successfully!");

    Ok(())
}
