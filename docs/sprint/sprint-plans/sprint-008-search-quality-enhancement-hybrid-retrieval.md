# **Sprint ID:** ZL-008  
**Sprint Name:** Search Quality Enhancement & Hybrid Retrieval Implementation  
**Start Date:** September 2, 2025  
**End Date:** October 14, 2025  
**Duration:** 30 working days (6 weeks)  
**Sprint Goal:** Implement high-ROI search quality improvements based on consultant review evaluation, focusing on hybrid retrieval, multi-query expansion, and comprehensive evaluation framework  
**Current Status:** PLANNED 📋 - Ready for Implementation  
**Related:** [ADR-044](../../adr/044_consultant_review_evaluation_and_recommendations.md), [ADR-043](../../adr/043_docsearch_enhancements.md), [Phase 4B ML/AI Implementation](../misc/artefacts/033_phase-4b-ml-ai-implementation-plan.md)  

---

## 🎯 Sprint Objective

Implement selective technical improvements from consultant review evaluation (ADR-044), focusing on high-ROI search quality enhancements while maintaining architectural integrity. Deliver hybrid retrieval, multi-query expansion, and comprehensive evaluation capabilities without operational complexity.

**STRATEGIC CONTEXT**: External consultant provided comprehensive enhancement strategy but underestimated current implementation maturity. This sprint extracts validated technical improvements while deferring agent orchestration complexity until product-market fit is clearer.

**Success Criteria:**
- [ ] Hybrid BM25 + vector retrieval with score transparency implemented
- [ ] Multi-query expansion pipeline operational for improved recall
- [ ] Comprehensive evaluation framework with NDCG@10 measurement
- [ ] P95 latency ≤350ms for hybrid search maintained
- [ ] ≥15% NDCG@10 improvement vs vector-only baseline achieved
- [ ] Zero performance regression on existing workloads validated
- [ ] A/B testing framework operational for future improvements

---

## 📋 Task Breakdown

### **Epic 1: Technical Foundation & Assessment (Week 1-2)**
**Priority:** High | **Effort:** 10 days | **Assignee:** Development Team

#### **ZL-008-001: BM25/Tantivy Integration Feasibility Assessment**
**Status:** PLANNED 📋  
**Priority:** High  
**Effort:** 3 days  
**Dependencies:** None  

**Description:**
Comprehensive technical assessment of BM25 (tantivy) integration with existing vector search architecture. Evaluate implementation complexity, performance implications, and architectural alignment.

**Acceptance Criteria:**
- [ ] Tantivy library integration complexity analysis completed
- [ ] Architecture compatibility assessment documented
- [ ] Performance impact estimation with benchmarks
- [ ] Integration approach design with existing search pipeline
- [ ] Resource requirements and timeline estimation
- [ ] Risk assessment and mitigation strategies identified
- [ ] Technical specification for hybrid implementation approved

**Files to Investigate:**
- `crates/zero-latency-search/src/vector_search.rs`
- `crates/zero-latency-search/src/traits.rs`
- `services/doc-indexer/src/infrastructure/search_enhancement.rs`
- `crates/zero-latency-vector/src/models.rs`

**Deliverables:**
- Technical feasibility report
- Integration architecture design
- Performance benchmark baseline
- Implementation timeline estimate

---

#### **ZL-008-002: Evaluation Dataset Creation & Metrics Framework**
**Status:** PLANNED 📋  
**Priority:** High  
**Effort:** 4 days  
**Dependencies:** None  

**Description:**
Create comprehensive evaluation dataset and metrics framework for search quality assessment. Establish baseline measurements and automated evaluation capabilities.

**Acceptance Criteria:**
- [ ] **CI gating step** with "fail if NDCG@10 drops >3%" configurable threshold
- [ ] **Small golden snapshot** to catch regressions in CI pipeline
- [ ] **Dataset with graded relevance** (0/1/2) for 50-100 labeled queries
- [ ] **Cost & latency panels** including tokens and rerank time tracking
- [ ] NDCG@10 calculation implementation completed
- [ ] Hit@K and precision@K metrics implemented
- [ ] Baseline performance measurements for current vector-only search
- [ ] Automated evaluation pipeline with CI integration
- [ ] Search quality metrics dashboard framework
- [ ] A/B testing infrastructure foundation established

