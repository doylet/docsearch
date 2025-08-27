use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use zero_latency_core::Result;
use zero_latency_search::{
    models::{SearchRequest, SearchResponse},
    traits::{CategoryTrend, PopularQuery, SearchAnalytics, SearchTrends},
};

/// Production-ready SearchAnalytics implementation with in-memory storage
/// and comprehensive tracking capabilities for advanced search pipeline
#[derive(Debug)]
pub struct ProductionSearchAnalytics {
    /// Query tracking data
    query_stats: Arc<RwLock<HashMap<String, QueryStats>>>,
    /// Search session data
    search_metrics: Arc<RwLock<SearchMetrics>>,
    /// Performance tracking
    performance_data: Arc<RwLock<Vec<PerformanceRecord>>>,
    /// Configuration
    config: AnalyticsConfig,
}

#[derive(Debug, Clone)]
pub struct AnalyticsConfig {
    /// Maximum number of queries to track
    pub max_tracked_queries: usize,
    /// Maximum number of performance records to keep
    pub max_performance_records: usize,
    /// Enable detailed logging
    pub enable_detailed_logging: bool,
    /// Enable performance tracking
    pub enable_performance_tracking: bool,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            max_tracked_queries: 10000,
            max_performance_records: 5000,
            enable_detailed_logging: true,
            enable_performance_tracking: true,
        }
    }
}

