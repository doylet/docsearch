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

    /// Enable stdio JSON-RPC transport
    #[arg(long, short = 's')]
    stdio: bool,

    /// Enable batch processing mode for stdio
    #[arg(long, short = 'b')]
    batch: bool,

    /// Print stdio usage information
    #[arg(long)]
    stdio_help: bool,

    /// Path to documentation directory to index
    #[arg(long, default_value = "~/Documents")]
    docs_path: std::path::PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Print environment example if requested
    if cli.env_example {
        println!("{}", Config::env_example());
        return Ok(());
    }

    // Print stdio help if requested
    if cli.stdio_help {
        infrastructure::stdio::utils::print_stdio_usage();
        return Ok(());
    }

    // Initialize logging
    init_logging(&cli.log_level, cli.structured_logs);

    info!("Starting doc-indexer service");

    // Load configuration
    let config = load_config(cli.config.as_deref(), cli.port, &cli.docs_path).await?;
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

    // Check if stdio mode is requested
    if cli.stdio || cli.batch {
        let app_state = infrastructure::http::handlers::AppState::new_async(container.clone()).await
            .map_err(|e| anyhow::Error::msg(format!("Failed to initialize app state: {}", e)))?;

        if cli.batch {
            info!("Starting stdio batch processing mode");
            let batch_processor = infrastructure::stdio::StdioBatchProcessor::new(app_state);
            if let Err(e) = batch_processor.process_batch().await {
                error!("Stdio batch processing error: {}", e);
                return Err(anyhow::Error::msg(format!("Stdio batch processing error: {}", e)));
            }
        } else {
            info!("Starting stdio JSON-RPC mode");
            let stdio_server = infrastructure::stdio::StdioServer::new(app_state);
            if let Err(e) = stdio_server.start().await {
                error!("Stdio server error: {}", e);
                return Err(anyhow::Error::msg(format!("Stdio server error: {}", e)));
            }
        }

        info!("Doc-indexer stdio mode stopped");
        return Ok(());
    }

    // Create and start HTTP server
    let server = HttpServer::new(config.server.clone(), container).await
        .map_err(|e| anyhow::Error::msg(format!("Failed to create HTTP server: {}", e)))?;
    
    info!("Starting HTTP server on {}:{}", config.server.host, config.server.port);
    
    if let Err(e) = server.start().await {
        error!("HTTP server error: {}", e);
        return Err(anyhow::Error::msg(format!("HTTP server error: {}", e)));
    }

    info!("Doc-indexer service stopped");
    Ok(())
}

/// Initialize logging and tracing based on configuration
fn init_logging(log_level: &str, structured: bool) {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("doc_indexer={},zero_latency_core=info,zero_latency_search=info,zero_latency_vector=info,zero_latency_observability=info", log_level)));

    // Create base subscriber
    let subscriber = tracing_subscriber::registry().with(env_filter);

    if structured {
        // Structured JSON logging for production
        subscriber
            .with(fmt::layer().json().with_target(false).with_current_span(false))
            .init();
    } else {
        // Human-readable logging for development
        subscriber
            .with(fmt::layer().with_target(false))
            .init();
    }
    
    tracing::info!("Tracing initialized with level: {}, structured: {}", log_level, structured);
}

/// Load configuration from file or environment
async fn load_config(config_file: Option<&str>, port_override: u16, docs_path: &std::path::Path) -> Result<Config> {
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

    // Override docs_path if provided via CLI (and not using default)
    let default_docs_path = if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join("Documents")
    } else {
        std::path::PathBuf::from("~/Documents")
    };
    
    if docs_path != default_docs_path {
        // Convert to absolute path using the current working directory
        if docs_path.is_absolute() {
            config.service.docs_path = docs_path.to_path_buf();
        } else {
            let absolute_path = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join(docs_path);
            // Try to canonicalize to resolve .. and . components
            config.service.docs_path = match absolute_path.canonicalize() {
                Ok(canonical) => canonical,
                Err(_) => {
                    // If canonicalize fails (e.g., path doesn't exist), clean manually
                    let mut components = Vec::new();
                    for component in absolute_path.components() {
                        match component {
                            std::path::Component::ParentDir => {
                                components.pop();
                            }
                            std::path::Component::CurDir => {
                                // Skip current directory
                            }
                            _ => {
                                components.push(component);
                            }
                        }
                    }
                    components.iter().collect()
                }
            };
        }
    } else {
        // Canonicalize the default path as well
        if config.service.docs_path.is_absolute() {
            // Already absolute, try to canonicalize
            config.service.docs_path = config.service.docs_path.canonicalize()
                .unwrap_or_else(|_| config.service.docs_path.clone());
        } else {
            let absolute_path = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join(&config.service.docs_path);
            config.service.docs_path = absolute_path.canonicalize()
                .unwrap_or_else(|_| absolute_path);
        }
    }

    // Validate configuration
    config.validate()?;

    info!("Configuration validation successful");
    info!("Documentation path: {}", config.service.docs_path.display());
    Ok(config)
}
