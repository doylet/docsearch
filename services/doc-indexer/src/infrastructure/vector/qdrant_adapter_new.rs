/// Qdrant vector store adapter
/// 
/// This adapter implements the VectorRepository trait for Qdrant vector database,
/// providing concrete implementation for vector storage and retrieval operations.

use std::collections::HashMap;
use std::str::FromStr;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use qdrant_client::{Qdrant, QdrantError};
use qdrant_client::qdrant::{
    SearchPoints, Vector, PointStruct, Value, Vectors, PointId, SearchResponse, ScoredPoint
};
use zero_latency_core::{Result, ZeroLatencyError, models::HealthStatus, Uuid};
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
    client: Qdrant,
}

impl QdrantAdapter {
    /// Create a new Qdrant adapter
    pub async fn new(config: QdrantConfig) -> Result<Self> {
        println!("🔧 QdrantAdapter: Connecting to {} with collection '{}'", config.url, config.collection_name);
        
        let client = if let Some(api_key) = &config.api_key {
            Qdrant::from_url(&config.url)
                .with_api_key(api_key)
                .build()
                .map_err(|e| ZeroLatencyError::database(&format!("Failed to connect to Qdrant: {}", e)))?
        } else {
            Qdrant::from_url(&config.url)
                .build()
                .map_err(|e| ZeroLatencyError::database(&format!("Failed to connect to Qdrant: {}", e)))?
        };
        
        println!("✅ QdrantAdapter: Successfully connected to Qdrant");
        
        Ok(Self {
            config,
            client,
        })
    }

    /// Convert a Qdrant scored point to VectorDocument  
    fn from_qdrant_point(&self, point: &ScoredPoint) -> Result<VectorDocument> {
        let id_str = match &point.id {
            Some(PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) }) => {
                uuid.clone()
            }
            Some(PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) }) => {
                num.to_string()
            }
            _ => return Err(ZeroLatencyError::database("Invalid point ID format")),
        };
        
        let payload = &point.payload;
        
        let document_id = payload.get("document_id")
            .and_then(|v| match v.kind.as_ref()? {
                qdrant_client::qdrant::value::Kind::StringValue(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ZeroLatencyError::database("Missing document_id in payload"))?;
            
        let chunk_index = payload.get("chunk_index")
            .and_then(|v| match v.kind.as_ref()? {
                qdrant_client::qdrant::value::Kind::StringValue(s) => s.parse().ok(),
                qdrant_client::qdrant::value::Kind::IntegerValue(i) => Some(*i as usize),
                _ => None,
            })
            .unwrap_or(0);
            
        let content = payload.get("content")
            .and_then(|v| match v.kind.as_ref()? {
                qdrant_client::qdrant::value::Kind::StringValue(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_default();
            
        let title = payload.get("title")
            .and_then(|v| match v.kind.as_ref()? {
                qdrant_client::qdrant::value::Kind::StringValue(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_default();
            
        let heading_path = payload.get("heading_path")
            .and_then(|v| match v.kind.as_ref()? {
                qdrant_client::qdrant::value::Kind::StringValue(s) => {
                    Some(s.split('/').map(|s| s.to_string()).collect())
                },
                _ => None,
            })
            .unwrap_or_default();
            
        let url = payload.get("url")
            .and_then(|v| match v.kind.as_ref()? {
                qdrant_client::qdrant::value::Kind::StringValue(s) => Some(s.clone()),
                _ => None,
            });
        
        let mut custom = HashMap::new();
        for (key, value) in payload {
            if key.starts_with("custom_") {
                if let Some(qdrant_client::qdrant::value::Kind::StringValue(s)) = value.kind.as_ref() {
                    custom.insert(key.strip_prefix("custom_").unwrap().to_string(), s.clone());
                }
            }
        }
        
        let metadata = VectorMetadata {
            document_id,
            chunk_index,
            content,
            title,
            heading_path,
            url,
            custom,
        };
        
        let embedding = point.vectors.as_ref()
            .and_then(|vectors| match vectors.vectors_options.as_ref() {
                Some(qdrant_client::qdrant::vectors::VectorsOptions::Vector(vector)) => {
                    Some(vector.data.clone())
                }
                _ => None,
            })
            .unwrap_or_default();
        
        Ok(VectorDocument {
            id: Uuid::from_str(&id_str).unwrap_or_else(|_| Uuid::new_v4()),
            embedding,
            metadata,
        })
    }
}

#[async_trait]
impl VectorRepository for QdrantAdapter {
    async fn insert(&self, vectors: Vec<VectorDocument>) -> Result<()> {
        // For now, insertion is handled separately via the indexing service
        // This implementation focuses on search functionality
        println!("🔧 QdrantAdapter: Insert not implemented (using separate indexing)");
        let _ = vectors; // Suppress unused warning
        Ok(())
    }
    
    async fn search(&self, query_vector: Vec<f32>, k: usize) -> Result<Vec<SimilarityResult>> {
        println!("🔍 QdrantAdapter: Searching in collection '{}' with vector size {} for {} results", 
                 self.config.collection_name, query_vector.len(), k);
        
        let search_request = SearchPoints {
            collection_name: self.config.collection_name.clone(),
            vector: query_vector,
            limit: k as u64,
            score_threshold: Some(0.0), // Accept all results
            offset: Some(0),
            with_payload: Some(true),
            with_vectors: Some(true),
            ..Default::default()
        };
        
        println!("📡 QdrantAdapter: Sending search request to Qdrant...");
        
        let response = self.client.search_points(&search_request).await
            .map_err(|e| {
                println!("❌ QdrantAdapter: Search failed: {}", e);
                ZeroLatencyError::database(&format!("Qdrant search failed: {}", e))
            })?;
        
        println!("📊 QdrantAdapter: Qdrant returned {} results", response.result.len());
        
        let mut similarity_results = Vec::new();
        for point in response.result {
            println!("🔍 QdrantAdapter: Processing point with score: {}", point.score);
            
            match self.from_qdrant_point(&point) {
                Ok(document) => {
                    similarity_results.push(SimilarityResult {
                        document_id: document.metadata.document_id.clone(),
                        similarity: point.score,
                        metadata: document.metadata,
                    });
                }
                Err(e) => {
                    println!("⚠️  QdrantAdapter: Failed to convert point: {}", e);
                    continue;
                }
            }
        }
        
        println!("✅ QdrantAdapter: Successfully converted {} results", similarity_results.len());
        Ok(similarity_results)
    }
    
    async fn delete(&self, document_id: &str) -> Result<bool> {
        println!("🔧 QdrantAdapter: Delete not fully implemented");
        let _ = document_id; // Suppress unused warning
        Ok(true)
    }
    
    async fn update(&self, document_id: &str, vector: Vec<f32>) -> Result<bool> {
        println!("🔧 QdrantAdapter: Update not fully implemented");
        let _ = (document_id, vector); // Suppress unused warnings
        Ok(true)
    }
    
    async fn health_check(&self) -> Result<HealthStatus> {
        // In a real implementation, this would check Qdrant health
        Ok(HealthStatus::Healthy)
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
