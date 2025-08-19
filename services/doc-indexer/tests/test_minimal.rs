// Minimal test to check if the basic runtime works

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Starting minimal test...");
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("✅ Logging initialized");
    
    println!("✅ Minimal test completed successfully!");
    Ok(())
}
