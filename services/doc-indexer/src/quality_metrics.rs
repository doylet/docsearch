use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tracing::{info, debug, warn};

use crate::chunking::{ChunkQuality, ChunkingConfig};

/// Aggregated quality metrics for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentQualityMetrics {
    pub doc_id: String,
    pub timestamp: DateTime<Utc>,
    pub total_chunks: usize,
    pub avg_coherence: f32,
    pub avg_completeness: f32,
    pub avg_size_score: f32,
    pub avg_context_preservation: f32,
    pub overall_quality: f32,
    pub chunk_size_distribution: ChunkSizeDistribution,
    pub strategy_used: String,
}

/// Distribution of chunk sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSizeDistribution {
    pub min_size: usize,
    pub max_size: usize,
    pub avg_size: f32,
    pub median_size: usize,
    pub std_dev: f32,
}

/// Individual chunk quality record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkQualityRecord {
    pub chunk_id: String,
    pub doc_id: String,
    pub timestamp: DateTime<Utc>,
    pub quality: ChunkQuality,
    pub chunk_size: usize,
    pub position_in_doc: usize,
    pub heading_context: Vec<String>,
}

/// Quality metrics collector and analyzer
#[derive(Debug)]
pub struct QualityMetricsCollector {
    /// Individual chunk quality records
    chunk_records: HashMap<String, ChunkQualityRecord>,
    /// Aggregated document metrics
    document_metrics: HashMap<String, DocumentQualityMetrics>,
    /// Configuration for quality evaluation
    config: ChunkingConfig,
}

impl QualityMetricsCollector {
    pub fn new(config: ChunkingConfig) -> Self {
        Self {
            chunk_records: HashMap::new(),
            document_metrics: HashMap::new(),
            config,
        }
    }

    /// Record quality metrics for chunks from a document
    pub fn record_document_chunks(
        &mut self,
        doc_id: &str,
        chunks: &[(String, String, Vec<String>)], // (chunk_id, content, heading_context)
    ) -> Result<DocumentQualityMetrics> {
        let timestamp = Utc::now();
        let mut chunk_qualities = Vec::new();
        let mut chunk_sizes = Vec::new();

        // Evaluate each chunk
        for (position, (chunk_id, content, heading_context)) in chunks.iter().enumerate() {
            let quality = ChunkQuality::evaluate(content, &self.config, heading_context);
            let chunk_size = content.len();

            // Record individual chunk quality
            let record = ChunkQualityRecord {
                chunk_id: chunk_id.clone(),
                doc_id: doc_id.to_string(),
                timestamp,
                quality: quality.clone(),
                chunk_size,
                position_in_doc: position,
                heading_context: heading_context.clone(),
            };

            self.chunk_records.insert(chunk_id.clone(), record);
            chunk_qualities.push(quality.clone());
            chunk_sizes.push(chunk_size);

            debug!(
                chunk_id = %chunk_id,
                coherence = quality.coherence,
                completeness = quality.completeness,
                size_score = quality.size_score,
                context_preservation = quality.context_preservation,
                overall = quality.overall_score(),
                "Chunk quality evaluated"
            );
        }

        // Calculate aggregated metrics
        let doc_metrics = self.calculate_document_metrics(
            doc_id,
            timestamp,
            &chunk_qualities,
            &chunk_sizes,
        )?;

        // Log quality summary
        info!(
            doc_id = %doc_id,
            total_chunks = doc_metrics.total_chunks,
            overall_quality = doc_metrics.overall_quality,
            avg_coherence = doc_metrics.avg_coherence,
            avg_size = doc_metrics.chunk_size_distribution.avg_size,
            strategy = %doc_metrics.strategy_used,
            "Document chunking quality metrics recorded"
        );

        // Check for quality concerns
        if doc_metrics.overall_quality < 0.6 {
            warn!(
                doc_id = %doc_id,
                quality = doc_metrics.overall_quality,
                "Low document chunking quality detected"
            );
        }

        self.document_metrics.insert(doc_id.to_string(), doc_metrics.clone());
        Ok(doc_metrics)
    }

