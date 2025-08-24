use clap::Args;
use colored::*;
use std::process::{Command, Stdio};

use zero_latency_core::{Result as ZeroLatencyResult, ZeroLatencyError};
use crate::application::{CliServiceContainer, ServerCommand as AppServerCommand};

/// CLI arguments for the server command
#[derive(Args)]
pub struct ServerCommand {
    /// Start the server locally (spawns doc-indexer process)
    #[arg(long)]
    pub start: bool,
    
    /// Start server locally in the background
    #[arg(long)]
    pub start_local: bool,
    
    /// Stop the background server (via API)
    #[arg(long)]
    pub stop: bool,
    
    /// Show server status (via API)
    #[arg(long)]
    pub status: bool,
    
    /// Port to run the server on
    #[arg(short, long, default_value = "8081")]
    pub port: u16,
    
    /// Documentation directory to index
    #[arg(short, long)]
    pub docs: Option<String>,
}

impl ServerCommand {
    /// Execute the server command using clean architecture pattern.
    /// 
    /// This method provides both direct server startup and API-based management.
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        // Handle direct server startup first
        if self.start || self.start_local {
            return self.start_server_directly().await;
        }
        
        // For other operations, use the API-based approach
        if self.status || self.stop {
            println!("{}", "Managing server via API...".bright_blue().bold());
            
            if self.status {
                use crate::application::services::cli_service::StatusCommand;
                
                let status_command = StatusCommand {
                    detailed: false, // You could add a --detailed flag later
                };
                
                // Try to get status, but provide helpful error if server isn't running
                match container.cli_service().status(status_command).await {
                    Ok(_) => {
                        println!("{}", "Server status retrieved successfully!".bright_green().bold());
                    }
                    Err(ZeroLatencyError::Network { .. }) => {
                        println!("{}", "Server is not running".red().bold());
                        println!("{} Start the server with: {}", "Tip:".yellow(), "mdx server --start".cyan());
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                }
            }
            
            if self.stop {
                let app_command = AppServerCommand {
                    port: self.port,
                    host: "localhost".to_string(),
                };
                
                container.cli_service().server(app_command).await?;
                println!("{}", "Server stop command sent!".bright_green().bold());
            }
            
            return Ok(());
        }
        
        // If no flags specified, show help
        println!("{}", "Server Management Commands:".bright_blue().bold());
        println!("  {} - Start server locally", "--start".green());
        println!("  {} - Start server in background", "--start-local".green());
        println!("  {} - Check server status", "--status".yellow());
        println!("  {} - Stop server", "--stop".red());
        println!();
        println!("{} {}", "Example:".bright_blue().bold(), "mdx server --start".cyan());
        
        Ok(())
    }
    
    /// Start the doc-indexer server directly by spawning the process
    async fn start_server_directly(&self) -> ZeroLatencyResult<()> {
        println!("{}", "Starting doc-indexer server...".bright_blue().bold());
        
        // Find the doc-indexer binary
        let binary_path = self.find_doc_indexer_binary()?;
        
        println!("Using binary: {}", binary_path.bright_cyan());
        
        // Build command arguments
        let args = vec![
            "--port".to_string(),
            self.port.to_string(),
        ];
        
        // Note: doc-indexer doesn't take docs path as an argument
        // It uses the default docs location or configuration
        if self.docs.is_some() {
            println!("{} doc-indexer will use its default docs configuration", "ℹ️".blue());
        }
        
        // Start the server
        if self.start_local {
            // Background mode
            println!("{}", "🔄 Starting server in background...".yellow());
            
            let child = Command::new(&binary_path)
                .args(&args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .map_err(|e| ZeroLatencyError::Configuration {
                    message: format!("Failed to start server: {}", e)
                })?;
                
            println!("Server started with PID: {}", child.id().to_string().green());
            println!("Server running on: {}", format!("http://localhost:{}", self.port).cyan());
        } else {
            // Foreground mode
            println!("{}", "Starting server in foreground (Ctrl+C to stop)...".yellow());
            println!("Server will run on: {}", format!("http://localhost:{}", self.port).cyan());
            
            let status = Command::new(&binary_path)
                .args(&args)
                .status()
                .map_err(|e| ZeroLatencyError::Configuration {
                    message: format!("Failed to start server: {}", e)
                })?;
                
            if !status.success() {
                return Err(ZeroLatencyError::Configuration {
                    message: format!("Server exited with code: {:?}", status.code())
                });
            }
        }
        
        Ok(())
    }
    
    /// Find the doc-indexer binary in expected locations
    fn find_doc_indexer_binary(&self) -> ZeroLatencyResult<String> {
        let possible_paths = vec![
            "./target/release/doc-indexer",
            "./target/debug/doc-indexer", 
            "./services/doc-indexer/target/release/doc-indexer",
            "./services/doc-indexer/target/debug/doc-indexer",
            "doc-indexer", // In PATH
        ];
        
        for path in possible_paths {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }
        
        Err(ZeroLatencyError::Configuration {
            message: "Could not find doc-indexer binary. Please build the project first with 'cargo build --release'".to_string()
        })
    }
}
