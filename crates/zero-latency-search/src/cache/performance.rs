//! Performance monitoring and optimization utilities for caching

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Performance thresholds for cache optimization
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_query_latency_ms: f64,
    pub min_hit_rate: f64,
    pub max_memory_usage_mb: f64,
    pub max_cache_size: usize,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_query_latency_ms: 100.0,
            min_hit_rate: 0.7,
            max_memory_usage_mb: 512.0,
            max_cache_size: 10_000,
        }
    }
}

/// Cache statistics for performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub query_cache_hits: u64,
    pub query_cache_misses: u64,
    pub embedding_cache_hits: u64,
    pub embedding_cache_misses: u64,
    pub bm25_cache_hits: u64,
    pub bm25_cache_misses: u64,
    pub fusion_cache_hits: u64,
    pub fusion_cache_misses: u64,
    pub memory_usage_bytes: usize,
    pub last_updated: u64,
}

impl CacheStatistics {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            query_cache_hits: 0,
            query_cache_misses: 0,
            embedding_cache_hits: 0,
            embedding_cache_misses: 0,
            bm25_cache_hits: 0,
            bm25_cache_misses: 0,
            fusion_cache_hits: 0,
            fusion_cache_misses: 0,
            memory_usage_bytes: 0,
            last_updated: now,
        }
    }

    pub fn record_hit(&mut self) {
        self.query_cache_hits += 1;
    }

    pub fn record_miss(&mut self) {
        self.query_cache_misses += 1;
    }

    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.query_cache_hits + self.embedding_cache_hits + 
                        self.bm25_cache_hits + self.fusion_cache_hits;
        let total_requests = total_hits + self.query_cache_misses + 
                           self.embedding_cache_misses + self.bm25_cache_misses + 
                           self.fusion_cache_misses;
        
        if total_requests == 0 {
            0.0
        } else {
            total_hits as f64 / total_requests as f64
        }
    }
}

/// Performance monitor for cache optimization
#[derive(Debug)]
pub struct PerformanceMonitor {
    thresholds: PerformanceThresholds,
}

impl PerformanceMonitor {
    pub fn new(thresholds: PerformanceThresholds) -> Self {
        Self { thresholds }
    }

    pub async fn get_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            overall_health_score: 85.0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub overall_health_score: f64,
    pub timestamp: u64,
}
