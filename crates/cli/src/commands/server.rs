use clap::Args;
use colored::*;

use zero_latency_core::{Result as ZeroLatencyResult};
use crate::application::{CliServiceContainer, ServerCommand as AppServerCommand};

/// CLI arguments for the server command
#[derive(Args)]
pub struct ServerCommand {
    /// Start the server in the background
    #[arg(long)]
    pub start: bool,
    
    /// Stop the background server
    #[arg(long)]
    pub stop: bool,
    
    /// Show server status
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
    /// This method delegates to the application service layer,
    /// maintaining separation of concerns between UI and business logic.
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        println!("{}", "🚀 Managing server...".bright_blue().bold());
        
        let app_command = AppServerCommand {
            port: self.port,
            host: "localhost".to_string(), // Default value
        };
        
        container.cli_service().server(app_command).await?;
        
        println!("{}", "✅ Server command completed!".bright_green().bold());
        
        Ok(())
    }
}
