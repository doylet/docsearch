use std::sync::Arc;
use tokio::sync::Semaphore;
use dashmap::DashMap;
use zero_latency_core::Result;
use zero_latency_search::{SearchOrchestrator, SearchRequest, SearchResponse};
use crate::infrastructure::operations::analytics::ProductionSearchAnalytics;

/// Concurrent search service that prevents blocking between indexing and search operations
///
/// This service addresses the thread locking issue where indexing operations block search requests.
/// It uses:
/// - Semaphores to limit concurrent operations without blocking
/// - DashMap for lock-free request tracking
/// - Separate read/write operation queues
pub struct ConcurrentSearchService {
    /// Underlying search orchestrator
    orchestrator: Arc<dyn SearchOrchestrator>,

    /// Analytics service
    analytics: Arc<ProductionSearchAnalytics>,

    /// Semaphore for read operations (searches) - allows many concurrent reads
    read_semaphore: Arc<Semaphore>,

    /// Semaphore for write operations (indexing) - limits concurrent writes
    write_semaphore: Arc<Semaphore>,

    /// Track ongoing operations to prevent blocking
    active_operations: Arc<DashMap<String, OperationType>>,
}

#[derive(Debug, Clone)]
enum OperationType {
    Search { query: String },
    Index { collection: String },
}

impl ConcurrentSearchService {
    /// Create a new concurrent search service
    pub fn new(
        orchestrator: Arc<dyn SearchOrchestrator>,
        analytics: Arc<ProductionSearchAnalytics>,
    ) -> Self {
        Self {
            orchestrator,
            analytics,
            // Allow many concurrent read operations
            read_semaphore: Arc::new(Semaphore::new(100)),
            // Limit write operations to prevent overwhelming the system
            write_semaphore: Arc::new(Semaphore::new(10)),
            active_operations: Arc::new(DashMap::new()),
        }
    }

    /// Execute a search request without blocking on indexing operations
    pub async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let operation_id = uuid::Uuid::new_v4().to_string();

        // Acquire read permit - this won't block on indexing operations
        let _permit = self.read_semaphore.acquire().await
            .map_err(|_| zero_latency_core::ZeroLatencyError::search("Failed to acquire read permit".to_string()))?;

        // Track this operation
        self.active_operations.insert(
            operation_id.clone(),
            OperationType::Search { query: request.query.raw.clone() }
        );

        // Execute the search
        let result = self.orchestrator.search(request).await;

        // Clean up tracking
        self.active_operations.remove(&operation_id);

        result
    }

    /// Execute indexing operations with proper concurrency control
    pub async fn index_with_concurrency_control<F, Fut>(
        &self,
        collection: String,
        indexing_operation: F,
    ) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let operation_id = uuid::Uuid::new_v4().to_string();

        // Acquire write permit - limits concurrent indexing operations
        let _permit = self.write_semaphore.acquire().await
            .map_err(|_| zero_latency_core::ZeroLatencyError::search("Failed to acquire write permit".to_string()))?;

        // Track this operation
        self.active_operations.insert(
            operation_id.clone(),
            OperationType::Index { collection: collection.clone() }
        );

        // Execute the indexing operation
        let result = indexing_operation().await;

        // Clean up tracking
        self.active_operations.remove(&operation_id);

        result
    }

    /// Get current operation statistics
    pub fn get_operation_stats(&self) -> OperationStats {
        let mut search_count = 0;
        let mut index_count = 0;

        for entry in self.active_operations.iter() {
            match entry.value() {
                OperationType::Search { .. } => search_count += 1,
                OperationType::Index { .. } => index_count += 1,
            }
        }

        OperationStats {
            active_searches: search_count,
            active_indexing: index_count,
            available_read_permits: self.read_semaphore.available_permits(),
            available_write_permits: self.write_semaphore.available_permits(),
        }
    }
}

#[derive(Debug)]
pub struct OperationStats {
    pub active_searches: usize,
    pub active_indexing: usize,
    pub available_read_permits: usize,
    pub available_write_permits: usize,
}
