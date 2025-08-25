use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use zero_latency_core::{ZeroLatencyError, Result as ZeroLatencyResult};
use zero_latency_contracts::api::{endpoints, urls};
use crate::commands::collection::{
    CollectionInfo, GetCollectionResponse, CreateCollectionRequest, 
    CreateCollectionResponse, DeleteCollectionResponse, GetCollectionStatsResponse
};

/// Response wrapper for list collections API call
#[derive(Debug, Deserialize, Serialize)]
struct ListCollectionsApiResponse {
    collections: Vec<CollectionInfo>,
}

/// HTTP client for collection management operations against the Zero Latency API.
/// 
/// This client is focused solely on collection management functionality, following the Single Responsibility Principle.
/// It handles HTTP communication, serialization, and error handling for collection-related operations.
pub struct CollectionApiClient {
    client: Client,
    base_url: String,
}

impl CollectionApiClient {
    /// Creates a new collection API client.
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

    /// List all collections
    pub async fn list_collections(&self) -> ZeroLatencyResult<Vec<CollectionInfo>> {
        let url = format!("{}{}", self.base_url, endpoints::COLLECTIONS);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("List collections request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "collection_api".to_string(),
                message: format!("List collections request failed: {}", response.status())
            });
        }
        
        let response_wrapper: ListCollectionsApiResponse = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse list collections response: {}", e)
            })?;
            
        Ok(response_wrapper.collections)
    }

    /// Get information about a specific collection
    pub async fn get_collection(&self, name: &str) -> ZeroLatencyResult<GetCollectionResponse> {
        let url = urls::collection_by_name(&self.base_url, name);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Get collection request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "collection_api".to_string(),
                message: format!("Get collection request failed: {}", response.status())
            });
        }
        
        let collection_response: GetCollectionResponse = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse get collection response: {}", e)
            })?;
            
        Ok(collection_response)
    }

    /// Create a new collection
    pub async fn create_collection(&self, request: CreateCollectionRequest) -> ZeroLatencyResult<CreateCollectionResponse> {
        let url = format!("{}{}", self.base_url, endpoints::COLLECTIONS);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Create collection request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "collection_api".to_string(),
                message: format!("Create collection request failed: {}", response.status())
            });
        }
        
        let create_response: CreateCollectionResponse = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse create collection response: {}", e)
            })?;
            
        Ok(create_response)
    }

    /// Delete a collection
    pub async fn delete_collection(&self, name: &str) -> ZeroLatencyResult<DeleteCollectionResponse> {
        let url = urls::collection_by_name(&self.base_url, name);
        
        let response = self.client
            .delete(&url)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Delete collection request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "collection_api".to_string(),
                message: format!("Delete collection request failed: {}", response.status())
            });
        }
        
        let delete_response: DeleteCollectionResponse = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse delete collection response: {}", e)
            })?;
            
        Ok(delete_response)
    }

    /// Get collection statistics
    pub async fn get_collection_stats(&self, name: &str) -> ZeroLatencyResult<GetCollectionStatsResponse> {
        let url = urls::collection_stats(&self.base_url, name);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Get collection stats request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "collection_api".to_string(),
                message: format!("Get collection stats request failed: {}", response.status())
            });
        }
        
        let stats_response: GetCollectionStatsResponse = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse get collection stats response: {}", e)
            })?;
            
        Ok(stats_response)
    }
}
