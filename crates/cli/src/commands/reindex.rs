use clap::Args;
use colored::*;

use zero_latency_core::{Result as ZeroLatencyResult};
use crate::application::{CliServiceContainer, ReindexCommand as AppReindexCommand};

/// Rebuild the document index from the configured source directory
/// 
/// This command clears and rebuilds the entire search index using the 
/// server's configured docs path. Supports the same filtering options as index.
/// 
/// Examples:
///   mdx reindex --safe-patterns "*.md" "*.rst"
///   mdx reindex --ignore-patterns "*.tmp" --yes
///   mdx reindex --clear-default-ignores --case-sensitive
#[derive(Args)]
pub struct ReindexCommand {
    /// Skip confirmation prompt and proceed immediately
    #[arg(short, long)]
    pub yes: bool,
    
    /// Force reindexing even if another operation is in progress
    #[arg(short, long)]
    pub force: bool,
    
    /// Only reindex files matching these glob patterns (allowlist)
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

impl ReindexCommand {
    /// Execute the reindex command using clean architecture pattern.
    /// 
    /// This method delegates to the application service layer,
    /// maintaining separation of concerns between UI and business logic.
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        // Confirmation prompt (unless --yes is specified)
        if !self.yes {
            println!("This will rebuild the entire index from source documents.");
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
        
        println!("{}", "Starting reindexing...".bright_blue().bold());
        
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
        
        let app_command = AppReindexCommand {
            safe_patterns: self.safe_patterns.clone(),
            ignore_patterns: self.ignore_patterns.clone(),
            clear_default_ignores: self.clear_default_ignores,
            follow_symlinks: self.follow_symlinks,
            case_sensitive: self.case_sensitive,
        };
        
        let result = container.cli_service().reindex(app_command).await?;
        
        // Use the index formatter to display results
        container.output_formatter().format_index_results(result).await?;
        
        Ok(())
    }
}
