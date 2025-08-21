/// Doc-Indexer service main entry point
/// 
/// This service provides document indexing and search capabilities using
/// a clean architecture with shared domain crates.

use std::sync::Arc;
use anyhow::Result;
use clap::Parser;
use tracing::{info, error};

mod config;
mod application;
mod infrastructure;

use config::Config;
use application::ServiceContainer;
use infrastructure::HttpServer;

#[derive(Parser)]
#[command(name = "doc-indexer")]
#[command(about = "Document indexing and search service")]
struct Cli {
    /// Configuration file path
    #[arg(long)]
    config: Option<String>,

    /// HTTP server port
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Enable structured logging
    #[arg(long)]
    structured_logs: bool,

    /// Print example environment variables
    #[arg(long)]
    env_example: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Print environment example if requested
    if cli.env_example {
        println!("{}", Config::env_example());
        return Ok(());
    }

    // Initialize logging
    init_logging(&cli.log_level, cli.structured_logs);

    info!("Starting doc-indexer service");

    // Load configuration
    let config = load_config(cli.config.as_deref(), cli.port).await?;
    info!("Configuration loaded successfully");

    // Create service container with all dependencies
    let container = match ServiceContainer::new(config.clone()).await {
        Ok(container) => {
            info!("Service container initialized successfully");
            Arc::new(container)
        }
        Err(e) => {
            error!("Failed to initialize service container: {}", e);
            return Err(e.into());
        }
    };

    // Create and start HTTP server
    let server = HttpServer::new(config.server.clone(), container);
    
    info!("Starting HTTP server on {}:{}", config.server.host, config.server.port);
    
    if let Err(e) = server.start().await {
        error!("HTTP server error: {}", e);
        return Err(anyhow::Error::msg(format!("HTTP server error: {}", e)));
    }

    info!("Doc-indexer service stopped");
    Ok(())
}

/// Initialize logging based on configuration
fn init_logging(log_level: &str, structured: bool) {
    use tracing_subscriber::{fmt, EnvFilter};
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("doc_indexer={},zero_latency_core=info,zero_latency_search=info,zero_latency_vector=info,zero_latency_observability=info", log_level)));

    if structured {
        fmt()
            .json()
            .with_env_filter(env_filter)
            .with_target(false)
            .with_current_span(false)
            .init();
    } else {
        fmt()
            .with_env_filter(env_filter)
            .with_target(false)
            .init();
    }
}

/// Load configuration from file or environment
async fn load_config(config_file: Option<&str>, port_override: u16) -> Result<Config> {
    let mut config = if let Some(path) = config_file {
        info!("Loading configuration from file: {}", path);
        Config::from_file(path)?
    } else {
        info!("Loading configuration from environment variables");
        Config::from_env().unwrap_or_else(|e| {
            info!("Failed to load config from environment ({}), using defaults", e);
            Config::default()
        })
    };

    // Override port if provided via CLI
    if port_override != 8080 {
        config.server.port = port_override;
    }

    // Validate configuration
    config.validate()?;

    info!("Configuration validation successful");
    Ok(config)
}
