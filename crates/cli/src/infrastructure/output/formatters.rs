use colored::*;
use comfy_table::{Table, presets::UTF8_FULL};
use serde_json;

use zero_latency_core::{ZeroLatencyError, Result as ZeroLatencyResult};
use zero_latency_search::SearchResponse;

use crate::application::services::cli_service::IndexResponse;
use crate::infrastructure::http::api_client::{StatusResponse, ServerInfo, ReindexResult};

/// Table-based output formatter for CLI command results.
/// 
/// This formatter provides clean tabular output for CLI commands,
/// with support for different output formats and colored text.
pub struct TableFormatter {
}

impl TableFormatter {
    /// Creates a new table formatter
    pub fn new() -> Self {
        Self {}
    }
    
    /// Helper method to create a new table with standard styling
    fn create_table(&self) -> Table {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table
    }
    
    /// Format search results
    pub async fn format_search_results(&self, response: SearchResponse, format: &str) -> ZeroLatencyResult<()> {
        match format {
            "json" => {
                let json = serde_json::to_string_pretty(&response)
                    .map_err(|e| ZeroLatencyError::Serialization { 
                        message: format!("Failed to serialize response: {}", e)
                    })?;
                println!("{}", json);
            }
            "simple" => {
                if response.results.is_empty() {
                    println!("{}", "No results found.".yellow());
                } else {
                    for (index, result) in response.results.iter().enumerate() {
                        println!("{}. {}", 
                            (index + 1).to_string().bold(),
                            result.content.trim()
                        );
                        println!("   Source: {}", result.document_path.dimmed());
                        println!();
                    }
                }
            }
            "table" | _ => {
                if response.results.is_empty() {
                    println!("{}", "No results found.".yellow());
                } else {
                    let mut table = self.create_table();
                    table.set_header(vec!["#", "Content", "Source"]);
                    
                    for (index, result) in response.results.iter().enumerate() {
                        let source = result.document_path.clone();
                        
                        table.add_row(vec![
                            (index + 1).to_string(),
                            result.content.trim().to_string(),
                            source,
                        ]);
                    }
                    
                    println!("{}", table);
                }
            }
        }
        Ok(())
    }
    
    /// Format index results
    pub async fn format_index_results(&self, response: IndexResponse) -> ZeroLatencyResult<()> {
        println!("{}", "Indexing completed successfully!".green().bold());
        
        let mut table = self.create_table();
        table.add_row(vec!["Documents Processed".to_string(), response.documents_processed.to_string()]);
        table.add_row(vec!["Processing Time (ms)".to_string(), response.processing_time_ms.to_string()]);
        table.add_row(vec!["Status".to_string(), response.status]);
        
        if let Some(message) = response.message {
            table.add_row(vec!["Message".to_string(), message]);
        }
        
        println!("{}", table);
        Ok(())
    }
    
    /// Format server status
    pub async fn format_status(&self, status: StatusResponse, detailed: bool) -> ZeroLatencyResult<()> {
        println!("{}", "Server Status".green().bold());
        
        let mut table = self.create_table();
        table.add_row(vec!["Status".to_string(), status.status]);
        table.add_row(vec!["Version".to_string(), status.version]);
        
        if detailed {
            table.add_row(vec!["Uptime (seconds)".to_string(), status.uptime_seconds.to_string()]);
            table.add_row(vec!["Total Documents".to_string(), status.total_documents.to_string()]);
            table.add_row(vec!["Index Size (bytes)".to_string(), status.index_size_bytes.to_string()]);
            
            if let Some(last_update) = status.last_index_update {
                table.add_row(vec!["Last Index Update".to_string(), last_update]);
            }
        }
        
        println!("{}", table);
        Ok(())
    }
    
    /// Format server information
    pub async fn format_server_info(&self, info: ServerInfo) -> ZeroLatencyResult<()> {
        println!("{}", "Server Information".green().bold());
        
        let mut table = self.create_table();
        table.add_row(vec!["Host".to_string(), info.host]);
        table.add_row(vec!["Port".to_string(), info.port.to_string()]);
        table.add_row(vec!["Status".to_string(), info.status]);
        table.add_row(vec!["Message".to_string(), info.message]);
        
        println!("{}", table);
        Ok(())
    }
    
    /// Format reindex results
    pub async fn format_reindex_results(&self, result: ReindexResult) -> ZeroLatencyResult<()> {
        println!("{}", "Reindexing completed!".green().bold());
        
        let mut table = self.create_table();
        table.add_row(vec!["Status".to_string(), result.status]);
        table.add_row(vec!["Documents Processed".to_string(), result.documents_processed.to_string()]);
        table.add_row(vec!["Processing Time (ms)".to_string(), result.processing_time_ms.to_string()]);
        
        if !result.errors.is_empty() {
            table.add_row(vec!["Errors".to_string(), result.errors.join(", ")]);
        }
        
        println!("{}", table);
        Ok(())
    }
}
