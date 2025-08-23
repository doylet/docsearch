use std::time::Duration;
use reqwest::Client;
use serde_json;

use zero_latency_core::{ZeroLatencyError, Result as ZeroLatencyResult, values::SearchQuery};
use zero_latency_search::SearchResponse;

use crate::application::services::cli_service::{IndexCommand, IndexResponse};

/// Status response from the API
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub total_documents: u64,
    pub index_size_bytes: u64,
    pub last_index_update: Option<String>,
}

/// Server information response
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ServerInfo {
    pub host: String,
    pub port: u16,
    pub status: String,
    pub message: String,
}

/// Reindex operation result
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ReindexResult {
    pub status: String,
    pub documents_processed: u64,
    pub processing_time_ms: f64,
    pub errors: Vec<String>,
}

/// HTTP client for communicating with the Zero Latency API.
/// 
/// This adapter provides concrete implementations for all API operations,
/// handling HTTP communication, serialization, and error handling.
pub struct HttpApiClient {
    client: Client,
    base_url: String,
    collection_name: String,
}

impl HttpApiClient {
    /// Creates a new HTTP API client.
    /// 
    /// # Arguments
    /// * `base_url` - The base URL of the Zero Latency API
    /// * `timeout` - Request timeout duration
    /// * `collection_name` - Collection name for vector storage operations
    pub fn new(base_url: String, timeout: Duration, collection_name: String) -> ZeroLatencyResult<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| ZeroLatencyError::Configuration { 
                message: format!("Failed to create HTTP client: {}", e)
            })?;
            
        Ok(Self {
            client,
            base_url,
            collection_name,
        })
    }

    /// Execute a search query against the API
    pub async fn search(&self, query: SearchQuery) -> ZeroLatencyResult<SearchResponse> {
        let url = format!("{}/api/search", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "query": query.effective_query(),
                "limit": 10,
                "collection_name": self.collection_name
            }))
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Search request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "search_api".to_string(),
                message: format!("Search request failed: {}", response.status())
            });
        }
        
        let search_response: SearchResponse = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse search response: {}", e)
            })?;
            
        Ok(search_response)
    }
    
    /// Execute an index operation via the API
    pub async fn index(&self, request: IndexCommand) -> ZeroLatencyResult<IndexResponse> {
        let url = format!("{}/api/index", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "path": request.path,
                "recursive": request.recursive,
                "force": request.force
            }))
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Index request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "index_api".to_string(),
                message: format!("Index request failed: {}", response.status())
            });
        }
        
        let index_response: IndexResponse = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse index response: {}", e)
            })?;
            
        Ok(index_response)
    }
    
    /// Get server status from the API
    pub async fn get_status(&self) -> ZeroLatencyResult<StatusResponse> {
        let url = format!("{}/api/status", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Status request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "status_api".to_string(),
                message: format!("Status request failed: {}", response.status())
            });
        }
        
        let status_response: StatusResponse = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse status response: {}", e)
            })?;
            
        Ok(status_response)
    }
    
    /// Start server via the API
    pub async fn start_server(&self, host: String, port: u16) -> ZeroLatencyResult<ServerInfo> {
        let url = format!("{}/api/server/start", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "host": host,
                "port": port
            }))
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Server start request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "server_api".to_string(),
                message: format!("Server start request failed: {}", response.status())
            });
        }
        
        let server_info: ServerInfo = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse server info: {}", e)
            })?;
            
        Ok(server_info)
    }
    
    /// Trigger reindexing via the API
    pub async fn reindex(&self, force: bool) -> ZeroLatencyResult<ReindexResult> {
        let url = format!("{}/api/reindex", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "force": force
            }))
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Reindex request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "reindex_api".to_string(),
                message: format!("Reindex request failed: {}", response.status())
            });
        }
        
        let reindex_result: ReindexResult = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse reindex result: {}", e)
            })?;
            
        Ok(reindex_result)
    }
}
