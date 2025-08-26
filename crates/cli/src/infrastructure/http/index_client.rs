use crate::application::services::cli_service::{IndexCommand, IndexResponse, ReindexCommand};
use reqwest::Client;
use std::time::Duration;
use zero_latency_core::{Result as ZeroLatencyResult, ZeroLatencyError};

/// HTTP client for indexing operations against the Zero Latency API.
///
/// This client is focused solely on document indexing functionality, following the Single Responsibility Principle.
/// It handles HTTP communication, serialization, and error handling for indexing-related operations.
pub struct IndexApiClient {
    client: Client,
    base_url: String,
    collection_name: String,
}

impl IndexApiClient {
    /// Creates a new indexing API client.
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the Zero Latency API
    /// * `timeout` - Request timeout duration
    /// * `collection_name` - Collection name for vector storage operations
    pub fn new(
        base_url: String,
        timeout: Duration,
        collection_name: String,
    ) -> ZeroLatencyResult<Self> {
        let client = Client::builder().timeout(timeout).build().map_err(|e| {
            ZeroLatencyError::Configuration {
                message: format!("Failed to create HTTP client: {}", e),
            }
        })?;

        Ok(Self {
            client,
            base_url,
            collection_name,
        })
    }

    /// Index documents from a path
    pub async fn index(&self, request: IndexCommand) -> ZeroLatencyResult<IndexResponse> {
        // Convert path to absolute path to ensure server processes the correct directory
        let absolute_path = std::path::Path::new(&request.path)
            .canonicalize()
            .map_err(|e| {
                ZeroLatencyError::validation(
                    "path",
                    &format!("Invalid path '{}': {}", request.path, e),
                )
            })?
            .to_string_lossy()
            .to_string();

        let json_body = serde_json::json!({
            "path": absolute_path,
            "collection": self.collection_name,  // Fixed: was "collection_name", should be "collection"
            "recursive": request.recursive,
            "force": request.force,
            "safe_patterns": request.safe_patterns,
            "ignore_patterns": request.ignore_patterns,
            "clear_default_ignores": request.clear_default_ignores,
            "follow_symlinks": request.follow_symlinks,
            "case_sensitive": request.case_sensitive
        });

        let url = format!("{}/api/index", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&json_body)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network {
                message: format!("Index request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "index_api".to_string(),
                message: format!("Index request failed: {}", response.status()),
            });
        }

        let index_response: IndexResponse =
            response
                .json()
                .await
                .map_err(|e| ZeroLatencyError::Serialization {
                    message: format!("Failed to parse index response: {}", e),
                })?;

        Ok(index_response)
    }

    /// Reindex documents
    pub async fn reindex(&self, request: ReindexCommand) -> ZeroLatencyResult<IndexResponse> {
        let json_body = serde_json::json!({
            "collection": self.collection_name,  // Fixed: was "collection_name", should be "collection"
            "safe_patterns": request.safe_patterns,
            "ignore_patterns": request.ignore_patterns,
            "clear_default_ignores": request.clear_default_ignores,
            "follow_symlinks": request.follow_symlinks,
            "case_sensitive": request.case_sensitive
        });

        let url = format!("{}/api/reindex", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&json_body)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network {
                message: format!("Reindex request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "reindex_api".to_string(),
                message: format!("Reindex request failed: {}", response.status()),
            });
        }

        let reindex_response: IndexResponse =
            response
                .json()
                .await
                .map_err(|e| ZeroLatencyError::Serialization {
                    message: format!("Failed to parse reindex response: {}", e),
                })?;

        Ok(reindex_response)
    }
}
