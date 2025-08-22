use clap::Args;
use colored::*;

use zero_latency_core::{Result as ZeroLatencyResult};
use crate::application::{CliServiceContainer, ReindexCommand as AppReindexCommand};

/// CLI arguments for the reindex command
#[derive(Args)]
pub struct ReindexCommand {
    /// Skip confirmation prompt
    #[arg(short, long)]
    pub yes: bool,
    
    /// Force reindexing even if already running
    #[arg(short, long)]
    pub force: bool,
}

impl ReindexCommand {
    /// Execute the reindex command using clean architecture pattern.
    /// 
    /// This method delegates to the application service layer,
    /// maintaining separation of concerns between UI and business logic.
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        // Confirmation prompt (unless --yes is specified)
        if !self.yes {
            println!("⚠️  This will rebuild the entire index from source documents.");
            print!("Are you sure you want to continue? [y/N]: ");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).map_err(|e| {
                zero_latency_core::ZeroLatencyError::Configuration { 
                    message: format!("Failed to read input: {}", e) 
                }
            })?;
            
            if !input.trim().to_lowercase().starts_with('y') {
                println!("{}", "Reindex cancelled".yellow());
                return Ok(());
            }
        }
        
        println!("{}", "🚀 Starting reindexing...".bright_blue().bold());
        
        let app_command = AppReindexCommand {
            force: self.force,
        };
        
        container.cli_service().reindex(app_command).await?;
        
        println!("{}", "✅ Reindexing completed successfully!".bright_green().bold());
        
        Ok(())
    }
}
