use clap::Args;
use anyhow::Result;
use crate::client::ApiClient;
use crate::commands::Command;
use crate::output::OutputFormatter;

#[derive(Args)]
pub struct StatusCommand {
    /// Output format: table, json
    #[arg(short, long, default_value = "table")]
    pub format: String,
}

impl Command for StatusCommand {
    async fn execute(&self, client: &ApiClient) -> Result<()> {
        let formatter = OutputFormatter::new(&self.format);
        
        // Check if server is reachable
        if !client.health_check().await? {
            formatter.display_error_message("API server is not reachable")?;
            return Ok(());
        }
        
        // Get status
        let response = client.status().await?;
        
        // Display based on format
        match self.format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&response)?;
                println!("{}", json);
            }
            _ => {
                formatter.display_status(&response)?;
            }
        }
        
        Ok(())
    }
}
