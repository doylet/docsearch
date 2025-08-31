use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Score normalization methods for hybrid search fusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizationMethod {
    /// Min-max normalization: (score - min) / (max - min)
    MinMax,
    /// Z-score normalization with clamping to [0,1]: clamp((score - mean) / stddev + 0.5, 0, 1)
    ZScore,
}

/// Raw and normalized scores for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Raw BM25 score (if available)
    pub bm25_raw: Option<f32>,
    /// Raw vector similarity score (if available)
    pub vector_raw: Option<f32>,
    /// Normalized BM25 score [0,1]
    pub bm25_normalized: Option<f32>,
    /// Normalized vector score [0,1] 
    pub vector_normalized: Option<f32>,
    /// Final fused score [0,1]
    pub fused: f32,
    /// Normalization method used
    pub normalization_method: NormalizationMethod,
}

/// Score normalization utilities
pub struct ScoreNormalizer;

impl ScoreNormalizer {
    /// Normalize scores using min-max method
    pub fn min_max_normalize(scores: &[f32]) -> Vec<f32> {
        if scores.is_empty() {
            return Vec::new();
        }
        
        let min_score = scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_score = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max_score - min_score).abs() < f32::EPSILON {
            // All scores are the same, return 0.5 for all
            return vec![0.5; scores.len()];
        }
        
        scores
            .iter()
            .map(|&score| (score - min_score) / (max_score - min_score))
            .collect()
    }
    
    /// Normalize scores using z-score method with clamping
    pub fn z_score_normalize(scores: &[f32]) -> Vec<f32> {
        if scores.is_empty() {
            return Vec::new();
        }
        
        if scores.len() == 1 {
            return vec![0.5];
        }
        
        let mean = scores.iter().sum::<f32>() / scores.len() as f32;
        let variance = scores
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / scores.len() as f32;
        let std_dev = variance.sqrt();
        
        if std_dev < f32::EPSILON {
            // All scores are the same, return 0.5 for all
            return vec![0.5; scores.len()];
        }
        
        scores
            .iter()
            .map(|&score| {
                let z_score = (score - mean) / std_dev;
                // Shift and clamp to [0,1]
                (z_score + 3.0).max(0.0).min(6.0) / 6.0
            })
            .collect()
    }
}

/// Configuration for score fusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionConfig {
    /// Weight for BM25 scores
    pub bm25_weight: f32,
    /// Weight for vector scores 
    pub vector_weight: f32,
    /// Normalization method to use
    pub normalization_method: NormalizationMethod,
}

impl Default for FusionConfig {
    fn default() -> Self {
        Self {
            bm25_weight: 0.3,
            vector_weight: 0.7,
            normalization_method: NormalizationMethod::MinMax,
        }
    }
}

impl FusionConfig {
    /// Create a new fusion config with specified weights
    pub fn new(bm25_weight: f32, vector_weight: f32) -> Self {
        Self {
            bm25_weight,
            vector_weight,
            normalization_method: NormalizationMethod::MinMax,
        }
    }
    
    /// Set the normalization method
    pub fn with_normalization(mut self, method: NormalizationMethod) -> Self {
        self.normalization_method = method;
        self
    }
    
    /// Validate weights sum to 1.0 (approximately)
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.bm25_weight + self.vector_weight;
        if (sum - 1.0).abs() > 0.01 {
            return Err(format!("Weights must sum to 1.0, got {}", sum));
        }
        Ok(())
    }
}

/// Score fusion engine for combining BM25 and vector scores
pub struct ScoreFusion {
    config: FusionConfig,
}

impl ScoreFusion {
    /// Create a new score fusion engine
    pub fn new(config: FusionConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self { config })
    }
    
    /// Fuse BM25 and vector scores according to configuration
    pub fn fuse_scores(
        &self,
        bm25_scores: &[f32],
        vector_scores: &[f32],
    ) -> Result<Vec<ScoreBreakdown>, String> {
        // Validate input lengths
        let max_len = bm25_scores.len().max(vector_scores.len());
        if max_len == 0 {
            return Ok(Vec::new());
        }
        
        // Normalize scores based on configuration
        let bm25_normalized = if !bm25_scores.is_empty() {
            Some(match self.config.normalization_method {
                NormalizationMethod::MinMax => ScoreNormalizer::min_max_normalize(bm25_scores),
                NormalizationMethod::ZScore => ScoreNormalizer::z_score_normalize(bm25_scores),
            })
        } else {
            None
        };
        
        let vector_normalized = if !vector_scores.is_empty() {
            Some(match self.config.normalization_method {
                NormalizationMethod::MinMax => ScoreNormalizer::min_max_normalize(vector_scores),
                NormalizationMethod::ZScore => ScoreNormalizer::z_score_normalize(vector_scores),
            })
        } else {
            None
        };
        
        let mut results = Vec::new();
        
        for i in 0..max_len {
            let bm25_raw = bm25_scores.get(i).copied();
            let vector_raw = vector_scores.get(i).copied();
            
            let bm25_norm = bm25_normalized.as_ref().and_then(|n| n.get(i).copied());
            let vector_norm = vector_normalized.as_ref().and_then(|n| n.get(i).copied());
            
            // Calculate fused score using weighted combination
            let fused = match (bm25_norm, vector_norm) {
                (Some(bm25), Some(vector)) => {
                    self.config.bm25_weight * bm25 + self.config.vector_weight * vector
                }
                (Some(bm25), None) => bm25, // Only BM25 available
                (None, Some(vector)) => vector, // Only vector available
                (None, None) => 0.0, // No scores available
            };
            
            results.push(ScoreBreakdown {
                bm25_raw,
                vector_raw,
                bm25_normalized: bm25_norm,
                vector_normalized: vector_norm,
                fused,
                normalization_method: self.config.normalization_method.clone(),
            });
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_max_normalization() {
        let scores = vec![1.0, 3.0, 5.0, 2.0, 4.0];
        let normalized = ScoreNormalizer::min_max_normalize(&scores);
        
        assert_eq!(normalized[0], 0.0);  // min -> 0
        assert_eq!(normalized[2], 1.0);  // max -> 1
        assert!((normalized[1] - 0.5).abs() < f32::EPSILON); // middle
    }

    #[test]
    fn test_z_score_normalization() {
        let scores = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = ScoreNormalizer::z_score_normalize(&scores);
        
        // All normalized values should be in [0,1]
        for &score in &normalized {
            assert!(score >= 0.0 && score <= 1.0);
        }
    }

    #[test]
    fn test_score_fusion() {
        let config = FusionConfig::new(0.4, 0.6);
        let fusion = ScoreFusion::new(config).unwrap();
        
        let bm25_scores = vec![1.0, 2.0, 3.0];
        let vector_scores = vec![0.8, 0.9, 0.7];
        
        let results = fusion.fuse_scores(&bm25_scores, &vector_scores).unwrap();
        
        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(result.fused >= 0.0 && result.fused <= 1.0);
            assert!(result.bm25_raw.is_some());
            assert!(result.vector_raw.is_some());
        }
    }

    #[test]
    fn test_fusion_config_validation() {
        let valid_config = FusionConfig::new(0.3, 0.7);
        assert!(valid_config.validate().is_ok());
        
        let invalid_config = FusionConfig::new(0.3, 0.6);
        assert!(invalid_config.validate().is_err());
    }
}
