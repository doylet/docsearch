use colored::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use serde_json;
use std::path::Path;

use zero_latency_core::{Result as ZeroLatencyResult, ZeroLatencyError};
use zero_latency_search::SearchResponse;

use crate::application::services::cli_service::IndexResponse;

/// Table-based output formatter for CLI command results.
///
/// This formatter provides clean tabular output for CLI commands,
/// with support for different output formats and colored text.
pub struct TableFormatter {}

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
    pub async fn format_search_results(
        &self,
        response: SearchResponse,
        format: &str,
    ) -> ZeroLatencyResult<()> {
        match format {
            "json" => {
                let json = serde_json::to_string_pretty(&response).map_err(|e| {
                    ZeroLatencyError::Serialization {
                        message: format!("Failed to serialize response: {}", e),
                    }
                })?;
                println!("{}", json);
            }
            "simple" => {
                if response.results.is_empty() {
                    println!("{}", "No results found.".yellow());
                } else {
                    for (index, result) in response.results.iter().enumerate() {
                        let score = format!("{:.3}", result.final_score.value());
                        println!(
                            "{}. {} {}",
                            (index + 1).to_string().bold(),
                            format!("({})", score).dimmed(),
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
                    table.set_header(vec!["#", "Score", "Content", "Source"]);

                    for (index, result) in response.results.iter().enumerate() {
                        // Format the score to 3 decimal places
                        let score = format!("{:.3}", result.final_score.value());
                        let source = if !result.title.is_empty()
                            && result.title != result.document_path
                        {
                            format!("{} ({})", result.title, result.document_path)
                        } else {
                            result.document_path.clone()
                        };

                        table.add_row(vec![
                            (index + 1).to_string(),
                            score,
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
        table.add_row(vec![
            "Docs Processed".to_string(),
            response.documents_processed.to_string(),
        ]);
        table.add_row(vec![
            "Time (ms)".to_string(),
            response.processing_time_ms.to_string(),
        ]);
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

    /// Format document list
    pub async fn format_document_list(
        &self,
        response: &crate::commands::document::ListDocumentsResponse,
        format: &str,
    ) -> ZeroLatencyResult<()> {
        match format {
            "json" => {
                let json = serde_json::to_string_pretty(response).map_err(|e| {
                    ZeroLatencyError::Serialization {
                        message: format!("Failed to serialize response: {}", e),
                    }
                })?;
                println!("{}", json);
            }
            "simple" => {
                if response.documents.is_empty() {
                    println!("{}", "No documents found.".yellow());
                } else {
                    for doc in &response.documents {
                        println!(
                            "{} - {} ({})",
                            doc.id.bright_blue(),
                            doc.title.white(),
                            doc.path.dimmed()
                        );
                    }
                }
                println!(
                    "\nTotal: {} documents, {} bytes",
                    response.total_count, response.index_size_bytes
                );
            }
            "table" | _ => {
                println!(
                    "{}",
                    format!(
                        "Documents (Page {} of {})",
                        response.page, response.total_pages
                    )
                    .green()
                    .bold()
                );

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
                            format!("...{}", &doc.path[doc.path.len() - 27..])
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

                println!(
                    "\n{} Total: {} documents, {:.2} MB",
                    "".bright_blue(),
                    response.total_count,
                    response.index_size_bytes as f64 / (1024.0 * 1024.0)
                );
            }
        }
        Ok(())
    }

    /// Format document detail
    pub async fn format_document_detail(
        &self,
        response: &crate::commands::document::GetDocumentResponse,
        format: &str,
    ) -> ZeroLatencyResult<()> {
        if !response.found {
            println!("Document '{}' not found", response.id.red());
            return Ok(());
        }

        match format {
            "json" => {
                let json = serde_json::to_string_pretty(response).map_err(|e| {
                    ZeroLatencyError::Serialization {
                        message: format!("Failed to serialize response: {}", e),
                    }
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
    pub async fn format_config(
        &self,
        config: &crate::config::CliConfig,
        config_file_path: &Path,
    ) -> ZeroLatencyResult<()> {
        println!("{}", "Current Configuration".blue().bold());
        println!();

        let mut table = self.create_table();
        table.set_header(vec!["Setting".to_string(), "Value".to_string()]);

        table.add_rows(vec![
            vec!["Server URL".to_string(), config.server_url.clone()],
            vec![
                "Collection Name".to_string(),
                config.collection_name.clone(),
            ],
            vec![
                "Default Limit".to_string(),
                config.default_limit.to_string(),
            ],
            vec!["Output Format".to_string(), config.output_format.clone()],
            vec!["Verbose".to_string(), config.verbose.to_string()],
        ]);

        println!("{}", table);
        println!();
        println!(
            "Config file: {}",
            config_file_path.display().to_string().dimmed()
        );

        Ok(())
    }
}
