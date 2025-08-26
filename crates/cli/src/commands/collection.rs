use crate::application::CliServiceContainer;
/// Collection management commands
///
/// This module provides CLI commands for managing vector collections,
/// including creating, listing, getting info, and deleting collections.
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use zero_latency_core::Result as ZeroLatencyResult;

/// Collection management command and subcommands
#[derive(Debug, Args)]
pub struct CollectionCommand {
    #[command(subcommand)]
    pub action: CollectionAction,
}

/// Collection action subcommands
#[derive(Debug, Subcommand)]
pub enum CollectionAction {
    /// List all collections
    List(ListArgs),
    /// Get information about a specific collection
    Get(GetArgs),
    /// Create a new collection
    Create(CreateArgs),
    /// Delete a collection
    Delete(DeleteArgs),
    /// Show collection statistics
    Stats(StatsArgs),
    /// Set the default collection for subsequent operations
    Set(SetArgs),
}

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Output format (table, json, simple)
    #[arg(long, default_value = "table")]
    format: String,
}

#[derive(Debug, Args)]
pub struct GetArgs {
    /// Collection name
    name: String,
    /// Output format (table, json, simple)
    #[arg(long, default_value = "table")]
    format: String,
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Collection name
    name: String,
    /// Vector dimension size
    #[arg(long, default_value = "384")]
    vector_size: u64,
    /// Distance metric (cosine, euclidean, dot)
    #[arg(long, default_value = "cosine")]
    distance_metric: String,
    /// Optional description
    #[arg(long)]
    description: Option<String>,
    /// Skip confirmation prompt
    #[arg(long)]
    yes: bool,
}

#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// Collection name
    name: String,
    /// Skip confirmation prompt
    #[arg(long)]
    yes: bool,
}

#[derive(Debug, Args)]
pub struct StatsArgs {
    /// Collection name
    name: String,
    /// Output format (table, json, simple)
    #[arg(long, default_value = "table")]
    format: String,
}

#[derive(Debug, Args)]
pub struct SetArgs {
    /// Collection name to set as default
    name: String,
}

