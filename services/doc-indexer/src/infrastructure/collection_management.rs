use crate::application::interfaces::VectorStorage;
use crate::infrastructure::enhanced_search::{
    EnhancedSearchRequest, EnhancedSearchResult, EnhancedSearchService,
};
use serde::{Deserialize, Serialize};
/// Collection Management System
///
/// Dynamic collection creation, management, and analytics for organizing
/// documents and optimizing search performance across different document types.
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Collection configuration and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// Unique collection identifier
    pub id: String,

    /// Human-readable collection name
    pub name: String,

    /// Collection description
    pub description: Option<String>,

    /// Vector store configuration
    pub vector_config: VectorStoreConfig,

    /// Search configuration
    pub search_config: SearchConfig,

    /// Creation timestamp
    pub created_at: u64,

    /// Last updated timestamp
    pub updated_at: u64,

    /// Collection metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Whether the collection is active
    pub active: bool,
}

/// Vector store configuration for a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    /// Vector dimension
    pub dimension: usize,

    /// Distance metric (cosine, euclidean, etc.)
    pub metric: String,

    /// Index type (flat, hnsw, etc.)
    pub index_type: String,

    /// Index parameters
    pub index_params: HashMap<String, serde_json::Value>,

    /// Maximum number of vectors
    pub max_vectors: Option<usize>,
}

/// Search configuration for a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Default search limit
    pub default_limit: usize,

    /// Maximum search limit
    pub max_limit: usize,
    // The PopularQuery struct is defined in zero-latency-search/src/traits.rs and is no longer needed here.
    // #[derive(Debug, Clone, Serialize, Deserialize)]
    // pub struct PopularQuery {
    //     pub query: String,
    //     pub count: u64,
    //     pub avg_results: f64,
    //     pub avg_confidence: f64,
    // }
    /// Enable metadata filtering
    pub enable_metadata_filtering: bool,

    /// Custom ranking weights
    pub ranking_weights: HashMap<String, f32>,
}

/// Collection statistics and analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    /// Collection identifier
    pub collection_id: String,

    /// Number of documents in the collection
    pub document_count: u64,

    /// Total size in bytes
    pub total_size_bytes: u64,

    /// Average document size
    pub avg_document_size: f64,

    /// Search statistics
    pub search_stats: SearchStats,

    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,

    /// Last updated timestamp
    pub last_updated: u64,
}

/// Search usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    /// Total number of searches performed
    pub total_searches: u64,

    /// Average results per search
    pub avg_results_per_search: f64,

    /// Average confidence score
    pub avg_confidence_score: f64,

    /// Most common search terms
    pub popular_queries: Vec<PopularQuery>,

    /// Search frequency by time period
    pub search_frequency: HashMap<String, u64>,
}

/// Popular query tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularQuery {
    pub query: String,
    pub count: u64,
    pub avg_results: f64,
    pub avg_confidence: f64,
}

/// Performance metrics for a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average search response time (milliseconds)
    pub avg_search_time_ms: f64,

    /// 95th percentile search time
    pub p95_search_time_ms: f64,

    /// Cache hit rate
    pub cache_hit_rate: f64,

    /// Memory usage (MB)
    pub memory_usage_mb: f64,

    /// Index efficiency score
    pub index_efficiency: f64,
}

/// Collection creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollectionRequest {
    /// Collection name
    pub name: String,

    /// Collection description
    pub description: Option<String>,

    /// Vector store configuration
    pub vector_config: VectorStoreConfig,

    /// Search configuration
    pub search_config: Option<SearchConfig>,

    /// Initial metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Collection update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCollectionRequest {
    /// New name
    pub name: Option<String>,

    /// New description
    pub description: Option<String>,

    /// Updated search configuration
    pub search_config: Option<SearchConfig>,

    /// Updated metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,

    /// Whether to activate/deactivate
    pub active: Option<bool>,
}

/// Cross-collection search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCollectionSearchRequest {
    /// Base search request
    pub search_request: EnhancedSearchRequest,

    /// Collections to search (if None, search all active collections)
    pub target_collections: Option<Vec<String>>,

    /// Whether to merge results or return per-collection
    pub merge_results: bool,

    /// Maximum results per collection (before merging)
    pub max_results_per_collection: Option<usize>,
}

/// Cross-collection search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCollectionSearchResponse {
    /// Merged results (if merge_results = true)
    pub merged_results: Option<Vec<EnhancedSearchResult>>,

    /// Results by collection (if merge_results = false)
    pub collection_results: Option<HashMap<String, Vec<EnhancedSearchResult>>>,

    /// Search statistics
    pub search_stats: CrossCollectionSearchStats,
}