**Files to Create/Modify:**
- `crates/zero-latency-search/src/evaluation/`
- `crates/zero-latency-search/src/evaluation/metrics.rs`
- `crates/zero-latency-search/src/evaluation/dataset.rs`
- `test/evaluation/labeled_dataset.json`
- `test/evaluation/evaluation_pipeline.rs`

**Deliverables:**
- Labeled evaluation dataset
- Metrics calculation library
- Baseline performance report
- Automated evaluation framework

---

#### **ZL-008-003: Hybrid Scoring Fusion Algorithm Design**
**Status:** PLANNED 📋  
**Priority:** High  
**Effort:** 3 days  
**Dependencies:** ZL-008-001  

**Description:**
Design and prototype score fusion algorithm for combining BM25 and vector similarity scores with transparency and configurability.

**Acceptance Criteria:**
- [ ] **Score normalization method specified**: min-max per list OR z-score + clamp to [0,1]
- [ ] **Fusion formula explicit**: `fused = w_bm25 * bm25_norm + w_vec * vec_norm` with configurable weights
- [ ] Score breakdown structure includes **raw and normalized scores** for transparency
- [ ] **Unit tests with known inputs/outputs** for normalization and fusion logic
- [ ] Performance impact assessment completed
- [ ] Configuration interface for per-collection weight tuning
- [ ] Documentation for score interpretation and tuning

**Files to Create:**
- `crates/zero-latency-search/src/fusion/`
- `crates/zero-latency-search/src/fusion/score_fusion.rs`
- `crates/zero-latency-search/src/fusion/normalization.rs`
- `crates/zero-latency-search/src/models.rs` (extend SearchResult with enhanced structure)
- `crates/zero-latency-core/src/doc_id.rs` (shared DocId definition)

**Enhanced SearchResult Model:**
```rust
pub struct SearchResult {
    pub doc_id: DocId,
    pub uri: String,
    pub title: String,
    pub snippet: String,
    pub section_path: Vec<String>,
    pub scores: Scores,        // raw + normalized + fused
    pub from_signals: FromSignals,  // {bm25: bool, vector: bool, variants: Vec<usize>}
    // ... existing fields
}
```

**Deliverables:**
- Score fusion algorithm implementation
- Configuration framework
- Performance benchmarks
- Technical documentation

---

### **Epic 2: Hybrid Retrieval Implementation (Week 3-4)**
**Priority:** High | **Effort:** 10 days | **Assignee:** Development Team

#### **ZL-008-004: Tantivy BM25 Search Engine Integration**
**Status:** PLANNED 📋  
**Priority:** High  
**Effort:** 5 days  
**Dependencies:** ZL-008-001, ZL-008-003  

**Description:**
Integrate tantivy BM25 search engine with existing search architecture, maintaining clean separation of concerns and performance characteristics.

**Acceptance Criteria:**
- [ ] **Stable DocId definition**: `{collection, external_id, version}` shared across BM25 and vector stores
- [ ] **Index Sync contract**: dual-write upsert/delete/batch operations with version/seqno logging
- [ ] **Dual-index reconciliation**: background task comparing counts & sample IDs with drift alerts
- [ ] **Index sync verified via reconciliation report** showing zero drift over 1k-doc sample
- [ ] Tantivy dependency added with appropriate feature flags
- [ ] BM25 index creation pipeline implemented
- [ ] Document indexing workflow updated for dual indexing
- [ ] BM25 search interface implementation completed
- [ ] Performance optimization for index operations
- [ ] Error handling and recovery mechanisms implemented

