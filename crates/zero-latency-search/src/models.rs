use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing;
use zero_latency_core::{values::*, DateTime, DocId, Utc, Uuid};

// Re-export fusion types for convenience
pub use crate::fusion::{FromSignals, ScoreBreakdown};

/// Search request with all parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: SearchQuery,
    pub limit: usize,
    pub offset: usize,
    pub filters: SearchFilters,
    pub options: SearchOptions,
}

impl SearchRequest {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: SearchQuery::new(query),
            limit: 20,
            offset: 0,
            filters: SearchFilters::default(),
            options: SearchOptions::default(),
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self.query = self.query.with_limit(limit as u32);
        self
    }

    pub fn with_filters(mut self, filters: SearchFilters) -> Self {
        self.filters = filters;
        self
    }
}

/// Search filters for refining results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    pub document_types: Vec<String>,
    pub date_range: Option<DateRange>,
    pub tags: Vec<String>,
    pub minimum_score: Option<Score>,
    pub custom: HashMap<String, String>,
}

/// Date range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

/// Search options and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub include_ranking_signals: bool,
    pub include_snippets: bool,
    pub snippet_length: usize,
    pub response_format: ResponseFormat,
    pub enable_query_enhancement: bool,
    pub enable_personalization: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            include_ranking_signals: false,
            include_snippets: true,
            snippet_length: 200,
            response_format: ResponseFormat::default(),
            enable_query_enhancement: true,
            enable_personalization: false,
        }
    }
}

/// Individual search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Stable document identifier
    pub doc_id: DocId,
    /// Legacy chunk ID for backward compatibility
    pub chunk_id: Uuid,
    /// Legacy document ID for backward compatibility  
    pub document_id: Uuid,
    /// URI/path to the document
    pub uri: String,
    /// Document title
    pub title: String,
    /// Document path (legacy)
    pub document_path: String,
    /// Full content of the chunk/document
    pub content: String,
    /// Extracted snippet for display
    pub snippet: Option<String>,
    /// Section path breadcrumb (e.g., ["Chapter 1", "Section 1.1"])
    pub section_path: Vec<String>,
    /// Legacy heading path for backward compatibility
    pub heading_path: Vec<String>,
    /// Score breakdown with raw, normalized, and fused scores
    pub scores: ScoreBreakdown,
    /// Legacy final score for backward compatibility
    pub final_score: Score,
    /// Which engines and query variants found this result
    pub from_signals: FromSignals,
    /// Legacy ranking signals for backward compatibility
    pub ranking_signals: Option<RankingSignals>,
    /// URL for accessing the document
    pub url: Option<String>,
    /// Collection this document belongs to
    pub collection: Option<String>,
    /// Custom metadata
    pub custom_metadata: std::collections::HashMap<String, String>,
}

impl SearchResult {
    /// Create a new search result with enhanced scoring
    pub fn new(
        doc_id: DocId,
        uri: String,
        title: String,
        content: String,
        scores: ScoreBreakdown,
        from_signals: FromSignals,
    ) -> Self {
        let chunk_id = Uuid::new_v4();
        let document_id = Uuid::new_v4(); // Legacy compatibility
        let final_score = Score::new(scores.fused).unwrap_or_else(|_| Score::zero());
        
        Self {
            doc_id,
            chunk_id,
            document_id,
            uri: uri.clone(),
            title: title.clone(),
            document_path: uri, // Legacy compatibility
            content,
            snippet: None,
            section_path: Vec::new(),
            heading_path: Vec::new(), // Legacy compatibility
            scores,
            final_score,
            from_signals,
            ranking_signals: None,
            url: None,
            collection: None,
            custom_metadata: HashMap::new(),
        }
    }
    
    /// Set the snippet for this result
    pub fn with_snippet(mut self, snippet: String) -> Self {
        self.snippet = Some(snippet);
        self
    }
    
    /// Set the section path
    pub fn with_section_path(mut self, section_path: Vec<String>) -> Self {
        self.section_path = section_path.clone();
        self.heading_path = section_path; // Legacy compatibility
        self
    }
    
    /// Set the collection
    pub fn with_collection(mut self, collection: String) -> Self {
        self.collection = Some(collection);
        self
    }
    
    /// Set custom metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.custom_metadata = metadata;
        self
    }
}

