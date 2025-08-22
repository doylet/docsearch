use std::sync::Arc;
use std::time::Duration;

use zero_latency_core::{Result as ZeroLatencyResult};
use zero_latency_config::Config;

use crate::application::services::CliServiceImpl;
use crate::infrastructure::http::{HttpApiClient};
use crate::infrastructure::output::{TableFormatter};
use crate::infrastructure::config::{FileConfigLoader};

/// Dependency injection container for the CLI application.
/// 
/// This container manages all dependencies and their lifetimes,
/// ensuring proper separation of concerns and testability.
/// 
/// Note: Using concrete types instead of trait objects to avoid 
/// async trait object safety issues.
pub struct CliServiceContainer {
    config: Arc<Config>,
    config_loader: Arc<FileConfigLoader>,
    api_client: Arc<HttpApiClient>,
    output_formatter: Arc<TableFormatter>,
    cli_service: Arc<CliServiceImpl>,
}

impl CliServiceContainer {
    /// Creates a new service container with all dependencies properly wired.
    /// 
    /// # Arguments
    /// * `config` - Application configuration
    /// 
    /// # Returns
    /// * `Result<Self>` - The configured container or an error
    pub async fn new(config: Config) -> ZeroLatencyResult<Self> {
        // Create infrastructure adapters
        let config_loader = Arc::new(FileConfigLoader::new());
        let api_client = Arc::new(HttpApiClient::new(
            config.server_url.clone(),
            Duration::from_secs(30)
        )?);
        let output_formatter = Arc::new(TableFormatter::new());
        
        // Create application services
        let cli_service = Arc::new(CliServiceImpl::new(
            api_client.clone(),
            output_formatter.clone(),
        ));
        
        Ok(Self {
            config: Arc::new(config),
            config_loader,
            api_client,
            output_formatter,
            cli_service,
        })
    }
    
    /// Returns the main CLI service for command execution.
    pub fn cli_service(&self) -> Arc<CliServiceImpl> {
        self.cli_service.clone()
    }
    
    /// Returns the configuration for access by components.
    pub fn config(&self) -> Arc<Config> {
        self.config.clone()
    }
    
    /// Returns the API client for direct access if needed.
    pub fn api_client(&self) -> Arc<HttpApiClient> {
        self.api_client.clone()
    }
    
    /// Returns the output formatter for direct access if needed.
    pub fn output_formatter(&self) -> Arc<TableFormatter> {
        self.output_formatter.clone()
    }
}
