use clap::{Parser, Subcommand};
use anyhow::Result;
use colored::*;

mod commands;
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
}

#[derive(Subcommand)]
enum Commands {
    /// Search documents with semantic similarity
    Search(search::SearchCommand),
    
    /// Index documents from a directory
    Index(index::IndexCommand),
    
    /// Show collection statistics and health
    Status(status::StatusCommand),
    
    /// Start the API server
    Server(server::ServerCommand),
    
    /// Rebuild the entire index
    Reindex(reindex::ReindexCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose {
        "debug"
    } else {
        "info"
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(format!("mdx={},doc_indexer={}", log_level, log_level))
        .with_target(false)
        .without_time()
        .init();
    
    // Create API client
    let client = client::ApiClient::new(cli.server.clone())?;
    
    // Execute command
    let result = match cli.command {
        Commands::Search(cmd) => cmd.execute(&client).await,
        Commands::Index(cmd) => cmd.execute(&client).await,
        Commands::Status(cmd) => cmd.execute(&client).await,
        Commands::Server(cmd) => cmd.execute(&client).await,
        Commands::Reindex(cmd) => cmd.execute(&client).await,
    };
    
    // Handle errors with user-friendly messages
    if let Err(e) = result {
        eprintln!("{} {}", "❌ Error:".red().bold(), e);
        
        // Add helpful suggestions based on error type
        if e.to_string().contains("Connection refused") || e.to_string().contains("network") {
            eprintln!("{} Try: {}", "💡".yellow(), "mdx server --start".cyan());
        } else if e.to_string().contains("404") || e.to_string().contains("Not Found") {
            eprintln!("{} The requested resource was not found", "💡".yellow());
        }
        
        std::process::exit(1);
    }
    
    Ok(())
}