    /// Calculate aggregated metrics for a document
    fn calculate_document_metrics(
        &self,
        doc_id: &str,
        timestamp: DateTime<Utc>,
        qualities: &[ChunkQuality],
        sizes: &[usize],
    ) -> Result<DocumentQualityMetrics> {
        if qualities.is_empty() {
            return Err(anyhow::anyhow!("No chunks to calculate metrics for"));
        }

        let total_chunks = qualities.len();
        
        // Calculate averages
        let avg_coherence = qualities.iter().map(|q| q.coherence).sum::<f32>() / total_chunks as f32;
        let avg_completeness = qualities.iter().map(|q| q.completeness).sum::<f32>() / total_chunks as f32;
        let avg_size_score = qualities.iter().map(|q| q.size_score).sum::<f32>() / total_chunks as f32;
        let avg_context_preservation = qualities.iter().map(|q| q.context_preservation).sum::<f32>() / total_chunks as f32;
        let overall_quality = qualities.iter().map(|q| q.overall_score()).sum::<f32>() / total_chunks as f32;

        // Calculate size distribution
        let chunk_size_distribution = self.calculate_size_distribution(sizes)?;

        Ok(DocumentQualityMetrics {
            doc_id: doc_id.to_string(),
            timestamp,
            total_chunks,
            avg_coherence,
            avg_completeness,
            avg_size_score,
            avg_context_preservation,
            overall_quality,
            chunk_size_distribution,
            strategy_used: format!("{:?}", self.config.strategy),
        })
    }

    /// Calculate chunk size distribution statistics
    fn calculate_size_distribution(&self, sizes: &[usize]) -> Result<ChunkSizeDistribution> {
        if sizes.is_empty() {
            return Err(anyhow::anyhow!("No sizes to calculate distribution for"));
        }

        let min_size = *sizes.iter().min().unwrap();
        let max_size = *sizes.iter().max().unwrap();
        let avg_size = sizes.iter().sum::<usize>() as f32 / sizes.len() as f32;

        // Calculate median
        let mut sorted_sizes = sizes.to_vec();
        sorted_sizes.sort_unstable();
        let median_size = if sorted_sizes.len() % 2 == 0 {
            (sorted_sizes[sorted_sizes.len() / 2 - 1] + sorted_sizes[sorted_sizes.len() / 2]) / 2
        } else {
            sorted_sizes[sorted_sizes.len() / 2]
        };

        // Calculate standard deviation
        let variance = sizes.iter()
            .map(|&size| {
                let diff = size as f32 - avg_size;
                diff * diff
            })
            .sum::<f32>() / sizes.len() as f32;
        let std_dev = variance.sqrt();

        Ok(ChunkSizeDistribution {
            min_size,
            max_size,
            avg_size,
            median_size,
            std_dev,
        })
    }

    /// Get quality metrics for a specific document
    pub fn get_document_metrics(&self, doc_id: &str) -> Option<&DocumentQualityMetrics> {
        self.document_metrics.get(doc_id)
    }

    /// Get quality records for chunks in a document
    pub fn get_chunk_records(&self, doc_id: &str) -> Vec<&ChunkQualityRecord> {
        self.chunk_records
            .values()
            .filter(|record| record.doc_id == doc_id)
            .collect()
    }

    /// Get overall quality statistics across all documents
    pub fn get_overall_statistics(&self) -> QualityStatistics {
        let doc_metrics: Vec<&DocumentQualityMetrics> = self.document_metrics.values().collect();
        
        if doc_metrics.is_empty() {
            return QualityStatistics::default();
        }

        let total_documents = doc_metrics.len();
        let total_chunks: usize = doc_metrics.iter().map(|m| m.total_chunks).sum();
        
        let avg_quality = doc_metrics.iter().map(|m| m.overall_quality).sum::<f32>() / total_documents as f32;
        let avg_chunks_per_doc = total_chunks as f32 / total_documents as f32;
        
        let strategies: HashMap<String, usize> = doc_metrics.iter()
            .fold(HashMap::new(), |mut acc, m| {
                *acc.entry(m.strategy_used.clone()).or_insert(0) += 1;
                acc
            });

        QualityStatistics {
            total_documents,
            total_chunks,
            avg_quality,
            avg_chunks_per_doc,
            strategy_distribution: strategies,
        }
    }
}

/// Overall quality statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStatistics {
    pub total_documents: usize,
    pub total_chunks: usize,
    pub avg_quality: f32,
    pub avg_chunks_per_doc: f32,
    pub strategy_distribution: HashMap<String, usize>,
}

impl Default for QualityStatistics {
    fn default() -> Self {
        Self {
            total_documents: 0,
            total_chunks: 0,
            avg_quality: 0.0,
            avg_chunks_per_doc: 0.0,
            strategy_distribution: HashMap::new(),
        }
    }
}
