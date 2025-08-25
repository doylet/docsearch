use std::time::Duration;
use reqwest::Client;
use zero_latency_core::{ZeroLatencyError, Result as ZeroLatencyResult};

/// Status response structure
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub total_documents: u64,
    pub index_size_bytes: u64,
    pub last_index_update: Option<String>,
    pub docs_path: Option<String>,
}

/// Server information response
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ServerInfo {
    pub host: String,
    pub port: u16,
    pub status: String,
    pub message: String,
}

/// HTTP client for server lifecycle operations against the Zero Latency API.
/// 
/// This client is focused solely on server management functionality, following the Single Responsibility Principle.
/// It handles HTTP communication, serialization, and error handling for server-related operations.
pub struct ServerApiClient {
    client: Client,
    base_url: String,
}

impl ServerApiClient {
    /// Creates a new server API client.
    /// 
    /// # Arguments
    /// * `base_url` - The base URL of the Zero Latency API
    /// * `timeout` - Request timeout duration
    pub fn new(base_url: String, timeout: Duration) -> ZeroLatencyResult<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| ZeroLatencyError::Configuration { 
                message: format!("Failed to create HTTP client: {}", e)
            })?;
            
        Ok(Self {
            client,
            base_url,
        })
    }

    /// Get server status information
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
                service: "server_api".to_string(),
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

    /// Start the server with specified configuration
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
                message: format!("Start server request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "server_api".to_string(),
                message: format!("Start server request failed: {}", response.status())
            });
        }
        
        let server_info: ServerInfo = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse start server response: {}", e)
            })?;
            
        Ok(server_info)
    }
}
