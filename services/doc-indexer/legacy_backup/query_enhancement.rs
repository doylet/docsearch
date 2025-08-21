use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Query enhancement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEnhancementConfig {
    pub enable_expansion: bool,
    pub enable_synonyms: bool,
    pub enable_stemming: bool,
    pub max_expansions: usize,
    pub synonym_weight: f32,
}

impl Default for QueryEnhancementConfig {
    fn default() -> Self {
        Self {
            enable_expansion: true,
            enable_synonyms: true,
            enable_stemming: false, // Keep disabled for now
            max_expansions: 3,
            synonym_weight: 0.8,
        }
    }
}

/// Enhanced query with original and expanded terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedQuery {
    pub original: String,
    pub expanded_terms: Vec<String>,
    pub enhanced_query: String,
    pub enhancement_metadata: EnhancementMetadata,
}

/// Query enhancement metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementMetadata {
    pub original_term_count: usize,
    pub expanded_term_count: usize,
    pub synonyms_added: usize,
    pub techniques_applied: Vec<String>,
}

/// Query enhancement engine
pub struct QueryEnhancer {
    config: QueryEnhancementConfig,
    synonym_map: HashMap<String, Vec<String>>,
    domain_vocabulary: HashSet<String>,
}

impl QueryEnhancer {
    pub fn new(config: QueryEnhancementConfig) -> Self {
        let mut enhancer = Self {
            config,
            synonym_map: HashMap::new(),
            domain_vocabulary: HashSet::new(),
        };
        
        enhancer.initialize_synonyms();
        enhancer.initialize_domain_vocabulary();
        enhancer
    }

    /// Enhance a query with expansion and synonyms
    pub fn enhance_query(&self, query: &str) -> Result<EnhancedQuery> {
        let original = query.to_string();
        let mut enhanced_terms = Vec::new();
        let mut techniques_applied = Vec::new();
        let mut synonyms_added = 0;

        // Tokenize the original query
        let original_terms: Vec<String> = self.tokenize_query(query);
        let original_term_count = original_terms.len();

        // Add original terms
        enhanced_terms.extend(original_terms.clone());

        // Add synonyms if enabled
        if self.config.enable_synonyms {
            for term in &original_terms {
                if let Some(synonyms) = self.get_synonyms(term) {
                    let mut added_for_term = 0;
                    for synonym in synonyms {
                        if enhanced_terms.len() < original_term_count + self.config.max_expansions
                            && added_for_term < 2 // Limit synonyms per term
                            && !enhanced_terms.contains(&synonym)
                        {
                            enhanced_terms.push(synonym);
                            synonyms_added += 1;
                            added_for_term += 1;
                        }
                    }
                }
            }
            techniques_applied.push("synonym_expansion".to_string());
        }

        // Add domain-specific expansions if enabled
        if self.config.enable_expansion {
            for term in &original_terms {
                if let Some(expansions) = self.get_domain_expansions(term) {
                    for expansion in expansions {
                        if enhanced_terms.len() < original_term_count + self.config.max_expansions
                            && !enhanced_terms.contains(&expansion)
                        {
                            enhanced_terms.push(expansion);
                        }
                    }
                }
            }
            techniques_applied.push("domain_expansion".to_string());
        }

        // Create enhanced query string
        let enhanced_query = enhanced_terms.join(" ");
        let expanded_term_count = enhanced_terms.len();

        Ok(EnhancedQuery {
            original,
            expanded_terms: enhanced_terms,
            enhanced_query,
            enhancement_metadata: EnhancementMetadata {
                original_term_count,
                expanded_term_count,
                synonyms_added,
                techniques_applied,
            },
        })
    }

    /// Tokenize query into meaningful terms
    fn tokenize_query(&self, query: &str) -> Vec<String> {
        query
            .to_lowercase()
            .split_whitespace()
            .map(|term| {
                // Remove common punctuation
                term.trim_matches(|c: char| c.is_ascii_punctuation())
                    .to_string()
            })
            .filter(|term| {
                // Filter out empty terms and common stop words
                !term.is_empty() && !self.is_stop_word(term)
            })
            .collect()
    }

    /// Check if a word is a stop word
    fn is_stop_word(&self, word: &str) -> bool {
        matches!(
            word,
            "a" | "an" | "and" | "are" | "as" | "at" | "be" | "by" | "for" | "from" | "has"
                | "he" | "in" | "is" | "it" | "its" | "of" | "on" | "that" | "the" | "to"
                | "was" | "will" | "with"
        )
    }

    /// Get synonyms for a term
    fn get_synonyms(&self, term: &str) -> Option<Vec<String>> {
        self.synonym_map.get(term).cloned()
    }

