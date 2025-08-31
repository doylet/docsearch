use crate::query_expansion::{ExpandedQuery, ExpansionType, QueryExpansionConfig};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};
use zero_latency_core::Result;

/// Trait for synonym expansion strategies
#[async_trait]
pub trait SynonymExpansion {
    async fn expand(
        &self,
        query: &str,
        config: &QueryExpansionConfig,
    ) -> Result<Vec<ExpandedQuery>>;
}

/// Trait for morphological expansion strategies  
#[async_trait]
pub trait MorphologicalExpansion {
    async fn expand(
        &self,
        query: &str,
        config: &QueryExpansionConfig,
    ) -> Result<Vec<ExpandedQuery>>;
}

/// Trait for contextual expansion strategies
#[async_trait]
pub trait ContextualExpansion {
    async fn expand(
        &self,
        query: &str,
        config: &QueryExpansionConfig,
    ) -> Result<Vec<ExpandedQuery>>;
}

/// Simple dictionary-based synonym expansion
pub struct DictionarySynonymExpansion {
    /// Synonym dictionary mapping words to their synonyms
    synonym_dict: HashMap<String, Vec<String>>,
}

impl DictionarySynonymExpansion {
    /// Create a new dictionary-based synonym expansion with a predefined dictionary
    pub fn new() -> Self {
        let mut synonym_dict = HashMap::new();
        
        // Add common technical synonyms
        synonym_dict.insert("search".to_string(), vec!["find".to_string(), "lookup".to_string(), "query".to_string()]);
        synonym_dict.insert("find".to_string(), vec!["search".to_string(), "locate".to_string(), "discover".to_string()]);
        synonym_dict.insert("document".to_string(), vec!["file".to_string(), "text".to_string(), "content".to_string()]);
        synonym_dict.insert("file".to_string(), vec!["document".to_string(), "record".to_string()]);
        synonym_dict.insert("data".to_string(), vec!["information".to_string(), "content".to_string()]);
        synonym_dict.insert("information".to_string(), vec!["data".to_string(), "details".to_string()]);
        synonym_dict.insert("algorithm".to_string(), vec!["method".to_string(), "approach".to_string(), "technique".to_string()]);
        synonym_dict.insert("method".to_string(), vec!["approach".to_string(), "technique".to_string(), "algorithm".to_string()]);
        synonym_dict.insert("implementation".to_string(), vec!["realization".to_string(), "execution".to_string()]);
        synonym_dict.insert("performance".to_string(), vec!["speed".to_string(), "efficiency".to_string()]);
        synonym_dict.insert("optimization".to_string(), vec!["improvement".to_string(), "enhancement".to_string()]);
        synonym_dict.insert("error".to_string(), vec!["bug".to_string(), "issue".to_string(), "problem".to_string()]);
        synonym_dict.insert("bug".to_string(), vec!["error".to_string(), "issue".to_string(), "defect".to_string()]);
        synonym_dict.insert("configuration".to_string(), vec!["setup".to_string(), "settings".to_string(), "config".to_string()]);
        synonym_dict.insert("interface".to_string(), vec!["api".to_string(), "endpoint".to_string()]);
        synonym_dict.insert("database".to_string(), vec!["db".to_string(), "storage".to_string(), "repository".to_string()]);
        
        Self { synonym_dict }
    }

    /// Create with a custom synonym dictionary
    pub fn with_dictionary(synonym_dict: HashMap<String, Vec<String>>) -> Self {
        Self { synonym_dict }
    }

    /// Add synonyms for a term
    pub fn add_synonyms(&mut self, term: String, synonyms: Vec<String>) {
        self.synonym_dict.insert(term, synonyms);
    }
}

