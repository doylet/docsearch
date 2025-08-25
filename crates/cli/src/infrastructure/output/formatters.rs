use colored::*;
use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, ContentArrangement};
use serde_json;
use std::path::Path;

use zero_latency_core::{ZeroLatencyError, Result as ZeroLatencyResult};
use zero_latency_search::SearchResponse;

use crate::application::services::cli_service::IndexResponse;
use crate::infrastructure::http::server_client::{StatusResponse, ServerInfo};

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
        table.apply_modifier(UTF8_ROUND_CORNERS);
        
        // Set column constraints to prevent line wrapping
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_width(80); // Limit total table width to 80 characters
        
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
        table.add_row(vec!["Docs Processed".to_string(), response.documents_processed.to_string()]);
        table.add_row(vec!["Time (ms)".to_string(), response.processing_time_ms.to_string()]);
        table.add_row(vec!["Status".to_string(), response.status]);
        
        if let Some(message) = response.message {
            // Truncate long messages to prevent line wrapping
            let truncated_message = if message.len() > 60 {
                format!("{}...", &message[..57])
            } else {
                message
            };
            table.add_row(vec!["Message".to_string(), truncated_message]);
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
        
        if let Some(docs_path) = &status.docs_path {
            // Convert relative path to absolute path for clarity
            let absolute_path = if Path::new(docs_path).is_absolute() {
                docs_path.clone()
            } else {
                // For relative paths, convert to absolute
                match std::env::current_dir() {
                    Ok(current_dir) => {
                        current_dir.join(docs_path).display().to_string()
                    }
                    Err(_) => docs_path.clone(), // Fallback to original if we can't get current dir
                }
            };
            
            // Truncate long paths
            let truncated_path = if absolute_path.len() > 50 {
                format!("...{}", &absolute_path[absolute_path.len()-47..])
            } else {
                absolute_path
            };
            table.add_row(vec!["Docs Path".to_string(), truncated_path]);
        }
        
        // Always show key operational metrics with shorter labels
        table.add_row(vec!["Documents".to_string(), status.total_documents.to_string()]);
        table.add_row(vec!["Index Size".to_string(), format!("{} bytes", status.index_size_bytes)]);
        table.add_row(vec!["Uptime".to_string(), format!("{}s", status.uptime_seconds)]);
        
        if detailed {
            if let Some(last_update) = status.last_index_update {
                table.add_row(vec!["Last Update".to_string(), last_update]);
            } else {
                table.add_row(vec!["Last Update".to_string(), "Never".to_string()]);
            }
        }
        
        println!("{}", table);
        
        // Add explanatory notes
        if status.total_documents == 0 {
            println!("{}", "Note: No documents indexed yet. The server is configured to index from the docs path above.".yellow());
        }
        
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
    
    /// Format document list
    pub async fn format_document_list(&self, response: &crate::commands::document::ListDocumentsResponse, format: &str) -> ZeroLatencyResult<()> {
        match format {
            "json" => {
                let json = serde_json::to_string_pretty(response)
                    .map_err(|e| ZeroLatencyError::Serialization { 
                        message: format!("Failed to serialize response: {}", e)
                    })?;
                println!("{}", json);
            }
            "simple" => {
                if response.documents.is_empty() {
                    println!("{}", "No documents found.".yellow());
                } else {
                    for doc in &response.documents {
                        println!("{} - {} ({})", 
                            doc.id.bright_blue(), 
                            doc.title.white(),
                            doc.path.dimmed()
                        );
                    }
                }
                println!("\nTotal: {} documents, {} bytes", response.total_count, response.index_size_bytes);
            }
            "table" | _ => {
                println!("{}", format!("Documents (Page {} of {})", response.page, response.total_pages).green().bold());
                
                if response.documents.is_empty() {
                    println!("{}", "No documents found.".yellow());
                } else {
                    let mut table = self.create_table();
                    table.set_header(vec!["ID", "Title", "Path", "Size", "Modified"]);
                    
                    for doc in &response.documents {
                        let truncated_id = if doc.id.len() > 8 { 
                            format!("{}...", &doc.id[..8]) 
                        } else { 
                            doc.id.clone() 
                        };
                        let truncated_title = if doc.title.len() > 25 { 
                            format!("{}...", &doc.title[..22]) 
                        } else { 
                            doc.title.clone() 
                        };
                        let truncated_path = if doc.path.len() > 30 { 
                            format!("...{}", &doc.path[doc.path.len()-27..]) 
                        } else { 
                            doc.path.clone() 
                        };
                        
                        table.add_row(vec![
                            truncated_id,
                            truncated_title,
                            truncated_path,
                            format!("{} B", doc.size),
                            doc.last_modified.clone(),
                        ]);
                    }
                    
                    println!("{}", table);
                }
                
                println!("\n{} Total: {} documents, {:.2} MB", 
                    "".bright_blue(),
                    response.total_count, 
                    response.index_size_bytes as f64 / (1024.0 * 1024.0)
                );
            }
        }
        Ok(())
    }
    
    /// Format document detail
    pub async fn format_document_detail(&self, response: &crate::commands::document::GetDocumentResponse, format: &str) -> ZeroLatencyResult<()> {
        if !response.found {
            println!("Document '{}' not found", response.id.red());
            return Ok(());
        }
        
        match format {
            "json" => {
                let json = serde_json::to_string_pretty(response)
                    .map_err(|e| ZeroLatencyError::Serialization { 
                        message: format!("Failed to serialize response: {}", e)
                    })?;
                println!("{}", json);
            }
            "metadata" => {
                println!("{}", format!("Document {}", response.id).green().bold());
                if let Some(metadata) = &response.metadata {
                    let formatted = serde_json::to_string_pretty(metadata)
                        .unwrap_or_else(|_| "Invalid metadata".to_string());
                    println!("{}", formatted);
                } else {
                    println!("{}", "No metadata available".yellow());
                }
            }
            "content" | _ => {
                println!("{}", format!("Document {}", response.id).green().bold());
                if let Some(content) = &response.content {
                    println!("{}", content);
                } else {
                    println!("{}", "No content available".yellow());
                }
            }
        }
        Ok(())
    }
    
    /// Format configuration display
    pub async fn format_config(&self, config: &crate::config::CliConfig, config_file_path: &Path) -> ZeroLatencyResult<()> {
        println!("{}", "Current Configuration".blue().bold());
        println!();
        
        let mut table = self.create_table();
        table.set_header(vec![
            "Setting".to_string(),
            "Value".to_string()
        ]);
        
        table.add_rows(vec![
            vec!["Server URL".to_string(), config.server_url.clone()],
            vec!["Collection Name".to_string(), config.collection_name.clone()],
            vec!["Default Limit".to_string(), config.default_limit.to_string()],
            vec!["Output Format".to_string(), config.output_format.clone()],
            vec!["Verbose".to_string(), config.verbose.to_string()],
        ]);
        
        println!("{}", table);
        println!();
        println!("Config file: {}", config_file_path.display().to_string().dimmed());
        
        Ok(())
    }
}
