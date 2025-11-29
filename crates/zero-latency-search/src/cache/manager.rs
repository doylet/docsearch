//! Multi-layer cache manager for hybrid search optimization
//!
//! This module provides a comprehensive caching solution that spans
//! multiple layers of the hybrid search pipeline.

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use super::{
    CacheConfig, LRUCache, QueryCacheKey,
};
use super::performance::CacheStatistics;
use crate::models::{SearchRequest, SearchResult};
use zero_latency_core::Result;

/// Multi-layer cache manager for hybrid search
pub struct HybridSearchCacheManager {
    /// Query result cache
    query_cache: Arc<RwLock<LRUCache<QueryCacheKey, Vec<SearchResult>>>>,
    /// Vector embedding cache
    embedding_cache: Arc<RwLock<LRUCache<String, Vec<f32>>>>,
    /// BM25 score cache
    bm25_cache: Arc<RwLock<LRUCache<String, HashMap<String, f64>>>>,
    /// Fusion result cache
    fusion_cache: Arc<RwLock<LRUCache<String, Vec<(String, f64)>>>>,
    /// Cache configuration
    config: CacheConfig,
    /// Global cache statistics
    statistics: Arc<RwLock<CacheStatistics>>,
    /// Cache statistics
    query_stats: Arc<RwLock<CacheStatistics>>,
    embedding_stats: Arc<RwLock<CacheStatistics>>,
    bm25_stats: Arc<RwLock<CacheStatistics>>,
    fusion_stats: Arc<RwLock<CacheStatistics>>,
}

impl HybridSearchCacheManager {
    pub fn new(config: CacheConfig) -> Self {
        let query_cache = Arc::new(RwLock::new(LRUCache::new(
            config.query_cache.max_entries,
            config.query_cache.max_size_mb * 1024 * 1024,
            Duration::from_secs(config.query_cache.ttl_seconds),
        )));

        let embedding_cache = Arc::new(RwLock::new(LRUCache::new(
            config.embedding_cache.max_entries,
            config.embedding_cache.max_size_mb * 1024 * 1024,
            Duration::from_secs(config.embedding_cache.ttl_seconds),
        )));

        let bm25_cache = Arc::new(RwLock::new(LRUCache::new(
            config.bm25_cache.max_entries,
            config.bm25_cache.max_size_mb * 1024 * 1024,
            Duration::from_secs(config.bm25_cache.ttl_seconds),
        )));

        let fusion_cache = Arc::new(RwLock::new(LRUCache::new(
            config.fusion_cache.max_entries,
            config.fusion_cache.max_size_mb * 1024 * 1024,
            Duration::from_secs(config.fusion_cache.ttl_seconds),
        )));

        let manager = Self {
            query_cache,
            embedding_cache,
            bm25_cache,
            fusion_cache,
            config: config.clone(),
            statistics: Arc::new(RwLock::new(CacheStatistics::new())),
            query_stats: Arc::new(RwLock::new(CacheStatistics::new())),
            embedding_stats: Arc::new(RwLock::new(CacheStatistics::new())),
            bm25_stats: Arc::new(RwLock::new(CacheStatistics::new())),
            fusion_stats: Arc::new(RwLock::new(CacheStatistics::new())),
        };

        // Start background cleanup task
        if config.global.enable_statistics {
            manager.start_cleanup_task();
        }

        manager
    }

    /// Get cached query results
    pub async fn get_query_results(&self, request: &SearchRequest) -> Option<Vec<SearchResult>> {
        let key = QueryCacheKey::new(request);
        let mut cache = self.query_cache.write().await;
        let mut stats = self.query_stats.write().await;

        if let Some(results) = cache.get(&key) {
            stats.record_hit();
            Some(results)
        } else {
            stats.record_miss();
            None
        }
    }

    /// Cache query results
    pub async fn cache_query_results(&self, request: &SearchRequest, results: Vec<SearchResult>) -> Result<()> {
        let key = QueryCacheKey::new(request);
        let size_bytes = self.estimate_results_size(&results);

        let mut cache = self.query_cache.write().await;
        cache.insert(key, results, size_bytes);

        Ok(())
    }

    /// Get cached document embedding
    pub async fn get_document_embedding(&self, doc_id: &str) -> Option<Vec<f32>> {
        let mut cache = self.embedding_cache.write().await;
        let mut stats = self.embedding_stats.write().await;

        if let Some(embedding) = cache.get(&doc_id.to_string()) {
            stats.record_hit();
            Some(embedding)
        } else {
            stats.record_miss();
            None
        }
    }

    /// Cache document embedding
    pub async fn cache_document_embedding(&self, doc_id: &str, embedding: Vec<f32>) -> Result<()> {
        let size_bytes = embedding.len() * std::mem::size_of::<f32>();

        let mut cache = self.embedding_cache.write().await;
        cache.insert(doc_id.to_string(), embedding, size_bytes);

        Ok(())
    }

    /// Get cached BM25 scores
    pub async fn get_bm25_scores(&self, query_hash: &str) -> Option<HashMap<String, f64>> {
        let mut cache = self.bm25_cache.write().await;
        let mut stats = self.bm25_stats.write().await;

        if let Some(scores) = cache.get(&query_hash.to_string()) {
            stats.record_hit();
            Some(scores)
        } else {
            stats.record_miss();
            None
        }
    }

    /// Cache BM25 scores
    pub async fn cache_bm25_scores(&self, query_hash: &str, scores: HashMap<String, f64>) -> Result<()> {
        let size_bytes = scores.len() * (std::mem::size_of::<String>() + std::mem::size_of::<f64>());

        let mut cache = self.bm25_cache.write().await;
        cache.insert(query_hash.to_string(), scores, size_bytes);

        Ok(())
    }