/// Ranking signals for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingSignals {
    pub vector_similarity: Score,
    pub term_frequency: Score,
    pub document_frequency: Score,
    pub title_boost: f32,
    pub freshness_boost: f32,
    pub custom_signals: HashMap<String, f32>,
}

/// Complete search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: Option<usize>,
    pub search_metadata: SearchMetadata,
    pub pagination: Option<zero_latency_core::models::Pagination>,
}

/// Search execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub query: SearchQuery,
    pub execution_time: Duration,
    pub query_enhancement_applied: bool,
    pub ranking_method: String,
    pub result_sources: Vec<String>,
    pub debug_info: Option<HashMap<String, serde_json::Value>>,
}

/// Enhanced query with synonyms and expansions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedQuery {
    pub original: String,
    pub enhanced: String,
    pub synonyms_added: Vec<String>,
    pub technical_terms: Vec<String>,
    pub expansion_strategy: String,
}

/// Query analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAnalysis {
    pub intent: QueryIntent,
    pub complexity: QueryComplexity,
    pub technical_terms: Vec<String>,
    pub entities: Vec<Entity>,
    pub suggestions: Vec<String>,
}

/// Detected query intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryIntent {
    Documentation,
    Code,
    Tutorial,
    Reference,
    Troubleshooting,
    Unknown,
}

/// Query complexity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryComplexity {
    Simple,   // Single term or phrase
    Moderate, // Multiple terms with basic operators
    Complex,  // Advanced queries with filters
}

/// Named entity in query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub text: String,
    pub entity_type: EntityType,
    pub confidence: Score,
}

/// Types of entities that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    #[serde(rename = "programming_language")]
    ProgrammingLanguage,
    #[serde(rename = "framework")]
    Framework,
    #[serde(rename = "library")]
    Library,
    #[serde(rename = "method")]
    Method,
    #[serde(rename = "class")]
    Class,
    #[serde(rename = "function")]
    Function,
    #[serde(rename = "concept")]
    Concept,
    #[serde(rename = "error")]
    Error,
}

/// Search pipeline step context
#[derive(Debug)]
pub struct SearchContext {
    pub request: SearchRequest,
    pub enhanced_query: Option<EnhancedQuery>,
    pub analysis: Option<QueryAnalysis>,
    pub raw_results: Vec<SearchResult>,
    pub ranked_results: Vec<SearchResult>,
    pub metadata: SearchMetadata,
    pub execution_start: DateTime<Utc>,
}

impl SearchContext {
    pub fn new(request: SearchRequest) -> Self {
        Self {
            request,
            enhanced_query: None,
            analysis: None,
            raw_results: Vec::new(),
            ranked_results: Vec::new(),
            metadata: SearchMetadata {
                query: SearchQuery::new(""),
                execution_time: Duration::from_millis(0),
                query_enhancement_applied: false,
                ranking_method: "unknown".to_string(),
                result_sources: Vec::new(),
                debug_info: None,
            },
            execution_start: Utc::now(),
        }
    }

    pub fn set_enhanced_query(&mut self, enhanced: EnhancedQuery) {
        self.enhanced_query = Some(enhanced);
        self.metadata.query_enhancement_applied = true;
    }

    pub fn set_analysis(&mut self, analysis: QueryAnalysis) {
        self.analysis = Some(analysis);
    }

    pub fn set_raw_results(&mut self, results: Vec<SearchResult>) {
        self.raw_results = results;
    }

    pub fn set_ranked_results(&mut self, results: Vec<SearchResult>) {
        self.ranked_results = results;
    }

    pub fn into_response(mut self) -> SearchResponse {
        self.metadata.execution_time = Utc::now()
            .signed_duration_since(self.execution_start)
            .to_std()
            .unwrap_or_default();

        // Use enhanced query if available, otherwise use the original request query
        if let Some(enhanced) = &self.enhanced_query {
            tracing::info!(
                "[SearchPipeline] Setting enhanced query: '{}' -> '{}'",
                enhanced.original,
                enhanced.enhanced
            );
            // Create an enhanced SearchQuery with the enhanced query text
            self.metadata.query = self
                .request
                .query
                .clone()
                .with_enhancement(&enhanced.enhanced);
        } else {
            tracing::info!("[SearchPipeline] No enhanced query available, using original");
            self.metadata.query = self.request.query.clone();
        }

        SearchResponse {
            results: self.ranked_results,
            total_count: None,
            search_metadata: self.metadata,
            pagination: None,
        }
    }
}
