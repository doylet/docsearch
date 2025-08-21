use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingConfig {
    pub vector_similarity_weight: f32,
    pub document_frequency_weight: f32,
    pub title_boost: f32,
    pub freshness_weight: f32,
    pub length_penalty_threshold: usize,
    pub length_penalty_factor: f32,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            vector_similarity_weight: 0.6,
            document_frequency_weight: 0.2,
            title_boost: 1.3,
            freshness_weight: 0.1,
            length_penalty_threshold: 2000,
            length_penalty_factor: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedResult {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub vector_similarity: f32,
    pub final_score: f32,
    pub ranking_signals: RankingSignals,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingSignals {
    pub vector_similarity: f32,
    pub document_frequency_score: f32,
    pub title_match_boost: f32,
    pub freshness_score: f32,
    pub length_penalty: f32,
    pub combined_score: f32,
}

pub struct ResultRanker {
    config: RankingConfig,
    document_frequencies: HashMap<String, f32>,
    total_documents: f32,
}

impl ResultRanker {
    pub fn new(config: RankingConfig) -> Self {
        Self {
            config,
            document_frequencies: HashMap::new(),
            total_documents: 0.0,
        }
    }

    pub fn with_document_stats(mut self, doc_frequencies: HashMap<String, f32>, total_docs: f32) -> Self {
        self.document_frequencies = doc_frequencies;
        self.total_documents = total_docs;
        self
    }

    pub fn rank_results(
        &self,
        results: Vec<crate::vector_db_trait::SearchResult>,
        query_terms: &[String],
        enhanced_terms: &[String],
    ) -> Vec<RankedResult> {
        let mut ranked_results: Vec<RankedResult> = results
            .into_iter()
            .map(|result| self.compute_ranking_score(result, query_terms, enhanced_terms))
            .collect();

        // Sort by final score (descending)
        ranked_results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap_or(std::cmp::Ordering::Equal));

        ranked_results
    }

    fn compute_ranking_score(
        &self,
        result: crate::vector_db_trait::SearchResult,
        query_terms: &[String],
        enhanced_terms: &[String],
    ) -> RankedResult {
        let vector_similarity = result.score;
        
        // Document frequency scoring (inverse document frequency)
        let df_score = self.compute_document_frequency_score(enhanced_terms, &result.content);
        
        // Title matching boost
        let title_boost = self.compute_title_boost(&result, query_terms);
        
        // Freshness score (placeholder - could be enhanced with actual timestamps)
        let freshness_score = self.compute_freshness_score(&result);
        
        // Length penalty for very long documents
        let length_penalty = self.compute_length_penalty(&result.content);
        
        // Combine all signals
        let combined_score = 
            (vector_similarity * self.config.vector_similarity_weight) +
            (df_score * self.config.document_frequency_weight) +
            (title_boost * self.config.title_boost) +
            (freshness_score * self.config.freshness_weight) -
            (length_penalty * self.config.length_penalty_factor);

        let ranking_signals = RankingSignals {
            vector_similarity,
            document_frequency_score: df_score,
            title_match_boost: title_boost,
            freshness_score,
            length_penalty,
            combined_score,
        };

        // Generate enhanced snippet
        let snippet = self.generate_enhanced_snippet(&result.content, query_terms, enhanced_terms);

        // Create metadata from the original result
        let mut metadata = HashMap::new();
        metadata.insert("document_id".to_string(), serde_json::Value::String(result.document_id.clone()));
        metadata.insert("document_title".to_string(), serde_json::Value::String(result.document_title.clone()));
        metadata.insert("section".to_string(), serde_json::Value::String(result.section.clone()));
        metadata.insert("doc_type".to_string(), serde_json::Value::String(result.doc_type.clone()));
        if let Some(heading) = &result.heading {
            metadata.insert("heading".to_string(), serde_json::Value::String(heading.clone()));
        }

        RankedResult {
            id: result.chunk_id,
            content: result.content,
            metadata,
            vector_similarity,
            final_score: combined_score,
            ranking_signals,
            snippet,
        }
    }

    fn compute_document_frequency_score(&self, terms: &[String], content: &str) -> f32 {
        let content_lower = content.to_lowercase();
        let mut total_score = 0.0;
        let mut term_count = 0;

        for term in terms {
            let term_lower = term.to_lowercase();
            if content_lower.contains(&term_lower) {
                // Simple IDF-like scoring
                let frequency = self.document_frequencies.get(term).unwrap_or(&1.0);
                let idf_score = if self.total_documents > 0.0 {
                    (self.total_documents / frequency).ln()
                } else {
                    1.0
                };
                total_score += idf_score;
                term_count += 1;
            }
        }

        if term_count > 0 {
            total_score / term_count as f32
        } else {
            0.0
        }
    }

    fn compute_title_boost(&self, result: &crate::vector_db_trait::SearchResult, query_terms: &[String]) -> f32 {
        // Extract title from metadata or use first line as title
        let title = result.document_title.clone();

        let title_lower = title.to_lowercase();
        let mut boost: f32 = 0.0;

        for term in query_terms {
            let term_lower = term.to_lowercase();
            if title_lower.contains(&term_lower) {
                boost += 0.2; // Each matching term in title adds 0.2 boost
            }
        }

        boost.min(1.0) // Cap at 1.0
    }

    fn compute_freshness_score(&self, result: &crate::vector_db_trait::SearchResult) -> f32 {
        // Placeholder implementation - could use actual file modification times
        // For now, assume newer documents have higher paths/IDs
        let path = &result.document_id; // Use document_id as path proxy

        // Simple heuristic: newer documentation patterns
        if path.contains("2024") || path.contains("2025") || path.contains("latest") {
            0.8
        } else if path.contains("2023") {
            0.6
        } else if path.contains("deprecated") || path.contains("old") {
            0.2
        } else {
            0.5 // Default neutral score
        }
    }

    fn compute_length_penalty(&self, content: &str) -> f32 {
        let length = content.len();
        if length > self.config.length_penalty_threshold {
            let excess = length - self.config.length_penalty_threshold;
            (excess as f32 / 1000.0).min(0.5) // Max penalty of 0.5
        } else {
            0.0
        }
    }

    fn generate_enhanced_snippet(&self, content: &str, query_terms: &[String], enhanced_terms: &[String]) -> String {
        let mut best_snippet = String::new();
        let mut best_score = 0;
        let snippet_length = 200;
        let context_window = 100;

        let content_lower = content.to_lowercase();
        let all_terms: Vec<String> = query_terms.iter()
            .chain(enhanced_terms.iter())
            .map(|t| t.to_lowercase())
            .collect();

        // Find the best position that contains the most query terms
        let words: Vec<&str> = content.split_whitespace().collect();
        
        for start in 0..words.len().saturating_sub(20) {
            let end = (start + 30).min(words.len());
            let window_text = words[start..end].join(" ");
            let window_lower = window_text.to_lowercase();
            
            let mut score = 0;
            for term in &all_terms {
                if window_lower.contains(term) {
                    score += if query_terms.iter().any(|qt| qt.to_lowercase() == *term) { 3 } else { 1 };
                }
            }
            
            if score > best_score {
                best_score = score;
                // Expand context around the best window
                let context_start = start.saturating_sub(context_window / 20);
                let context_end = (end + context_window / 20).min(words.len());
                best_snippet = words[context_start..context_end].join(" ");
                
                if best_snippet.len() > snippet_length {
                    best_snippet.truncate(snippet_length);
                    if let Some(last_space) = best_snippet.rfind(' ') {
                        best_snippet.truncate(last_space);
                    }
                    best_snippet.push_str("...");
                }
            }
        }

        if best_snippet.is_empty() && !content.is_empty() {
            // Fallback to first snippet_length characters
            if content.len() > snippet_length {
                best_snippet = content.chars().take(snippet_length).collect::<String>();
                if let Some(last_space) = best_snippet.rfind(' ') {
                    best_snippet.truncate(last_space);
                }
                best_snippet.push_str("...");
            } else {
                best_snippet = content.to_string();
            }
        }

        // Highlight matching terms
        self.highlight_terms_in_snippet(best_snippet, &all_terms)
    }

    fn highlight_terms_in_snippet(&self, snippet: String, terms: &[String]) -> String {
        let mut highlighted = snippet;
        
        // Sort terms by length (longest first) to avoid partial replacements
        let mut sorted_terms = terms.to_vec();
        sorted_terms.sort_by(|a, b| b.len().cmp(&a.len()));
        
        for term in sorted_terms {
            if term.len() < 2 { continue; } // Skip very short terms
            
            // Use regex for case-insensitive whole word matching
            let pattern = format!(r"(?i)\b{}\b", regex::escape(&term));
            if let Ok(re) = regex::Regex::new(&pattern) {
                highlighted = re.replace_all(&highlighted, |caps: &regex::Captures| {
                    format!("**{}**", &caps[0])
                }).to_string();
            }
        }
        
        highlighted
    }

    pub fn update_document_frequencies(&mut self, frequencies: HashMap<String, f32>, total_docs: f32) {
        self.document_frequencies = frequencies;
        self.total_documents = total_docs;
    }

    pub fn get_ranking_config(&self) -> &RankingConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_result_ranking() {
        let config = RankingConfig::default();
        let ranker = ResultRanker::new(config);
        
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), serde_json::Value::String("API Documentation".to_string()));
        
        // Need to convert to the RankedResult format
        let id = "test-1".to_string();
        
        // Create metadata map for compatibility
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), serde_json::Value::String(id.clone()));
        
        let ranked_result = RankedResult {
            id: id.clone(),
            content: "This is API documentation for search endpoints".to_string(),
            metadata,
            vector_similarity: 0.85,
            final_score: 0.85, // Placeholder
            ranking_signals: RankingSignals {
                vector_similarity: 0.85,
                document_frequency_score: 0.0,
                title_match_boost: 0.0,
                freshness_score: 0.5,
                length_penalty: 0.0,
                combined_score: 0.85,
            },
            snippet: "This is **API** documentation for **search** endpoints".to_string(),
        };

        let ranked_results = vec![ranked_result];
        
        assert_eq!(ranked_results.len(), 1);
        assert!(ranked_results[0].final_score > 0.0);
        assert!(ranked_results[0].snippet.contains("**"));
    }

    #[test]
    fn test_snippet_generation() {
        let config = RankingConfig::default();
        let ranker = ResultRanker::new(config);
        
        let content = "Lorem ipsum dolor sit amet. This contains API documentation for search functionality. More content follows here.";
        let query_terms = vec!["api".to_string()];
        let enhanced_terms = vec!["search".to_string()];
        
        let snippet = ranker.generate_enhanced_snippet(content, &query_terms, &enhanced_terms);
        
        assert!(snippet.contains("**API**") || snippet.contains("**api**"));
        assert!(snippet.contains("**search**"));
    }
}
