//! Cache performance testing and demonstration
//!
//! This module demonstrates the caching system capabilities
//! and provides performance validation.

use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, Instant};

use zero_latency_core::Result;
use crate::cache::{
    CacheConfig, HybridSearchCacheManager, PerformanceMonitor, 
    CachedHybridSearchPipeline, 
    performance::PerformanceThresholds,
};
use crate::models::SearchRequest;

/// Cache performance demonstration
pub struct CachePerformanceDemo {
    pipeline: CachedHybridSearchPipeline,
    cache_manager: Arc<HybridSearchCacheManager>,
    performance_monitor: Arc<PerformanceMonitor>,
}

impl CachePerformanceDemo {
    pub fn new() -> Self {
        let config = CacheConfig::default();
        let thresholds = PerformanceThresholds::default();
        
        let cache_manager = Arc::new(HybridSearchCacheManager::new(config));
        let performance_monitor = Arc::new(PerformanceMonitor::new(thresholds));
        
        let pipeline = CachedHybridSearchPipeline::new(
            Arc::clone(&cache_manager),
            Arc::clone(&performance_monitor),
        );

        Self {
            pipeline,
            cache_manager,
            performance_monitor,
        }
    }

    /// Run comprehensive cache performance test
    pub async fn run_performance_test(&self) -> Result<CacheTestReport> {
        println!("🚀 Starting Cache Performance Test");
        println!("================================");

        let mut report = CacheTestReport::new();
        
        // Test 1: Cold cache performance
        println!("📊 Test 1: Cold Cache Performance");
        let cold_performance = self.test_cold_cache_performance().await?;
        report.cold_cache_ms = cold_performance;
        println!("   Cold cache average: {:.2}ms", cold_performance);

        // Test 2: Warm cache performance
        println!("📊 Test 2: Warm Cache Performance");
        let warm_performance = self.test_warm_cache_performance().await?;
        report.warm_cache_ms = warm_performance;
        println!("   Warm cache average: {:.2}ms", warm_performance);

        // Test 3: Cache hit rate test
        println!("📊 Test 3: Cache Hit Rate Analysis");
        let hit_rate = self.test_cache_hit_rates().await?;
        report.cache_hit_rate = hit_rate;
        println!("   Overall hit rate: {:.2}%", hit_rate * 100.0);

        // Test 4: Memory efficiency test
        println!("📊 Test 4: Memory Efficiency Analysis");
        let memory_efficiency = self.test_memory_efficiency().await?;
        report.memory_efficiency_score = memory_efficiency;
        println!("   Memory efficiency: {:.2}/10", memory_efficiency);

        // Test 5: Concurrent access test
        println!("📊 Test 5: Concurrent Access Performance");
        let concurrent_performance = self.test_concurrent_access().await?;
        report.concurrent_performance_ms = concurrent_performance;
        println!("   Concurrent average: {:.2}ms", concurrent_performance);

        // Generate final report
        let cache_stats = self.cache_manager.get_statistics().await;
        let performance_report = self.performance_monitor.get_performance_report().await;
        
        report.cache_statistics = cache_stats;
        report.performance_report = performance_report;
        report.calculate_overall_score();

        println!("\n📋 Cache Performance Summary");
        println!("============================");
        println!("Overall Score: {:.1}/100", report.overall_score);
        println!("Cache Hit Rate: {:.1}%", report.cache_hit_rate * 100.0);
        println!("Performance Improvement: {:.1}x", 
            report.cold_cache_ms / report.warm_cache_ms.max(1.0));
        println!("Memory Efficiency: {:.1}/10", report.memory_efficiency_score);

        Ok(report)
    }

    async fn test_cold_cache_performance(&self) -> Result<f64> {
        // Clear all caches
        self.cache_manager.clear_all().await;
        
        let test_queries = self.generate_test_queries();
        let mut total_time = 0.0;
        
        for query in &test_queries {
            let start = Instant::now();
            let _ = self.pipeline.search(query).await?;
            total_time += start.elapsed().as_millis() as f64;
        }
        
        Ok(total_time / test_queries.len() as f64)
    }

    async fn test_warm_cache_performance(&self) -> Result<f64> {
        let test_queries = self.generate_test_queries();
        
        // Prime the cache
        for query in &test_queries {
            let _ = self.pipeline.search(query).await?;
        }
        
        // Measure performance with warm cache
        let mut total_time = 0.0;
        
        for query in &test_queries {
            let start = Instant::now();
            let _ = self.pipeline.search(query).await?;
            total_time += start.elapsed().as_millis() as f64;
        }
        
        Ok(total_time / test_queries.len() as f64)
    }

    async fn test_cache_hit_rates(&self) -> Result<f64> {
        // Clear caches and run test sequence
        self.cache_manager.clear_all().await;
        
        let test_queries = self.generate_test_queries();
        
        // Execute queries multiple times to generate hits
        for _ in 0..3 {
            for query in &test_queries {
                let _ = self.pipeline.search(query).await?;
            }
        }
        
        let stats = self.cache_manager.get_statistics().await;
        Ok(stats.overall_hit_rate())
    }

    async fn test_memory_efficiency(&self) -> Result<f64> {
        let stats = self.cache_manager.get_statistics().await;
        let memory_mb = stats.memory_usage_mb();
        let hit_rate = stats.overall_hit_rate();
        
        // Score based on hit rate per MB of memory used
        let efficiency = if memory_mb > 0.0 {
            (hit_rate * 100.0 / memory_mb).min(10.0)
        } else {
            0.0
        };
        
        Ok(efficiency)
    }

