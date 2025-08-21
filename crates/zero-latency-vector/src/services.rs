use crate::{traits::*, models::*};

/// Cosine similarity calculator
pub struct CosineSimilarityCalculator;

impl SimilarityCalculator for CosineSimilarityCalculator {
    fn calculate_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    fn batch_similarities(&self, query: &[f32], candidates: &[Vec<f32>]) -> Vec<f32> {
        candidates
            .iter()
            .map(|candidate| self.calculate_similarity(query, candidate))
            .collect()
    }

    fn metric(&self) -> SimilarityMetric {
        SimilarityMetric::Cosine
    }
}
