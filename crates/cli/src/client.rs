use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub total_results: u32,
    pub results: Vec<SearchResult>,
    pub search_metadata: SearchMetadata,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub score: f32,
    pub document_title: String,
    pub content: String,
    pub snippet: String,
    pub section: String,
    pub doc_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchMetadata {
    pub embedding_time_ms: u64,
    pub search_time_ms: u64,
    pub total_time_ms: u64,
    pub model_used: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub collection_info: CollectionInfo,
    pub server_info: ServerInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionInfo {
    pub name: String,
    pub vectors_count: u64,
    pub points_count: u64,
    pub indexed_documents: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerInfo {
    pub version: String,
    pub uptime_seconds: u64,
    pub embedding_model: String,
}

#[derive(Debug, Serialize)]
pub struct IndexRequest {
    pub path: String,
    pub recursive: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IndexResponse {
    pub status: String,
    pub indexed_documents: u32,
    pub total_chunks: u32,
    pub errors: Vec<String>,
    pub processing_time_ms: u64,
}

impl ApiClient {
    pub fn new(base_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }
    
    pub async fn search(&self, query: &str, limit: Option<u32>) -> Result<SearchResponse> {
        let url = format!("{}/api/search", self.base_url);
        let request = SearchRequest {
            query: query.to_string(),
            limit,
        };
        
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send search request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Search request failed with status {}: {}", status, text);
        }
        
        response
            .json::<SearchResponse>()
            .await
            .context("Failed to parse search response")
    }
    
    pub async fn status(&self) -> Result<StatusResponse> {
        let url = format!("{}/api/status", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send status request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Status request failed with status {}: {}", status, text);
        }
        
        response
            .json::<StatusResponse>()
            .await
            .context("Failed to parse status response")
    }
    
    pub async fn index(&self, path: &str, recursive: bool) -> Result<IndexResponse> {
        let url = format!("{}/api/index", self.base_url);
        let request = IndexRequest {
            path: path.to_string(),
            recursive,
        };
        
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send index request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Index request failed with status {}: {}", status, text);
        }
        
        response
            .json::<IndexResponse>()
            .await
            .context("Failed to parse index response")
    }
    
    pub async fn reindex(&self) -> Result<IndexResponse> {
        let url = format!("{}/api/reindex", self.base_url);
        
        let response = self
            .client
            .post(&url)
            .send()
            .await
            .context("Failed to send reindex request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Reindex request failed with status {}: {}", status, text);
        }
        
        response
            .json::<IndexResponse>()
            .await
            .context("Failed to parse reindex response")
    }
    
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