    async fn test_concurrent_access(&self) -> Result<f64> {
        let test_queries = self.generate_test_queries();
        let num_concurrent = 10;
        
        let start = Instant::now();
        
        let mut handles = Vec::new();
        for _ in 0..num_concurrent {
            let pipeline = &self.pipeline;
            let queries = test_queries.clone();
            
            let handle = tokio::spawn(async move {
                for query in queries {
                    let _ = pipeline.search(&query).await;
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all concurrent tasks
        for handle in handles {
            let _ = handle.await;
        }
        
        let total_time = start.elapsed().as_millis() as f64;
        let total_queries = num_concurrent * test_queries.len();
        
        Ok(total_time / total_queries as f64)
    }

    fn generate_test_queries(&self) -> Vec<SearchRequest> {
        vec![
            SearchRequest::new("rust programming language")
                .with_limit(10),
            SearchRequest::new("API documentation")
                .with_limit(5),
            SearchRequest::new("configuration setup guide")
                .with_limit(15),
            SearchRequest::new("performance optimization")
                .with_limit(8),
            SearchRequest::new("search functionality")
                .with_limit(12),
        ]
    }
}

use serde::{Deserialize, Serialize};
use crate::cache::{CacheManagerStatistics, PerformanceReport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheTestReport {
    pub cold_cache_ms: f64,
    pub warm_cache_ms: f64,
    pub cache_hit_rate: f64,
    pub memory_efficiency_score: f64,
    pub concurrent_performance_ms: f64,
    pub overall_score: f64,
    pub cache_statistics: CacheManagerStatistics,
    pub performance_report: PerformanceReport,
}

impl CacheTestReport {
    pub fn new() -> Self {
        Self {
            cold_cache_ms: 0.0,
            warm_cache_ms: 0.0,
            cache_hit_rate: 0.0,
            memory_efficiency_score: 0.0,
            concurrent_performance_ms: 0.0,
            overall_score: 0.0,
            cache_statistics: CacheManagerStatistics {
                query_cache: crate::cache::CacheLayerStats {
                    statistics: crate::cache::performance::CacheStatistics {
                        hits: 0,
                        misses: 0,
                        evictions: 0,
                        size: 0,
                        memory_usage: 0,
                        hit_rate: 0.0,
                        last_updated: 0,
                    },
                    size: 0,
                    memory_usage: 0,
                },
                embedding_cache: crate::cache::CacheLayerStats {
                    statistics: crate::cache::performance::CacheStatistics {
                        hits: 0,
                        misses: 0,
                        evictions: 0,
                        size: 0,
                        memory_usage: 0,
                        hit_rate: 0.0,
                        last_updated: 0,
                    },
                    size: 0,
                    memory_usage: 0,
                },
                bm25_cache: crate::cache::CacheLayerStats {
                    statistics: crate::cache::performance::CacheStatistics {
                        hits: 0,
                        misses: 0,
                        evictions: 0,
                        size: 0,
                        memory_usage: 0,
                        hit_rate: 0.0,
                        last_updated: 0,
                    },
                    size: 0,
                    memory_usage: 0,
                },
                fusion_cache: crate::cache::CacheLayerStats {
                    statistics: crate::cache::performance::CacheStatistics {
                        hits: 0,
                        misses: 0,
                        evictions: 0,
                        size: 0,
                        memory_usage: 0,
                        hit_rate: 0.0,
                        last_updated: 0,
                    },
                    size: 0,
                    memory_usage: 0,
                },
                total_memory_usage: 0,
            },
            performance_report: PerformanceReport {
                query_metrics: crate::cache::performance::QueryMetrics {
                    total_queries: 0,
                    avg_execution_time_ms: 0.0,
                    p95_execution_time_ms: 0.0,
                    p99_execution_time_ms: 0.0,
                    queries_per_second: 0.0,
                    recent_execution_times: Vec::new(),
                    error_count: 0,
                    last_updated: Instant::now(),
                },
                cache_metrics: crate::cache::performance::CachePerformanceMetrics {
                    hit_rate_impact: 0.0,
                    avg_time_saved_ms: 0.0,
                    cache_overhead_ms: 0.0,
                    memory_efficiency: 0.0,
                    layer_effectiveness: std::collections::HashMap::new(),
                },
                resource_metrics: crate::cache::performance::ResourceMetrics {
                    memory_usage_bytes: 0,
                    cpu_usage_percent: 0.0,
                    disk_iops: 0.0,
                    network_bytes_per_sec: 0.0,
                    thread_pool_utilization: 0.0,
                },
                thresholds: PerformanceThresholds::default(),
                recommendations: Vec::new(),
                overall_health_score: 0.0,
            },
        }
    }

    pub fn calculate_overall_score(&mut self) {
        let mut score = 0.0;
        
        // Performance improvement score (30%)
        if self.cold_cache_ms > 0.0 && self.warm_cache_ms > 0.0 {
            let improvement = self.cold_cache_ms / self.warm_cache_ms;
            score += (improvement.min(10.0) / 10.0) * 30.0;
        }
        
        // Cache hit rate score (25%)
        score += self.cache_hit_rate * 25.0;
        
        // Memory efficiency score (20%)
        score += (self.memory_efficiency_score / 10.0) * 20.0;
        
        // Concurrent performance score (15%)
        if self.warm_cache_ms > 0.0 && self.concurrent_performance_ms > 0.0 {
            let concurrent_efficiency = self.warm_cache_ms / self.concurrent_performance_ms;
            score += (concurrent_efficiency.min(2.0) / 2.0) * 15.0;
        }
        
        // Overall health score (10%)
        score += (self.performance_report.overall_health_score / 100.0) * 10.0;
        
        self.overall_score = score.min(100.0);
    }
}
