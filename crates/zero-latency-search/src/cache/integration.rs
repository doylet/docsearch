//! Cache integration for hybrid search pipeline

use super::HybridSearchCacheManager;
use crate::models::{SearchRequest, SearchResult};
use std::sync::Arc;
use zero_latency_core::Result;

/// Cache-aware hybrid search pipeline
pub struct CachedHybridSearchPipeline {
    cache_manager: Arc<HybridSearchCacheManager>,
}

impl CachedHybridSearchPipeline {
    pub fn new(cache_manager: Arc<HybridSearchCacheManager>) -> Self {
        Self { cache_manager }
    }

    pub async fn search(&self, _request: &SearchRequest) -> Result<Vec<SearchResult>> {
        Ok(Vec::new())
    }

    pub fn cache_manager(&self) -> &Arc<HybridSearchCacheManager> {
        &self.cache_manager
    }
}
