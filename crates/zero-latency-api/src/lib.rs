//! Zero-Latency API Client Library
//! 
//! Generated types and client code for the Zero-Latency document indexing and search API.
//! This crate provides type-safe access to all API endpoints with support for multiple protocols.

pub mod endpoints;

pub mod types {
    //! API type definitions
    
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use chrono::{DateTime, Utc};

    /// Error response structure
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ApiError {
        pub error: String,
        pub message: String,
        pub code: String,
        pub trace_id: Option<String>,
        pub details: Option<serde_json::Value>,
    }

    /// Health check result
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HealthCheckResult {
        pub status: String,
        pub timestamp: DateTime<Utc>,
        pub version: Option<String>,
    }

    /// API status response
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ApiStatusResponse {
        pub status: String,
        pub version: String,
        pub uptime_seconds: u64,
    }

    /// Search request
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchRequest {
        pub query: String,
        pub limit: Option<i32>,
        pub offset: Option<i32>,
    }

    /// Search response
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchResponse {
        pub results: Vec<SearchResult>,
        pub total: i64,
        pub query_time_ms: i64,
    }

    /// Search result
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchResult {
        pub id: String,
        pub title: Option<String>,
        pub content: String,
        pub score: f64,
    }

    /// Index request
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IndexRequest {
        pub path: String,
        pub collection: Option<String>,
        pub recursive: Option<bool>,
    }

    /// Index response
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IndexResponse {
        pub indexed_documents: i64,
        pub errors: Vec<String>,
    }

    /// Collection information
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Collection {
        pub name: String,
        pub description: Option<String>,
        pub document_count: i64,
        pub created_at: DateTime<Utc>,
    }
}

pub mod client {
    //! HTTP client for Zero-Latency API
    
    use crate::types::*;
    use reqwest::Client;
    use serde_json::Value;
    use std::collections::HashMap;
    use uuid::Uuid;
    
    /// API client configuration
    #[derive(Debug, Clone)]
    pub struct ApiClientConfig {
        pub base_url: String,
        pub tenant_id: Option<Uuid>,
        pub timeout_seconds: u64,
        pub user_agent: String,
    }
    
    impl Default for ApiClientConfig {
        fn default() -> Self {
            Self {
                base_url: "http://localhost:8081".to_string(),
                tenant_id: None,
                timeout_seconds: 30,
                user_agent: "zero-latency-api-client/1.0.0".to_string(),
            }
        }
    }
    
    /// Zero-Latency API Client
    #[derive(Debug, Clone)]
    pub struct ZeroLatencyApiClient {
        client: Client,
        config: ApiClientConfig,
    }
    
    impl ZeroLatencyApiClient {
        /// Create a new API client with default configuration
        pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            Self::with_config(ApiClientConfig::default())
        }
        
        /// Create a new API client with custom configuration
        pub fn with_config(config: ApiClientConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            let client = Client::builder()
                .timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .user_agent(&config.user_agent)
                .build()?;
            
            Ok(Self { client, config })
        }
        
        /// Set tenant ID for multi-tenant operations
        pub fn with_tenant_id(mut self, tenant_id: Uuid) -> Self {
            self.config.tenant_id = Some(tenant_id);
            self
        }
        
        /// Build request headers including tenant ID if configured
        fn build_headers(&self) -> reqwest::header::HeaderMap {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                "application/json".parse().unwrap(),
            );
            
            if let Some(tenant_id) = self.config.tenant_id {
                headers.insert(
                    "x-tenant-id",
                    tenant_id.to_string().parse().unwrap(),
                );
            }
            