**Files to Create/Modify:**
- `services/doc-indexer/Cargo.toml` (add tantivy dependency)
- `crates/zero-latency-search/src/bm25/`
- `crates/zero-latency-search/src/bm25/tantivy_adapter.rs`
- `crates/zero-latency-search/src/bm25/index_manager.rs`
- `services/doc-indexer/src/infrastructure/persistence/bm25/`
- `crates/zero-latency-search/src/traits.rs` (extend SearchStep)

**Deliverables:**
- Tantivy BM25 search implementation
- Dual indexing pipeline
- Performance benchmarks
- Integration tests

---

#### **ZL-008-005: Vector + BM25 Hybrid Search Pipeline**
**Status:** PLANNED 📋  
**Priority:** High  
**Effort:** 5 days  
**Dependencies:** ZL-008-004, ZL-008-003  

**Description:**
Implement unified hybrid search pipeline that combines vector similarity and BM25 results with score fusion and transparent breakdown.

**Acceptance Criteria:**
- [ ] **Parallel engine execution** with `tokio::try_join!` and per-engine timeouts
- [ ] **Partial failure handling**: use available engine if one times out, mark `partial=true`
- [ ] **Enhanced SearchResult model** with score breakdown and provenance tracking
- [ ] **Score breakdown present in API & docs** with raw/normalized/fused scores
- [ ] **Per-collection feature flagging & rollout** documentation completed
- [ ] Hybrid search orchestration pipeline implemented
- [ ] Result fusion with score breakdown completed
- [ ] Search response format updated with score transparency
- [ ] Performance optimization to meet ≤350ms P95 target
- [ ] Configuration interface for hybrid search parameters

**Files to Create/Modify:**
- `crates/zero-latency-search/src/hybrid_search.rs`
- `crates/zero-latency-search/src/pipeline/hybrid_step.rs`
- `crates/zero-latency-search/src/models.rs` (extend SearchResult)
- `services/doc-indexer/src/infrastructure/search_enhancement.rs`
- `crates/zero-latency-api/src/dto/search.rs`

**Deliverables:**
- Hybrid search pipeline
- Performance-optimized implementation
- Enhanced search response format
- Configuration framework

---

### **Epic 3: Multi-Query Expansion Implementation (Week 5)**
**Priority:** Medium | **Effort:** 5 days | **Assignee:** Development Team

#### **ZL-008-006: Query Paraphrase Generation Pipeline**
**Status:** PLANNED 📋  
**Priority:** Medium  
**Effort:** 3 days  
**Dependencies:** ZL-008-005  

**Description:**
Extend existing query enhancement pipeline to generate 2-3 query paraphrases for improved recall and semantic coverage.

**Acceptance Criteria:**
- [ ] **Global MQE latency budget** ≤150ms total with concurrent paraphrase generation
- [ ] **Paraphrase caching** for 5-15 minutes keyed by `(norm_query, filters)`
- [ ] **Fallback mechanism** if paraphrase generator errors or times out
- [ ] Query paraphrase generation algorithm implemented
- [ ] Integration with existing query enhancement pipeline
- [ ] Configurable number of paraphrases (2-3 default)
- [ ] Quality filtering for generated paraphrases
- [ ] A/B testing support for paraphrase effectiveness

**Files to Create/Modify:**
- `services/doc-indexer/src/infrastructure/search_enhancement.rs`
- `crates/zero-latency-search/src/enhancement/`
- `crates/zero-latency-search/src/enhancement/query_expansion.rs`
- `crates/zero-latency-search/src/enhancement/paraphrase_generator.rs`

**Deliverables:**
- Multi-query expansion implementation
- Performance benchmarks
- Quality assessment framework
- Configuration interface

---

#### **ZL-008-007: Result Deduplication and Merging Logic**
**Status:** PLANNED 📋  
**Priority:** Medium  
**Effort:** 2 days  
**Dependencies:** ZL-008-006  

**Description:**
Implement result deduplication and intelligent merging for multiple query variations to prevent duplicate results and maintain ranking quality.

