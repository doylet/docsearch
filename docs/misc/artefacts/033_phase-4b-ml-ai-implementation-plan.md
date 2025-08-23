# 033 - Phase 4B Implementation Plan: Advanced ML/AI Features

**Date:** August 21, 2025  
**Status:** ✅ COMPLETE  
**Target:** Enhance search intelligence with machine learning capabilities  
**Timeline:** 4 weeks  
**Priority:** 1 (Immediate implementation)  
**Related:** [031](031_phase-4-strategic-roadmap-analysis.md), [032](032_native-search-integration-plan.md), [034](034_phase-4a-frontend-ui-plans.md)

## 🎯 **Phase 4B Overview**

Building on Phase 3's 78% relevance improvements, Phase 4B will add sophisticated ML capabilities to achieve state-of-the-art search intelligence.

## 🚀 **Week 1: BERT Re-ranking Integration**

### **Goal:** Implement cross-encoder re-ranking for semantic similarity

**Technical Implementation:**

```rust
// services/doc-indexer/src/reranking_service.rs
pub struct BertReranker {
    model: Arc<OrtSession>,
    tokenizer: Arc<Tokenizer>,
    config: RerankingConfig,
}

impl BertReranker {
    pub async fn rerank_candidates(
        &self,
        query: &str,
        candidates: Vec<SearchResult>,
        top_k: usize,
    ) -> Result<Vec<RankedResult>> {
        // 1. Create query-document pairs
        // 2. Tokenize and encode pairs
        // 3. Run BERT inference
        // 4. Sort by cross-encoder scores
        // 5. Return top-k results
    }
}
```

**Models to Integrate:**
- **Primary:** `sentence-transformers/ms-marco-MiniLM-L-6-v2` (80MB)
- **Alternative:** `cross-encoder/ms-marco-TinyBERT-L-2-v2` (45MB)
- **Format:** ONNX for consistent runtime

**Search Pipeline Enhancement:**
```rust
// Enhanced search flow
SearchPipeline {
    1. Query Enhancement      // Phase 3 (existing)
    2. Vector Similarity      // Phase 3 (existing) → Top 50
    3. BERT Re-ranking        // Phase 4B (NEW) → Top 10
    4. Multi-factor Scoring   // Phase 3 (enhanced)
    5. Response Assembly      // Phase 3 (existing)
}
```

**Performance Targets:**
- **Latency:** <200ms total (100ms vector + 100ms BERT)
- **Accuracy:** +20% relevance improvement over Phase 3
- **Throughput:** Maintain 5+ searches/second

**Week 1 Deliverables:**
- [ ] ONNX BERT model integration
- [ ] Cross-encoder inference pipeline
- [ ] Re-ranking service implementation
- [ ] Performance benchmarking
- [ ] A/B testing framework

## ⚡ **Week 2: Query Intent Classification**

### **Goal:** Understand query intent for intelligent routing

**Intent Categories:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum QueryIntent {
    Technical,     // "API authentication", "error handling"
    Conceptual,    // "what is", "how does", "explain"
    Code,          // "example", "implementation", "snippet"
    Navigation,    // "getting started", "installation"
    Troubleshooting, // "fix", "error", "problem", "issue"
}
```

**Implementation Strategy:**
```rust
// services/doc-indexer/src/intent_classifier.rs
pub struct IntentClassifier {
    model: Arc<OrtSession>,
    tokenizer: Arc<Tokenizer>,
    label_mapping: HashMap<usize, QueryIntent>,
}

impl IntentClassifier {
    pub async fn classify_query(&self, query: &str) -> Result<IntentPrediction> {
        // 1. Tokenize query
        // 2. Run classification model
        // 3. Return intent + confidence
    }
}

