use async_trait::async_trait;
/// Enhanced search pipeline components for advanced search features
///
/// This module provides concrete implementations of QueryEnhancer and ResultRanker
/// to enable sophisticated search capabilities including query expansion,
/// term enhancement, and multi-factor result ranking.
use std::collections::HashMap;
use zero_latency_core::Result;
use zero_latency_search::{
    EnhancedQuery, Entity, EntityType, QueryAnalysis, QueryComplexity, QueryEnhancer, QueryIntent,
    RankingSignals, ResultRanker, SearchResult,
};

/// Context information for query analysis
#[derive(Debug, Clone)]
pub struct QueryContext {
    pub is_question: bool,
    pub is_troubleshooting: bool,
    pub is_tutorial: bool,
    pub is_setup: bool,
    pub complexity_score: f32,
}

/// Simple query enhancer that expands queries with technical terms and synonyms
pub struct SimpleQueryEnhancer {
    // Domain-specific term mappings for documentation search
    technical_terms: HashMap<String, Vec<String>>,
    synonyms: HashMap<String, Vec<String>>,
    // Context-aware expansion rules
    context_rules: HashMap<String, Vec<String>>,
    // Common technical patterns
    pattern_expansions: HashMap<String, Vec<String>>,
}

impl SimpleQueryEnhancer {
    pub fn new() -> Self {
        let mut technical_terms = HashMap::new();
        let mut synonyms = HashMap::new();
        let mut context_rules = HashMap::new();
        let mut pattern_expansions = HashMap::new();

        // Add comprehensive technical term expansions
        technical_terms.insert(
            "api".to_string(),
            vec![
                "application programming interface".to_string(),
                "endpoint".to_string(),
                "rest".to_string(),
                "restful".to_string(),
                "graphql".to_string(),
                "web service".to_string(),
                "http".to_string(),
            ],
        );

        technical_terms.insert(
            "config".to_string(),
            vec![
                "configuration".to_string(),
                "settings".to_string(),
                "parameters".to_string(),
                "options".to_string(),
                "environment".to_string(),
                "env".to_string(),
            ],
        );

        technical_terms.insert(
            "auth".to_string(),
            vec![
                "authentication".to_string(),
                "authorization".to_string(),
                "security".to_string(),
                "login".to_string(),
                "token".to_string(),
                "jwt".to_string(),
                "oauth".to_string(),
            ],
        );

        technical_terms.insert(
            "db".to_string(),
            vec![
                "database".to_string(),
                "storage".to_string(),
                "persistence".to_string(),
                "sql".to_string(),
                "nosql".to_string(),
            ],
        );

        technical_terms.insert(
            "test".to_string(),
            vec![
                "testing".to_string(),
                "unit test".to_string(),
                "integration test".to_string(),
                "benchmark".to_string(),
                "validation".to_string(),
            ],
        );

        // Document and content-related terms
        technical_terms.insert(
            "markdown".to_string(),
            vec![
                "md".to_string(),
                "markup".to_string(),
                "documentation".to_string(),
                "format".to_string(),
                "text formatting".to_string(),
                "content".to_string(),
            ],
        );

        technical_terms.insert(
            "mdx".to_string(),
            vec![
                "markdown".to_string(),
                "jsx".to_string(),
                "react markdown".to_string(),
                "component".to_string(),
                "interactive documentation".to_string(),
            ],
        );

        technical_terms.insert(
            "html".to_string(),
            vec![
                "hypertext markup language".to_string(),
                "web".to_string(),
                "markup".to_string(),
                "tag".to_string(),
                "element".to_string(),
                "dom".to_string(),
            ],
        );

        technical_terms.insert(
            "content".to_string(),
            vec![
                "document".to_string(),
                "text".to_string(),
                "information".to_string(),
                "data".to_string(),
                "material".to_string(),
            ],
        );

        technical_terms.insert(
            "processing".to_string(),
            vec![
                "parsing".to_string(),
                "transformation".to_string(),
                "conversion".to_string(),
                "handling".to_string(),
                "indexing".to_string(),
            ],
        );

        // Add enhanced synonyms
        synonyms.insert(
            "search".to_string(),
            vec![
                "find".to_string(),
                "query".to_string(),
                "lookup".to_string(),
                "retrieve".to_string(),
                "discover".to_string(),
            ],
        );

        synonyms.insert(
            "error".to_string(),
            vec![
                "issue".to_string(),
                "problem".to_string(),
                "bug".to_string(),
                "failure".to_string(),
                "exception".to_string(),
                "fault".to_string(),
            ],
        );

        synonyms.insert(
            "build".to_string(),
            vec![
                "compile".to_string(),
                "construct".to_string(),
                "create".to_string(),
                "generate".to_string(),
            ],
        );

        synonyms.insert(
            "run".to_string(),
            vec![
                "execute".to_string(),
                "start".to_string(),
                "launch".to_string(),
                "invoke".to_string(),
            ],
        );

        // Context-aware rules for better expansion
        context_rules.insert(
            "how to".to_string(),
            vec![
                "tutorial".to_string(),
                "guide".to_string(),
                "example".to_string(),
                "documentation".to_string(),
                "instructions".to_string(),
            ],
        );

        context_rules.insert(
            "setup".to_string(),
            vec![
                "installation".to_string(),
                "configure".to_string(),
                "initialize".to_string(),
                "prepare".to_string(),
            ],
        );

        // Pattern-based expansions for common phrases
        pattern_expansions.insert(
            "get started".to_string(),
            vec![
                "introduction".to_string(),
                "quick start".to_string(),
                "getting started".to_string(),
                "first steps".to_string(),
                "setup guide".to_string(),
            ],
        );

        pattern_expansions.insert(
            "best practices".to_string(),
            vec![
                "recommendations".to_string(),
                "guidelines".to_string(),
                "standards".to_string(),
                "conventions".to_string(),
            ],
        );

        Self {
            technical_terms,
            synonyms,
            context_rules,
            pattern_expansions,
        }
    }
}