#[async_trait]
impl SynonymExpansion for DictionarySynonymExpansion {
    async fn expand(
        &self,
        query: &str,
        config: &QueryExpansionConfig,
    ) -> Result<Vec<ExpandedQuery>> {
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let mut expansions = Vec::new();
        
        // Track which terms have been expanded to avoid duplicates
        let mut expanded_terms: HashSet<String> = HashSet::new();
        
        for term in &query_terms {
            let term_lower = term.to_lowercase();
            
            if let Some(synonyms) = self.synonym_dict.get(&term_lower) {
                for synonym in synonyms.iter().take(config.max_terms_per_expansion) {
                    if expanded_terms.contains(synonym) {
                        continue;
                    }
                    
                    // Create expanded query by replacing the term with its synonym
                    let mut expanded_query_terms = query_terms.clone();
                    for (_i, query_term) in expanded_query_terms.iter_mut().enumerate() {
                        if query_term.to_lowercase() == term_lower {
                            *query_term = synonym;
                            break; // Only replace first occurrence
                        }
                    }
                    
                    let expanded_query = expanded_query_terms.join(" ");
                    expanded_terms.insert(synonym.clone());
                    
                    expansions.push(ExpandedQuery {
                        query: expanded_query,
                        expansion_type: ExpansionType::Synonym,
                        weight: config.expansion_weight,
                        source_terms: vec![term.to_string()],
                        added_terms: vec![synonym.clone()],
                    });
                    
                    if expansions.len() >= config.max_expansions {
                        break;
                    }
                }
            }
            
            if expansions.len() >= config.max_expansions {
                break;
            }
        }
        
        debug!("Generated {} synonym expansions", expansions.len());
        Ok(expansions)
    }
}

/// Simple morphological expansion using basic rules
pub struct RuleMorphologicalExpansion {
    /// Enable plural/singular variations
    pub enable_plurals: bool,
    /// Enable past/present tense variations
    pub enable_tenses: bool,
    /// Enable common suffix variations
    pub enable_suffixes: bool,
}

impl Default for RuleMorphologicalExpansion {
    fn default() -> Self {
        Self {
            enable_plurals: true,
            enable_tenses: true,
            enable_suffixes: true,
        }
    }
}

impl RuleMorphologicalExpansion {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate plural/singular variants
    fn generate_plural_variants(&self, word: &str) -> Vec<String> {
        let mut variants = Vec::new();
        let word_lower = word.to_lowercase();
        
        // Basic pluralization rules
        if word_lower.ends_with('s') && word_lower.len() > 1 {
            // Might be plural, try singular
            variants.push(word_lower[..word_lower.len() - 1].to_string());
            if word_lower.ends_with("es") && word_lower.len() > 2 {
                variants.push(word_lower[..word_lower.len() - 2].to_string());
            }
        } else {
            // Try plural forms
            if word_lower.ends_with('y') && word_lower.len() > 1 {
                variants.push(format!("{}ies", &word_lower[..word_lower.len() - 1]));
            } else if word_lower.ends_with(&['s', 'x', 'z']) || 
                     word_lower.ends_with("ch") || 
                     word_lower.ends_with("sh") {
                variants.push(format!("{}es", word_lower));
            } else {
                variants.push(format!("{}s", word_lower));
            }
        }
        
        variants
    }

    /// Generate tense variants
    fn generate_tense_variants(&self, word: &str) -> Vec<String> {
        let mut variants = Vec::new();
        let word_lower = word.to_lowercase();
        
        // Basic tense rules
        if word_lower.ends_with("ed") && word_lower.len() > 2 {
            // Past tense, try present
            variants.push(word_lower[..word_lower.len() - 2].to_string());
            if word_lower.ends_with("ied") && word_lower.len() > 3 {
                variants.push(format!("{}y", &word_lower[..word_lower.len() - 3]));
            }
        } else if word_lower.ends_with("ing") && word_lower.len() > 3 {
            // Present continuous, try base form
            variants.push(word_lower[..word_lower.len() - 3].to_string());
            // Try with 'e' added back
            variants.push(format!("{}e", &word_lower[..word_lower.len() - 3]));
        } else {
            // Try adding past tense
            if word_lower.ends_with('e') {
                variants.push(format!("{}d", word_lower));
            } else if word_lower.ends_with('y') && word_lower.len() > 1 {
                variants.push(format!("{}ied", &word_lower[..word_lower.len() - 1]));
            } else {
                variants.push(format!("{}ed", word_lower));
            }
            
            // Try adding -ing
            if word_lower.ends_with('e') && word_lower.len() > 1 {
                variants.push(format!("{}ing", &word_lower[..word_lower.len() - 1]));
            } else {
                variants.push(format!("{}ing", word_lower));
            }
        }
        
        variants
    }

    /// Generate suffix variants
    fn generate_suffix_variants(&self, word: &str) -> Vec<String> {
        let mut variants = Vec::new();
        let word_lower = word.to_lowercase();
        
        // Common suffix transformations
        let suffix_pairs = vec![
            ("tion", "te"),
            ("tion", "tive"),
            ("ness", ""),
            ("ment", ""),
            ("able", ""),
            ("ible", ""),
            ("ful", ""),
            ("less", ""),
            ("ly", ""),
        ];
        
        for (suffix, replacement) in suffix_pairs {
            if word_lower.ends_with(suffix) && word_lower.len() > suffix.len() {
                let base = &word_lower[..word_lower.len() - suffix.len()];
                if !replacement.is_empty() {
                    variants.push(format!("{}{}", base, replacement));
                } else {
                    variants.push(base.to_string());
                }
            }
        }
        
        variants
    }
}

