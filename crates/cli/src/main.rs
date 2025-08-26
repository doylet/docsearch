use clap::{Parser, Subcommand};
use colored::*;

use crate::config::CliConfig;
use zero_latency_core::{Result as ZeroLatencyResult, ZeroLatencyError};

// Clean architecture modules
mod application;
mod commands;
mod infrastructure;

// Legacy modules for gradual migration
mod config;

#[derive(Parser)]
#[command(name = "mdx")]
#[command(about = "Zero Latency Documentation Search CLI")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Zero Latency Team")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// API server URL
    #[arg(long, global = true)]
    server: Option<String>,

    /// Collection name for vector storage
    #[arg(long, global = true)]
    collection: Option<String>,

    /// Configuration file path (or use 'config' command for advanced management)
    #[arg(long, global = true)]
    config: Option<std::path::PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Search documents with semantic similarity
    Search(commands::search::SearchCommand),

    /// Index documents from a directory
    Index(commands::index::IndexCommand),

    /// Document discovery operations (list, get)
    Document(commands::document::DocumentCommand),

    /// Collection management operations (list, get, create, delete, stats)
    Collection(commands::collection::CollectionCommand),

    /// Show collection statistics and health
    Status(commands::status::StatusCommand),

    /// Start the API server
    Server(commands::server::ServerCommand),

    /// Rebuild the entire index
    Reindex(commands::reindex::ReindexCommand),

    /// Configuration management (show, set, export, reset)
    Config(commands::config::ConfigCommand),
}

#[tokio::main]
async fn main() -> ZeroLatencyResult<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(format!("mdx={},doc_indexer={}", log_level, log_level))
        .with_target(false)
        .without_time()
        .init();

    // Load configuration
    let config = load_config(&cli).await?;

    // Create service container with dependency injection
    let container = application::CliServiceContainer::new(config).await?;

    // Execute command using clean architecture
    let result = match cli.command {
        Commands::Search(cmd) => cmd.execute(&container).await,
        Commands::Index(cmd) => cmd.execute(&container).await,
        Commands::Document(cmd) => cmd.execute(&container).await,
        Commands::Collection(cmd) => cmd.execute(&container).await,
        Commands::Status(cmd) => cmd.execute(&container).await,
        Commands::Server(cmd) => cmd.execute(&container).await,
        Commands::Reindex(cmd) => cmd.execute(&container).await,
        Commands::Config(cmd) => cmd.execute(&container).await,
    };

    // Handle errors with user-friendly messages
    if let Err(e) = result {
        eprintln!("Error: {}", e.to_string().red().bold());

        // Provide helpful suggestions based on error type
        match &e {
            ZeroLatencyError::Network { .. } => {
                eprintln!("Tip: Try: {}", "mdx server --start".cyan());
            }
            ZeroLatencyError::NotFound { .. } => {
                eprintln!("Tip: The requested resource was not found");
            }
            ZeroLatencyError::Configuration { message } => {
                eprintln!("Tip: Check your configuration: {}", message);
            }
            _ => {
                eprintln!("Tip: Check the server status: {}", "mdx status".cyan());
            }
        }

        std::process::exit(1);
    }

    Ok(())
}

/// Load configuration from various sources with CLI override
async fn load_config(cli: &Cli) -> ZeroLatencyResult<CliConfig> {
    // Load base configuration from file
    let mut config = if let Some(config_path) = &cli.config {
        // Load from specified config file
        if !config_path.exists() {
            return Err(ZeroLatencyError::not_found(&format!(
                "Config file not found: {}",
                config_path.display()
            )));
        }

        let content = std::fs::read_to_string(config_path).map_err(|e| {
            ZeroLatencyError::configuration(&format!("Failed to read config file: {}", e))
        })?;

        toml::from_str::<CliConfig>(&content).map_err(|e| {
            ZeroLatencyError::configuration(&format!(
                "Invalid config format in {}: {}",
                config_path.display(),
                e
            ))
        })?
    } else {
        // Load from default config location or use defaults
        CliConfig::load().map_err(|e| {
            ZeroLatencyError::configuration(&format!("Failed to load default config: {}", e))
        })?
    };

    // Override with CLI arguments (only if explicitly provided)
    if let Some(server) = &cli.server {
        config.server_url = server.clone();
    }
    if let Some(collection) = &cli.collection {
        config.collection_name = collection.clone();
    }
    if cli.verbose {
        config.verbose = true;
    }

    Ok(config)
}
