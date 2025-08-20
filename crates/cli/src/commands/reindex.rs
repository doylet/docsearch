use clap::Args;
use anyhow::Result;
use crate::client::ApiClient;
use crate::commands::Command;
use crate::output::OutputFormatter;

#[derive(Args)]
pub struct ReindexCommand {
    /// Output format: table, json
    #[arg(short, long, default_value = "table")]
    pub format: String,
    
    /// Skip confirmation prompt
    #[arg(short, long)]
    pub yes: bool,
}

impl Command for ReindexCommand {
    async fn execute(&self, client: &ApiClient) -> Result<()> {
        let formatter = OutputFormatter::new(&self.format);
        
        // Confirmation prompt (unless --yes is specified)
        if !self.yes {
            println!("⚠️  This will rebuild the entire index from source documents.");
            print!("Are you sure you want to continue? [y/N]: ");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if !input.trim().to_lowercase().starts_with('y') {
                formatter.display_info_message("Reindex cancelled")?;
                return Ok(());
            }
        }
        
        // Show reindexing start message
        formatter.display_progress_start("Rebuilding entire index from source documents")?;
        
        // Send reindex request
        let response = client.reindex().await?;
        
        // Display results
        match self.format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&response)?;
                println!("{}", json);
            }
            _ => {
                if response.errors.is_empty() {
                    formatter.display_success_message(&format!(
                        "Reindexed {} documents ({} chunks) in {}ms",
                        response.indexed_documents,
                        response.total_chunks,
                        response.processing_time_ms
                    ))?;
                } else {
                    formatter.display_warning_message(&format!(
                        "Reindexed {} documents with {} errors",
                        response.indexed_documents,
                        response.errors.len()
                    ))?;
                    
                    for error in &response.errors {
                        formatter.display_error_message(error)?;
                    }
                }
            }
        }
        
        Ok(())
    }
}