**Acceptance Criteria:**
- [ ] **Deduplication stable across doc reorderings** with consistent tie-breaking
- [ ] **Ties broken by fused score then recency** for deterministic ranking
- [ ] **Union + dedupe with per-variant provenance** for debugging support
- [ ] Document-level deduplication algorithm implemented
- [ ] Score aggregation strategy for duplicate results
- [ ] Ranking preservation across merged results
- [ ] Performance optimization for large result sets
- [ ] Configurable deduplication sensitivity
- [ ] Metrics tracking for deduplication effectiveness

**Files to Create/Modify:**
- `crates/zero-latency-search/src/fusion/result_merger.rs`
- `crates/zero-latency-search/src/fusion/deduplication.rs`
- `crates/zero-latency-search/src/pipeline/merge_step.rs`

**Deliverables:**
- Deduplication algorithm
- Result merging logic
- Performance benchmarks
- Quality metrics

---

### **Epic 4: Validation & Optimization (Week 6)**
**Priority:** High | **Effort:** 5 days | **Assignee:** Development Team

#### **ZL-008-008: Comprehensive Search Quality Evaluation**
**Status:** PLANNED 📋  
**Priority:** High  
**Effort:** 3 days  
**Dependencies:** ZL-008-007, ZL-008-002  

**Description:**
Conduct comprehensive evaluation of hybrid search implementation against baseline and quality targets using established metrics framework.

**Acceptance Criteria:**
- [ ] **A/B testing with statistical significance** using randomization test or similar method
- [ ] **Report includes per-query diffs and aggregated deltas** with confidence intervals
- [ ] NDCG@10 improvement ≥15% vs vector-only baseline validated
- [ ] Hit@K and precision@K improvements measured and documented
- [ ] Query recall improvement ≥20% with multi-query expansion validated
- [ ] Search relevance quality assessment completed
- [ ] Performance regression testing completed
- [ ] User experience impact assessment documented

**Files to Create:**
- `test/evaluation/hybrid_search_evaluation.rs`
- `docs/evaluation/search_quality_report.md`
- `test/evaluation/performance_regression_tests.rs`

**Deliverables:**
- Comprehensive evaluation report
- Performance benchmark results
- Quality improvement documentation
- Regression test suite

---

#### **ZL-008-009: Performance Optimization & Caching**
**Status:** PLANNED 📋  
**Priority:** High  
**Effort:** 2 days  
**Dependencies:** ZL-008-008  

**Description:**
Implement performance optimizations and result caching to ensure latency targets are maintained with enhanced search capabilities.

**Acceptance Criteria:**
- [ ] **Cache key includes flags + collection version** for proper invalidation
- [ ] **≥40% cache hit-rate on mixed workload** benchmark achieved
- [ ] **Collection version-based invalidation** prevents phantom cache wins
- [ ] **Small rerank cache** for `(doc_id, query_hash) -> rerank_score` tail latency reduction
- [ ] P95 latency ≤350ms for hybrid search achieved
- [ ] Query result caching with TTL implemented
- [ ] Vector store connection pooling optimized
- [ ] BM25 index warming strategies implemented
- [ ] Memory usage optimization for hybrid operations
- [ ] Performance monitoring and alerting setup

**Files to Create/Modify:**
- `crates/zero-latency-search/src/caching/`
- `crates/zero-latency-search/src/caching/result_cache.rs`
- `services/doc-indexer/src/infrastructure/caching/`
- `services/doc-indexer/src/infrastructure/performance/`

**Deliverables:**
- Performance optimization implementation
- Caching framework
- Monitoring setup
- Performance documentation

---

## 🎯 Success Metrics

### **Technical Metrics**
- [ ] **NDCG@10 Improvement**: ≥15% vs vector-only baseline
- [ ] **Search Latency**: P95 ≤350ms for hybrid search
- [ ] **Recall Improvement**: ≥20% with multi-query expansion  
- [ ] **Performance Regression**: Zero degradation on existing workloads
- [ ] **Test Coverage**: ≥90% for all new search components
- [ ] **Documentation Coverage**: 100% for public APIs and configuration

