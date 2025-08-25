use std::time::Duration;
use reqwest::Client;
use zero_latency_core::{ZeroLatencyError, Result as ZeroLatencyResult};
use crate::commands::document::{ListDocumentsResponse, GetDocumentResponse};

/// HTTP client for document operations against the Zero Latency API.
/// 
/// This client is focused solely on document management functionality, following the Single Responsibility Principle.
/// It handles HTTP communication, serialization, and error handling for document-related operations.
pub struct DocumentApiClient {
    client: Client,
    base_url: String,
    collection_name: String,
}

impl DocumentApiClient {
    /// Creates a new document API client.
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

    /// List documents in the collection
    pub async fn list_documents(&self, page: u64, limit: u64) -> ZeroLatencyResult<ListDocumentsResponse> {
        let url = format!("{}/api/documents", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[
                ("collection_name", &self.collection_name),
                ("page", &page.to_string()),
                ("limit", &limit.to_string())
            ])
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("List documents request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "document_api".to_string(),
                message: format!("List documents request failed: {}", response.status())
            });
        }
        
        let list_response: ListDocumentsResponse = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse list documents response: {}", e)
            })?;
            
        Ok(list_response)
    }

    /// Get a specific document by ID
    pub async fn get_document(&self, id: &str) -> ZeroLatencyResult<GetDocumentResponse> {
        let url = format!("{}/api/documents/{}", self.base_url, id);
        
        let response = self.client
            .get(&url)
            .query(&[("collection_name", &self.collection_name)])
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network { 
                message: format!("Get document request failed: {}", e)
            })?;
            
        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "document_api".to_string(),
                message: format!("Get document request failed: {}", response.status())
            });
        }
        
        let document_response: GetDocumentResponse = response
            .json()
            .await
            .map_err(|e| ZeroLatencyError::Serialization { 
                message: format!("Failed to parse get document response: {}", e)
            })?;
            
        Ok(document_response)
    }
}