#[async_trait]
impl MorphologicalExpansion for RuleMorphologicalExpansion {
    async fn expand(
        &self,
        query: &str,
        config: &QueryExpansionConfig,
    ) -> Result<Vec<ExpandedQuery>> {
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let mut expansions = Vec::new();
        let mut seen_expansions: HashSet<String> = HashSet::new();
        
        for term in &query_terms {
            if term.len() < 3 {
                continue; // Skip very short terms
            }
            
            let mut term_variants = Vec::new();
            
            if self.enable_plurals {
                term_variants.extend(self.generate_plural_variants(term));
            }
            
            if self.enable_tenses {
                term_variants.extend(self.generate_tense_variants(term));
            }
            
            if self.enable_suffixes {
                term_variants.extend(self.generate_suffix_variants(term));
            }
            
            // Create expansions for each variant
            for variant in term_variants.iter().take(config.max_terms_per_expansion) {
                if variant == &term.to_lowercase() || seen_expansions.contains(variant) {
                    continue;
                }
                
                // Create expanded query by replacing the term with its variant
                let mut expanded_query_terms = query_terms.clone();
                for (_i, query_term) in expanded_query_terms.iter_mut().enumerate() {
                    if query_term.to_lowercase() == term.to_lowercase() {
                        *query_term = variant;
                        break; // Only replace first occurrence
                    }
                }
                
                let expanded_query = expanded_query_terms.join(" ");
                seen_expansions.insert(variant.clone());
                
                expansions.push(ExpandedQuery {
                    query: expanded_query,
                    expansion_type: ExpansionType::Morphological,
                    weight: config.expansion_weight,
                    source_terms: vec![term.to_string()],
                    added_terms: vec![variant.clone()],
                });
                
                if expansions.len() >= config.max_expansions {
                    break;
                }
            }
            
            if expansions.len() >= config.max_expansions {
                break;
            }
        }
        
        debug!("Generated {} morphological expansions", expansions.len());
        Ok(expansions)
    }
}

/// Placeholder for corpus-based contextual expansion
/// This would require access to the document corpus or external NLP services
pub struct CorpusContextualExpansion {
    /// Whether this expansion is enabled (placeholder)
    enabled: bool,
}

impl Default for CorpusContextualExpansion {
    fn default() -> Self {
        Self { enabled: false }
    }
}

impl CorpusContextualExpansion {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ContextualExpansion for CorpusContextualExpansion {
    async fn expand(
        &self,
        _query: &str,
        _config: &QueryExpansionConfig,
    ) -> Result<Vec<ExpandedQuery>> {
        if !self.enabled {
            debug!("Contextual expansion is disabled");
            return Ok(vec![]);
        }
        
        // Placeholder implementation
        // In a real implementation, this would:
        // 1. Analyze query terms against document corpus
        // 2. Find frequently co-occurring terms
        // 3. Generate contextually relevant expansions
        // 4. Possibly use external NLP services or embeddings
        
        warn!("Contextual expansion not yet implemented");
        Ok(vec![])
    }
}

/// Factory for creating default expansion strategies
pub struct ExpansionStrategyFactory;

impl ExpansionStrategyFactory {
    /// Create default expansion strategies
    pub fn create_default() -> crate::query_expansion::ExpansionStrategies {
        crate::query_expansion::ExpansionStrategies {
            synonym: Box::new(DictionarySynonymExpansion::new()),
            morphological: Box::new(RuleMorphologicalExpansion::new()),
            contextual: Some(Box::new(CorpusContextualExpansion::new())),
        }
    }
    
    /// Create strategies with custom synonym dictionary
    pub fn create_with_synonyms(
        synonym_dict: HashMap<String, Vec<String>>
    ) -> crate::query_expansion::ExpansionStrategies {
        crate::query_expansion::ExpansionStrategies {
            synonym: Box::new(DictionarySynonymExpansion::with_dictionary(synonym_dict)),
            morphological: Box::new(RuleMorphologicalExpansion::new()),
            contextual: Some(Box::new(CorpusContextualExpansion::new())),
        }
    }
}
