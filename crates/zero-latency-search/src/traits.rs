use crate::models::*;
use async_trait::async_trait;
use zero_latency_core::Result;

/// Query enhancement capabilities
#[async_trait]
pub trait QueryEnhancer: Send + Sync {
    async fn enhance(&self, query: &str) -> Result<EnhancedQuery>;
    async fn analyze(&self, query: &str) -> Result<QueryAnalysis>;
}

/// Query validation
#[async_trait]
pub trait QueryValidator: Send + Sync {
    async fn validate(&self, query: &str) -> Result<bool>;
    async fn suggest_corrections(&self, query: &str) -> Result<Vec<String>>;
}

/// Result ranking capabilities
#[async_trait]
pub trait ResultRanker: Send + Sync {
    async fn rank(&self, results: Vec<SearchResult>) -> Result<Vec<SearchResult>>;
    async fn explain_ranking(&self, result: &SearchResult) -> Result<RankingSignals>;
}

/// Search step in a pipeline
#[async_trait]
pub trait SearchStep: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, context: &mut SearchContext) -> Result<()>;
}

/// Search orchestration
#[async_trait]
pub trait SearchOrchestrator: Send + Sync {
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse>;
}

/// Search analytics and insights
#[async_trait]
pub trait SearchAnalytics: Send + Sync {
    async fn record_search(&self, request: &SearchRequest, response: &SearchResponse)
        -> Result<()>;
    async fn get_popular_queries(&self, limit: usize) -> Result<Vec<PopularQuery>>;
    async fn get_search_trends(&self) -> Result<SearchTrends>;
}

/// Search personalization
#[async_trait]
pub trait SearchPersonalizer: Send + Sync {
    async fn personalize_query(&self, query: &str, user_context: &UserContext) -> Result<String>;
    async fn personalize_results(
        &self,
        results: Vec<SearchResult>,
        user_context: &UserContext,
    ) -> Result<Vec<SearchResult>>;
}

/// Popular query information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PopularQuery {
    pub query: String,
    pub count: usize,
    pub avg_results: f32,
    pub success_rate: f32,
}

/// Search trends data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchTrends {
    pub total_searches: usize,
    pub unique_queries: usize,
    pub avg_response_time: f32,
    pub top_categories: Vec<CategoryTrend>,
}

/// Category trend information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CategoryTrend {
    pub category: String,
    pub search_count: usize,
    pub growth_rate: f32,
}

/// User context for personalization
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: Option<String>,
    pub session_id: String,
    pub preferences: UserPreferences,
    pub search_history: Vec<HistoricalSearch>,
}

/// User search preferences
#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub preferred_formats: Vec<String>,
    pub language: Option<String>,
    pub skill_level: SkillLevel,
    pub topics_of_interest: Vec<String>,
}

/// User skill level for content filtering
#[derive(Debug, Clone)]
pub enum SkillLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Historical search entry
#[derive(Debug, Clone)]
pub struct HistoricalSearch {
    pub query: String,
    pub timestamp: zero_latency_core::DateTime<zero_latency_core::Utc>,
    pub clicked_results: Vec<String>,
    pub satisfaction_score: Option<f32>,
}