            headers
        }
        
        /// Health check endpoint
        pub async fn health_check(&self) -> Result<HealthCheckResult, Box<dyn std::error::Error + Send + Sync>> {
            let url = format!("{}/health", self.config.base_url);
            let response = self.client
                .get(&url)
                .headers(self.build_headers())
                .send()
                .await?;
            
            if response.status().is_success() {
                Ok(response.json().await?)
            } else {
                let error: ApiError = response.json().await?;
                Err(format!("API Error: {}", error.message).into())
            }
        }
        
        /// Get API status
        pub async fn api_status(&self) -> Result<ApiStatusResponse, Box<dyn std::error::Error + Send + Sync>> {
            let url = format!("{}/api/status", self.config.base_url);
            let response = self.client
                .get(&url)
                .headers(self.build_headers())
                .send()
                .await?;
            
            if response.status().is_success() {
                Ok(response.json().await?)
            } else {
                let error: ApiError = response.json().await?;
                Err(format!("API Error: {}", error.message).into())
            }
        }
        
        /// Search documents
        pub async fn search_documents(&self, request: SearchRequest) -> Result<SearchResponse, Box<dyn std::error::Error + Send + Sync>> {
            let url = format!("{}/api/search", self.config.base_url);
            let response = self.client
                .post(&url)
                .headers(self.build_headers())
                .json(&request)
                .send()
                .await?;
            
            if response.status().is_success() {
                Ok(response.json().await?)
            } else {
                let error: ApiError = response.json().await?;
                Err(format!("Search Error: {}", error.message).into())
            }
        }
        
        /// Index documents from path
        pub async fn index_documents(&self, request: IndexRequest) -> Result<IndexResponse, Box<dyn std::error::Error + Send + Sync>> {
            let url = format!("{}/api/index", self.config.base_url);
            let response = self.client
                .post(&url)
                .headers(self.build_headers())
                .json(&request)
                .send()
                .await?;
            
            if response.status().is_success() {
                Ok(response.json().await?)
            } else {
                let error: ApiError = response.json().await?;
                Err(format!("Index Error: {}", error.message).into())
            }
        }
        
        /// List collections
        pub async fn list_collections(&self, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<Collection>, Box<dyn std::error::Error + Send + Sync>> {
            let mut url = format!("{}/api/collections", self.config.base_url);
            let mut params = Vec::new();
            
            if let Some(limit) = limit {
                params.push(format!("limit={}", limit));
            }
            if let Some(offset) = offset {
                params.push(format!("offset={}", offset));
            }
            
            if !params.is_empty() {
                url.push('?');
                url.push_str(&params.join("&"));
            }
            
            let response = self.client
                .get(&url)
                .headers(self.build_headers())
                .send()
                .await?;
            
            if response.status().is_success() {
                #[derive(serde::Deserialize)]
                struct ListResponse {
                    collections: Vec<Collection>,
                }
                let list: ListResponse = response.json().await?;
                Ok(list.collections)
            } else {
                let error: ApiError = response.json().await?;
                Err(format!("List Collections Error: {}", error.message).into())
            }
        }
        
        /// Get collection by name
        pub async fn get_collection(&self, name: &str) -> Result<Collection, Box<dyn std::error::Error + Send + Sync>> {
            let url = format!("{}/api/collections/{}", self.config.base_url, name);
            let response = self.client
                .get(&url)
                .headers(self.build_headers())
                .send()
                .await?;
            
            if response.status().is_success() {
                Ok(response.json().await?)
            } else {
                let error: ApiError = response.json().await?;
                Err(format!("Get Collection Error: {}", error.message).into())
            }
        }
    }
    
    impl Default for ZeroLatencyApiClient {
        fn default() -> Self {
            Self::new().expect("Failed to create default API client")
        }
    }
}

// Re-export commonly used items
pub use types::*;
pub use client::{ZeroLatencyApiClient, ApiClientConfig};

/// Convenience function to create a new API client
pub fn new_client() -> Result<ZeroLatencyApiClient, Box<dyn std::error::Error + Send + Sync>> {
    ZeroLatencyApiClient::new()
}

/// Convenience function to create a client with custom base URL
pub fn new_client_with_url(base_url: String) -> Result<ZeroLatencyApiClient, Box<dyn std::error::Error + Send + Sync>> {
    let config = ApiClientConfig {
        base_url,
        ..Default::default()
    };
    ZeroLatencyApiClient::with_config(config)
}
