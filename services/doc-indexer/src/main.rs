use anyhow::Result;
use clap::Parser;
/// Doc-Indexer service main entry point
///
/// This service provides document indexing and search capabilities using
/// a clean architecture with shared domain crates.
use std::sync::Arc;
use tracing::{error, info};
use zero_latency_config::{load_config_from_file, load_config, AppConfig, validate_config};

mod application;
mod config;
mod infrastructure;

use application::ServiceContainer;
use config::Config;
use infrastructure::HttpServer;

#[derive(Parser)]
#[command(name = "doc-indexer")]
#[command(about = "Document indexing and search service")]
struct Cli {
    /// Configuration file path
    #[arg(long)]
    config: Option<String>,

    /// HTTP server port (overrides config)
    #[arg(long)]
    port: Option<u16>,

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

    /// Path to documentation directory to index (overrides config)
    #[arg(long)]
    docs_path: Option<std::path::PathBuf>,
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

    // Load configuration using zero-latency-config system
    let app_config = load_effective_config(&cli).await?;
    info!("Configuration loaded successfully");

    // Validate configuration
    if let Err(e) = validate_config(&app_config) {
        error!("Configuration validation failed: {}", e);
        return Err(e.into());
    }

    // Convert to service-specific config (compatibility layer)
    let config = Config::from_app_config(app_config);

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
        let app_state = infrastructure::http::handlers::AppState::new_async(container.clone())
            .await
            .map_err(|e| anyhow::Error::msg(format!("Failed to initialize app state: {}", e)))?;

        if cli.batch {
            info!("Starting stdio batch processing mode");
            let batch_processor = infrastructure::stdio::StdioBatchProcessor::new(app_state);
            if let Err(e) = batch_processor.process_batch().await {
                error!("Stdio batch processing error: {}", e);
                return Err(anyhow::Error::msg(format!(
                    "Stdio batch processing error: {}",
                    e
                )));
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

    // Create protocol adapters using the new adapter factory
    let adapter_factory = infrastructure::protocol_adapters::ProtocolAdapterFactory::new(container.clone());
    
    // Create and start HTTP server with protocol adapters
    let server = HttpServer::new_with_adapters(config.server.clone(), adapter_factory)
        .await
        .map_err(|e| anyhow::Error::msg(format!("Failed to create HTTP server: {}", e)))?;

    info!(
        "Starting HTTP server with protocol adapters on {}:{}",
        config.server.host, config.server.port
    );

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
            .with(
                fmt::layer()
                    .json()
                    .with_target(false)
                    .with_current_span(false),
            )
            .init();
    } else {
        // Human-readable logging for development
        subscriber.with(fmt::layer().with_target(false)).init();
    }

    tracing::info!(
        "Tracing initialized with level: {}, structured: {}",
        log_level,
        structured
    );
}

/// Load effective configuration with CLI argument overrides
async fn load_effective_config(cli: &Cli) -> Result<AppConfig> {
    // Load base configuration
    let mut app_config = if let Some(config_file) = &cli.config {
        info!("Loading configuration from file: {}", config_file);
        load_config_from_file(config_file).map_err(|e| {
            anyhow::Error::msg(format!("Failed to load config file: {}", e))
        })?
    } else {
        info!("Loading configuration from environment and defaults");
        load_config().map_err(|e| {
            anyhow::Error::msg(format!("Failed to load configuration: {}", e))
        })?
    };

    // Apply CLI argument overrides
    if let Some(port) = cli.port {
        app_config.server.port = port;
        info!("Port overridden to: {}", port);
    }

    if let Some(docs_path) = &cli.docs_path {
        app_config.server.docs_path = Some(docs_path.to_string_lossy().to_string());
        info!("Docs path overridden to: {:?}", docs_path);
    }

    // Override log level from CLI
    app_config.app.log_level = cli.log_level.clone();

    info!("Configuration: server={}:{}, docs_path={:?}", 
        app_config.server.host, 
        app_config.server.port,
        app_config.server.docs_path
    );

    Ok(app_config)
}
