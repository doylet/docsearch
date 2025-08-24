use clap::Args;
use colored::*;

use zero_latency_core::{Result as ZeroLatencyResult};
use crate::application::{CliServiceContainer, IndexCommand as AppIndexCommand};

/// CLI arguments for the index command
#[derive(Args)]
pub struct IndexCommand {
    /// Directory path to index
    pub path: String,
    
    /// Maximum number of files to process
    #[arg(short, long)]
    pub limit: Option<u32>,
    
    /// Batch size for processing
    #[arg(short, long, default_value = "100")]
    pub batch_size: u32,
    
    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl IndexCommand {
    /// Execute the index command using clean architecture pattern.
    /// 
    /// This method delegates to the application service layer,
    /// maintaining separation of concerns between UI and business logic.
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        println!("{}", "Starting document indexing...".bright_blue().bold());
        
        let app_command = AppIndexCommand {
            path: self.path.clone(),
            recursive: false, // Default value
            force: false,     // Default value
        };
        
        container.cli_service().index(app_command).await?;
        
        println!("{}", "Indexing completed successfully!".bright_green().bold());
        
        Ok(())
    }
}