#[async_trait]
impl QueryEnhancer for SimpleQueryEnhancer {
    async fn enhance(&self, query: &str) -> Result<EnhancedQuery> {
        let original_query = query.to_string();

        tracing::info!(
            "[AdvancedSearch] QueryEnhancementStep active: enhancing query '{}'.",
            query
        );

        // Step 1: Analyze query context and intent
        let analysis = self.analyze_query_context(query);

        // Step 2: Apply context-aware expansion
        let context_terms = self.expand_with_context(query, &analysis);

        // Step 3: Apply technical term expansion
        let technical_expansion = self.expand_query_terms(query);

        // Step 4: Apply pattern-based expansion
        let pattern_expansion = self.expand_with_patterns(query);

        // Step 5: Combine and deduplicate expansions
        let mut all_expansions = Vec::new();
        all_expansions.extend(context_terms);
        all_expansions.extend(technical_expansion);
        all_expansions.extend(pattern_expansion);

        // Remove duplicates and limit expansion size
        all_expansions.sort();
        all_expansions.dedup();

        // Limit to most relevant terms (max 8 additional terms)
        if all_expansions.len() > 8 {
            all_expansions.truncate(8);
        }

        println!(
            "🔍 QueryEnhancementStep: Expanding query '{}' with {} terms: {:?}",
            query,
            all_expansions.len(),
            all_expansions
        );

        let enhanced_query = if !all_expansions.is_empty() {
            format!("{} {}", query, all_expansions.join(" "))
        } else {
            query.to_string()
        };

        println!(
            "✨ QueryEnhancementStep: Enhanced query: '{}'",
            enhanced_query
        );

        // Track enhancement metrics
        tracing::info!(
            "[AdvancedSearch] Query enhancement metrics - Original length: {}, Enhanced length: {}, Terms added: {}",
            original_query.len(),
            enhanced_query.len(),
            all_expansions.len()
        );

        let expansion_count = all_expansions.len();
        let synonyms_added = self.extract_synonyms_from_expansion(&all_expansions);

        Ok(EnhancedQuery {
            original: original_query,
            enhanced: enhanced_query,
            synonyms_added,
            technical_terms: all_expansions,
            expansion_strategy: format!("context_aware_multi_factor({})", expansion_count),
        })
    }

