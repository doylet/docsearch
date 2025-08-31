//! Hybrid search pipeline combining BM25 and vector search

use async_trait::async_trait;
use tracing;

use zero_latency_core::Result;

use crate::fusion::ScoreFusion;
use crate::models::SearchContext;
use crate::traits::SearchStep;
use crate::vector_search::VectorSearchStep;
use crate::bm25::BM25SearchStep;

/// Hybrid search step that combines BM25 and vector search results
pub struct HybridSearchStep {
    bm25_step: BM25SearchStep,
    vector_step: VectorSearchStep,
    score_fusion: ScoreFusion,
}

impl HybridSearchStep {
    pub fn new(
        bm25_step: BM25SearchStep,
        vector_step: VectorSearchStep,
        score_fusion: ScoreFusion,
    ) -> Self {
        Self {
            bm25_step,
            vector_step,
            score_fusion,
        }
    }
}

#[async_trait]
impl SearchStep for HybridSearchStep {
    fn name(&self) -> &str {
        "hybrid_search"
    }

    async fn execute(&self, context: &mut SearchContext) -> Result<()> {
        tracing::info!("🔀 HybridSearchStep: Starting parallel BM25 and vector search");
        
        // Create separate contexts for each search engine
        let mut bm25_context = context.clone();
        let mut vector_context = context.clone();
        
        // Execute both searches in parallel
        let (_bm25_result, _vector_result) = tokio::try_join!(
            self.bm25_step.execute(&mut bm25_context),
            self.vector_step.execute(&mut vector_context)
        )?;
        
        tracing::info!(
            "📊 HybridSearchStep: BM25 found {} results, Vector found {} results",
            bm25_context.raw_results.len(),
            vector_context.raw_results.len()
        );
        
        // Collect all results for fusion
        let mut all_results = Vec::new();
        all_results.extend(bm25_context.raw_results);
        all_results.extend(vector_context.raw_results);
        
        // Apply score fusion
        let fused_results = self.score_fusion.fuse_results(all_results)?;
        
        tracing::info!(
            "✨ HybridSearchStep: Fusion produced {} results",
            fused_results.len()
        );
        
        // Update context with fused results
        context.raw_results = fused_results;
        context.metadata.result_sources.extend([
            "hybrid".to_string(),
            "bm25".to_string(), 
            "vector".to_string()
        ]);
        context.metadata.ranking_method = "hybrid_fusion".to_string();
        
        Ok(())
    }
}

/// Advanced hybrid search step with configurable execution modes
pub struct AdvancedHybridSearchStep {
    bm25_step: BM25SearchStep,
    vector_step: VectorSearchStep,
    score_fusion: ScoreFusion,
    execution_mode: HybridExecutionMode,
}

#[derive(Debug, Clone)]
pub enum HybridExecutionMode {
    /// Execute both searches in parallel (default)
    Parallel,
    /// Execute vector search first, then BM25 for refinement
    Sequential {
        /// If vector search returns fewer than this, run BM25 to supplement
        vector_threshold: usize,
    },
    /// Execute BM25 first to get candidates, then vector search for reranking
    BM25ThenVector {
        /// Number of top BM25 results to rerank with vector search
        rerank_count: usize,
    },
}

impl Default for HybridExecutionMode {
    fn default() -> Self {
        Self::Parallel
    }
}

impl AdvancedHybridSearchStep {
    pub fn new(
        bm25_step: BM25SearchStep,
        vector_step: VectorSearchStep,
        score_fusion: ScoreFusion,
        execution_mode: HybridExecutionMode,
    ) -> Self {
        Self {
            bm25_step,
            vector_step,
            score_fusion,
            execution_mode,
        }
    }
}

#[async_trait]
impl SearchStep for AdvancedHybridSearchStep {
    fn name(&self) -> &str {
        "advanced_hybrid_search"
    }

    async fn execute(&self, context: &mut SearchContext) -> Result<()> {
        match &self.execution_mode {
            HybridExecutionMode::Parallel => {
                self.execute_parallel(context).await
            }
            HybridExecutionMode::Sequential { vector_threshold } => {
                self.execute_sequential(context, *vector_threshold).await
            }
            HybridExecutionMode::BM25ThenVector { rerank_count } => {
                self.execute_bm25_then_vector(context, *rerank_count).await
            }
        }
    }
}

