//! Performance optimization and caching layer for hybrid search
//!
//! This module implements comprehensive caching strategies to optimize
//! hybrid search performance while maintaining quality improvements.

pub mod manager;
pub mod performance;
pub mod integration;
pub mod cache_demo;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::models::SearchRequest;

// Re-export simplified types
pub use manager::HybridSearchCacheManager;
pub use integration::CachedHybridSearchPipeline;

/// Cache configuration for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Query result cache settings
    pub query_cache: QueryCacheConfig,
    /// Vector embedding cache settings
    pub embedding_cache: EmbeddingCacheConfig,
    /// BM25 score cache settings
    pub bm25_cache: BM25CacheConfig,
    /// Fusion result cache settings
    pub fusion_cache: FusionCacheConfig,
    /// Global cache settings
    pub global: GlobalCacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCacheConfig {
    /// Maximum number of cached query results
    pub max_entries: usize,
    /// Time-to-live for cached results
    pub ttl_seconds: u64,
    /// Enable query normalization for cache keys
    pub normalize_queries: bool,
    /// Cache size limit in MB
    pub max_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCacheConfig {
    /// Maximum number of cached embeddings
    pub max_entries: usize,
    /// Time-to-live for cached embeddings
    pub ttl_seconds: u64,
    /// Cache size limit in MB
    pub max_size_mb: usize,
    /// Preload frequently accessed embeddings
    pub enable_preloading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25CacheConfig {
    /// Maximum number of cached BM25 scores
    pub max_entries: usize,
    /// Time-to-live for cached scores
    pub ttl_seconds: u64,
    /// Cache per-term scores separately
    pub cache_term_scores: bool,
    /// Cache size limit in MB
    pub max_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionCacheConfig {
    /// Maximum number of cached fusion results
    pub max_entries: usize,
    /// Time-to-live for cached fusion results
    pub ttl_seconds: u64,
    /// Cache size limit in MB
    pub max_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCacheConfig {
    /// Enable cache statistics collection
    pub enable_statistics: bool,
    /// Cache cleanup interval in seconds
    pub cleanup_interval_seconds: u64,
    /// Memory pressure threshold (0.0-1.0)
    pub memory_pressure_threshold: f64,
    /// Enable cache warming on startup
    pub enable_cache_warming: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            query_cache: QueryCacheConfig {
                max_entries: 1000,
                ttl_seconds: 300, // 5 minutes
                normalize_queries: true,
                max_size_mb: 50,
            },
            embedding_cache: EmbeddingCacheConfig {
                max_entries: 5000,
                ttl_seconds: 3600, // 1 hour
                max_size_mb: 200,
                enable_preloading: true,
            },
            bm25_cache: BM25CacheConfig {
                max_entries: 2000,
                ttl_seconds: 600, // 10 minutes
                cache_term_scores: true,
                max_size_mb: 30,
            },
            fusion_cache: FusionCacheConfig {
                max_entries: 500,
                ttl_seconds: 180, // 3 minutes
                max_size_mb: 20,
            },
            global: GlobalCacheConfig {
                enable_statistics: true,
                cleanup_interval_seconds: 60,
                memory_pressure_threshold: 0.8,
                enable_cache_warming: true,
            },
        }
    }
}

/// Cache entry with TTL and metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub size_bytes: usize,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, size_bytes: usize) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            size_bytes,
        }
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }

    pub fn access(&mut self) -> &T {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.value
    }
}

/// LRU cache with TTL support
#[derive(Debug)]
pub struct LRUCache<K, V> 
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    entries: HashMap<K, CacheEntry<V>>,
    access_order: Vec<K>,
    max_entries: usize,
    max_size_bytes: usize,
    current_size_bytes: usize,
    ttl: Duration,
}

impl<K, V> LRUCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(max_entries: usize, max_size_bytes: usize, ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: Vec::new(),
            max_entries,
            max_size_bytes,
            current_size_bytes: 0,
            ttl,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.get_mut(key) {
            if entry.is_expired(self.ttl) {
                self.remove(key);
                return None;
            }

            // Update access order
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            self.access_order.push(key.clone());

            Some(entry.access().clone())
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V, size_bytes: usize) {
        // Remove existing entry if present
        if self.entries.contains_key(&key) {
            self.remove(&key);
        }

        // Evict entries if necessary
        self.evict_if_needed(size_bytes);

        // Insert new entry
        let entry = CacheEntry::new(value, size_bytes);
        self.current_size_bytes += size_bytes;
        self.entries.insert(key.clone(), entry);
        self.access_order.push(key);
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.remove(key) {
            self.current_size_bytes = self.current_size_bytes.saturating_sub(entry.size_bytes);
            
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            
            Some(entry.value)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.current_size_bytes = 0;
    }

    pub fn cleanup_expired(&mut self) {
        let expired_keys: Vec<K> = self.entries
            .iter()
            .filter(|(_, entry)| entry.is_expired(self.ttl))
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.remove(&key);
        }
    }

    fn evict_if_needed(&mut self, incoming_size: usize) {
        // Check if we need to evict based on count or size
        while (self.entries.len() >= self.max_entries || 
               self.current_size_bytes + incoming_size > self.max_size_bytes) &&
              !self.access_order.is_empty() {
            
            let lru_key = self.access_order.remove(0);
            if let Some(entry) = self.entries.remove(&lru_key) {
                self.current_size_bytes = self.current_size_bytes.saturating_sub(entry.size_bytes);
            }
        }
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn memory_usage(&self) -> usize {
        self.current_size_bytes
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
    pub memory_usage: usize,
    pub hit_rate: f64,
    pub last_updated: u64,
}

impl CacheStatistics {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            size: 0,
            memory_usage: 0,
            hit_rate: 0.0,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn record_hit(&mut self) {
        self.hits += 1;
        self.update_hit_rate();
    }

    pub fn record_miss(&mut self) {
        self.misses += 1;
        self.update_hit_rate();
    }

    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    fn update_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Query cache key for consistent hashing
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct QueryCacheKey {
    pub query: String,
    pub limit: usize,
    pub offset: usize,
    pub filters: Vec<String>,
}

impl QueryCacheKey {
    pub fn new(request: &SearchRequest) -> Self {
        let mut filters = Vec::new();
        
        // Add document type filters
        for doc_type in &request.filters.document_types {
            filters.push(format!("doc_type:{}", doc_type));
        }

        // Add tag filters
        for tag in &request.filters.tags {
            filters.push(format!("tag:{}", tag));
        }

        // Add custom filters
        for (key, value) in &request.filters.custom {
            filters.push(format!("{}:{}", key, value));
        }

        filters.sort(); // Ensure consistent ordering

        Self {
            query: request.query.normalized.clone(),
            limit: request.limit,
            offset: request.offset,
            filters,
        }
    }
}
