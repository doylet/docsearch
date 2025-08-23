/// Enhanced search pipeline components for advanced search features
/// 
/// This module provides concrete implementations of QueryEnhancer and ResultRanker
/// to enable sophisticated search capabilities including query expansion,
/// term enhancement, and multi-factor result ranking.

use std::collections::HashMap;
use async_trait::async_trait;
use zero_latency_core::Result;
use zero_latency_search::{
    QueryEnhancer, ResultRanker, EnhancedQuery, QueryAnalysis, RankingSignals,
    SearchResult, QueryIntent, QueryComplexity, Entity, EntityType
};

/// Simple query enhancer that expands queries with technical terms and synonyms
pub struct SimpleQueryEnhancer {
    // Domain-specific term mappings for documentation search
    technical_terms: HashMap<String, Vec<String>>,
    synonyms: HashMap<String, Vec<String>>,
}

impl SimpleQueryEnhancer {
    pub fn new() -> Self {
        let mut technical_terms = HashMap::new();
        let mut synonyms = HashMap::new();
        
        // Add technical term expansions
        technical_terms.insert("api".to_string(), vec![
            "application programming interface".to_string(),
            "endpoint".to_string(),
            "rest".to_string(),
            "graphql".to_string(),
        ]);
        
        technical_terms.insert("config".to_string(), vec![
            "configuration".to_string(),
            "settings".to_string(),
            "parameters".to_string(),
            "options".to_string(),
        ]);
        
        technical_terms.insert("auth".to_string(), vec![
            "authentication".to_string(),
            "authorization".to_string(),
            "security".to_string(),
            "login".to_string(),
        ]);
        
        // Add synonyms
        synonyms.insert("search".to_string(), vec![
            "find".to_string(),
            "query".to_string(),
            "lookup".to_string(),
        ]);
        
        synonyms.insert("error".to_string(), vec![
            "issue".to_string(),
            "problem".to_string(),
            "bug".to_string(),
            "failure".to_string(),
        ]);
        
        Self {
            technical_terms,
            synonyms,
        }
    }
}

#[async_trait]
impl QueryEnhancer for SimpleQueryEnhancer {
    async fn enhance(&self, query: &str) -> Result<EnhancedQuery> {
        let original_query = query.to_string();
        
        // Expand query with technical terms and synonyms
        let expanded_terms = self.expand_query_terms(query);
        println!("🔍 QueryEnhancementStep: Expanding query '{}' with {} terms: {:?}", 
                query, expanded_terms.len(), expanded_terms);
        
        let enhanced_query = if expanded_terms.len() > 1 {
            format!("{} {}", query, expanded_terms.join(" "))
        } else {
            query.to_string()
        };
        
        println!("✨ QueryEnhancementStep: Enhanced query: '{}'", enhanced_query);
        
        Ok(EnhancedQuery {
            original: original_query,
            enhanced: enhanced_query,
            synonyms_added: vec![], // Simple implementation
            technical_terms: expanded_terms,
            expansion_strategy: "technical_terms_and_synonyms".to_string(),
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
        let technical_terms: Vec<String> = words.iter()
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
    // Boost factors for different signals
    title_boost_factor: f32,
    freshness_boost_factor: f32,
    term_frequency_weight: f32,
}

impl MultiFactorResultRanker {
    pub fn new() -> Self {
        Self {
            title_boost_factor: 1.5,
            freshness_boost_factor: 1.2,
            term_frequency_weight: 0.3,
        }
    }
}

#[async_trait]
impl ResultRanker for MultiFactorResultRanker {
    async fn rank(&self, mut results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        // Sort by enhanced score combining multiple factors
        results.sort_by(|a, b| {
            let score_a = self.calculate_enhanced_score(a);
            let score_b = self.calculate_enhanced_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(results)
    }
    
    async fn explain_ranking(&self, result: &SearchResult) -> Result<RankingSignals> {
        let _base_score = result.final_score;
        
        // Calculate boosts
        let title_boost = if result.content.to_lowercase().contains("title") {
            self.title_boost_factor
        } else {
            1.0
        };
        
        let freshness_boost = self.freshness_boost_factor; // Simplified for now
        
        Ok(RankingSignals {
            vector_similarity: result.final_score,
            term_frequency: zero_latency_core::values::Score::new(0.5).unwrap_or_default(), // Placeholder
            document_frequency: zero_latency_core::values::Score::new(0.3).unwrap_or_default(), // Placeholder
            title_boost,
            freshness_boost,
            custom_signals: HashMap::new(),
        })
    }
}

impl MultiFactorResultRanker {
    fn calculate_enhanced_score(&self, result: &SearchResult) -> f32 {
        let base_score = result.final_score.value();
        
        // Apply title boost if content appears to be from a title/header
        let title_boost = if result.content.len() < 100 && 
                            (result.content.starts_with('#') || result.content.to_uppercase() == result.content) {
            self.title_boost_factor
        } else {
            1.0
        };
        
        // Apply freshness boost (simplified - could be based on document modification time)
        let freshness_boost = self.freshness_boost_factor;
        
        // Calculate enhanced score
        base_score * title_boost * freshness_boost
    }
}