/// Statistics for cross-collection search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCollectionSearchStats {
    /// Collections searched
    pub collections_searched: Vec<String>,

    /// Total results found
    pub total_results: usize,

    /// Results per collection
    pub results_per_collection: HashMap<String, usize>,

    /// Total processing time
    pub total_processing_time_ms: u64,

    /// Processing time per collection
    pub processing_time_per_collection: HashMap<String, u64>,
}

/// Main collection manager
pub struct CollectionManager {
    /// Collections by ID
    collections: Arc<RwLock<HashMap<String, CollectionConfig>>>,
    /// Vector stores by collection ID
    vector_stores: Arc<RwLock<HashMap<String, Arc<dyn VectorStorage>>>>,
    /// Search services by collection ID
    search_services: Arc<RwLock<HashMap<String, Arc<EnhancedSearchService>>>>,
    /// Statistics by collection ID
    statistics: Arc<RwLock<HashMap<String, CollectionStats>>>,
    /// Configuration
    config: CollectionManagerConfig,
}

/// Configuration for collection manager
#[derive(Debug, Clone)]
pub struct CollectionManagerConfig {
    pub default_vector_dimension: usize,
    pub default_search_limit: usize,
    pub max_collections: usize,
    pub stats_update_interval: u64,
    pub enable_auto_optimization: bool,
    pub data_directory: std::path::PathBuf,
}

