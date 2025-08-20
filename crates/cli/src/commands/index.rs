use clap::Args;
use anyhow::Result;
use std::path::PathBuf;
use crate::client::ApiClient;
use crate::commands::Command;
use crate::output::OutputFormatter;

#[derive(Args)]
pub struct IndexCommand {
    /// Path to directory containing documents to index
    pub path: PathBuf,
    
    /// Index directories recursively
    #[arg(short, long)]
    pub recursive: bool,
    
    /// Output format: table, json
    #[arg(short, long, default_value = "table")]
    pub format: String,
}

impl Command for IndexCommand {
    async fn execute(&self, client: &ApiClient) -> Result<()> {
        let formatter = OutputFormatter::new(&self.format);
        
        // Validate path exists
        if !self.path.exists() {
            anyhow::bail!("Path does not exist: {}", self.path.display());
        }
        
        if !self.path.is_dir() {
            anyhow::bail!("Path is not a directory: {}", self.path.display());
        }
        
        // Show indexing start message
        formatter.display_progress_start(&format!(
            "Indexing documents from {} (recursive: {})",
            self.path.display(),
            self.recursive
        ))?;
        
        // Send index request
        let path_str = self.path.to_string_lossy().to_string();
        let response = client.index(&path_str, self.recursive).await?;
        
        // Display results
        match self.format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&response)?;
                println!("{}", json);
            }
            _ => {
                if response.errors.is_empty() {
                    formatter.display_success_message(&format!(
                        "Indexed {} documents ({} chunks) in {}ms",
                        response.indexed_documents,
                        response.total_chunks,
                        response.processing_time_ms
                    ))?;
                } else {
                    formatter.display_warning_message(&format!(
                        "Indexed {} documents with {} errors",
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
