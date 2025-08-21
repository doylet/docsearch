use clap::{Parser, Subcommand};
use colored::*;

use zero_latency_core::{ZeroLatencyError, Result as ZeroLatencyResult};
use zero_latency_config::Config;

// Clean architecture modules
mod application;
mod infrastructure;
mod commands;

// Legacy modules for gradual migration
mod client;
mod config;
mod output;

use commands::*;

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
    #[arg(long, global = true, default_value = "http://localhost:8081")]
    server: String,
    
    /// Configuration file path
    #[arg(long, global = true)]
    config: Option<std::path::PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Search documents with semantic similarity
    Search(commands::search::SearchCommand),
    
    /// Index documents from a directory
    Index(commands::index::IndexCommand),
    
    /// Show collection statistics and health
    Status(commands::status::StatusCommand),
    
    /// Start the API server
    Server(commands::server::ServerCommand),
    
    /// Rebuild the entire index
    Reindex(commands::reindex::ReindexCommand),
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
        Commands::Status(cmd) => cmd.execute(&container).await,
        Commands::Server(cmd) => cmd.execute(&container).await,
        Commands::Reindex(cmd) => cmd.execute(&container).await,
    };
    
    // Handle errors with user-friendly messages
    if let Err(e) = result {
        eprintln!("{} {}", "❌ Error:".red().bold(), e);
        
        // Add helpful suggestions based on error type
        match &e {
            ZeroLatencyError::Network { message } if message.contains("Connection refused") => {
                eprintln!("{} Try: {}", "💡".yellow(), "mdx server --start".cyan());
            }
            ZeroLatencyError::ExternalService { service: _, message } if message.contains("404") => {
                eprintln!("{} The requested resource was not found", "💡".yellow());
            }
            ZeroLatencyError::Configuration { message } => {
                eprintln!("{} Check your configuration: {}", "💡".yellow(), message);
            }
            _ => {
                eprintln!("{} Check the server status: {}", "💡".yellow(), "mdx status".cyan());
            }
        }
        
        std::process::exit(1);
    }
    
    Ok(())
}

/// Load configuration from various sources with CLI override
async fn load_config(cli: &Cli) -> ZeroLatencyResult<Config> {
    // Create base configuration
    let mut config = Config {
        server_url: cli.server.clone(),
        timeout_seconds: 30,
        max_retries: 3,
        log_level: if cli.verbose { "debug".to_string() } else { "info".to_string() },
        output_format: "table".to_string(),
    };
    
    // Load from config file if specified
    if let Some(_config_path) = &cli.config {
        // TODO: Use FileConfigLoader to load and merge config
        // For now, use CLI values as override
    }
    
    Ok(config)
}
