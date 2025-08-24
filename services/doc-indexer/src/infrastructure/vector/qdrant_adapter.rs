/// Qdrant vector store adapter
/// 
/// This adapter implements the VectorRepository trait for Qdrant vector database,
/// providing concrete implementation for vector storage and retrieval operations.

use std::collections::HashMap;
use std::str::FromStr;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use zero_latency_core::{Result, ZeroLatencyError, models::HealthStatus, Uuid, values::Score};
use zero_latency_vector::{VectorRepository, VectorDocument, VectorMetadata, SimilarityResult};

/// Qdrant-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub collection_name: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
}

/// Qdrant vector store adapter
pub struct QdrantAdapter {
    config: QdrantConfig,
    client: Client,
}

impl QdrantAdapter {
    /// Create a new Qdrant adapter
    pub async fn new(config: QdrantConfig) -> Result<Self> {
        tracing::info!("QdrantAdapter: Setting up REST client for {} with collection '{}'", config.url, config.collection_name);
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| ZeroLatencyError::database(&format!("Failed to create HTTP client: {}", e)))?;
        
        tracing::info!("QdrantAdapter: Successfully created REST client");
        
        Ok(Self {
            config,
            client,
        })
    }

    /// Convert a Qdrant REST API search result to VectorDocument  
    fn from_qdrant_rest_result(&self, result: &QdrantSearchResult) -> Result<VectorDocument> {
        let document_id = result.payload.get("document_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ZeroLatencyError::database("Missing document_id in payload"))?;
            
        let chunk_index = result.payload.get("chunk_index")
            .and_then(|v| v.as_u64().map(|i| i as usize))
            .unwrap_or(0);
            
        let content = result.payload.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
            
        let title = result.payload.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
            
        let heading_path = result.payload.get("heading_path")
            .and_then(|v| v.as_str())
            .map(|s| s.split('/').map(|s| s.to_string()).collect())
            .unwrap_or_default();
            
        let url = result.payload.get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let mut custom = HashMap::new();
        for (key, value) in &result.payload {
            if key.starts_with("custom_") {
                if let Some(s) = value.as_str() {
                    custom.insert(key.strip_prefix("custom_").unwrap().to_string(), s.to_string());
                }
            }
        }
        
        let metadata = VectorMetadata {
            document_id: Uuid::from_str(document_id).unwrap_or_else(|_| Uuid::new_v4()),
            chunk_index,
            content,
            title,
            heading_path,
            url,
            custom,
        };
        
        // Convert id to string 
        let id_str = match &result.id {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => Uuid::new_v4().to_string(),
        };
        
        Ok(VectorDocument {
            id: Uuid::from_str(&id_str).unwrap_or_else(|_| Uuid::new_v4()),
            embedding: result.vector.clone().unwrap_or_default(),
            metadata,
        })
    }
}

#[derive(Debug, Deserialize)]
struct QdrantSearchResponse {
    result: Vec<QdrantSearchResult>,
}

#[derive(Debug, Deserialize)]
struct QdrantSearchResult {
    id: serde_json::Value, // Can be string or number
    score: f32,
    payload: HashMap<String, serde_json::Value>,
    vector: Option<Vec<f32>>,
}

#[async_trait]
impl VectorRepository for QdrantAdapter {
    async fn insert(&self, vectors: Vec<VectorDocument>) -> Result<()> {
        // For now, insertion is handled separately via the indexing service
        // This implementation focuses on search functionality
        tracing::debug!("QdrantAdapter: Insert not implemented (using separate indexing)");
        let _ = vectors; // Suppress unused warning
        Ok(())
    }
    
    async fn search(&self, query_vector: Vec<f32>, k: usize) -> Result<Vec<SimilarityResult>> {
        println!("🔍 QdrantAdapter: Searching in collection '{}' with vector size {} for {} results", 
                 self.config.collection_name, query_vector.len(), k);
        
        // Create search request payload
        let search_payload = serde_json::json!({
            "vector": query_vector,
            "limit": k,
            "with_payload": true,
            "with_vector": true,
            "score_threshold": 0.0
        });
        
        let url = format!("{}/collections/{}/points/search", 
                         self.config.url, self.config.collection_name);
        
        println!("📡 QdrantAdapter: Sending REST search request to {}", url);
        
        let mut request = self.client.post(&url)
            .header("Content-Type", "application/json")
            .json(&search_payload);
        
        // Add API key if configured
        if let Some(api_key) = &self.config.api_key {
            request = request.header("api-key", api_key);
        }
        
        let response = request.send().await
            .map_err(|e| {
                tracing::error!("QdrantAdapter: HTTP request failed: {}", e);
                ZeroLatencyError::database(&format!("Qdrant HTTP request failed: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("QdrantAdapter: HTTP error {}: {}", status, error_text);
            return Err(ZeroLatencyError::database(&format!("Qdrant HTTP error {}: {}", status, error_text)));
        }
        
        let search_response: QdrantSearchResponse = response.json().await
            .map_err(|e| {
                tracing::error!("QdrantAdapter: Failed to parse response: {}", e);
                ZeroLatencyError::database(&format!("Failed to parse Qdrant response: {}", e))
            })?;
        
        println!("📊 QdrantAdapter: Qdrant returned {} results", search_response.result.len());
        
        let mut similarity_results = Vec::new();
        for result in &search_response.result {
            println!("🔍 QdrantAdapter: Processing result with score: {}", result.score);
            
            match self.from_qdrant_rest_result(result) {
                Ok(document) => {
                    similarity_results.push(SimilarityResult {
                        document_id: document.metadata.document_id.clone(),
                        similarity: Score::new(result.score).unwrap_or_default(),
                        metadata: document.metadata,
                    });
                }
                Err(e) => {
                    println!("⚠️  QdrantAdapter: Failed to convert result: {}", e);
                    continue;
                }
            }
        }
        
        tracing::debug!("QdrantAdapter: Successfully converted {} results", similarity_results.len());
        Ok(similarity_results)
    }
    
    async fn delete(&self, document_id: &str) -> Result<bool> {
        tracing::debug!("QdrantAdapter: Delete not fully implemented");
        let _ = document_id; // Suppress unused warning
        Ok(true)
    }
    
    async fn update(&self, document_id: &str, vector: Vec<f32>) -> Result<bool> {
        tracing::debug!("QdrantAdapter: Update not fully implemented");
        let _ = (document_id, vector); // Suppress unused warnings
        Ok(true)
    }
    
    async fn health_check(&self) -> Result<HealthStatus> {
        // In a real implementation, this would check Qdrant health
        Ok(HealthStatus::Healthy)
    }
    
    async fn count(&self) -> Result<usize> {
        // For now, return a placeholder count
        // In a real implementation, this would query Qdrant for collection stats
        tracing::debug!("QdrantAdapter: Count not fully implemented");
        Ok(0)
    }
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6333".to_string(),
            collection_name: "zero_latency_docs".to_string(),
            api_key: None,
            timeout_seconds: 30,
        }
    }
}
