use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, warn};

mod config;
mod document;
mod indexer;
mod vectordb_simple;
mod watcher_v2;
mod vector_db_trait;
mod qdrant_client;
mod chunking;
mod advanced_chunker;
mod embedding_provider;
mod search_service;
mod api_server;

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

    /// Start HTTP API server
    #[arg(long)]
    api_server: bool,

    /// Port for HTTP API server
    #[arg(long, default_value = "3000")]
    api_port: u16,

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
    let mut indexer = DocumentIndexer::new(config.clone()).await?;

    // Perform initial indexing
    info!("Performing initial documentation indexing...");
    indexer.index_all_documents().await?;
    info!("Initial indexing complete");

    if cli.index_only {
        info!("Index-only mode: exiting");
        return Ok(());
    }

    // Start API server if requested
    if cli.api_server {
        info!("Starting API server mode on port {}", cli.api_port);
        
        // Create search service
        use embedding_provider::{EmbeddingConfig, OpenAIEmbedder, MockEmbedder, LocalEmbedder};
        use search_service::SearchService;
        use api_server::ApiServer;
        
        let mut embedding_config = EmbeddingConfig::default();
        // Configure for local gte-small model
        embedding_config.model = "gte-small".to_string();
        embedding_config.dimensions = Some(384);
        
        let embedder: Box<dyn embedding_provider::EmbeddingProvider> = {
            // Try local embedder first (preferred for Step 4)
            match LocalEmbedder::new(embedding_config.clone()) {
                Ok(local_embedder) => {
                    info!("Using local embedding model: gte-small");
                    Box::new(local_embedder)
                }
                Err(e) => {
                    warn!("Failed to initialize local embedder: {}. Falling back to cloud provider.", e);
                    
                    // Fall back to OpenAI if available
                    if let Some(api_key) = config.openai_api_key.clone() {
                        info!("Using OpenAI embeddings as fallback");
                        // Reset config for OpenAI
                        embedding_config.model = "text-embedding-3-small".to_string();
                        Box::new(OpenAIEmbedder::new(api_key, embedding_config)?)
                    } else {
                        info!("No OpenAI API key provided, using mock embedder");
                        Box::new(MockEmbedder::new(embedding_config))
                    }
                }
            }
        };
        
        // Get vector database from indexer
        let vectordb = indexer.create_vectordb_for_search().await?;
        let search_service = SearchService::new(vectordb, embedder);
        
        // Start API server
        let api_server = ApiServer::new(search_service);
        api_server.serve(cli.api_port).await?;
        
        return Ok(());
    }

    // Start watching for changes
    info!("Starting file watcher...");
    indexer.start_watching().await?;

    Ok(())
}
