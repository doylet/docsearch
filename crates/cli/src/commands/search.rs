use clap::Args;
use colored::*;

use zero_latency_core::{Result as ZeroLatencyResult};
use crate::application::{CliServiceContainer, SearchCommand as AppSearchCommand};

/// CLI arguments for the search command
#[derive(Args)]
pub struct SearchCommand {
    /// Search query
    pub query: String,
    
    /// Maximum number of results to return
    #[arg(short, long, default_value = "10")]
    pub limit: u32,
    
    /// Output format: table, json, simple
    #[arg(short, long, default_value = "table")]
    pub format: String,
    
    /// Show only the best result
    #[arg(long)]
    pub best: bool,
}

impl SearchCommand {
    /// Execute the search command using clean architecture pattern.
    /// 
    /// This method delegates to the application service layer,
    /// maintaining separation of concerns between UI and business logic.
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        // Show search indicator
        println!("{} Searching for: {}", "".blue(), self.query.cyan().bold());
        
        // Convert CLI args to application command
        let app_command = AppSearchCommand {
            query: self.query.clone(),
            limit: if self.best { 1 } else { self.limit },
            format: self.format.clone(),
            best: self.best,
        };
        
        // Delegate to application service
        container.cli_service().search(app_command).await?;
        
        println!("{}", "Search completed successfully!".bright_green().bold());
        
        Ok(())
    }
}