impl Default for CollectionManagerConfig {
    fn default() -> Self {
        Self {
            default_vector_dimension: std::env::var("COLLECTION_DEFAULT_DIMENSION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(384),
            default_search_limit: std::env::var("COLLECTION_DEFAULT_SEARCH_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            max_collections: std::env::var("COLLECTION_MAX_COLLECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            stats_update_interval: std::env::var("COLLECTION_STATS_UPDATE_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300), // 5 minutes
            enable_auto_optimization: std::env::var("COLLECTION_AUTO_OPTIMIZATION")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            data_directory: std::env::var("COLLECTION_DATA_DIRECTORY")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::path::PathBuf::from("./data/collections")),
        }
    }
}

impl CollectionManager {
    pub fn new(config: CollectionManagerConfig) -> Self {
        Self {
            collections: Arc::new(RwLock::new(HashMap::new())),
            vector_stores: Arc::new(RwLock::new(HashMap::new())),
            search_services: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create a new collection
    pub async fn create_collection(
        &self,
        request: CreateCollectionRequest,
    ) -> Result<CollectionConfig, Box<dyn std::error::Error + Send + Sync>> {
        let mut collections = self.collections.write().await;

        // Check collection limit
        if collections.len() >= self.config.max_collections {
            return Err(format!(
                "Maximum number of collections ({}) reached",
                self.config.max_collections
            )
            .into());
        }

        // Generate unique ID
        let collection_id = self.generate_collection_id(&request.name);

        // Check for duplicate names
        for config in collections.values() {
            if config.name == request.name {
                return Err(
                    format!("Collection with name '{}' already exists", request.name).into(),
                );
            }
        }

        let now = chrono::Utc::now().timestamp() as u64;

        // Create collection configuration
        let collection_config = CollectionConfig {
            id: collection_id.clone(),
            name: request.name,
            description: request.description,
            vector_config: request.vector_config,
            search_config: request.search_config.unwrap_or_else(|| SearchConfig {
                default_limit: self.config.default_search_limit,
                max_limit: self.config.default_search_limit * 10,
                enable_metadata_filtering: true,
                ranking_weights: HashMap::new(),
            }),
            created_at: now,
            updated_at: now,
            metadata: request.metadata.unwrap_or_default(),
            active: true,
        };

        // Create vector store for the collection
        let vector_store = self.create_vector_store(&collection_config).await?;

        // Create search service for the collection
        let search_service = self
            .create_search_service(&collection_config, vector_store.clone())
            .await?;

        // Initialize statistics
        let initial_stats = CollectionStats {
            collection_id: collection_id.clone(),
            document_count: 0,
            total_size_bytes: 0,
            avg_document_size: 0.0,
            search_stats: SearchStats {
                total_searches: 0,
                avg_results_per_search: 0.0,
                avg_confidence_score: 0.0,
                popular_queries: Vec::new(),
                search_frequency: HashMap::new(),
            },
            performance_metrics: PerformanceMetrics {
                avg_search_time_ms: 0.0,
                p95_search_time_ms: 0.0,
                cache_hit_rate: 0.0,
                memory_usage_mb: 0.0,
                index_efficiency: 1.0,
            },
            last_updated: now,
        };

        // Store everything
        collections.insert(collection_id.clone(), collection_config.clone());
        drop(collections);

        self.vector_stores
            .write()
            .await
            .insert(collection_id.clone(), vector_store);
        self.search_services
            .write()
            .await
            .insert(collection_id.clone(), search_service);
        self.statistics
            .write()
            .await
            .insert(collection_id.clone(), initial_stats);

        // Create collection directory
        let collection_dir = self.config.data_directory.join(&collection_id);
        std::fs::create_dir_all(&collection_dir)?;

        // Save collection configuration
        self.save_collection_config(&collection_config).await?;

        println!(
            "✅ Created collection '{}' with ID: {}",
            collection_config.name, collection_id
        );

        Ok(collection_config)
    }

    /// Update an existing collection
    pub async fn update_collection(
        &self,
        collection_id: &str,
        request: UpdateCollectionRequest,
    ) -> Result<CollectionConfig, Box<dyn std::error::Error + Send + Sync>> {
        let mut collections = self.collections.write().await;

        let collection = collections
            .get_mut(collection_id)
            .ok_or_else(|| format!("Collection '{}' not found", collection_id))?;

        // Update fields
        if let Some(name) = request.name {
            collection.name = name;
        }

        if let Some(description) = request.description {
            collection.description = Some(description);
        }

        if let Some(search_config) = request.search_config {
            collection.search_config = search_config;
        }

        if let Some(metadata) = request.metadata {
            collection.metadata = metadata;
        }

        if let Some(active) = request.active {
            collection.active = active;
        }

        collection.updated_at = chrono::Utc::now().timestamp() as u64;

        let updated_config = collection.clone();
        drop(collections);

        // Save updated configuration
        self.save_collection_config(&updated_config).await?;

        println!("✅ Updated collection '{}'", collection_id);

        Ok(updated_config)
    }

    /// Delete a collection
    pub async fn delete_collection(
        &self,
        collection_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Remove from all maps
        let removed_config = self.collections.write().await.remove(collection_id);
        self.vector_stores.write().await.remove(collection_id);
        self.search_services.write().await.remove(collection_id);
        self.statistics.write().await.remove(collection_id);

        if removed_config.is_none() {
            return Err(format!("Collection '{}' not found", collection_id).into());
        }

        // Remove collection directory
        let collection_dir = self.config.data_directory.join(collection_id);
        if collection_dir.exists() {
            std::fs::remove_dir_all(&collection_dir)?;
        }

        println!("✅ Deleted collection '{}'", collection_id);

        Ok(())
    }

    /// List all collections
    pub async fn list_collections(&self) -> Vec<CollectionConfig> {
        self.collections.read().await.values().cloned().collect()
    }

    /// Get collection by ID
    pub async fn get_collection(&self, collection_id: &str) -> Option<CollectionConfig> {
        self.collections.read().await.get(collection_id).cloned()
    }

    /// Get collection statistics
    pub async fn get_collection_stats(&self, collection_id: &str) -> Option<CollectionStats> {
        self.statistics.read().await.get(collection_id).cloned()
    }

    /// Perform cross-collection search
    pub async fn cross_collection_search(
        &self,
        request: CrossCollectionSearchRequest,
    ) -> Result<CrossCollectionSearchResponse, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();

        // Determine target collections
        let target_collections = if let Some(targets) = request.target_collections {
            targets
        } else {
            // Get all active collections
            self.collections
                .read()
                .await
                .values()
                .filter(|config| config.active)
                .map(|config| config.id.clone())
                .collect()
        };

        let mut collection_results = HashMap::new();
        let mut processing_times = HashMap::new();
        let mut total_results = 0;

        // Search each collection
        for collection_id in &target_collections {
            let collection_start = std::time::Instant::now();

            if let Some(search_service) = self.search_services.read().await.get(collection_id) {
                let mut search_request = request.search_request.clone();

                // Apply per-collection limit if specified
                if let Some(max_per_collection) = request.max_results_per_collection {
                    search_request.limit = Some(max_per_collection);
                }

                match search_service.enhanced_search(search_request).await {
                    Ok(mut results) => {
                        // Tag results with collection ID
                        for result in &mut results {
                            result.collection = Some(collection_id.clone());
                        }

                        total_results += results.len();
                        collection_results.insert(collection_id.clone(), results);
                    }
                    Err(error) => {
                        eprintln!("Error searching collection {}: {}", collection_id, error);
                        collection_results.insert(collection_id.clone(), Vec::new());
                    }
                }
            }

            processing_times.insert(
                collection_id.clone(),
                collection_start.elapsed().as_millis() as u64,
            );
        }

        // Merge results if requested
        let merged_results = if request.merge_results {
            let mut all_results = Vec::new();
            for results in collection_results.values() {
                all_results.extend(results.clone());
            }

            // Sort by confidence score
            all_results
                .sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());

            // Apply global limit
            if let Some(limit) = request.search_request.limit {
                all_results.truncate(limit);
            }

            Some(all_results)
        } else {
            None
        };

        let results_per_collection = collection_results
            .iter()
            .map(|(id, results)| (id.clone(), results.len()))
            .collect();

        let search_stats = CrossCollectionSearchStats {
            collections_searched: target_collections,
            total_results,
            results_per_collection,
            total_processing_time_ms: start_time.elapsed().as_millis() as u64,
            processing_time_per_collection: processing_times,
        };

        Ok(CrossCollectionSearchResponse {
            merged_results,
            collection_results: if request.merge_results {
                None
            } else {
                Some(collection_results)
            },
            search_stats,
        })
    }

    /// Update collection statistics
    pub async fn update_statistics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let collections = self.collections.read().await;
        let mut statistics = self.statistics.write().await;

        for (collection_id, config) in collections.iter() {
            if !config.active {
                continue;
            }

            // Get current stats or create new ones
            let mut stats =
                statistics
                    .get(collection_id)
                    .cloned()
                    .unwrap_or_else(|| CollectionStats {
                        collection_id: collection_id.clone(),
                        document_count: 0,
                        total_size_bytes: 0,
                        avg_document_size: 0.0,
                        search_stats: SearchStats {
                            total_searches: 0,
                            avg_results_per_search: 0.0,
                            avg_confidence_score: 0.0,
                            popular_queries: Vec::new(),
                            search_frequency: HashMap::new(),
                        },
                        performance_metrics: PerformanceMetrics {
                            avg_search_time_ms: 0.0,
                            p95_search_time_ms: 0.0,
                            cache_hit_rate: 0.0,
                            memory_usage_mb: 0.0,
                            index_efficiency: 1.0,
                        },
                        last_updated: chrono::Utc::now().timestamp() as u64,
                    });

            // Update statistics (this would integrate with actual metrics collection)
            stats.last_updated = chrono::Utc::now().timestamp() as u64;

            // TODO: Integrate with actual metrics collection from vector stores and search services
            // For now, we'll just update the timestamp

            statistics.insert(collection_id.clone(), stats);
        }

        Ok(())
    }

    // Helper methods

    fn generate_collection_id(&self, name: &str) -> String {
        // Generate a unique collection ID based on name and timestamp
        let sanitized_name = name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>()
            .to_lowercase();

        let timestamp = chrono::Utc::now().timestamp();
        format!("{}_{}", sanitized_name, timestamp)
    }

    async fn create_vector_store(
        &self,
        _config: &CollectionConfig,
    ) -> Result<Arc<dyn VectorStorage>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Create actual vector store based on configuration
        // For now, return a placeholder
        Err("Vector store creation not implemented".into())
    }

