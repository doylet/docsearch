use clap::Args;
use colored::*;

use zero_latency_core::{Result as ZeroLatencyResult};
use crate::application::{CliServiceContainer, StatusCommand as AppStatusCommand};

/// CLI arguments for the status command
#[derive(Args)]
pub struct StatusCommand {
    /// Show detailed health information
    #[arg(short, long)]
    pub detailed: bool,
    
    /// Output format: table, json, simple
    #[arg(short, long, default_value = "table")]
    pub format: String,
    
    /// Check specific collection
    #[arg(short, long)]
    pub collection: Option<String>,
}

impl StatusCommand {
    /// Execute the status command using clean architecture pattern.
    /// 
    /// This method delegates to the application service layer,
    /// maintaining separation of concerns between UI and business logic.
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        println!("{}", "Checking system status...".bright_blue().bold());
        
        let app_command = AppStatusCommand {
            detailed: self.detailed,
        };
        
        container.cli_service().status(app_command).await?;
        
        println!("{}", "Status check completed!".bright_green().bold());
        
        Ok(())
    }
}
