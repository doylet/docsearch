use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Configuration for document chunking strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Primary chunking strategy to use
    pub strategy: ChunkingStrategy,
    /// Maximum size of a chunk in characters
    pub max_chunk_size: usize,
    /// Minimum size of a chunk in characters (to avoid tiny chunks)
    pub min_chunk_size: usize,
    /// Number of characters to overlap between chunks for context
    pub chunk_overlap: usize,
    /// Whether to respect sentence boundaries when chunking
    pub respect_sentence_boundaries: bool,
    /// Whether to respect paragraph boundaries when chunking
    pub respect_paragraph_boundaries: bool,
    /// Maximum heading depth to use for chunking (1-6)
    pub max_heading_depth: u8,
    /// Whether to include heading context in chunk content
    pub include_heading_context: bool,
    /// Whether to preserve code blocks as single chunks
    pub preserve_code_blocks: bool,
    /// Whether to preserve tables as single chunks
    pub preserve_tables: bool,
}

/// Different strategies for chunking documents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChunkingStrategy {
    /// Chunk by markdown headings, creating hierarchical sections
    ByHeading,
    /// Chunk by character count with smart boundary detection
    BySize,
    /// Hybrid approach: primarily by headings, but split large sections by size
    Hybrid,
    /// Semantic chunking that preserves meaning and context
    Semantic,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            strategy: ChunkingStrategy::Hybrid,
            max_chunk_size: 2048,  // ~500 tokens for most LLMs
            min_chunk_size: 100,   // Avoid tiny chunks
            chunk_overlap: 200,    // 10% overlap for context
            respect_sentence_boundaries: true,
            respect_paragraph_boundaries: true,
            max_heading_depth: 4,  // H1-H4 levels
            include_heading_context: true,
            preserve_code_blocks: true,
            preserve_tables: true,
        }
    }
}

impl ChunkingConfig {
    /// Create configuration optimized for documentation
    pub fn for_documentation() -> Self {
        Self {
            strategy: ChunkingStrategy::ByHeading,
            max_chunk_size: 1536,  // Smaller chunks for precise search
            min_chunk_size: 50,
            chunk_overlap: 150,
            respect_sentence_boundaries: true,
            respect_paragraph_boundaries: true,
            max_heading_depth: 6,  // Use all heading levels
            include_heading_context: true,
            preserve_code_blocks: true,
            preserve_tables: true,
        }
    }

    /// Create configuration optimized for code documentation
    pub fn for_code_docs() -> Self {
        Self {
            strategy: ChunkingStrategy::Hybrid,
            max_chunk_size: 3072,  // Larger chunks for code context
            min_chunk_size: 200,
            chunk_overlap: 300,
            respect_sentence_boundaries: false, // Code doesn't follow sentence rules
            respect_paragraph_boundaries: true,
            max_heading_depth: 4,
            include_heading_context: true,
            preserve_code_blocks: true,
            preserve_tables: true,
        }
    }

    /// Create configuration optimized for API documentation
    pub fn for_api_docs() -> Self {
        Self {
            strategy: ChunkingStrategy::ByHeading,
            max_chunk_size: 1024,  // Small chunks for precise API search
            min_chunk_size: 100,
            chunk_overlap: 100,
            respect_sentence_boundaries: true,
            respect_paragraph_boundaries: true,
            max_heading_depth: 5,
            include_heading_context: true,
            preserve_code_blocks: true,
            preserve_tables: true,
        }
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        if self.max_chunk_size <= self.min_chunk_size {
            return Err(anyhow::anyhow!("max_chunk_size must be greater than min_chunk_size"));
        }
        
        if self.chunk_overlap >= self.max_chunk_size {
            return Err(anyhow::anyhow!("chunk_overlap must be less than max_chunk_size"));
        }
        
        if self.max_heading_depth == 0 || self.max_heading_depth > 6 {
            return Err(anyhow::anyhow!("max_heading_depth must be between 1 and 6"));
        }
        
        Ok(())
    }
}

/// Quality metrics for evaluating chunk effectiveness
#[derive(Debug, Clone)]
pub struct ChunkQuality {
    /// Coherence score (0.0-1.0) - how well the chunk maintains context
    pub coherence: f32,
    /// Completeness score (0.0-1.0) - whether chunk contains complete thoughts
    pub completeness: f32,
    /// Size score (0.0-1.0) - whether chunk is appropriately sized
    pub size_score: f32,
    /// Context preservation score (0.0-1.0) - how well headings/structure is preserved
    pub context_preservation: f32,
}

impl ChunkQuality {
    /// Calculate overall quality score
    pub fn overall_score(&self) -> f32 {
        (self.coherence + self.completeness + self.size_score + self.context_preservation) / 4.0
    }
    
    /// Evaluate chunk quality based on content and configuration
    pub fn evaluate(chunk_content: &str, config: &ChunkingConfig, heading_context: &[String]) -> Self {
        let coherence = Self::calculate_coherence(chunk_content);
        let completeness = Self::calculate_completeness(chunk_content, config);
        let size_score = Self::calculate_size_score(chunk_content, config);
        let context_preservation = Self::calculate_context_preservation(heading_context);
        
        Self {
            coherence,
            completeness,
            size_score,
            context_preservation,
        }
    }
    
    fn calculate_coherence(content: &str) -> f32 {
        // Basic coherence metrics
        let lines = content.lines().count();
        let has_complete_sentences = content.ends_with('.') || content.ends_with('!') || content.ends_with('?');
        let has_proper_structure = content.contains('\n') || lines == 1;
        
        let mut score: f32 = 0.0;
        if has_complete_sentences { score += 0.4; }
        if has_proper_structure { score += 0.3; }
        if lines > 0 { score += 0.3; }
        
        score.min(1.0)
    }
    
    fn calculate_completeness(content: &str, config: &ChunkingConfig) -> f32 {
        let len = content.len();
        
        // Penalize very short chunks
        if len < config.min_chunk_size {
            return (len as f32) / (config.min_chunk_size as f32);
        }
        
        // Reward chunks that end at natural boundaries
        let ends_naturally = content.ends_with('.')
            || content.ends_with('\n')
            || content.ends_with("```")
            || content.ends_with('|');
            
        if ends_naturally { 1.0 } else { 0.8 }
    }
    
    fn calculate_size_score(content: &str, config: &ChunkingConfig) -> f32 {
        let len = content.len();
        let target_size = (config.max_chunk_size + config.min_chunk_size) / 2;
        
        if len <= config.max_chunk_size && len >= config.min_chunk_size {
            // Within bounds - score based on how close to target
            let distance_from_target = ((len as f32) - (target_size as f32)).abs();
            let max_distance = (config.max_chunk_size - config.min_chunk_size) as f32 / 2.0;
            1.0 - (distance_from_target / max_distance).min(1.0)
        } else if len > config.max_chunk_size {
            // Too large - gradually decrease score
            0.5 * (config.max_chunk_size as f32 / len as f32)
        } else {
            // Too small - already handled in completeness
            0.6
        }
    }
    
    fn calculate_context_preservation(heading_context: &[String]) -> f32 {
        if heading_context.is_empty() {
            0.5 // Neutral score for no headings
        } else {
            // Score based on heading depth and clarity
            let depth_score = (heading_context.len() as f32 * 0.2).min(1.0);
            let clarity_score = if heading_context.iter().all(|h| !h.is_empty()) { 0.5 } else { 0.3 };
            depth_score + clarity_score
        }
    }
}
