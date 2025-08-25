use std::sync::Arc;
use std::time::Duration;

use zero_latency_core::{Result as ZeroLatencyResult};
use crate::config::CliConfig;

use crate::application::services::CliServiceImpl;
use crate::infrastructure::http::{
    SearchApiClient, IndexApiClient, DocumentApiClient, 
    CollectionApiClient, ServerApiClient
};
use crate::infrastructure::output::{TableFormatter};
use crate::infrastructure::config::{FileConfigLoader};

/// Dependency injection container for the CLI application.
/// 
/// This container manages all dependencies and their lifetimes,
/// ensuring proper separation of concerns and testability.
/// 
/// Using domain-specific API clients instead of a monolithic client
/// to follow the Single Responsibility Principle and improve maintainability.
pub struct CliServiceContainer {
    config: Arc<CliConfig>,
    config_loader: Arc<FileConfigLoader>,
    search_client: Arc<SearchApiClient>,
    index_client: Arc<IndexApiClient>,
    document_client: Arc<DocumentApiClient>,
    collection_client: Arc<CollectionApiClient>,
    server_client: Arc<ServerApiClient>,
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
    pub async fn new(config: CliConfig) -> ZeroLatencyResult<Self> {
        let timeout = Duration::from_secs(30);
        
        // Create infrastructure adapters
        let config_loader = Arc::new(FileConfigLoader::new());
        
        // Create domain-specific API clients
        let search_client = Arc::new(SearchApiClient::new(
            config.server_url.clone(),
            timeout,
            config.collection_name.clone()
        )?);
        
        let index_client = Arc::new(IndexApiClient::new(
            config.server_url.clone(),
            timeout,
            config.collection_name.clone()
        )?);
        
        let document_client = Arc::new(DocumentApiClient::new(
            config.server_url.clone(),
            timeout,
            config.collection_name.clone()
        )?);
        
        let collection_client = Arc::new(CollectionApiClient::new(
            config.server_url.clone(),
            timeout
        )?);
        
        let server_client = Arc::new(ServerApiClient::new(
            config.server_url.clone(),
            timeout
        )?);
        
        let output_formatter = Arc::new(TableFormatter::new());
        
        // Create application services with domain-specific clients
        let cli_service = Arc::new(CliServiceImpl::new(
            search_client.clone(),
            index_client.clone(),
            server_client.clone(),
            output_formatter.clone(),
        ));
        
        Ok(Self {
            config: Arc::new(config),
            config_loader,
            search_client,
            index_client,
            document_client,
            collection_client,
            server_client,
            output_formatter,
            cli_service,
        })
    }
    
    /// Returns the main CLI service for command execution.
    pub fn cli_service(&self) -> Arc<CliServiceImpl> {
        self.cli_service.clone()
    }
    
    /// Returns the configuration for access by components.
    pub fn config(&self) -> Arc<CliConfig> {
        self.config.clone()
    }
    
    /// Returns the search API client for direct access if needed.
    pub fn search_client(&self) -> Arc<SearchApiClient> {
        self.search_client.clone()
    }
    
    /// Returns the index API client for direct access if needed.
    pub fn index_client(&self) -> Arc<IndexApiClient> {
        self.index_client.clone()
    }
    
    /// Returns the document API client for direct access if needed.
    pub fn document_client(&self) -> Arc<DocumentApiClient> {
        self.document_client.clone()
    }
    
    /// Returns the collection API client for direct access if needed.
    pub fn collection_client(&self) -> Arc<CollectionApiClient> {
        self.collection_client.clone()
    }
    
    /// Returns the server API client for direct access if needed.
    pub fn server_client(&self) -> Arc<ServerApiClient> {
        self.server_client.clone()
    }
    
    /// Returns the output formatter for direct access if needed.
    pub fn output_formatter(&self) -> Arc<TableFormatter> {
        self.output_formatter.clone()
    }
}