pub struct IntentPrediction {
    pub intent: QueryIntent,
    pub confidence: f32,
    pub alternatives: Vec<(QueryIntent, f32)>,
}
```

**Model Training Approach:**
1. **Data Collection:** Analyze existing search logs for patterns
2. **Labeling:** Manual classification of 1000+ queries
3. **Model Selection:** Fine-tuned DistilBERT or lightweight transformer
4. **Validation:** Cross-validation with held-out test set

**Intent-Aware Search Routing:**
```rust
// Different search strategies per intent
match intent.intent {
    QueryIntent::Technical => {
        // Emphasize technical docs, API references
        ranking_weights.technical_boost = 1.5;
    },
    QueryIntent::Code => {
        // Prioritize code examples, tutorials
        ranking_weights.code_snippet_boost = 2.0;
    },
    QueryIntent::Conceptual => {
        // Focus on explanations, overviews
        ranking_weights.conceptual_boost = 1.3;
    },
    // ... other intents
}
```

**Week 2 Deliverables:**
- [ ] Intent classification model training
- [ ] Query intent detection service
- [ ] Intent-aware search routing
- [ ] Classification accuracy metrics
- [ ] Intent distribution analytics

## 📊 **Week 3: User Behavior Analytics Foundation**

### **Goal:** Build analytics infrastructure for learning from user interactions

**Event Tracking System:**
```rust
// services/doc-indexer/src/analytics.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchEvent {
    pub session_id: String,
    pub query: String,
    pub intent: Option<QueryIntent>,
    pub results_shown: Vec<SearchResultAnalytics>,
    pub timestamp: DateTime<Utc>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClickEvent {
    pub session_id: String,
    pub query: String,
    pub clicked_position: usize,
    pub result_id: String,
    pub dwell_time: Option<Duration>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultAnalytics {
    pub result_id: String,
    pub position: usize,
    pub score: f32,
    pub ranking_signals: RankingSignals,
}
```

**Analytics Storage:**
```rust
// Time-series database for analytics
pub trait AnalyticsStore {
    async fn record_search_event(&self, event: SearchEvent) -> Result<()>;
    async fn record_click_event(&self, event: ClickEvent) -> Result<()>;
    async fn get_query_analytics(&self, query: &str) -> Result<QueryAnalytics>;
    async fn get_user_patterns(&self, session_id: &str) -> Result<UserPatterns>;
}

// Implementation options:
// 1. SQLite for development
// 2. PostgreSQL for production
// 3. ClickHouse for high-volume analytics
```

**Learning Metrics:**
```rust
#[derive(Debug)]
pub struct QueryAnalytics {
    pub query: String,
    pub total_searches: u64,
    pub avg_click_position: f32,
    pub click_through_rate: f32,
    pub no_click_rate: f32,
    pub reformulation_rate: f32,
    pub top_clicked_results: Vec<String>,
}
```

**Week 3 Deliverables:**
- [ ] Event tracking infrastructure
- [ ] Analytics data models
- [ ] Storage implementation (SQLite → PostgreSQL)
- [ ] Privacy-compliant data collection
- [ ] Basic analytics dashboard

## 🧠 **Week 4: ML Pipeline Optimization & Learning**

### **Goal:** Implement feedback loops and ranking optimization

**Click-Through Rate (CTR) Learning:**
```rust
// services/doc-indexer/src/learning.rs
pub struct CTRLearner {
    analytics_store: Arc<dyn AnalyticsStore>,
    model_cache: LruCache<String, CTRModel>,
}

impl CTRLearner {
    pub async fn learn_query_preferences(&self, query: &str) -> Result<QueryPreferences> {
        let analytics = self.analytics_store.get_query_analytics(query).await?;
        
        QueryPreferences {
            preferred_doc_types: self.analyze_click_patterns(&analytics),
            optimal_result_count: self.optimize_result_count(&analytics),
            ranking_adjustments: self.compute_ranking_adjustments(&analytics),
        }
    }
}
```

**Ranking Model Updates:**
```rust
// Feedback-driven ranking improvements
pub struct AdaptiveRanker {
    base_ranker: ResultRanker,
    ctr_learner: CTRLearner,
    preferences_cache: Arc<RwLock<HashMap<String, QueryPreferences>>>,
}

impl AdaptiveRanker {
    pub async fn rank_with_learning(
        &self,
        results: Vec<SearchResult>,
        query: &str,
    ) -> Result<Vec<RankedResult>> {
        // 1. Get base ranking (Phase 3)
        let base_ranked = self.base_ranker.rank_results(results, query)?;
        
        // 2. Apply learned preferences
        if let Some(preferences) = self.get_query_preferences(query).await? {
            return self.apply_learned_ranking(base_ranked, preferences);
        }
        
        Ok(base_ranked)
    }
}
```

**A/B Testing Framework:**
```rust
// Controlled experimentation
pub struct ExperimentManager {
    experiments: HashMap<String, Experiment>,
    user_assignments: Arc<RwLock<HashMap<String, String>>>,
}

#[derive(Debug)]
pub struct Experiment {
    pub name: String,
    pub traffic_split: f32,  // 0.1 = 10% of traffic
    pub ranking_variant: RankingVariant,
    pub metrics: ExperimentMetrics,
}
```

**Week 4 Deliverables:**
- [ ] CTR learning algorithms
- [ ] Adaptive ranking implementation
- [ ] A/B testing framework
- [ ] Performance monitoring
- [ ] ML pipeline documentation

## 📊 **Success Metrics for Phase 4B**

### **Relevance Metrics:**
- **NDCG@10:** Normalized Discounted Cumulative Gain improvement
- **MRR:** Mean Reciprocal Rank enhancement
- **Click-through Rate:** User engagement improvement
- **Precision@K:** Accuracy of top results

### **Performance Metrics:**
- **Latency:** <200ms end-to-end response time
- **Throughput:** 5+ searches/second sustained
- **Model Size:** <200MB total model footprint
- **Memory Usage:** <2GB RAM for ML components

### **Learning Metrics:**
- **Intent Accuracy:** 95%+ query classification accuracy
- **Adaptation Speed:** Ranking improvements within 100 searches
- **Coverage:** Analytics for 100% of search sessions

## 🛠 **Technical Dependencies**

### **New Crate Dependencies:**
```toml
# Cargo.toml additions
[dependencies]
# ONNX Runtime for BERT
ort = { version = "1.16", features = ["copy-dylibs"] }

# Analytics and time-series data
sqlx = { version = "0.7", features = ["postgres", "chrono", "uuid"] }
clickhouse = "0.11"  # Optional for high-volume analytics

# ML utilities
ndarray = "0.15"
candle-core = "0.3"  # Alternative to ONNX for some models

# Caching and performance
lru = "0.12"
dashmap = "5.5"  # Concurrent HashMap
```

### **Model Downloads:**
```bash
# Model preparation script
mkdir -p models/reranking models/classification

# BERT re-ranking model
wget https://huggingface.co/sentence-transformers/ms-marco-MiniLM-L-6-v2/resolve/main/model.onnx \
  -O models/reranking/bert-reranker.onnx

# Intent classification model (to be trained)
# Custom model training pipeline needed
```

## 🎯 **Integration with Phase 3**

Phase 4B enhances existing Phase 3 components:

- **Query Enhancement:** Add intent-aware synonym expansion
- **Result Ranking:** Integrate BERT scores into multi-factor ranking
- **Observability:** Extend metrics with ML performance tracking
- **Search Service:** Orchestrate ML pipeline with existing services

This creates a powerful, learning search system that continuously improves based on user interactions while maintaining the performance and quality gains from Phase 3.

---

**Next:** Phase 4C architectural refactoring to support these ML features in a scalable, maintainable way.