    async fn analyze(&self, query: &str) -> Result<QueryAnalysis> {
        let words: Vec<&str> = query.split_whitespace().collect();
        let word_count = words.len();

        // Determine query intent based on keywords
        let intent = if query.contains("how") || query.contains("what") || query.contains("?") {
            QueryIntent::Documentation
        } else if query.contains("error") || query.contains("issue") || query.contains("problem") {
            QueryIntent::Troubleshooting
        } else if query.contains("config") || query.contains("setup") || query.contains("install") {
            QueryIntent::Reference
        } else if query.contains("tutorial") || query.contains("guide") {
            QueryIntent::Tutorial
        } else if query.contains("code") || query.contains("example") {
            QueryIntent::Code
        } else {
            QueryIntent::Unknown
        };

        // Determine complexity
        let complexity = match word_count {
            1..=2 => QueryComplexity::Simple,
            3..=5 => QueryComplexity::Moderate,
            _ => QueryComplexity::Complex,
        };

        // Find technical terms and create entities
        let mut entities = Vec::new();
        let technical_terms: Vec<String> = words
            .iter()
            .filter(|word| self.technical_terms.contains_key(&word.to_lowercase()))
            .map(|word| {
                // Create entity for technical term
                entities.push(Entity {
                    text: word.to_string(),
                    entity_type: EntityType::Concept,
                    confidence: zero_latency_core::values::Score::new(0.8).unwrap_or_default(),
                });
                word.to_string()
            })
            .collect();

        Ok(QueryAnalysis {
            intent,
            complexity,
            technical_terms,
            entities,
            suggestions: Vec::new(), // Could add query suggestions in future
        })
    }
}

impl SimpleQueryEnhancer {
    /// Analyze query context to determine expansion strategy
    fn analyze_query_context(&self, query: &str) -> QueryContext {
        let query_lower = query.to_lowercase();

        let context = QueryContext {
            is_question: query.contains("?")
                || query_lower.starts_with("how")
                || query_lower.starts_with("what")
                || query_lower.starts_with("why"),
            is_troubleshooting: query_lower.contains("error")
                || query_lower.contains("issue")
                || query_lower.contains("problem")
                || query_lower.contains("fix"),
            is_tutorial: query_lower.contains("tutorial")
                || query_lower.contains("guide")
                || query_lower.contains("how to"),
            is_setup: query_lower.contains("setup")
                || query_lower.contains("install")
                || query_lower.contains("configure"),
            complexity_score: self.calculate_complexity_score(query),
        };

        context
    }

    /// Calculate complexity score for query
    fn calculate_complexity_score(&self, query: &str) -> f32 {
        let word_count = query.split_whitespace().count();
        let has_technical_terms = query
            .split_whitespace()
            .any(|word| self.technical_terms.contains_key(&word.to_lowercase()));

        let base_score = word_count as f32 * 0.1;
        let technical_bonus = if has_technical_terms { 0.3 } else { 0.0 };

        (base_score + technical_bonus).min(1.0)
    }

    /// Apply context-aware expansion based on query analysis
    fn expand_with_context(&self, query: &str, context: &QueryContext) -> Vec<String> {
        let mut expansions = Vec::new();
        let query_lower = query.to_lowercase();

        // Apply context rules
        for (pattern, terms) in &self.context_rules {
            if query_lower.contains(pattern) {
                expansions.extend(terms.clone());
            }
        }

        // Add context-specific terms based on analysis
        if context.is_question {
            expansions.extend(vec!["documentation".to_string(), "reference".to_string()]);
        }

        if context.is_troubleshooting {
            expansions.extend(vec![
                "debug".to_string(),
                "fix".to_string(),
                "resolve".to_string(),
            ]);
        }

        if context.is_tutorial {
            expansions.extend(vec!["example".to_string(), "step by step".to_string()]);
        }

        if context.is_setup {
            expansions.extend(vec![
                "installation".to_string(),
                "getting started".to_string(),
            ]);
        }

        expansions
    }

