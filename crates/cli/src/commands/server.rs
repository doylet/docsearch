use clap::Args;
use anyhow::Result;
use crate::client::ApiClient;
use crate::commands::Command;
use crate::output::OutputFormatter;

#[derive(Args)]
pub struct ServerCommand {
    /// Start the server in the background
    #[arg(long)]
    pub start: bool,
    
    /// Stop the background server
    #[arg(long)]
    pub stop: bool,
    
    /// Show server status
    #[arg(long)]
    pub status: bool,
    
    /// Port to run the server on
    #[arg(short, long, default_value = "8081")]
    pub port: u16,
    
    /// Documentation directory to index
    #[arg(short, long)]
    pub docs: Option<String>,
}

impl Command for ServerCommand {
    async fn execute(&self, client: &ApiClient) -> Result<()> {
        let formatter = OutputFormatter::new("table");
        
        if self.status {
            // Show server status
            if client.health_check().await? {
                formatter.display_success_message("API server is running")?;
            } else {
                formatter.display_error_message("API server is not running")?;
            }
            return Ok(());
        }
        
        if self.stop {
            formatter.display_info_message("Server stop functionality not yet implemented")?;
            formatter.display_info_message("Use Ctrl+C to stop the server if running in foreground")?;
            return Ok(());
        }
        
        if self.start {
            formatter.display_info_message("Starting API server...")?;
            
            // Build command to start the doc-indexer server
            let mut cmd = std::process::Command::new("cargo");
            cmd.args(&["run", "--bin", "doc-indexer", "--"]);
            cmd.args(&["--api-server", "--api-port", &self.port.to_string()]);
            
            if let Some(docs_path) = &self.docs {
                cmd.args(&["--docs-path", docs_path]);
            }
            
            // Add default Qdrant URL if not specified
            cmd.args(&["--qdrant-url", "http://localhost:6334"]);
            
            formatter.display_info_message(&format!(
                "Starting server on port {} with docs: {}",
                self.port,
                self.docs.as_deref().unwrap_or("default")
            ))?;
            
            // Execute the command
            let status = cmd.status()?;
            
            if status.success() {
                formatter.display_success_message("Server started successfully")?;
            } else {
                formatter.display_error_message("Failed to start server")?;
            }
            
            return Ok(());
        }
        
        // Default: show usage
        formatter.display_info_message("Server command options:")?;
        println!("  --start    Start the API server");
        println!("  --stop     Stop the API server");
        println!("  --status   Check server status");
        println!("  --port     Specify port (default: 8081)");
        println!("  --docs     Specify docs directory");
        
        Ok(())
    }
}
