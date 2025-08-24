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
                "limit": query.limit,
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
    
    /// Index documents from a path
    pub async fn index(&self, request: IndexCommand) -> ZeroLatencyResult<IndexResponse> {
        let mut json_body = serde_json::json!({
            "path": request.path,
            "recursive": request.recursive,
            "force": request.force
        });
        
        // Add filtering parameters if present
        if !request.safe_patterns.is_empty() {
            json_body["safe_patterns"] = serde_json::Value::Array(
                request.safe_patterns.into_iter().map(serde_json::Value::String).collect()
            );
        }
        
        if !request.ignore_patterns.is_empty() {
            json_body["ignore_patterns"] = serde_json::Value::Array(
                request.ignore_patterns.into_iter().map(serde_json::Value::String).collect()
            );
        }
        
        if request.clear_default_ignores {
            json_body["clear_default_ignores"] = serde_json::Value::Bool(request.clear_default_ignores);
        }
        
        if request.follow_symlinks {
            json_body["follow_symlinks"] = serde_json::Value::Bool(request.follow_symlinks);
        }
        
        if request.case_sensitive {
            json_body["case_sensitive"] = serde_json::Value::Bool(request.case_sensitive);
        }

        let response = self.client
            .post(&format!("{}/api/index", self.base_url))
            .json(&json_body)
            .send()
            .await
            .map_err(|e| zero_latency_core::ZeroLatencyError::network(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            let index_response: IndexResponse = response
                .json()
                .await
                .map_err(|e| zero_latency_core::ZeroLatencyError::serialization(format!("Failed to parse response: {}", e)))?;
            Ok(index_response)
        } else {
            let error_text = response
                .text()
                .await
                .map_err(|e| zero_latency_core::ZeroLatencyError::network(format!("Failed to read error response: {}", e)))?;
            Err(zero_latency_core::ZeroLatencyError::external_service("doc-indexer", format!("Index failed: {}", error_text)))
        }
    }    /// Get server status from the API
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
    pub async fn reindex(&self, request: crate::application::services::cli_service::ReindexCommand) -> ZeroLatencyResult<IndexResponse> {
        // Get the server status to find the current docs path
        let status = self.get_status().await?;
        
        let docs_path = status.docs_path.ok_or_else(|| {
            zero_latency_core::ZeroLatencyError::configuration("Server has no configured docs path for reindexing")
        })?;
        
        // Create an IndexCommand with the server's docs path and force=true
        let index_request = crate::application::services::cli_service::IndexCommand {
            path: docs_path,
            recursive: true, // Always recursive for reindex
            force: true,     // Always force for reindex
            safe_patterns: request.safe_patterns,
            ignore_patterns: request.ignore_patterns,
            clear_default_ignores: request.clear_default_ignores,
            follow_symlinks: request.follow_symlinks,
            case_sensitive: request.case_sensitive,
        };
        
        // Use the existing index method
        self.index(index_request).await
    }
    
    // Document CRUD operations
    
    /// List all documents with pagination
    pub async fn list_documents(&self, page: u64, limit: u64) -> ZeroLatencyResult<crate::commands::document::ListDocumentsResponse> {
        let url = format!("{}/documents?page={}&limit={}", self.base_url, page, limit);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Failed to list documents: {}", e)
            })?;
        
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService { 
                service: "doc-indexer".to_string(),
                message: format!("List documents request failed: {}", response.status())
            });
        }
        
        let list_response = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse document list: {}", e)
            })?;
            
        Ok(list_response)
    }
    
    /// Get a specific document by ID
    pub async fn get_document(&self, id: &str) -> ZeroLatencyResult<crate::commands::document::GetDocumentResponse> {
        let url = format!("{}/documents/{}", self.base_url, id);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Failed to get document: {}", e)
            })?;
        
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService { 
                service: "doc-indexer".to_string(),
                message: format!("Get document request failed: {}", response.status())
            });
        }
        
        let document_response = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse document: {}", e)
            })?;
            
        Ok(document_response)
    }
    
    //
    // Collection Management Methods
    //
    
    /// List all collections
    pub async fn list_collections(&self) -> ZeroLatencyResult<Vec<crate::commands::collection::CollectionInfo>> {
        let url = format!("{}/collections", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Failed to list collections: {}", e)
            })?;
        
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService { 
                service: "doc-indexer".to_string(),
                message: format!("List collections request failed: {}", response.status())
            });
        }
        
        let list_response: ListCollectionsApiResponse = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse collections list: {}", e)
            })?;
            
        Ok(list_response.collections)
    }
    
    /// Get a specific collection by name
    pub async fn get_collection(&self, name: &str) -> ZeroLatencyResult<crate::commands::collection::GetCollectionResponse> {
        let url = format!("{}/collections/{}", self.base_url, name);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Failed to get collection: {}", e)
            })?;
        
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService { 
                service: "doc-indexer".to_string(),
                message: format!("Get collection request failed: {}", response.status())
            });
        }
        
        let get_response = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse collection info: {}", e)
            })?;
            
        Ok(get_response)
    }
    
    /// Create a new collection
    pub async fn create_collection(&self, request: crate::commands::collection::CreateCollectionRequest) -> ZeroLatencyResult<crate::commands::collection::CreateCollectionResponse> {
        let url = format!("{}/collections", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Failed to create collection: {}", e)
            })?;
        
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService { 
                service: "doc-indexer".to_string(),
                message: format!("Create collection request failed: {}", response.status())
            });
        }
        
        let create_response = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse create collection response: {}", e)
            })?;
            
        Ok(create_response)
    }
    
    /// Delete a collection
    pub async fn delete_collection(&self, name: &str) -> ZeroLatencyResult<crate::commands::collection::DeleteCollectionResponse> {
        let url = format!("{}/collections/{}", self.base_url, name);
        
        let response = self.client
            .delete(&url)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Failed to delete collection: {}", e)
            })?;
        
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService { 
                service: "doc-indexer".to_string(),
                message: format!("Delete collection request failed: {}", response.status())
            });
        }
        
        let delete_response = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse delete collection response: {}", e)
            })?;
            
        Ok(delete_response)
    }
    
    /// Get collection statistics
    pub async fn get_collection_stats(&self, name: &str) -> ZeroLatencyResult<crate::commands::collection::GetCollectionStatsResponse> {
        let url = format!("{}/collections/{}/stats", self.base_url, name);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Failed to get collection stats: {}", e)
            })?;
        
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService { 
                service: "doc-indexer".to_string(),
                message: format!("Get collection stats request failed: {}", response.status())
            });
        }
        
        let stats_response = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse collection stats: {}", e)
            })?;
            
        Ok(stats_response)
    }
}

/// API response types for collection operations
#[derive(Debug, serde::Deserialize)]
struct ListCollectionsApiResponse {
    pub collections: Vec<crate::commands::collection::CollectionInfo>,
}