    /// Apply pattern-based expansion for common phrases
    fn expand_with_patterns(&self, query: &str) -> Vec<String> {
        let mut expansions = Vec::new();
        let query_lower = query.to_lowercase();

        for (pattern, terms) in &self.pattern_expansions {
            if query_lower.contains(pattern) {
                expansions.extend(terms.clone());
            }
        }

        expansions
    }

    /// Extract synonyms from the expansion list
    fn extract_synonyms_from_expansion(&self, expansions: &[String]) -> Vec<String> {
        // For now, return a subset of expansions that are synonyms
        // In a more sophisticated implementation, this would distinguish between
        // technical terms, synonyms, and contextual expansions
        expansions
            .iter()
            .filter(|term| {
                // Check if this term appears in our synonyms mapping
                self.synonyms
                    .values()
                    .any(|synonyms| synonyms.contains(term))
            })
            .cloned()
            .collect()
    }

    /// Expand query terms with technical synonyms and related terms
    fn expand_query_terms(&self, query: &str) -> Vec<String> {
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut expanded_terms = Vec::new();

        for word in words {
            let word_lower = word.to_lowercase();

            // Check for technical terms
            if let Some(tech_terms) = self.technical_terms.get(&word_lower) {
                expanded_terms.extend(tech_terms.clone());
            }

            // Check for synonyms
            if let Some(synonyms) = self.synonyms.get(&word_lower) {
                expanded_terms.extend(synonyms.clone());
            }
        }

        expanded_terms
    }

    /// Extract entities from query text
    fn extract_entities(&self, query: &str) -> Vec<Entity> {
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut entities = Vec::new();

        for word in words {
            let word_lower = word.to_lowercase();

            // Check if word is a technical term
            if self.technical_terms.contains_key(&word_lower) {
                entities.push(Entity {
                    text: word.to_string(),
                    entity_type: EntityType::Concept,
                    confidence: zero_latency_core::values::Score::new(0.8).unwrap_or_default(),
                });
            }
        }

        entities
    }
}

/// Multi-factor result ranker that improves search relevance
pub struct MultiFactorResultRanker {
    // Scoring weights for different factors
    vector_similarity_weight: f32,
    content_relevance_weight: f32,
    title_boost_weight: f32,
    recency_weight: f32,
    metadata_relevance_weight: f32,

    // Boost factors
    title_boost_factor: f32,
    heading_boost_factor: f32,
    exact_match_boost: f32,

    // Content analysis thresholds
    short_content_threshold: usize,
    keyword_density_threshold: f32,
}

impl MultiFactorResultRanker {
    pub fn new() -> Self {
        Self {
            // Scoring weights (must sum to ~1.0 for balanced scoring)
            vector_similarity_weight: 0.4,  // Base vector similarity
            content_relevance_weight: 0.25, // Content analysis score
            title_boost_weight: 0.15,       // Title/heading boost
            recency_weight: 0.1,            // Document freshness
            metadata_relevance_weight: 0.1, // Metadata matching

            // Boost factors
            title_boost_factor: 2.0,   // Strong boost for titles
            heading_boost_factor: 1.5, // Moderate boost for headings
            exact_match_boost: 1.8,    // Boost for exact query matches

            // Analysis thresholds
            short_content_threshold: 150, // Consider content under 150 chars as titles/headings
            keyword_density_threshold: 0.1, // 10% keyword density threshold
        }
    }

    /// Configure custom scoring weights
    pub fn with_weights(
        vector_weight: f32,
        content_weight: f32,
        title_weight: f32,
        recency_weight: f32,
        metadata_weight: f32,
    ) -> Self {
        Self {
            vector_similarity_weight: vector_weight,
            content_relevance_weight: content_weight,
            title_boost_weight: title_weight,
            recency_weight,
            metadata_relevance_weight: metadata_weight,
            ..Self::new()
        }
    }
}

