use std::sync::Arc;

use zero_latency_core::{Result as ZeroLatencyResult, values::SearchQuery};

use crate::infrastructure::http::{
    SearchApiClient, IndexApiClient, ServerApiClient
};
use crate::infrastructure::output::{TableFormatter};

/// Command data structures for CLI operations
#[derive(Debug, Clone)]
pub struct SearchCommand {
    pub query: String,
    pub limit: u32,
    pub format: String,
}

#[derive(Debug, Clone)]
pub struct IndexCommand {
    pub path: String,
    pub recursive: bool,
    pub force: bool,
    pub safe_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub clear_default_ignores: bool,
    pub follow_symlinks: bool,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone)]
pub struct StatusCommand {
}

#[derive(Debug, Clone)]
pub struct ServerCommand {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone)]
pub struct ReindexCommand {
    pub safe_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub clear_default_ignores: bool,
    pub follow_symlinks: bool,
    pub case_sensitive: bool,
}

// Temporary response types until we create proper domain models
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct IndexResponse {
    pub documents_processed: u64,
    pub processing_time_ms: f64,
    pub status: String,
    pub message: Option<String>,
}

/// CLI service implementation that handles command orchestration,
/// delegating to domain-specific infrastructure adapters for I/O operations.
pub struct CliServiceImpl {
    search_client: Arc<SearchApiClient>,
    index_client: Arc<IndexApiClient>,
    server_client: Arc<ServerApiClient>,
    output_formatter: Arc<TableFormatter>,
}

impl CliServiceImpl {
    /// Creates a new CLI service implementation with domain-specific clients.
    /// 
    /// # Arguments
    /// * `search_client` - HTTP client for search operations
    /// * `index_client` - HTTP client for indexing operations
    /// * `server_client` - HTTP client for server operations
    /// * `output_formatter` - Formatter for command output
    pub fn new(
        search_client: Arc<SearchApiClient>,
        index_client: Arc<IndexApiClient>,
        server_client: Arc<ServerApiClient>,
        output_formatter: Arc<TableFormatter>,
    ) -> Self {
        Self {
            search_client,
            index_client,
            server_client,
            output_formatter,
        }
    }
    
    /// Execute a search command
    pub async fn search(&self, request: SearchCommand) -> ZeroLatencyResult<()> {
        // Convert CLI command to domain model
        let search_query = SearchQuery::new(request.query).with_limit(request.limit);
        
        // Use the search-specific client
        let response = self.search_client.search(search_query).await?;
        
        // Format and display results
        self.output_formatter.format_search_results(response, &request.format).await?;
        Ok(())
    }
    
    /// Execute an index command
    pub async fn index(&self, request: IndexCommand) -> ZeroLatencyResult<IndexResponse> {
        // Use the index-specific client
        let response = self.index_client.index(request).await?;
        
        println!("Indexing completed: {} documents processed", response.documents_processed);
        Ok(response)
    }
    
    /// Execute a status command
    pub async fn status(&self, _request: StatusCommand) -> ZeroLatencyResult<()> {
        // Use the server-specific client
        let status = self.server_client.get_status().await?;
        
        println!("Server Status: {}", status.status);
        println!("Version: {}", status.version);
        println!("Uptime: {} seconds", status.uptime_seconds);
        println!("Total Documents: {}", status.total_documents);
        Ok(())
    }
    
    /// Execute a start server command
    pub async fn start_server(&self, request: ServerCommand) -> ZeroLatencyResult<()> {
        // Use the server-specific client
        let server_info = self.server_client.start_server(request.host, request.port).await?;
        
        println!("Server started: {}", server_info.message);
        Ok(())
    }
    
    /// Execute a reindex command
    pub async fn reindex(&self, command: ReindexCommand) -> ZeroLatencyResult<IndexResponse> {
        // Use the index-specific client
        self.index_client.reindex(command).await
    }
}