    async fn create_search_service(
        &self,
        _config: &CollectionConfig,
        _vector_store: Arc<dyn VectorStorage>,
    ) -> Result<Arc<EnhancedSearchService>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Create actual search service
        // For now, return a placeholder
        Err("Search service creation not implemented".into())
    }

    async fn save_collection_config(
        &self,
        config: &CollectionConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config_path = self
            .config
            .data_directory
            .join(&config.id)
            .join("config.json");

        std::fs::create_dir_all(config_path.parent().unwrap())?;

        let config_json = serde_json::to_string_pretty(config)?;
        std::fs::write(config_path, config_json)?;

        Ok(())
    }

    /// Start periodic statistics updates
    pub async fn start_stats_updater(self: std::sync::Arc<Self>) {
        let interval = std::time::Duration::from_secs(self.config.stats_update_interval);
        let manager = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                if let Err(error) = manager.update_statistics().await {
                    eprintln!("Error updating collection statistics: {}", error);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collection_manager_creation() {
        let config = CollectionManagerConfig::default();
        let manager = CollectionManager::new(config);

        let collections = manager.list_collections().await;
        assert!(collections.is_empty());
    }

    #[test]
    fn test_vector_store_config_serialization() {
        let config = VectorStoreConfig {
            dimension: 384,
            metric: "cosine".to_string(),
            index_type: "hnsw".to_string(),
            index_params: HashMap::new(),
            max_vectors: Some(100000),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: VectorStoreConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.dimension, deserialized.dimension);
        assert_eq!(config.metric, deserialized.metric);
    }
}