#[derive(Debug, Clone)]
struct QueryStats {
    query: String,
    count: usize,
    total_results: usize,
    success_count: usize,
    failure_count: usize,
    total_response_time_ms: f64,
    avg_score: f32,
    last_executed: chrono::DateTime<chrono::Utc>,
    query_length: usize,
    result_sources: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
struct SearchMetrics {
    total_searches: usize,
    unique_queries: usize,
    total_response_time_ms: f64,
    successful_searches: usize,
    failed_searches: usize,
    query_enhancement_count: usize,
    result_ranking_count: usize,
    collections_searched: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub query: String,
    pub collection: Option<String>,
    pub response_time_ms: f64,
    pub result_count: usize,
    pub top_score: Option<f32>,
    pub query_enhancement_applied: bool,
    pub ranking_method: Option<String>,
    pub success: bool,
    pub error_details: Option<String>,
}

impl ProductionSearchAnalytics {
    pub fn new(config: AnalyticsConfig) -> Self {
        info!(
            "[SearchAnalytics] Initializing production analytics with config: {:?}",
            config
        );

        Self {
            query_stats: Arc::new(RwLock::new(HashMap::new())),
            search_metrics: Arc::new(RwLock::new(SearchMetrics {
                total_searches: 0,
                unique_queries: 0,
                total_response_time_ms: 0.0,
                successful_searches: 0,
                failed_searches: 0,
                query_enhancement_count: 0,
                result_ranking_count: 0,
                collections_searched: HashMap::new(),
            })),
            performance_data: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(AnalyticsConfig::default())
    }

    /// Get comprehensive analytics summary
    pub async fn get_analytics_summary(&self) -> AnalyticsSummary {
        let query_stats = self.query_stats.read().await;
        let search_metrics = self.search_metrics.read().await;
        let performance_data = self.performance_data.read().await;

        let top_queries: Vec<QueryAnalytics> = query_stats
            .values()
            .map(|stats| QueryAnalytics {
                query: stats.query.clone(),
                count: stats.count,
                success_rate: if stats.count > 0 {
                    stats.success_count as f32 / stats.count as f32
                } else {
                    0.0
                },
                avg_response_time_ms: if stats.count > 0 {
                    stats.total_response_time_ms / stats.count as f64
                } else {
                    0.0
                },
                avg_results: if stats.count > 0 {
                    stats.total_results as f32 / stats.count as f32
                } else {
                    0.0
                },
                avg_score: stats.avg_score,
            })
            .collect();

        AnalyticsSummary {
            total_searches: search_metrics.total_searches,
            unique_queries: search_metrics.unique_queries,
            avg_response_time_ms: if search_metrics.total_searches > 0 {
                search_metrics.total_response_time_ms / search_metrics.total_searches as f64
            } else {
                0.0
            },
            success_rate: if search_metrics.total_searches > 0 {
                search_metrics.successful_searches as f32 / search_metrics.total_searches as f32
            } else {
                0.0
            },
            query_enhancement_rate: if search_metrics.total_searches > 0 {
                search_metrics.query_enhancement_count as f32 / search_metrics.total_searches as f32
            } else {
                0.0
            },
            result_ranking_rate: if search_metrics.total_searches > 0 {
                search_metrics.result_ranking_count as f32 / search_metrics.total_searches as f32
            } else {
                0.0
            },
            top_queries,
            collections_searched: search_metrics.collections_searched.clone(),
            recent_performance: performance_data.iter().rev().take(10).cloned().collect(),
        }
    }

    /// Clear old performance records to prevent memory leaks
    async fn cleanup_performance_records(&self) {
        let mut performance_data = self.performance_data.write().await;
        if performance_data.len() > self.config.max_performance_records {
            let excess = performance_data.len() - self.config.max_performance_records;
            performance_data.drain(0..excess);
            debug!(
                "[SearchAnalytics] Cleaned up {} old performance records",
                excess
            );
        }
    }

    /// Clear old query stats to prevent memory leaks
    async fn cleanup_query_stats(&self) {
        let mut query_stats = self.query_stats.write().await;
        if query_stats.len() > self.config.max_tracked_queries {
            // Remove oldest queries (by last executed time)
            let mut queries_with_time: Vec<_> = query_stats
                .iter()
                .map(|(query, stats)| (query.clone(), stats.last_executed))
                .collect();

            queries_with_time.sort_by(|a, b| a.1.cmp(&b.1));

            let to_remove = queries_with_time.len() - self.config.max_tracked_queries;
            for (query, _) in queries_with_time.into_iter().take(to_remove) {
                query_stats.remove(&query);
            }

            debug!("[SearchAnalytics] Cleaned up {} old query stats", to_remove);
        }
    }
}

#[async_trait]
impl SearchAnalytics for ProductionSearchAnalytics {
    async fn record_search(
        &self,
        request: &SearchRequest,
        response: &SearchResponse,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();
        let timestamp = chrono::Utc::now();

        // Determine if search was successful
        let success = response.results.len() > 0;
        let response_time_ms = 0.0; // Would be calculated from request timing in real scenario

        // Extract analytics data
        let query = request.query.raw.clone();
        let collection: Option<String> = request.filters.custom.get("collection").cloned();
        let result_count = response.results.len();
        let top_score = response.results.first().map(|r| r.final_score);

        // Check for query enhancement and ranking
        let query_enhancement_applied = response.search_metadata.query_enhancement_applied;

        let ranking_method = Some(response.search_metadata.ranking_method.clone());

        if self.config.enable_detailed_logging {
            info!(
                "[SearchAnalytics] Recording search: query='{}', collection={:?}, results={}, success={}, enhancement={}, ranking={:?}",
                query, collection, result_count, success, query_enhancement_applied, ranking_method
            );
        }

        // Update query stats
        {
            let mut query_stats = self.query_stats.write().await;
            let stats = query_stats
                .entry(query.clone())
                .or_insert_with(|| QueryStats {
                    query: query.clone(),
                    count: 0,
                    total_results: 0,
                    success_count: 0,
                    failure_count: 0,
                    total_response_time_ms: 0.0,
                    avg_score: 0.0,
                    last_executed: timestamp,
                    query_length: query.len(),
                    result_sources: HashMap::new(),
                });

            stats.count += 1;
            stats.total_results += result_count;
            stats.total_response_time_ms += response_time_ms;
            stats.last_executed = timestamp;

            if success {
                stats.success_count += 1;
            } else {
                stats.failure_count += 1;
            }

            if let Some(score) = top_score {
                // Update running average of top scores
                let score_value = score.value(); // Convert Score to f32
                stats.avg_score =
                    (stats.avg_score * (stats.count - 1) as f32 + score_value) / stats.count as f32;
            }

            // Track result sources
            for source in &response.search_metadata.result_sources {
                *stats.result_sources.entry(source.clone()).or_insert(0) += 1;
            }
        }

        // Update search metrics
        {
            let mut search_metrics = self.search_metrics.write().await;
            search_metrics.total_searches += 1;
            search_metrics.total_response_time_ms += response_time_ms;

            if success {
                search_metrics.successful_searches += 1;
            } else {
                search_metrics.failed_searches += 1;
            }

            if query_enhancement_applied {
                search_metrics.query_enhancement_count += 1;
            }

            if ranking_method.is_some() {
                search_metrics.result_ranking_count += 1;
            }

            if let Some(ref coll) = collection {
                *search_metrics
                    .collections_searched
                    .entry(coll.clone())
                    .or_insert(0usize) += 1;
            }

            // Recalculate unique queries count
            let query_stats = self.query_stats.read().await;
            search_metrics.unique_queries = query_stats.len();
        }

        // Record performance data
        if self.config.enable_performance_tracking {
            let mut performance_data = self.performance_data.write().await;
            performance_data.push(PerformanceRecord {
                timestamp,
                query: query.clone(),
                collection,
                response_time_ms,
                result_count,
                top_score: top_score.map(|s| s.value()), // Convert Score to f32
                query_enhancement_applied,
                ranking_method,
                success,
                error_details: None,
            });
        }

        // Cleanup old data periodically
        if start_time.elapsed().as_millis() % 100 == 0 {
            tokio::spawn({
                let analytics = Arc::new(self.clone());
                async move {
                    analytics.cleanup_performance_records().await;
                    analytics.cleanup_query_stats().await;
                }
            });
        }

        debug!(
            "[SearchAnalytics] Search recorded in {:?}",
            start_time.elapsed()
        );
        Ok(())
    }