/// Enhanced scoring signals for comprehensive ranking
#[derive(Debug, Clone)]
pub struct EnhancedScoringSignals {
    pub vector_similarity_score: f32,
    pub content_relevance_score: f32,
    pub title_boost_score: f32,
    pub recency_score: f32,
    pub metadata_relevance_score: f32,
    pub keyword_density: f32,
    pub exact_match_bonus: f32,
    pub final_combined_score: f32,
}

#[async_trait]
impl ResultRanker for MultiFactorResultRanker {
    async fn rank(&self, results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        tracing::info!(
            "[AdvancedSearch] ResultRankingStep active: ranking {} results.",
            results.len()
        );

        // Store original query for relevance analysis
        // For now, we'll extract common terms - in a real implementation,
        // this would come from the search context
        let query_terms = self.extract_common_query_terms(&results);

        // Calculate enhanced scores for all results
        let mut scored_results: Vec<(SearchResult, EnhancedScoringSignals)> = Vec::new();

        for result in results {
            let signals = self
                .calculate_comprehensive_score(&result, &query_terms)
                .await;
            scored_results.push((result, signals));
        }

        // Sort by final combined score (descending)
        scored_results.sort_by(|a, b| {
            b.1.final_combined_score
                .partial_cmp(&a.1.final_combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Log ranking details for top 3 results
        for (i, (result, signals)) in scored_results.iter().take(3).enumerate() {
            tracing::debug!(
                "[AdvancedSearch] Rank #{}: Score {:.3} (Vector: {:.3}, Content: {:.3}, Title: {:.3}, Recency: {:.3}, Metadata: {:.3})",
                i + 1,
                signals.final_combined_score,
                signals.vector_similarity_score,
                signals.content_relevance_score,
                signals.title_boost_score,
                signals.recency_score,
                signals.metadata_relevance_score
            );
        }

        // Extract ranked results and update their final scores
        let ranked_results: Vec<SearchResult> = scored_results
            .into_iter()
            .map(|(mut result, signals)| {
                // Update the result's final score with the enhanced score
                result.final_score =
                    zero_latency_core::values::Score::new(signals.final_combined_score)
                        .unwrap_or(result.final_score);
                result
            })
            .collect();

        tracing::info!(
            "[AdvancedSearch] Multi-factor ranking complete. Top score: {:.3}",
            ranked_results
                .first()
                .map(|r| r.final_score.value())
                .unwrap_or(0.0)
        );

        Ok(ranked_results)
    }

    async fn explain_ranking(&self, result: &SearchResult) -> Result<RankingSignals> {
        // Extract query terms for analysis (simplified)
        let query_terms = vec!["api".to_string(), "config".to_string()]; // Placeholder
        let signals = self
            .calculate_comprehensive_score(result, &query_terms)
            .await;

        Ok(RankingSignals {
            vector_similarity: result.final_score,
            term_frequency: zero_latency_core::values::Score::new(signals.keyword_density)
                .unwrap_or_default(),
            document_frequency: zero_latency_core::values::Score::new(
                signals.content_relevance_score,
            )
            .unwrap_or_default(),
            title_boost: signals.title_boost_score,
            freshness_boost: signals.recency_score,
            custom_signals: {
                let mut map = HashMap::new();
                map.insert(
                    "metadata_relevance".to_string(),
                    signals.metadata_relevance_score,
                );
                map.insert("exact_match_bonus".to_string(), signals.exact_match_bonus);
                map.insert(
                    "final_combined_score".to_string(),
                    signals.final_combined_score,
                );
                map
            },
        })
    }
}

impl MultiFactorResultRanker {
    /// Calculate comprehensive score using all ranking factors
    async fn calculate_comprehensive_score(
        &self,
        result: &SearchResult,
        query_terms: &[String],
    ) -> EnhancedScoringSignals {
        // 1. Vector Similarity Score (base score from vector search)
        let vector_similarity_score = result.final_score.value();

        // 2. Content Relevance Score
        let content_relevance_score = self.calculate_content_relevance_score(result, query_terms);

        // 3. Title/Heading Boost Score
        let title_boost_score = self.calculate_title_boost_score(result);

        // 4. Recency Score (based on document freshness)
        let recency_score = self.calculate_recency_score(result);

        // 5. Metadata Relevance Score
        let metadata_relevance_score = self.calculate_metadata_relevance_score(result, query_terms);

        // 6. Additional signals
        let keyword_density = self.calculate_keyword_density(result, query_terms);
        let exact_match_bonus = self.calculate_exact_match_bonus(result, query_terms);

        // 7. Combine all scores with weights
        let final_combined_score = (vector_similarity_score * self.vector_similarity_weight)
            + (content_relevance_score * self.content_relevance_weight)
            + (title_boost_score * self.title_boost_weight)
            + (recency_score * self.recency_weight)
            + (metadata_relevance_score * self.metadata_relevance_weight)
            + exact_match_bonus; // Bonus is additive

        EnhancedScoringSignals {
            vector_similarity_score,
            content_relevance_score,
            title_boost_score,
            recency_score,
            metadata_relevance_score,
            keyword_density,
            exact_match_bonus,
            final_combined_score,
        }
    }

    /// Calculate content relevance based on term matching and keyword density
    fn calculate_content_relevance_score(
        &self,
        result: &SearchResult,
        query_terms: &[String],
    ) -> f32 {
        if query_terms.is_empty() {
            return 0.5; // Neutral score if no query terms
        }

        let content_lower = result.content.to_lowercase();
        let words: Vec<&str> = content_lower.split_whitespace().collect();

        if words.is_empty() {
            return 0.0;
        }

        // Count term matches
        let mut matched_terms = 0;
        let mut total_matches = 0;

        for term in query_terms {
            let term_lower = term.to_lowercase();
            let matches = words
                .iter()
                .filter(|word| word.contains(&term_lower))
                .count();
            if matches > 0 {
                matched_terms += 1;
                total_matches += matches;
            }
        }

        // Calculate relevance score
        let term_coverage = matched_terms as f32 / query_terms.len() as f32;
        let keyword_density = total_matches as f32 / words.len() as f32;

        // Combine term coverage and keyword density
        (term_coverage * 0.7 + keyword_density.min(0.3) * 0.3).min(1.0)
    }

    /// Calculate title/heading boost based on content characteristics
    fn calculate_title_boost_score(&self, result: &SearchResult) -> f32 {
        let content = &result.content;
        let content_length = content.len();

        // Check for title-like characteristics
        let mut boost = 1.0;

        // Short content is likely to be a title or heading
        if content_length < self.short_content_threshold {
            boost *= self.title_boost_factor;
        }

        // Content starts with markdown header
        if content.starts_with('#') {
            boost *= self.heading_boost_factor;
        }

        // Content is mostly uppercase (might be a header)
        let uppercase_ratio = content.chars().filter(|c| c.is_uppercase()).count() as f32
            / content.chars().count() as f32;
        if uppercase_ratio > 0.6 {
            boost *= 1.3;
        }

        // Check for heading-like patterns in heading_path
        if !result.heading_path.is_empty() {
            boost *= 1.2; // Moderate boost for results with heading context
        }

        boost.min(3.0) // Cap the boost to prevent extreme values
    }

    /// Calculate recency score based on document freshness
    fn calculate_recency_score(&self, _result: &SearchResult) -> f32 {
        // For now, return a neutral score
        // In a real implementation, this would analyze:
        // - Document modification time
        // - Content freshness indicators
        // - Version information
        0.5
    }

    /// Calculate metadata relevance score
    fn calculate_metadata_relevance_score(
        &self,
        result: &SearchResult,
        query_terms: &[String],
    ) -> f32 {
        if query_terms.is_empty() {
            return 0.5;
        }

        let mut relevance_score = 0.0;
        let mut metadata_fields_checked = 0;

        // Check document title
        let title_lower = result.document_title.to_lowercase();
        let title_matches = query_terms
            .iter()
            .filter(|term| title_lower.contains(&term.to_lowercase()))
            .count();

        if title_matches > 0 {
            relevance_score += (title_matches as f32 / query_terms.len() as f32) * 0.4;
        }
        metadata_fields_checked += 1;

        // Check heading path
        if !result.heading_path.is_empty() {
            let heading_text = result.heading_path.join(" ");
            let heading_lower = heading_text.to_lowercase();
            let heading_matches = query_terms
                .iter()
                .filter(|term| heading_lower.contains(&term.to_lowercase()))
                .count();

            if heading_matches > 0 {
                relevance_score += (heading_matches as f32 / query_terms.len() as f32) * 0.3;
            }
            metadata_fields_checked += 1;
        }

        // Check document path
        let path_lower = result.document_path.to_lowercase();
        let path_matches = query_terms
            .iter()
            .filter(|term| path_lower.contains(&term.to_lowercase()))
            .count();

        if path_matches > 0 {
            relevance_score += (path_matches as f32 / query_terms.len() as f32) * 0.2;
        }
        metadata_fields_checked += 1;

        // Check URL if available
        if let Some(ref url) = result.url {
            let url_lower = url.to_lowercase();
            let url_matches = query_terms
                .iter()
                .filter(|term| url_lower.contains(&term.to_lowercase()))
                .count();

            if url_matches > 0 {
                relevance_score += (url_matches as f32 / query_terms.len() as f32) * 0.1;
            }
            metadata_fields_checked += 1;
        }

        // Normalize by number of metadata fields checked
        if metadata_fields_checked > 0 {
            relevance_score / metadata_fields_checked as f32
        } else {
            0.5
        }
    }

    /// Calculate keyword density in content
    fn calculate_keyword_density(&self, result: &SearchResult, query_terms: &[String]) -> f32 {
        if query_terms.is_empty() {
            return 0.0;
        }

        let content_lower = result.content.to_lowercase();
        let words: Vec<&str> = content_lower.split_whitespace().collect();

        if words.is_empty() {
            return 0.0;
        }

        let total_keyword_matches = query_terms
            .iter()
            .map(|term| {
                let term_lower = term.to_lowercase();
                words
                    .iter()
                    .filter(|word| word.contains(&term_lower))
                    .count()
            })
            .sum::<usize>();

        total_keyword_matches as f32 / words.len() as f32
    }

    /// Calculate exact match bonus for queries that exactly match content
    fn calculate_exact_match_bonus(&self, result: &SearchResult, query_terms: &[String]) -> f32 {
        if query_terms.is_empty() {
            return 0.0;
        }

        let content_lower = result.content.to_lowercase();
        let query_phrase = query_terms.join(" ").to_lowercase();

        // Check for exact phrase match
        if content_lower.contains(&query_phrase) {
            return 0.2; // 20% bonus for exact phrase match
        }

        // Check for exact term matches
        let exact_matches = query_terms
            .iter()
            .filter(|term| {
                let term_lower = term.to_lowercase();
                content_lower
                    .split_whitespace()
                    .any(|word| word == term_lower)
            })
            .count();

        if exact_matches == query_terms.len() {
            return 0.1; // 10% bonus if all terms have exact matches
        }

        // Partial exact match bonus
        (exact_matches as f32 / query_terms.len() as f32) * 0.05
    }

    /// Extract common query terms from search results (heuristic)
    fn extract_common_query_terms(&self, results: &[SearchResult]) -> Vec<String> {
        // This is a simplified heuristic - in a real implementation,
        // the original query would be passed through the ranking context

        if results.is_empty() {
            return vec![];
        }

        // Look for common short words that appear frequently across results
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        for result in results.iter().take(5) {
            // Analyze top 5 results
            let content_lower = result.content.to_lowercase();
            let words: Vec<&str> = content_lower
                .split_whitespace()
                .filter(|word| word.len() >= 3 && word.len() <= 15) // Reasonable term length
                .collect();

            for word in words {
                *word_counts.entry(word.to_string()).or_insert(0) += 1;
            }
        }

        // Return words that appear in multiple results
        let min_appearances = (results.len() / 2).max(1);
        word_counts
            .into_iter()
            .filter(|(_, count)| *count >= min_appearances)
            .map(|(word, _)| word)
            .take(5) // Limit to 5 terms
            .collect()
    }
}