### **Operational Metrics**
- [ ] **Implementation Timeline**: Delivery within 6-week sprint duration
- [ ] **Production Incidents**: Zero incidents during deployment
- [ ] **Integration Success**: All existing integrations remain functional
- [ ] **Resource Utilization**: Memory usage increase ≤20% vs baseline
- [ ] **Index Size**: BM25 index overhead ≤30% of vector index size

### **Product Metrics**
- [ ] **Search Relevance**: Qualitative improvement in result quality
- [ ] **User Satisfaction**: Improved search experience feedback
- [ ] **API Compatibility**: 100% backward compatibility maintained
- [ ] **Feature Adoption**: Hybrid search enabled by default for new collections

---

## 📊 Sprint Planning

### **Week 1-2: Foundation (Days 1-10)**
**Focus**: Technical assessment and framework establishment
- Days 1-3: BM25/Tantivy feasibility assessment
- Days 4-7: Evaluation dataset and metrics framework  
- Days 8-10: Hybrid scoring fusion algorithm design

### **Week 3-4: Implementation (Days 11-20)**
**Focus**: Core hybrid retrieval implementation
- Days 11-15: Tantivy BM25 integration
- Days 16-20: Hybrid search pipeline development

### **Week 5: Enhancement (Days 21-25)**
**Focus**: Multi-query expansion and result optimization
- Days 21-23: Query paraphrase generation
- Days 24-25: Result deduplication and merging

### **Week 6: Validation (Days 26-30)**
**Focus**: Quality evaluation and performance optimization
- Days 26-28: Comprehensive search quality evaluation
- Days 29-30: Performance optimization and caching

---

## 🔧 Technical Configuration

### **Dependencies**
- **Tantivy**: BM25 full-text search engine
- **Existing Infrastructure**: Vector search, query enhancement, search pipeline
- **Evaluation Framework**: NDCG calculation, A/B testing infrastructure
- **Performance Monitoring**: Latency tracking, resource utilization

### **Feature Flags**
- `hybrid_search`: Enable hybrid BM25 + vector search
- `multi_query_expansion`: Enable query paraphrase generation  
- `result_caching`: Enable query result caching
- `evaluation_metrics`: Enable comprehensive search quality metrics

### **Configuration Parameters**
```yaml
search:
  hybrid:
    enabled: true
    bm25_weight: 0.3
    vector_weight: 0.7
    fusion_method: "normalized_sum"
    normalization: "min_max"  # or "z_score"
    engine_timeout_ms: 200
  
  multi_query:
    enabled: true
    max_paraphrases: 3
    quality_threshold: 0.7
    global_timeout_ms: 150
    cache_ttl_minutes: 10
  
  caching:
    enabled: true
    ttl_seconds: 300
    max_entries: 10000
    include_collection_version: true

  evaluation:
    enabled: true
    dataset_path: "test/evaluation/labeled_dataset.json"
    metrics: ["ndcg@10", "hit@5", "precision@10"]
    ci_regression_threshold: 0.03
```

---

## 🎯 Quick Wins (This Week)

Based on consultant feedback, implement these foundational changes immediately:

### **Week 0: Immediate Implementation (3 days)**
1. **Define DocId + extend SearchResult** with score breakdown & provenance
2. **Implement parallel vector+BM25 execution** with timeout guards (stub BM25 initially)  
3. **Ship evaluation harness + CI gate** (NDCG@10 baseline, hybrid stubbed)

These changes keep the sprint on-track while making hybrid rollout safer, explainable, and easier to iterate.

---

## 📋 Definition of Ready

### **Technical Prerequisites**
- [ ] Current search pipeline architecture documented
- [ ] Performance baseline measurements established
- [ ] Evaluation dataset requirements defined
- [ ] Development environment configured with tantivy dependencies

