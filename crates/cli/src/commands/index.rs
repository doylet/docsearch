use clap::Args;
use colored::*;

use zero_latency_core::{Result as ZeroLatencyResult};
use crate::application::{CliServiceContainer, IndexCommand as AppIndexCommand};

/// Index documents from a directory for semantic search
/// 
/// This command processes files in the specified directory and creates 
/// searchable embeddings. Supports filtering with glob patterns.
/// 
/// Examples:
///   mdx index ./docs --safe-patterns "*.md" "*.txt"
///   mdx index ./src --ignore-patterns "target" "*.log" --recursive
///   mdx index ./project --clear-default-ignores --force
#[derive(Args)]
pub struct IndexCommand {
    /// Directory or file path to index
    pub path: String,
    
    /// Maximum number of files to process (0 = unlimited)
    #[arg(short, long)]
    pub limit: Option<u32>,
    
    /// Number of files to process in each batch
    #[arg(short, long, default_value = "100")]
    pub batch_size: u32,
    
    /// Enable verbose output showing processing details
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Process subdirectories recursively
    #[arg(short, long, default_value = "true")]
    pub recursive: bool,
    
    /// Overwrite existing documents in the index
    #[arg(short, long)]
    pub force: bool,
    
    /// Only index files matching these glob patterns (allowlist)
    /// Example: --safe-patterns "*.md" "*.txt" "docs/**"
    #[arg(long = "safe-patterns", value_name = "PATTERN")]
    pub safe_patterns: Vec<String>,
    
    /// Skip files matching these glob patterns (denylist)
    /// Example: --ignore-patterns "*.log" "target" ".git"
    #[arg(long = "ignore-patterns", value_name = "PATTERN")]
    pub ignore_patterns: Vec<String>,
    
    /// Disable default ignore patterns (build artifacts, VCS files, etc.)
    #[arg(long)]
    pub clear_default_ignores: bool,
    
    /// Follow symbolic links when traversing directories
    #[arg(long)]
    pub follow_symlinks: bool,
    
    /// Use case-sensitive pattern matching for filters
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
            recursive: self.recursive,
            force: self.force,
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
