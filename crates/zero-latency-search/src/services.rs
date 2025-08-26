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