### **Organizational Prerequisites**
- [ ] Sprint goals aligned with product strategy
- [ ] Resource allocation confirmed for 6-week sprint
- [ ] Stakeholder approval for selective implementation approach
- [ ] Testing and deployment strategy approved

---

## ✅ Definition of Done

### **Implementation Requirements**
- [ ] All acceptance criteria met for each task
- [ ] Unit tests implemented with ≥90% coverage
- [ ] Integration tests validate end-to-end functionality
- [ ] Performance benchmarks meet latency targets
- [ ] Documentation updated for all new features

### **Quality Assurance**
- [ ] Code review completed by senior developers
- [ ] Security review for new components completed
- [ ] Performance regression testing passed
- [ ] Backward compatibility validation completed
- [ ] Production deployment checklist verified

### **Documentation Requirements**
- [ ] Technical documentation updated
- [ ] API documentation reflects new capabilities
- [ ] Configuration guide updated
- [ ] Troubleshooting documentation created
- [ ] Performance tuning guide documented

---

## 🚨 Risks & Mitigation Strategies

### **Technical Risks**

#### **Risk 1: BM25 Integration Complexity**
**Probability**: Medium | **Impact**: High
**Mitigation**: 
- Comprehensive feasibility assessment in Week 1
- Prototype implementation before full integration
- Fallback to vector-only search if integration fails

#### **Risk 2: Performance Degradation**
**Probability**: Medium | **Impact**: High  
**Mitigation**:
- Continuous performance monitoring during development
- Aggressive caching and optimization strategies
- A/B testing to validate performance impact

#### **Risk 3: Search Quality Regression**
**Probability**: Low | **Impact**: High
**Mitigation**:
- Comprehensive evaluation framework with baseline comparison
- Gradual rollout with feature flags
- Rollback capability if quality decreases

### **Organizational Risks**

#### **Risk 4: Scope Creep**
**Probability**: Medium | **Impact**: Medium
**Mitigation**:
- Strict adherence to ADR-044 selective implementation strategy
- Regular sprint review meetings with scope validation
- Deferred backlog for agent orchestration features

#### **Risk 5: Timeline Overrun**
**Probability**: Medium | **Impact**: Medium
**Mitigation**:
- Conservative time estimates with buffer
- Daily progress tracking and early issue identification
- Scope reduction options identified for each epic

---

## 📈 Success Tracking

### **Weekly Checkpoints**
- **Week 1**: Feasibility assessment complete, evaluation framework established
- **Week 2**: Technical foundation solid, hybrid scoring design approved
- **Week 3**: BM25 integration operational, basic hybrid search working
- **Week 4**: Full hybrid pipeline complete, performance targets met
- **Week 5**: Multi-query expansion operational, quality improvements validated
- **Week 6**: Comprehensive evaluation complete, production-ready implementation

### **Key Performance Indicators**
- Technical milestone completion rate
- Search quality improvement metrics
- Performance benchmark adherence
- Test coverage and documentation completion
- Stakeholder satisfaction with sprint progress

---

## 🎯 Post-Sprint Planning

### **Immediate Next Steps** (Sprint ZL-009)
- Cross-encoder reranking implementation (Phase 2 of ADR-044)
- Advanced caching and performance optimization
- Enhanced observability and monitoring

### **Future Considerations**
- Agent orchestration capabilities evaluation
- Session memory and personalization assessment
- Enterprise security features if scaling requirements emerge

### **Continuous Improvement**
- Search quality metrics monitoring
- User feedback collection and analysis
- Performance optimization based on production usage patterns

---

**Sprint Lead**: Development Team  
**Technical Advisor**: Senior Architect  
**Stakeholders**: Product Owner, Engineering Manager  
**Review Schedule**: Weekly sprint reviews, daily standups  

---

*This sprint plan implements the selective technical improvements identified in ADR-044, focusing on high-ROI search quality enhancements while maintaining architectural integrity and deferring complex agent orchestration features until product-market fit is clearer.*
