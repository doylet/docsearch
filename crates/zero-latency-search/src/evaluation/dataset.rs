use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zero_latency_core::{DocId, Result, ZeroLatencyError};

/// Relevance rating for a query-document pair
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelevanceRating {
    /// Not relevant (0)
    NotRelevant = 0,
    /// Somewhat relevant (1)
    SomewhatRelevant = 1,
    /// Highly relevant (2)
    HighlyRelevant = 2,
}

impl From<u8> for RelevanceRating {
    fn from(value: u8) -> Self {
        match value {
            0 => RelevanceRating::NotRelevant,
            1 => RelevanceRating::SomewhatRelevant,
            2 => RelevanceRating::HighlyRelevant,
            _ => RelevanceRating::NotRelevant,
        }
    }
}

impl From<RelevanceRating> for f64 {
    fn from(rating: RelevanceRating) -> Self {
        rating as u8 as f64
    }
}

/// A labeled query-document pair for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledExample {
    /// Unique identifier for this example
    pub id: String,
    /// The search query
    pub query: String,
    /// The document identifier
    pub doc_id: DocId,
    /// Human-assigned relevance rating
    pub relevance: RelevanceRating,
    /// Optional category for grouping examples
    pub category: Option<String>,
    /// Optional metadata for this example
    pub metadata: HashMap<String, String>,
}

impl LabeledExample {
    pub fn new(
        id: impl Into<String>,
        query: impl Into<String>,
        doc_id: DocId,
        relevance: RelevanceRating,
    ) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            doc_id,
            relevance,
            category: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
    
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Collection of labeled examples for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationDataset {
    /// Dataset name/version
    pub name: String,
    /// Description of the dataset
    pub description: String,
    /// Version or timestamp
    pub version: String,
    /// All labeled examples
    pub examples: Vec<LabeledExample>,
    /// Dataset-level metadata
    pub metadata: HashMap<String, String>,
}

impl EvaluationDataset {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            version: chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string(),
            examples: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn add_example(&mut self, example: LabeledExample) {
        self.examples.push(example);
    }
    
    pub fn add_examples(&mut self, examples: Vec<LabeledExample>) {
        self.examples.extend(examples);
    }
    
    /// Get all unique queries in the dataset
    pub fn get_queries(&self) -> Vec<String> {
        let mut queries: Vec<String> = self
            .examples
            .iter()
            .map(|ex| ex.query.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        queries.sort();
        queries
    }
    
    /// Get examples for a specific query
    pub fn get_examples_for_query(&self, query: &str) -> Vec<&LabeledExample> {
        self.examples
            .iter()
            .filter(|ex| ex.query == query)
            .collect()
    }
    
    /// Get examples by category
    pub fn get_examples_by_category(&self, category: &str) -> Vec<&LabeledExample> {
        self.examples
            .iter()
            .filter(|ex| ex.category.as_deref() == Some(category))
            .collect()
    }
    
    /// Load dataset from JSON file
    pub fn from_json_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            ZeroLatencyError::io(format!("Failed to read dataset file: {}", e))
        })?;
        
        let dataset: EvaluationDataset = serde_json::from_str(&content).map_err(|e| {
            ZeroLatencyError::configuration(format!("Failed to parse dataset JSON: {}", e))
        })?;
        
        Ok(dataset)
    }
    
    /// Save dataset to JSON file
    pub fn to_json_file(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            ZeroLatencyError::configuration(format!("Failed to serialize dataset: {}", e))
        })?;
        
        std::fs::write(path, content).map_err(|e| {
            ZeroLatencyError::io(format!("Failed to write dataset file: {}", e))
        })?;
        
        Ok(())
    }
    
    /// Create a sample dataset for testing
    pub fn create_sample() -> Self {
        let mut dataset = Self::new(
            "sample_evaluation_dataset",
            "Sample dataset for testing search quality metrics",
        );
        
        // Add some sample examples
        let examples = vec![
            LabeledExample::new(
                "ex1",
                "rust programming language",
                DocId::new("docs", "rust-lang-intro", 1),
                RelevanceRating::HighlyRelevant,
            ).with_category("programming"),
            
            LabeledExample::new(
                "ex2", 
                "rust programming language",
                DocId::new("docs", "javascript-guide", 1),
                RelevanceRating::NotRelevant,
            ).with_category("programming"),
            
            LabeledExample::new(
                "ex3",
                "vector database search",
                DocId::new("docs", "qdrant-setup", 1),
                RelevanceRating::HighlyRelevant,
            ).with_category("database"),
            
            LabeledExample::new(
                "ex4",
                "vector database search", 
                DocId::new("docs", "mysql-tutorial", 1),
                RelevanceRating::SomewhatRelevant,
            ).with_category("database"),
        ];
        
        dataset.add_examples(examples);
        dataset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_dataset_creation() {
        let dataset = EvaluationDataset::create_sample();
        assert_eq!(dataset.name, "sample_evaluation_dataset");
        assert_eq!(dataset.examples.len(), 4);
        
        let queries = dataset.get_queries();
        assert_eq!(queries.len(), 2);
        assert!(queries.contains(&"rust programming language".to_string()));
        assert!(queries.contains(&"vector database search".to_string()));
    }

    #[test]
    fn test_query_filtering() {
        let dataset = EvaluationDataset::create_sample();
        let rust_examples = dataset.get_examples_for_query("rust programming language");
        assert_eq!(rust_examples.len(), 2);
        
        let programming_examples = dataset.get_examples_by_category("programming");
        assert_eq!(programming_examples.len(), 2);
    }

    #[test]
    fn test_json_serialization() {
        let dataset = EvaluationDataset::create_sample();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        
        dataset.to_json_file(path).unwrap();
        let loaded_dataset = EvaluationDataset::from_json_file(path).unwrap();
        
        assert_eq!(dataset.name, loaded_dataset.name);
        assert_eq!(dataset.examples.len(), loaded_dataset.examples.len());
    }

    #[test]
    fn test_relevance_rating_conversion() {
        assert_eq!(RelevanceRating::from(0), RelevanceRating::NotRelevant);
        assert_eq!(RelevanceRating::from(1), RelevanceRating::SomewhatRelevant);
        assert_eq!(RelevanceRating::from(2), RelevanceRating::HighlyRelevant);
        assert_eq!(RelevanceRating::from(99), RelevanceRating::NotRelevant); // Default
        
        assert_eq!(f64::from(RelevanceRating::NotRelevant), 0.0);
        assert_eq!(f64::from(RelevanceRating::SomewhatRelevant), 1.0);
        assert_eq!(f64::from(RelevanceRating::HighlyRelevant), 2.0);
    }
}
