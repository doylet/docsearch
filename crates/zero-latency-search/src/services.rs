/// Stub implementation of SearchAnalytics for testing
pub struct StubSearchAnalytics;

#[async_trait]
impl SearchAnalytics for StubSearchAnalytics {
    async fn record_search(
        &self,
        request: &SearchRequest,
        response: &SearchResponse,
    ) -> Result<()> {
        println!(
            "[StubAnalytics] record_search: query='{}', results={}",
            request.query.raw,
            response.results.len()
        );
        Ok(())
    }

    async fn get_popular_queries(&self, _limit: usize) -> Result<Vec<PopularQuery>> {
        Ok(vec![])
    }

    async fn get_search_trends(&self) -> Result<SearchTrends> {
        Ok(SearchTrends {
            total_searches: 0,
            unique_queries: 0,
            avg_response_time: 0.0,
            top_categories: vec![],
        })
    }
}
use crate::{models::*, traits::*};
use async_trait::async_trait;
use std::sync::Arc;
use zero_latency_core::Result;

/// Search pipeline that executes steps in sequence
pub struct SearchPipeline {
    steps: Vec<Box<dyn SearchStep>>,
}

impl SearchPipeline {
    pub fn builder() -> SearchPipelineBuilder {
        SearchPipelineBuilder::new()
    }

    pub async fn execute(&self, request: SearchRequest) -> Result<SearchResponse> {
        let mut context = SearchContext::new(request);

        for step in &self.steps {
            step.execute(&mut context).await?;
        }

        Ok(context.into_response())
    }
}

/// Builder for search pipelines
pub struct SearchPipelineBuilder {
    steps: Vec<Box<dyn SearchStep>>,
}

impl SearchPipelineBuilder {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(mut self, step: Box<dyn SearchStep>) -> Self {
        self.steps.push(step);
        self
    }

    pub fn build(self) -> SearchPipeline {
        SearchPipeline { steps: self.steps }
    }
}

/// Analytics step for recording search analytics
pub struct AnalyticsStep {
    analytics: Arc<dyn SearchAnalytics>,
}

impl AnalyticsStep {
    pub fn new(analytics: Arc<dyn SearchAnalytics>) -> Self {
        Self { analytics }
    }
}

#[async_trait]
impl SearchStep for AnalyticsStep {
    fn name(&self) -> &str {
        "analytics"
    }

    async fn execute(&self, context: &mut SearchContext) -> Result<()> {
        // Build a SearchResponse for analytics without consuming context
        let response = SearchResponse {
            results: context.ranked_results.clone(),
            total_count: None,
            search_metadata: context.metadata.clone(),
            pagination: None,
        };
        // Ignore errors from analytics for now
        let _ = self
            .analytics
            .record_search(&context.request, &response)
            .await;
        Ok(())
    }
}

impl Default for SearchPipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic query enhancement step
pub struct QueryEnhancementStep {
    enhancer: Arc<dyn QueryEnhancer>,
}

impl QueryEnhancementStep {
    pub fn new(enhancer: Arc<dyn QueryEnhancer>) -> Self {
        Self { enhancer }
    }
}

#[async_trait]
impl SearchStep for QueryEnhancementStep {
    fn name(&self) -> &str {
        "query_enhancement"
    }

    async fn execute(&self, context: &mut SearchContext) -> Result<()> {
        let enhanced = self.enhancer.enhance(&context.request.query.raw).await?;
        context.set_enhanced_query(enhanced);
        Ok(())
    }
}

/// Result ranking step
pub struct ResultRankingStep {
    ranker: Arc<dyn ResultRanker>,
}

impl ResultRankingStep {
    pub fn new(ranker: Arc<dyn ResultRanker>) -> Self {
        Self { ranker }
    }
}

#[async_trait]
impl SearchStep for ResultRankingStep {
    fn name(&self) -> &str {
        "result_ranking"
    }

    async fn execute(&self, context: &mut SearchContext) -> Result<()> {
        let ranked_results = self.ranker.rank(context.raw_results.clone()).await?;
        context.set_ranked_results(ranked_results);
        context.metadata.ranking_method = "multi_factor".to_string();
        Ok(())
    }
}

/// Simple search orchestrator implementation
pub struct SimpleSearchOrchestrator {
    pipeline: SearchPipeline,
}

impl SimpleSearchOrchestrator {
    pub fn new(pipeline: SearchPipeline) -> Self {
        Self { pipeline }
    }
}

#[async_trait]
impl SearchOrchestrator for SimpleSearchOrchestrator {
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        self.pipeline.execute(request).await
    }
}