impl CollectionCommand {
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        match &self.action {
            CollectionAction::List(args) => self.list_collections(container, args).await,
            CollectionAction::Get(args) => self.get_collection(container, args).await,
            CollectionAction::Create(args) => self.create_collection(container, args).await,
            CollectionAction::Delete(args) => self.delete_collection(container, args).await,
            CollectionAction::Stats(args) => self.get_collection_stats(container, args).await,
            CollectionAction::Set(args) => self.set_default_collection(container, args).await,
        }
    }

    async fn list_collections(
        &self,
        container: &CliServiceContainer,
        args: &ListArgs,
    ) -> ZeroLatencyResult<()> {
        let collections = container.collection_client().list_collections().await?;

        match args.format.as_str() {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&collections)?);
            }
            "simple" => {
                for collection in &collections {
                    println!("{}", collection.name);
                }
            }
            _ => {
                // Table format
                println!(
                    "{:<20} {:<10} {:<10} {:<10} {:<20}",
                    "Name", "Vectors", "Size", "Status", "Created"
                );
                println!("{:-<75}", "");

                for collection in &collections {
                    let size_str = format_bytes(collection.size_bytes);
                    let created_str = collection
                        .created_at
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    let status_str = format!("{:?}", collection.status);

                    println!(
                        "{:<20} {:<10} {:<10} {:<10} {:<20}",
                        collection.name, collection.vector_count, size_str, status_str, created_str
                    );
                }

                println!("\nTotal: {} collection(s)", collections.len());
            }
        }

        Ok(())
    }

    async fn get_collection(
        &self,
        container: &CliServiceContainer,
        args: &GetArgs,
    ) -> ZeroLatencyResult<()> {
        let response = container
            .collection_client()
            .get_collection(&args.name)
            .await?;

        if !response.found {
            println!("Collection '{}' not found", args.name);
            return Ok(());
        }

        let collection = response.collection.unwrap();

        match args.format.as_str() {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&collection)?);
            }
            "simple" => {
                println!("{}", collection.name);
            }
            _ => {
                // Table format
                println!("{:<20} {}", "Property", "Value");
                println!("{:-<40}", "");

                println!("{:<20} {}", "Name", collection.name);
                println!("{:<20} {}", "Vector Count", collection.vector_count);
                println!("{:<20} {}", "Size", format_bytes(collection.size_bytes));
                println!("{:<20} {:?}", "Status", collection.status);

                if let Some(vector_size) = collection.vector_size {
                    println!("{:<20} {}", "Vector Size", vector_size);
                }

                if let Some(created) = collection.created_at {
                    println!(
                        "{:<20} {}",
                        "Created",
                        created.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                }

                if let Some(modified) = collection.last_modified {
                    println!(
                        "{:<20} {}",
                        "Last Modified",
                        modified.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                }
            }
        }

        Ok(())
    }

    async fn create_collection(
        &self,
        container: &CliServiceContainer,
        args: &CreateArgs,
    ) -> ZeroLatencyResult<()> {
        if !args.yes {
            println!("Create new collection:");
            println!("  Name: {}", args.name);
            println!("  Vector Size: {}", args.vector_size);
            println!("  Distance Metric: {}", args.distance_metric);
            if let Some(desc) = &args.description {
                println!("  Description: {}", desc);
            }

            print!("Continue? [y/N] ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
                println!("Operation cancelled");
                return Ok(());
            }
        }

        let request = CreateCollectionRequest {
            name: args.name.clone(),
            vector_size: args.vector_size,
            distance_metric: Some(args.distance_metric.clone()),
            description: args.description.clone(),
        };

        let response = container
            .collection_client()
            .create_collection(request)
            .await?;

        if response.success {
            println!("Collection created successfully: {}", response.message);
            println!(
                "Collection '{}' created with {} vector dimensions",
                response.collection.name,
                response.collection.vector_size.unwrap_or(0)
            );
        } else {
            println!("Failed to create collection");
        }

        Ok(())
    }

    async fn delete_collection(
        &self,
        container: &CliServiceContainer,
        args: &DeleteArgs,
    ) -> ZeroLatencyResult<()> {
        if !args.yes {
            println!(
                "WARNING: This will permanently delete collection '{}' and all its data.",
                args.name
            );
            print!("Continue? [y/N] ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
                println!("Operation cancelled");
                return Ok(());
            }
        }

        let response = container
            .collection_client()
            .delete_collection(&args.name)
            .await?;

        if response.success {
            println!("Collection deleted successfully: {}", response.message);
        } else {
            println!("Failed to delete collection: {}", response.message);
        }

        Ok(())
    }

    async fn get_collection_stats(
        &self,
        container: &CliServiceContainer,
        args: &StatsArgs,
    ) -> ZeroLatencyResult<()> {
        let response = container
            .collection_client()
            .get_collection_stats(&args.name)
            .await?;

        if !response.found {
            println!("Collection '{}' not found", args.name);
            return Ok(());
        }

        let stats = response.stats.unwrap();

        match args.format.as_str() {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&stats)?);
            }
            "simple" => {
                println!(
                    "{}: {} vectors, {}",
                    stats.name,
                    stats.vector_count,
                    format_bytes(stats.size_bytes)
                );
            }
            _ => {
                // Table format
                println!("{:<20} {}", "Metric", "Value");
                println!("{:-<40}", "");

                println!("{:<20} {}", "Collection", stats.name);
                println!("{:<20} {}", "Vector Count", stats.vector_count);
                println!("{:<20} {}", "Size", format_bytes(stats.size_bytes));
                println!("{:<20} {:.1}", "Avg Vector Size", stats.average_vector_size);
                println!(
                    "{:<20} {:.1}%",
                    "Index Efficiency",
                    stats.index_efficiency * 100.0
                );

                if let Some(last_indexed) = stats.last_indexed {
                    println!(
                        "{:<20} {}",
                        "Last Indexed",
                        last_indexed.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                }
            }
        }

        Ok(())
    }

    async fn set_default_collection(
        &self,
        _container: &CliServiceContainer,
        args: &SetArgs,
    ) -> ZeroLatencyResult<()> {
        use crate::config::CliConfig;
        use zero_latency_core::ZeroLatencyError;

        let mut config = CliConfig::load().map_err(|e| ZeroLatencyError::Configuration {
            message: format!("Failed to load config: {}", e),
        })?;

        config
            .set_collection(args.name.clone())
            .map_err(|e| ZeroLatencyError::Configuration {
                message: format!("Failed to save config: {}", e),
            })?;

        println!("Default collection set to '{}'", args.name);
        println!(
            "Note: This will be used for subsequent commands unless overridden with --collection"
        );

        // Show the config file location
        if let Ok(config_file) = CliConfig::config_file() {
            println!("Configuration saved to: {}", config_file.display());
        }

        Ok(())
    }
}

/// Helper function to format bytes in human-readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Collection request types for API communication
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub vector_size: u64,
    pub distance_metric: Option<String>,
    pub description: Option<String>,
}

/// Collection response types
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub name: String,
    pub vector_count: u64,
    pub size_bytes: u64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub vector_size: Option<u64>,
    pub status: CollectionStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CollectionStatus {
    Active,
    Indexing,
    Error,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionStats {
    pub name: String,
    pub vector_count: u64,
    pub size_bytes: u64,
    pub average_vector_size: f64,
    pub last_indexed: Option<chrono::DateTime<chrono::Utc>>,
    pub index_efficiency: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCollectionResponse {
    pub found: bool,
    pub collection: Option<CollectionInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCollectionResponse {
    pub success: bool,
    pub collection: CollectionInfo,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteCollectionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCollectionStatsResponse {
    pub found: bool,
    pub stats: Option<CollectionStats>,
}