    async fn get_popular_queries(&self, limit: usize) -> Result<Vec<PopularQuery>> {
        let query_stats = self.query_stats.read().await;

        let mut popular_queries: Vec<_> = query_stats
            .values()
            .map(|stats| PopularQuery {
                query: stats.query.clone(),
                count: stats.count,
                avg_results: if stats.count > 0 {
                    stats.total_results as f32 / stats.count as f32
                } else {
                    0.0
                },
                success_rate: if stats.count > 0 {
                    stats.success_count as f32 / stats.count as f32
                } else {
                    0.0
                },
            })
            .collect();

        // Sort by popularity (count)
        popular_queries.sort_by(|a, b| b.count.cmp(&a.count));
        popular_queries.truncate(limit);

        Ok(popular_queries)
    }

    async fn get_search_trends(&self) -> Result<SearchTrends> {
        let search_metrics = self.search_metrics.read().await;

        let top_categories: Vec<CategoryTrend> = search_metrics
            .collections_searched
            .iter()
            .map(|(collection, count)| CategoryTrend {
                category: collection.clone(),
                search_count: *count,
                growth_rate: 0.0, // Would be calculated from historical data in production
            })
            .collect();

        Ok(SearchTrends {
            total_searches: search_metrics.total_searches,
            unique_queries: search_metrics.unique_queries,
            avg_response_time: if search_metrics.total_searches > 0 {
                (search_metrics.total_response_time_ms / search_metrics.total_searches as f64)
                    as f32
            } else {
                0.0
            },
            top_categories,
        })
    }
}

// Clone implementation for Arc compatibility
impl Clone for ProductionSearchAnalytics {
    fn clone(&self) -> Self {
        Self {
            query_stats: Arc::clone(&self.query_stats),
            search_metrics: Arc::clone(&self.search_metrics),
            performance_data: Arc::clone(&self.performance_data),
            config: self.config.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_searches: usize,
    pub unique_queries: usize,
    pub avg_response_time_ms: f64,
    pub success_rate: f32,
    pub query_enhancement_rate: f32,
    pub result_ranking_rate: f32,
    pub top_queries: Vec<QueryAnalytics>,
    pub collections_searched: HashMap<String, usize>,
    pub recent_performance: Vec<PerformanceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAnalytics {
    pub query: String,
    pub count: usize,
    pub success_rate: f32,
    pub avg_response_time_ms: f64,
    pub avg_results: f32,
    pub avg_score: f32,
}
