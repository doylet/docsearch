use clap::Args;
use anyhow::Result;
use colored::*;
use crate::client::ApiClient;
use crate::commands::Command;
use crate::output::OutputFormatter;

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

impl Command for SearchCommand {
    async fn execute(&self, client: &ApiClient) -> Result<()> {
        let formatter = OutputFormatter::new(&self.format);
        
        // Show search indicator
        println!("{} Searching for: {}", "🔍".blue(), self.query.cyan().bold());
        
        // Perform search
        let limit = if self.best { Some(1) } else { Some(self.limit) };
        let response = client.search(&self.query, limit).await?;
        
        // Display results
        if response.results.is_empty() {
            println!("{} No results found for '{}'", "❌".red(), self.query);
            return Ok(());
        }
        
        // Format output based on requested format
        match self.format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&response)?;
                println!("{}", json);
            }
            "simple" => {
                for (i, result) in response.results.iter().enumerate() {
                    println!("{}. {} (score: {:.3})", 
                        i + 1, 
                        result.document_title.green().bold(), 
                        result.score
                    );
                    println!("   {}", result.snippet.dimmed());
                    println!();
                }
            }
            _ => { // table format (default)
                formatter.display_search_results(&response)?;
            }
        }
        
        // Show search summary
        let timing = &response.search_metadata;
        let summary = format!(
            "✅ Found {} results in {}ms (embedding: {}ms, search: {}ms) using {}",
            response.total_results,
            timing.total_time_ms,
            timing.embedding_time_ms,
            timing.search_time_ms,
            timing.model_used.cyan()
        );
        
        if response.total_results > 0 {
            println!("{}", summary.green());
        } else {
            println!("{}", summary.yellow());
        }
        
        Ok(())
    }
}
