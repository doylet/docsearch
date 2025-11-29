use serde::{Deserialize, Serialize};

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

    /// Create with default configuration
    pub fn default() -> Self {
        Self {
            config: FusionConfig::default(),
        }
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

    /// Fuse search results by combining duplicate documents and applying score fusion
    pub fn fuse_results(&self, results: Vec<crate::models::SearchResult>) -> Result<Vec<crate::models::SearchResult>, String> {
        use std::collections::HashMap;
        use crate::models::SearchResult;

        if results.is_empty() {
            return Ok(Vec::new());
        }

        // Group results by document ID to handle duplicates
        let mut doc_groups: HashMap<String, Vec<SearchResult>> = HashMap::new();

        for result in results {
            let doc_key = result.doc_id.to_index_key();
            doc_groups.entry(doc_key).or_insert_with(Vec::new).push(result);
        }

        let mut fused_results = Vec::new();

        for (_doc_key, doc_results) in doc_groups {
            if doc_results.len() == 1 {
                // Single result, no fusion needed
                fused_results.push(doc_results.into_iter().next().unwrap());
            } else {
                // Multiple results for same document, need to fuse scores
                let bm25_scores: Vec<f32> = doc_results.iter()
                    .filter_map(|r| r.scores.bm25_raw)
                    .collect();
                let vector_scores: Vec<f32> = doc_results.iter()
                    .filter_map(|r| r.scores.vector_raw)
                    .collect();

                // Use the first result as the base and update its scores
                let mut fused_result = doc_results.into_iter().next().unwrap();

                // Calculate new score breakdown
                let fused_scores = self.fuse_scores(
                    if bm25_scores.is_empty() { &[0.0] } else { &bm25_scores },
                    if vector_scores.is_empty() { &[0.0] } else { &vector_scores }
                )?;

                if let Some(new_scores) = fused_scores.into_iter().next() {
                    fused_result.scores = new_scores;

                    // Update final score for legacy compatibility
                    fused_result.final_score = zero_latency_core::values::Score::new(fused_result.scores.fused)
                        .unwrap_or_else(|_| zero_latency_core::values::Score::zero());

                    // Merge from_signals to reflect both engines contributed
                    let has_bm25 = fused_result.scores.bm25_raw.is_some();
                    let has_vector = fused_result.scores.vector_raw.is_some();

                    if has_bm25 && has_vector {
                        fused_result.from_signals = crate::fusion::FromSignals::hybrid();
                    } else if has_bm25 {
                        fused_result.from_signals = crate::fusion::FromSignals::bm25_only();
                    } else if has_vector {
                        fused_result.from_signals = crate::fusion::FromSignals::vector_only();
                    }
                }

                fused_results.push(fused_result);
            }
        }

        // Sort by fused score (descending)
        fused_results.sort_by(|a, b| {
            b.scores.fused.partial_cmp(&a.scores.fused).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(fused_results)
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
