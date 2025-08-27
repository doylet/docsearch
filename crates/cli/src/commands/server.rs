use clap::Args;
use colored::*;
use std::process::{Command, Stdio};

use crate::application::{CliServiceContainer, ServerCommand as AppServerCommand};
use zero_latency_core::{Result as ZeroLatencyResult, ZeroLatencyError};
use zero_latency_config::{load_config, AppConfig};

/// CLI arguments for the server command
#[derive(Args)]
pub struct ServerCommand {
    /// Start the server locally (spawns doc-indexer process)
    #[arg(long)]
    pub start: bool,

    /// Start server locally in the background
    #[arg(long)]
    pub start_local: bool,

    /// Stop the background server (via API)
    #[arg(long)]
    pub stop: bool,

    /// Show server status (via API)
    #[arg(long)]
    pub status: bool,

    /// Port to run the server on (overrides config)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Documentation directory to index (overrides config)
    #[arg(short, long)]
    pub docs: Option<String>,

    /// Configuration file path
    #[arg(long)]
    pub config_file: Option<String>,
}

impl ServerCommand {
    /// Execute the server command using clean architecture pattern.
    ///
    /// This method provides both direct server startup and API-based management.
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        // Load configuration with CLI argument overrides
        let config = self.load_effective_config()?;
        
        // Handle direct server startup first
        if self.start || self.start_local {
            return self.start_server_directly(&config).await;
        }

        // For other operations, use the API-based approach
        if self.status || self.stop {
            println!("{}", "Managing server via API...".bright_blue().bold());

            if self.status {
                use crate::application::services::cli_service::StatusCommand;

                let status_command = StatusCommand {};

                // Try to get status, but provide helpful error if server isn't running
                match container.cli_service().status(status_command).await {
                    Ok(_) => {
                        println!(
                            "{}",
                            "Server status retrieved successfully!"
                                .bright_green()
                                .bold()
                        );
                    }
                    Err(ZeroLatencyError::Network { .. }) => {
                        println!("{}", "Server is not running".red().bold());
                        println!(
                            "{} Start the server with: {}",
                            "Tip:".yellow(),
                            "mdx server --start".cyan()
                        );
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                }
            }

            if self.stop {
                let app_command = AppServerCommand {
                    port: config.server.port,
                    host: config.server.host.clone(),
                };

                container.cli_service().start_server(app_command).await?;
                println!("{}", "Server stop command sent!".bright_green().bold());
            }

            return Ok(());
        }

        // If no flags specified, show help
        println!("{}", "Server Management Commands:".bright_blue().bold());
        println!("  {} - Start server locally", "--start".green());
        println!("  {} - Start server in background", "--start-local".green());
        println!("  {} - Check server status", "--status".yellow());
        println!("  {} - Stop server", "--stop".red());
        println!();
        println!(
            "{} {}",
            "Example:".bright_blue().bold(),
            "mdx server --start".cyan()
        );

        Ok(())
    }

    /// Load effective configuration with CLI argument overrides
    fn load_effective_config(&self) -> ZeroLatencyResult<AppConfig> {
        use zero_latency_config::{ConfigResolver, load_config_from_file};
        
        // Load base configuration
        let mut config = if let Some(config_file) = &self.config_file {
            load_config_from_file(config_file).map_err(|e| ZeroLatencyError::Configuration {
                message: format!("Failed to load config file: {}", e),
            })?
        } else {
            load_config().map_err(|e| ZeroLatencyError::Configuration {
                message: format!("Failed to load configuration: {}", e),
            })?
        };
        
        // Apply CLI argument overrides
        if let Some(port) = self.port {
            config.server.port = port;
        }
        
        if let Some(docs_path) = &self.docs {
            config.server.docs_path = Some(docs_path.clone());
        }
        
        Ok(config)
    }

    /// Start the doc-indexer server directly by spawning the process
    async fn start_server_directly(&self, config: &AppConfig) -> ZeroLatencyResult<()> {
        println!("{}", "Starting doc-indexer server...".bright_blue().bold());

        // Find the doc-indexer binary
        let binary_path = self.find_doc_indexer_binary()?;

        println!("Using binary: {}", binary_path.bright_cyan());

        // Build command arguments using configuration
        let mut args = vec!["--port".to_string(), config.server.port.to_string()];

        if let Some(docs_path) = &config.server.docs_path {
            args.push("--docs".to_string());
            args.push(docs_path.clone());
        }

        // Start the server
        if self.start_local {
            // Background mode
            println!("{}", "Starting server in background...".yellow());

            let child = Command::new(&binary_path)
                .args(&args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .map_err(|e| ZeroLatencyError::Configuration {
                    message: format!("Failed to start server: {}", e),
                })?;

            println!(
                "Server started with PID: {}",
                child.id().to_string().green()
            );
            println!(
                "Server running on: {}",
                format!("http://{}:{}", config.server.host, config.server.port).cyan()
            );
        } else {
            // Foreground mode
            println!(
                "{}",
                "Starting server in foreground (Ctrl+C to stop)...".yellow()
            );
            println!(
                "Server will run on: {}",
                format!("http://{}:{}", config.server.host, config.server.port).cyan()
            );

            let status = Command::new(&binary_path)
                .args(&args)
                .status()
                .map_err(|e| ZeroLatencyError::Configuration {
                    message: format!("Failed to start server: {}", e),
                })?;

            if !status.success() {
                return Err(ZeroLatencyError::Configuration {
                    message: format!("Server exited with code: {:?}", status.code()),
                });
            }
        }

        Ok(())
    }

    /// Find the doc-indexer binary in expected locations
    fn find_doc_indexer_binary(&self) -> ZeroLatencyResult<String> {
        let local_paths = vec![
            "./target/release/doc-indexer",
            "./target/debug/doc-indexer",
            "./services/doc-indexer/target/release/doc-indexer",
            "./services/doc-indexer/target/debug/doc-indexer",
        ];

        // Check local paths first
        for path in local_paths {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // Check if doc-indexer is available in PATH by trying to run it with --help
        if let Ok(output) = std::process::Command::new("doc-indexer")
            .arg("--help")
            .output()
        {
            if output.status.success() {
                return Ok("doc-indexer".to_string());
            }
        }

        Err(ZeroLatencyError::Configuration {
            message: "Could not find doc-indexer binary. Please build the project first with 'cargo build --release'".to_string()
        })
    }
}