impl AdvancedHybridSearchStep {
    async fn execute_parallel(&self, context: &mut SearchContext) -> Result<()> {
        tracing::info!("🔀 AdvancedHybridSearchStep: Parallel execution mode");
        
        let mut bm25_context = context.clone();
        let mut vector_context = context.clone();
        
        let (_bm25_result, _vector_result) = tokio::try_join!(
            self.bm25_step.execute(&mut bm25_context),
            self.vector_step.execute(&mut vector_context)
        )?;
        
        let mut all_results = Vec::new();
        all_results.extend(bm25_context.raw_results);
        all_results.extend(vector_context.raw_results);
        
        let fused_results = self.score_fusion.fuse_results(all_results)?;
        
        context.raw_results = fused_results;
        context.metadata.result_sources.push("hybrid_parallel".to_string());
        
        Ok(())
    }
    
    async fn execute_sequential(&self, context: &mut SearchContext, vector_threshold: usize) -> Result<()> {
        tracing::info!("🔄 AdvancedHybridSearchStep: Sequential execution mode (threshold: {})", vector_threshold);
        
        // Execute vector search first
        let mut vector_context = context.clone();
        self.vector_step.execute(&mut vector_context).await?;
        
        let vector_count = vector_context.raw_results.len();
        tracing::info!("📊 Vector search returned {} results", vector_count);
        
        let mut all_results = vector_context.raw_results;
        
        // If vector search didn't return enough results, supplement with BM25
        if vector_count < vector_threshold {
            tracing::info!("🔍 Vector results below threshold, running BM25 search");
            let mut bm25_context = context.clone();
            self.bm25_step.execute(&mut bm25_context).await?;
            all_results.extend(bm25_context.raw_results);
        }
        
        let fused_results = if all_results.len() > vector_count {
            // We have results from both engines, apply fusion
            self.score_fusion.fuse_results(all_results)?
        } else {
            // Only vector results, no fusion needed
            all_results
        };
        
        context.raw_results = fused_results;
        context.metadata.result_sources.push("hybrid_sequential".to_string());
        
        Ok(())
    }
    
    async fn execute_bm25_then_vector(&self, context: &mut SearchContext, rerank_count: usize) -> Result<()> {
        tracing::info!("🎯 AdvancedHybridSearchStep: BM25-then-Vector execution mode (rerank: {})", rerank_count);
        
        // Execute BM25 search first to get candidates
        let mut bm25_context = context.clone();
        self.bm25_step.execute(&mut bm25_context).await?;
        
        tracing::info!("📊 BM25 search returned {} candidates", bm25_context.raw_results.len());
        
        if bm25_context.raw_results.is_empty() {
            context.raw_results = Vec::new();
            context.metadata.result_sources.push("bm25_only".to_string());
            return Ok(());
        }
        
        // Take top candidates for vector reranking
        let candidates_to_rerank = bm25_context.raw_results.len().min(rerank_count);
        let top_candidates = bm25_context.raw_results.into_iter().take(candidates_to_rerank).collect::<Vec<_>>();
        
        // Execute vector search for reranking
        let mut vector_context = context.clone();
        self.vector_step.execute(&mut vector_context).await?;
        
        // Merge results and apply fusion
        let mut all_results = top_candidates;
        all_results.extend(vector_context.raw_results);
        
        let fused_results = self.score_fusion.fuse_results(all_results)?;
        
        context.raw_results = fused_results;
        context.metadata.result_sources.push("hybrid_bm25_then_vector".to_string());
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test helper to create a mock hybrid search step
    async fn _create_test_hybrid_step() -> Result<HybridSearchStep> {
        // This would need actual implementations in a real test
        todo!("Implement test hybrid step creation")
    }
    
    #[tokio::test]
    #[ignore] // Remove ignore when implementing actual tests
    async fn test_parallel_execution() {
        // Test would verify that both BM25 and vector searches execute in parallel
        // and results are properly fused
    }
    
    #[tokio::test]
    #[ignore] // Remove ignore when implementing actual tests
    async fn test_sequential_execution() {
        // Test would verify sequential execution with threshold-based BM25 fallback
    }
    
    #[tokio::test]
    #[ignore] // Remove ignore when implementing actual tests
    async fn test_bm25_then_vector_execution() {
        // Test would verify BM25 candidate retrieval followed by vector reranking
    }
}
