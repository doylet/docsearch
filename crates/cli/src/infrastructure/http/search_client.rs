use reqwest::Client;
use std::time::Duration;
use zero_latency_core::{values::SearchQuery, Result as ZeroLatencyResult, ZeroLatencyError};
use zero_latency_search::SearchResponse;
use zero_latency_api::{SearchRequest, SearchFilters};

/// HTTP client for search operations against the Zero Latency API.
///
/// This client is focused solely on search functionality, following the Single Responsibility Principle.
/// It handles HTTP communication, serialization, and error handling for search-related operations.
pub struct SearchApiClient {
    client: Client,
    base_url: String,
    collection_name: String,
}

impl SearchApiClient {
    /// Creates a new search API client.
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

    /// Execute a search query against the API
    pub async fn search(&self, query: SearchQuery) -> ZeroLatencyResult<SearchResponse> {
        let url = format!("{}/api/search", self.base_url);

        // Create filters with collection name
        let filters = SearchFilters {
            collection_name: Some(self.collection_name.clone()),
            ..Default::default()
        };

        // Create comprehensive search request using generated types
        let search_request = SearchRequest {
            query: query.effective_query().to_string(),
            limit: Some(query.limit as i32),
            filters: Some(Box::new(filters)),
            ..Default::default()
        };

        let response = self
            .client
            .post(&url)
            .json(&search_request)
            .send()
            .await
            .map_err(|e| ZeroLatencyError::Network {
                message: format!("Search request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(ZeroLatencyError::ExternalService {
                service: "search_api".to_string(),
                message: format!("Search request failed: {}", response.status()),
            });
        }

        let search_response: SearchResponse =
            response
                .json()
                .await
                .map_err(|e| ZeroLatencyError::Serialization {
                    message: format!("Failed to parse search response: {}", e),
                })?;

        Ok(search_response)
    }
}