    /// Get domain-specific expansions for a term
    fn get_domain_expansions(&self, term: &str) -> Option<Vec<String>> {
        // Domain-specific expansion logic for technical documentation
        match term {
            "api" => Some(vec!["endpoint".to_string(), "interface".to_string()]),
            "search" => Some(vec!["query".to_string(), "retrieval".to_string()]),
            "index" => Some(vec!["indexing".to_string(), "document".to_string()]),
            "vector" => Some(vec!["embedding".to_string(), "similarity".to_string()]),
            "chunk" => Some(vec!["segment".to_string(), "fragment".to_string()]),
            "quality" => Some(vec!["metrics".to_string(), "performance".to_string()]),
            "config" => Some(vec!["configuration".to_string(), "settings".to_string()]),
            "server" => Some(vec!["service".to_string(), "daemon".to_string()]),
            _ => None,
        }
    }

    /// Initialize synonym mappings
    fn initialize_synonyms(&mut self) {
        // Technical documentation synonyms
        self.synonym_map.insert(
            "api".to_string(),
            vec!["endpoint".to_string(), "interface".to_string(), "service".to_string()],
        );
        self.synonym_map.insert(
            "search".to_string(),
            vec!["query".to_string(), "find".to_string(), "retrieval".to_string()],
        );
        self.synonym_map.insert(
            "document".to_string(),
            vec!["file".to_string(), "text".to_string(), "content".to_string()],
        );
        self.synonym_map.insert(
            "index".to_string(),
            vec!["indexing".to_string(), "catalog".to_string()],
        );
        self.synonym_map.insert(
            "vector".to_string(),
            vec!["embedding".to_string(), "representation".to_string()],
        );
        self.synonym_map.insert(
            "chunk".to_string(),
            vec!["segment".to_string(), "piece".to_string(), "fragment".to_string()],
        );
        self.synonym_map.insert(
            "quality".to_string(),
            vec!["performance".to_string(), "metrics".to_string(), "score".to_string()],
        );
        self.synonym_map.insert(
            "configuration".to_string(),
            vec!["config".to_string(), "settings".to_string(), "setup".to_string()],
        );
        self.synonym_map.insert(
            "error".to_string(),
            vec!["issue".to_string(), "problem".to_string(), "failure".to_string()],
        );
        self.synonym_map.insert(
            "implementation".to_string(),
            vec!["code".to_string(), "solution".to_string(), "approach".to_string()],
        );
    }

    /// Initialize domain-specific vocabulary
    fn initialize_domain_vocabulary(&mut self) {
        let domain_terms = vec![
            "qdrant", "embedding", "vector", "semantic", "similarity", "chunking",
            "indexing", "retrieval", "nlp", "tokenizer", "transformer", "bert",
            "api", "endpoint", "microservice", "docker", "kubernetes", "rust",
            "async", "tokio", "serde", "json", "http", "rest", "grpc",
            "monitoring", "logging", "metrics", "observability", "tracing",
            "configuration", "yaml", "toml", "environment", "deployment",
        ];

        for term in domain_terms {
            self.domain_vocabulary.insert(term.to_string());
        }
    }

    /// Update synonym mappings dynamically
    pub fn add_synonym(&mut self, term: String, synonyms: Vec<String>) {
        self.synonym_map.insert(term, synonyms);
    }

    /// Get enhancement statistics
    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("synonym_mappings".to_string(), self.synonym_map.len());
        stats.insert("domain_vocabulary_size".to_string(), self.domain_vocabulary.len());
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_enhancement_basic() {
        let enhancer = QueryEnhancer::new(QueryEnhancementConfig::default());
        let result = enhancer.enhance_query("api search").unwrap();

        assert_eq!(result.original, "api search");
        assert!(result.expanded_terms.contains(&"api".to_string()));
        assert!(result.expanded_terms.contains(&"search".to_string()));
        assert!(result.expanded_terms.len() > 2); // Should have expansions
        assert!(result.enhancement_metadata.synonyms_added > 0);
    }

    #[test]
    fn test_tokenization() {
        let enhancer = QueryEnhancer::new(QueryEnhancementConfig::default());
        let tokens = enhancer.tokenize_query("How to configure the API endpoint?");

        assert!(tokens.contains(&"configure".to_string()));
        assert!(tokens.contains(&"api".to_string()));
        assert!(tokens.contains(&"endpoint".to_string()));
        assert!(!tokens.contains(&"the".to_string())); // Stop word removed
        assert!(!tokens.contains(&"to".to_string())); // Stop word removed
    }

    #[test]
    fn test_synonym_expansion() {
        let enhancer = QueryEnhancer::new(QueryEnhancementConfig::default());
        let result = enhancer.enhance_query("vector search").unwrap();

        let enhanced_terms_str = result.enhanced_query;
        assert!(enhanced_terms_str.contains("vector"));
        assert!(enhanced_terms_str.contains("search"));
        // Should contain some synonyms
        assert!(
            enhanced_terms_str.contains("embedding") 
            || enhanced_terms_str.contains("query")
            || enhanced_terms_str.contains("retrieval")
        );
    }

    #[test]
    fn test_stop_word_filtering() {
        let enhancer = QueryEnhancer::new(QueryEnhancementConfig::default());
        let tokens = enhancer.tokenize_query("the api and the search");

        assert!(tokens.contains(&"api".to_string()));
        assert!(tokens.contains(&"search".to_string()));
        assert!(!tokens.contains(&"the".to_string()));
        assert!(!tokens.contains(&"and".to_string()));
    }
}
