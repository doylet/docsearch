use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::{info};

mod config;
mod document;
mod indexer;
mod vectordb_simple;
mod watcher_v2;
mod vector_db_trait;
mod qdrant_client;
mod chunking;
mod advanced_chunker;

use config::Config;
use indexer::DocumentIndexer;

#[derive(Parser)]
#[command(name = "doc-indexer")]
#[command(about = "A daemon that monitors documentation and maintains a vector database")]
struct Cli {
    /// Path to the docs directory to monitor
    #[arg(long, default_value = "./docs")]
    docs_path: PathBuf,

    /// Qdrant server URL
    #[arg(long, default_value = "http://localhost:6333")]
    qdrant_url: String,

    /// Collection name in Qdrant
    #[arg(long, default_value = "zero_latency_docs")]
    collection_name: String,

    /// OpenAI API key for embeddings
    #[arg(long, env = "OPENAI_API_KEY")]
    openai_api_key: Option<String>,

    /// Run initial indexing then exit (don't watch for changes)
    #[arg(long)]
    index_only: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("doc_indexer={},warn", log_level))
        .with_target(false)
        .init();

    info!("Starting Zero Latency Documentation Indexer");
    info!("Monitoring: {}", cli.docs_path.display());
    info!("Vector DB: {} (collection: {})", cli.qdrant_url, cli.collection_name);

    // Validate docs path exists
    if !cli.docs_path.exists() {
        anyhow::bail!("Docs path does not exist: {}", cli.docs_path.display());
    }

    // Create config
    let config = Config {
        docs_directory: cli.docs_path,
        qdrant_url: cli.qdrant_url,
        collection_name: cli.collection_name,
        openai_api_key: cli.openai_api_key,
    };

    // Initialize the indexer
    let indexer = DocumentIndexer::new(config).await?;

    // Perform initial indexing
    info!("Performing initial documentation indexing...");
    indexer.index_all_documents().await?;
    info!("Initial indexing complete");

    if cli.index_only {
        info!("Index-only mode: exiting");
        return Ok(());
    }

    // Start watching for changes
    info!("Starting file watcher...");
    indexer.start_watching().await?;

    Ok(())
}
