use std::sync::Arc;

use zero_latency_core::{Result as ZeroLatencyResult, values::SearchQuery};
use zero_latency_search::SearchResponse;

use crate::infrastructure::http::{HttpApiClient};
use crate::infrastructure::output::{TableFormatter};

/// Command data structures for CLI operations
#[derive(Debug, Clone)]
pub struct SearchCommand {
    pub query: String,
    pub limit: u32,
    pub format: String,
    pub best: bool,
}

#[derive(Debug, Clone)]
pub struct IndexCommand {
    pub path: String,
    pub recursive: bool,
    pub force: bool,
}

#[derive(Debug, Clone)]
pub struct StatusCommand {
    pub detailed: bool,
}

#[derive(Debug, Clone)]
pub struct ServerCommand {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone)]
pub struct ReindexCommand {
    pub force: bool,
}

// Temporary response types until we create proper domain models
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct IndexResponse {
    pub documents_processed: u64,
    pub processing_time_ms: f64,
    pub status: String,
    pub message: Option<String>,
}

/// CLI service implementation using concrete types.
/// 
/// This service orchestrates business logic for CLI commands,
/// delegating to infrastructure adapters for I/O operations.
pub struct CliServiceImpl {
    api_client: Arc<HttpApiClient>,
    output_formatter: Arc<TableFormatter>,
}

impl CliServiceImpl {
    /// Creates a new CLI service implementation.
    /// 
    /// # Arguments
    /// * `api_client` - HTTP client for API communication
    /// * `output_formatter` - Formatter for command output
    pub fn new(
        api_client: Arc<HttpApiClient>,
        output_formatter: Arc<TableFormatter>,
    ) -> Self {
        Self {
            api_client,
            output_formatter,
        }
    }
    
    /// Execute a search command
    pub async fn search(&self, request: SearchCommand) -> ZeroLatencyResult<()> {
        // Convert CLI command to domain model
        let search_query = SearchQuery::new(request.query);
        
        // Execute search via API client
        let response = self.api_client.search(search_query).await?;
        
        // Format and display results
        self.output_formatter.format_search_results(response, &request.format).await?;
        
        Ok(())
    }
    
    /// Execute an index command
    pub async fn index(&self, request: IndexCommand) -> ZeroLatencyResult<()> {
        // Execute indexing via API client
        let response = self.api_client.index(request).await?;
        
        // Format and display results
        self.output_formatter.format_index_results(response).await?;
        
        Ok(())
    }
    
    /// Execute a status command
    pub async fn status(&self, request: StatusCommand) -> ZeroLatencyResult<()> {
        // Get server status via API client
        let status = self.api_client.get_status().await?;
        
        // Format and display status
        self.output_formatter.format_status(status, request.detailed).await?;
        
        Ok(())
    }
    
    /// Execute a server command
    pub async fn server(&self, request: ServerCommand) -> ZeroLatencyResult<()> {
        // Start server via API client
        let server_info = self.api_client.start_server(request.host, request.port).await?;
        
        // Format and display server information
        self.output_formatter.format_server_info(server_info).await?;
        
        Ok(())
    }
    
    /// Execute a reindex command
    pub async fn reindex(&self, request: ReindexCommand) -> ZeroLatencyResult<()> {
        // Trigger reindexing via API client
        let result = self.api_client.reindex(request.force).await?;
        
        // Format and display reindexing results
        self.output_formatter.format_reindex_results(result).await?;
        
        Ok(())
    }
}
