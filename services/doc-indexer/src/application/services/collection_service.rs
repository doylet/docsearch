/// Collection management service
/// 
/// This service handles collection-level operations including creating, listing,
/// and managing vector collections in the storage backend.

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use zero_latency_core::{Result, ZeroLatencyError};
use crate::application::ServiceContainer;

/// Collection metadata and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub name: String,
    pub vector_count: u64,
    pub size_bytes: u64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub vector_size: Option<u64>,
    pub status: CollectionStatus,
}

/// Collection status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectionStatus {
    Active,
    Indexing,
    Error,
    Unknown,
}

/// Collection creation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub vector_size: u64,
    pub distance_metric: Option<String>,
    pub description: Option<String>,
}

/// Service for managing collections
#[derive(Clone)]
pub struct CollectionService {
    container: Arc<ServiceContainer>,
    // In-memory registry of collections (in production, this would be persistent storage)
    collections: Arc<tokio::sync::RwLock<HashMap<String, CollectionInfo>>>,
}

impl CollectionService {
    /// Create a new collection service
    pub fn new(container: &Arc<ServiceContainer>) -> Self {
        let collections = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        
        Self {
            container: container.clone(),
            collections,
        }
    }

    /// Initialize collection with actual data from vector repository
    pub async fn initialize(&self) -> Result<()> {
        let mut collections_guard = self.collections.write().await;
        
        // Get actual vector count from the vector repository
        let vector_count = self.container.vector_repository().count().await.unwrap_or(0) as u64;
        let estimated_size = vector_count * (384 * 4 + 100); // ~1640 bytes per vector with metadata
        
        // Add the default collection with actual vector store data
        let default_collection = CollectionInfo {
            name: "zero_latency_docs".to_string(),
            vector_count,
            size_bytes: estimated_size,
            created_at: Some(chrono::Utc::now() - chrono::Duration::days(30)),
            last_modified: Some(chrono::Utc::now()),
            vector_size: Some(384),
            status: CollectionStatus::Active,
        };
        
        collections_guard.insert("zero_latency_docs".to_string(), default_collection);
        
        tracing::info!(
            "Initialized collection 'zero_latency_docs' with {} vectors ({} bytes)",
            vector_count,
            estimated_size
        );
        
        Ok(())
    }

    /// Update collection statistics when documents are indexed
    pub async fn update_collection_stats(&self, collection_name: &str, vector_count: u64, size_bytes: u64) -> Result<()> {
        let mut collections_guard = self.collections.write().await;
        if let Some(collection) = collections_guard.get_mut(collection_name) {
            collection.vector_count = vector_count;
            collection.size_bytes = size_bytes;
            collection.last_modified = Some(chrono::Utc::now());
        }
        Ok(())
    }

    /// List all available collections
    pub async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let collections_guard = self.collections.read().await;
        let collections: Vec<_> = collections_guard.values().cloned().collect();
        
        // Return collections as-is, without overwriting with shared vector store data
        // In a full implementation, each collection would have its own vector store instance
        // For now, we track collection statistics independently
        Ok(collections)
    }

    /// Get information about a specific collection
    pub async fn get_collection_info(&self, name: &str) -> Result<Option<CollectionInfo>> {
        let collections_guard = self.collections.read().await;
        if let Some(collection) = collections_guard.get(name).cloned() {
            // Return the collection as-is from our registry
            // In a full implementation, we would query the collection-specific vector store
            Ok(Some(collection))
        } else {
            Ok(None)
        }
    }

    /// Create a new collection
    pub async fn create_collection(&self, request: CreateCollectionRequest) -> Result<CollectionInfo> {
        if request.name.is_empty() {
            return Err(ZeroLatencyError::validation("name", "Collection name cannot be empty"));
        }

        if request.vector_size == 0 {
            return Err(ZeroLatencyError::validation("vector_size", "Vector size must be greater than 0"));
        }

        // Check if collection already exists
        {
            let collections_guard = self.collections.read().await;
            if collections_guard.contains_key(&request.name) {
                return Err(ZeroLatencyError::validation("name", "Collection already exists"));
            }
        }

        // Create the collection info
        let collection = CollectionInfo {
            name: request.name.clone(),
            vector_count: 0,
            size_bytes: 0,
            created_at: Some(chrono::Utc::now()),
            last_modified: Some(chrono::Utc::now()),
            vector_size: Some(request.vector_size),
            status: CollectionStatus::Active,
        };

        // Add to registry
        {
            let mut collections_guard = self.collections.write().await;
            collections_guard.insert(request.name.clone(), collection.clone());
        }

        tracing::info!("Created collection: {}", collection.name);
        Ok(collection)
    }

    /// Delete a collection
    pub async fn delete_collection(&self, name: &str) -> Result<bool> {
        let mut collections_guard = self.collections.write().await;
        if collections_guard.remove(name).is_some() {
            println!("🗑️ Deleted collection: {}", name);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get statistics for a collection
    pub async fn get_collection_stats(&self, name: &str) -> Result<Option<CollectionStats>> {
        if let Some(collection) = self.get_collection_info(name).await? {
            let stats = CollectionStats {
                name: collection.name,
                vector_count: collection.vector_count,
                size_bytes: collection.size_bytes,
                average_vector_size: collection.vector_size.unwrap_or(0) as f64,
                last_indexed: collection.last_modified,
                index_efficiency: 0.95, // Mock efficiency
            };
            Ok(Some(stats))
        } else {
            Ok(None)
        }
    }

    /// Private helper to get collection size
    async fn get_collection_size(&self) -> Result<u64> {
        // Estimate size based on vector count and dimensions
        if let Ok(count) = self.container.vector_repository().count().await {
            let estimated_size = count * 384 * 4; // 384 dimensions * 4 bytes per f32
            Ok(estimated_size as u64)
        } else {
            Ok(0)
        }
    }
}

/// Collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub name: String,
    pub vector_count: u64,
    pub size_bytes: u64,
    pub average_vector_size: f64,
    pub last_indexed: Option<chrono::DateTime<chrono::Utc>>,
    pub index_efficiency: f64,
}
