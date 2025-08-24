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
    
    /// Safe list patterns - only index files matching these patterns
    /// Can be specified multiple times. If not specified, all files are allowed (subject to ignore list).
    /// Supports glob patterns like *.rs, *.md, etc.
    #[arg(long = "safe", value_name = "PATTERN")]
    pub safe_patterns: Vec<String>,
    
    /// Ignore list patterns - skip files matching these patterns
    /// Can be specified multiple times. These override safe list patterns.
    /// Supports glob patterns like *.log, .git, target, etc.
    #[arg(long = "ignore", value_name = "PATTERN")]
    pub ignore_patterns: Vec<String>,
    
    /// Clear default ignore patterns before applying custom ones
    /// By default, common build artifacts and system files are ignored
    #[arg(long)]
    pub clear_default_ignores: bool,
    
    /// Follow symbolic links during directory traversal
    #[arg(long)]
    pub follow_symlinks: bool,
    
    /// Use case-sensitive pattern matching
    #[arg(long)]
    pub case_sensitive: bool,
}

impl IndexCommand {
    /// Execute the index command using clean architecture pattern.
    /// 
    /// This method delegates to the application service layer,
    /// maintaining separation of concerns between UI and business logic.
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        println!("{}", "Starting document indexing...".bright_blue().bold());
        
        // Display filtering configuration if any custom patterns are specified
        if !self.safe_patterns.is_empty() || !self.ignore_patterns.is_empty() || self.clear_default_ignores {
            println!("{}", "Filtering Configuration:".bright_yellow().bold());
            
            if !self.safe_patterns.is_empty() {
                println!("  Safe list: {}", self.safe_patterns.join(", ").green());
            }
            
            if !self.ignore_patterns.is_empty() {
                println!("  Ignore list: {}", self.ignore_patterns.join(", ").red());
            }
            
            if self.clear_default_ignores {
                println!("  Default ignores: {}", "cleared".yellow());
            } else {
                println!("  Default ignores: {}", "active".cyan());
            }
            
            if self.follow_symlinks {
                println!("  Symlinks: {}", "following".green());
            }
            
            if self.case_sensitive {
                println!("  Case sensitivity: {}", "enabled".cyan());
            }
            
            println!(); // Empty line for better readability
        }
        
        let app_command = AppIndexCommand {
            path: self.path.clone(),
            recursive: false, // Default value
            force: false,     // Default value
            safe_patterns: self.safe_patterns.clone(),
            ignore_patterns: self.ignore_patterns.clone(),
            clear_default_ignores: self.clear_default_ignores,
            follow_symlinks: self.follow_symlinks,
            case_sensitive: self.case_sensitive,
        };
        
        container.cli_service().index(app_command).await?;
        
        println!("{}", "Indexing completed successfully!".bright_green().bold());
        
        Ok(())
    }
}