    /// Get cached fusion results
    pub async fn get_fusion_results(&self, fusion_key: &str) -> Option<Vec<(String, f64)>> {
        let mut cache = self.fusion_cache.write().await;
        let mut stats = self.fusion_stats.write().await;

        if let Some(results) = cache.get(&fusion_key.to_string()) {
            stats.record_hit();
            Some(results)
        } else {
            stats.record_miss();
            None
        }
    }

    /// Cache fusion results
    pub async fn cache_fusion_results(&self, fusion_key: &str, results: Vec<(String, f64)>) -> Result<()> {
        let size_bytes = results.len() * (std::mem::size_of::<String>() + std::mem::size_of::<f64>());

        let mut cache = self.fusion_cache.write().await;
        cache.insert(fusion_key.to_string(), results, size_bytes);

        Ok(())
    }

    /// Get comprehensive cache statistics
    pub async fn get_statistics(&self) -> CacheManagerStatistics {
        let query_stats = self.query_stats.read().await.clone();
        let embedding_stats = self.embedding_stats.read().await.clone();
        let bm25_stats = self.bm25_stats.read().await.clone();
        let fusion_stats = self.fusion_stats.read().await.clone();

        let query_cache = self.query_cache.read().await;
        let embedding_cache = self.embedding_cache.read().await;
        let bm25_cache = self.bm25_cache.read().await;
        let fusion_cache = self.fusion_cache.read().await;

        CacheManagerStatistics {
            query_cache: CacheLayerStats {
                statistics: query_stats,
                size: query_cache.size(),
                memory_usage: query_cache.memory_usage(),
            },
            embedding_cache: CacheLayerStats {
                statistics: embedding_stats,
                size: embedding_cache.size(),
                memory_usage: embedding_cache.memory_usage(),
            },
            bm25_cache: CacheLayerStats {
                statistics: bm25_stats,
                size: bm25_cache.size(),
                memory_usage: bm25_cache.memory_usage(),
            },
            fusion_cache: CacheLayerStats {
                statistics: fusion_stats,
                size: fusion_cache.size(),
                memory_usage: fusion_cache.memory_usage(),
            },
            total_memory_usage: query_cache.memory_usage() +
                              embedding_cache.memory_usage() +
                              bm25_cache.memory_usage() +
                              fusion_cache.memory_usage(),
        }
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        let mut query_cache = self.query_cache.write().await;
        let mut embedding_cache = self.embedding_cache.write().await;
        let mut bm25_cache = self.bm25_cache.write().await;
        let mut fusion_cache = self.fusion_cache.write().await;

        query_cache.clear();
        embedding_cache.clear();
        bm25_cache.clear();
        fusion_cache.clear();
    }

    /// Warm up caches with frequently accessed data
    pub async fn warm_up(&self, popular_queries: Vec<SearchRequest>) -> Result<()> {
        if !self.config.global.enable_cache_warming {
            return Ok(());
        }

        // This would be implemented to pre-populate caches
        // with frequently accessed data
        for _query in popular_queries {
            // Implementation would:
            // 1. Execute search for popular queries
            // 2. Cache the results
            // 3. Pre-load embeddings for common documents
            // 4. Cache BM25 scores for frequent terms
        }

        Ok(())
    }

    /// Record a cache hit
    pub async fn record_cache_hit(&self) {
        let mut stats = self.statistics.write().await;
        stats.record_hit();
    }

    /// Record a cache miss
    pub async fn record_cache_miss(&self) {
        let mut stats = self.statistics.write().await;
        stats.record_miss();
    }

    fn start_cleanup_task(&self) {
        let query_cache = Arc::clone(&self.query_cache);
        let embedding_cache = Arc::clone(&self.embedding_cache);
        let bm25_cache = Arc::clone(&self.bm25_cache);
        let fusion_cache = Arc::clone(&self.fusion_cache);
        let cleanup_interval = Duration::from_secs(self.config.global.cleanup_interval_seconds);

        tokio::spawn(async move {
            let mut interval = interval(cleanup_interval);

            loop {
                interval.tick().await;

                // Cleanup expired entries
                {
                    let mut cache = query_cache.write().await;
                    cache.cleanup_expired();
                }
                {
                    let mut cache = embedding_cache.write().await;
                    cache.cleanup_expired();
                }
                {
                    let mut cache = bm25_cache.write().await;
                    cache.cleanup_expired();
                }
                {
                    let mut cache = fusion_cache.write().await;
                    cache.cleanup_expired();
                }
            }
        });
    }

    fn estimate_results_size(&self, results: &[SearchResult]) -> usize {
        results.iter().map(|r| {
            std::mem::size_of::<SearchResult>() +
            r.uri.len() +
            r.content.len() +
            r.custom_metadata.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>()
        }).sum()
    }
}

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Cache layer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheLayerStats {
    pub statistics: CacheStatistics,
    pub size: usize,
    pub memory_usage: usize,
}

/// Comprehensive cache manager statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheManagerStatistics {
    pub query_cache: CacheLayerStats,
    pub embedding_cache: CacheLayerStats,
    pub bm25_cache: CacheLayerStats,
    pub fusion_cache: CacheLayerStats,
    pub total_memory_usage: usize,
}

impl CacheManagerStatistics {
    /// Calculate overall cache efficiency
    pub fn overall_hit_rate(&self) -> f64 {
        0.85 // Placeholder implementation
    }

    /// Get memory usage in MB
    pub fn memory_usage_mb(&self) -> f64 {
        self.total_memory_usage as f64 / (1024.0 * 1024.0)
    }
}
