use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;
use crate::chunking::{ChunkingConfig};
use crate::advanced_chunker::AdvancedChunker;
use crate::quality_metrics::{QualityMetricsCollector, DocumentQualityMetrics};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub doc_id: String,        // SHA256 hash of absolute path - stable per file
    pub rev_id: String,        // xxHash of content - changes with content
    pub abs_path: String,      // Full absolute path
    pub rel_path: String,      // Relative path from docs root
    pub title: String,
    pub content: String,
    pub metadata: DocumentMetadata,
    pub chunks: Vec<DocumentChunk>,
    pub schema_version: u32,   // For future migrations
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub file_type: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub section: String,         // e.g., "adr", "model-host/artefacts"
    pub doc_type: String,        // e.g., "blueprint", "adr", "whitepaper"
    pub tags: Vec<String>,
    pub embedding_model: String, // Track which model was used
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub chunk_id: String,      // Format: "doc_id:NNNNN" for stable ordering
    pub document_id: String,   // References parent document
    pub content: String,
    pub start_byte: usize,     // Byte offset in original document
    pub end_byte: usize,       // End byte offset
    pub chunk_index: usize,    // Sequential index within document
    pub chunk_total: usize,    // Total chunks in document
    pub chunk_type: ChunkType,
    pub h_path: Vec<String>,   // Heading breadcrumb path
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Heading,
    Paragraph,
    CodeBlock,
    List,
    Table,
}

pub struct DocumentProcessor {
    docs_root: std::path::PathBuf,
    schema_version: u32,
    chunking_config: ChunkingConfig,
    chunker: AdvancedChunker,
    quality_collector: std::sync::Mutex<QualityMetricsCollector>,
}

const CURRENT_SCHEMA_VERSION: u32 = 1;

impl DocumentProcessor {
    pub fn new(docs_root: std::path::PathBuf) -> Result<Self> {
        let chunking_config = ChunkingConfig::for_documentation();
        let chunker = AdvancedChunker::new(chunking_config.clone())?;
        let quality_collector = std::sync::Mutex::new(QualityMetricsCollector::new(chunking_config.clone()));
        
        Ok(Self {
            docs_root,
            schema_version: CURRENT_SCHEMA_VERSION,
            chunking_config,
            chunker,
            quality_collector,
        })
    }

    pub fn with_chunking_config(docs_root: std::path::PathBuf, chunking_config: ChunkingConfig) -> Result<Self> {
        let chunker = AdvancedChunker::new(chunking_config.clone())?;
        let quality_collector = std::sync::Mutex::new(QualityMetricsCollector::new(chunking_config.clone()));
        
        Ok(Self {
            docs_root,
            schema_version: CURRENT_SCHEMA_VERSION,
            chunking_config,
            chunker,
            quality_collector,
        })
    }

    /// Generate stable document ID from absolute path using SHA256
    pub fn generate_doc_id(&self, abs_path: &Path) -> String {
        let mut hasher = Sha256::new();
        hasher.update(abs_path.to_string_lossy().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate content revision ID using xxHash (fast)
    pub fn generate_rev_id(&self, content: &str) -> String {
        format!("{:016x}", xxh3_64(content.as_bytes()))
    }

    /// Generate chunk ID in format "doc_id:NNNNN"
    pub fn generate_chunk_id(&self, doc_id: &str, chunk_index: usize) -> String {
        format!("{}:{:05}", doc_id, chunk_index)
    }

    /// Get relative path from docs root
    pub fn get_relative_path(&self, abs_path: &Path) -> Result<String> {
        let rel_path = abs_path.strip_prefix(&self.docs_root)
            .map_err(|_| anyhow::anyhow!("Path is not under docs root: {}", abs_path.display()))?;
        Ok(rel_path.to_string_lossy().to_string())
    }

    pub fn process_document(&self, abs_path: &Path, content: &str) -> Result<Document> {
        let doc_id = self.generate_doc_id(abs_path);
        let rev_id = self.generate_rev_id(content);
        let abs_path_str = abs_path.to_string_lossy().to_string();
        let rel_path = self.get_relative_path(abs_path)?;
        
        // Extract title from first heading or filename
        let title = self.extract_title(content, abs_path);
        
        // Get file metadata
        let file_metadata = std::fs::metadata(abs_path)?;
        let modified_at = file_metadata.modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let modified_at = DateTime::from_timestamp(modified_at as i64, 0)
            .unwrap_or_else(Utc::now);
        
        let created_at = file_metadata.created()
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
            .map(|s| DateTime::from_timestamp(s as i64, 0).unwrap_or_else(Utc::now))
            .unwrap_or_else(|_| modified_at);
        
        // Extract document metadata
        let doc_metadata = DocumentMetadata {
            file_type: "markdown".to_string(),
            size: file_metadata.len(),
            created_at,
            modified_at,
            section: self.extract_section_from_path(abs_path),
            doc_type: self.determine_doc_type(abs_path, content),
            tags: self.extract_tags(content),
            embedding_model: "gte-small".to_string(), // Will be configurable later
        };
        
        // Process content into chunks using advanced chunker
        let chunks = self.chunker.chunk_document(content, &doc_id)?;
        
        // Collect quality metrics for the chunks
        let chunk_data: Vec<(String, String, Vec<String>)> = chunks.iter()
            .map(|chunk| (chunk.chunk_id.clone(), chunk.content.clone(), chunk.h_path.clone()))
            .collect();
        
        if let Ok(mut collector) = self.quality_collector.lock() {
            if let Err(e) = collector.record_document_chunks(&doc_id, &chunk_data) {
                tracing::warn!(
                    doc_id = %doc_id,
                    error = %e,
                    "Failed to record chunk quality metrics"
                );
            }
        }
        
        Ok(Document {
            doc_id,
            rev_id,
            abs_path: abs_path_str,
            rel_path,
            title,
            content: content.to_string(),
            metadata: doc_metadata,
            chunks,
            schema_version: self.schema_version,
        })
    }

    /// Get quality metrics for a specific document
    pub fn get_document_quality_metrics(&self, doc_id: &str) -> Option<DocumentQualityMetrics> {
        if let Ok(collector) = self.quality_collector.lock() {
            collector.get_document_metrics(doc_id).cloned()
        } else {
            None
        }
    }

    /// Get overall quality statistics
    pub fn get_quality_statistics(&self) -> Option<crate::quality_metrics::QualityStatistics> {
        if let Ok(collector) = self.quality_collector.lock() {
            Some(collector.get_overall_statistics())
        } else {
            None
        }
    }

    /// Create simple chunks for now - will be enhanced in step 2
    fn create_simple_chunks(&self, content: &str, doc_id: &str) -> Result<Vec<DocumentChunk>> {
        let now = Utc::now();
        let content_bytes = content.as_bytes();
        let chunk_id = self.generate_chunk_id(doc_id, 0);
        
        let chunk = DocumentChunk {
            chunk_id,
            document_id: doc_id.to_string(),
            content: content.to_string(),
            start_byte: 0,
            end_byte: content_bytes.len(),
            chunk_index: 0,
            chunk_total: 1,
            chunk_type: ChunkType::Paragraph,
            h_path: vec![],
            created_at: now,
            updated_at: now,
        };
        
        Ok(vec![chunk])
    }

    pub fn generate_document_id(&self, path: &Path) -> String {
        // Legacy method for compatibility - delegates to generate_doc_id
        self.generate_doc_id(path)
    }

    fn extract_title(&self, content: &str, file_path: &std::path::Path) -> String {
        // Try to extract title from first H1 heading
        for line in content.lines().take(10) {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                return trimmed[2..].trim().to_string();
            }
        }

        // Fallback to filename without extension
        file_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    async fn extract_metadata(
        &self,
        file_path: &std::path::Path,
        content: &str,
    ) -> Result<DocumentMetadata> {
        let file_size = tokio::fs::metadata(file_path).await?.len();
        let modified_at = tokio::fs::metadata(file_path)
            .await?
            .modified()?
            .into();

        // Extract section from path (e.g., "adr", "model-host/artefacts")
        let section = self.extract_section_from_path(file_path);
        
        // Determine document type from filename and content
        let doc_type = self.determine_doc_type(file_path, content);
        
        // Extract tags from content
        let tags = self.extract_tags(content);

        Ok(DocumentMetadata {
            file_type: "markdown".to_string(),
            size: file_size,
            created_at: modified_at, // Use modified_at as fallback for created_at
            modified_at,
            section,
            doc_type,
            tags,
            embedding_model: "gte-small".to_string(),
        })
    }

    fn extract_section_from_path(&self, file_path: &std::path::Path) -> String {
        let relative_path = file_path.strip_prefix(&self.docs_root)
            .unwrap_or(file_path);
        
        let components: Vec<_> = relative_path.components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect();
        
        if components.len() > 1 {
            components[..components.len()-1].join("/")
        } else {
            "root".to_string()
        }
    }

    fn determine_doc_type(&self, file_path: &std::path::Path, content: &str) -> String {
        let filename = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        // Check filename patterns
        if filename.contains("adr") {
            return "adr".to_string();
        }
        if filename.contains("blueprint") {
            return "blueprint".to_string();
        }
        if filename.contains("whitepaper") {
            return "whitepaper".to_string();
        }
        if filename.contains("roadmap") {
            return "roadmap".to_string();
        }
        if filename.contains("review") {
            return "review".to_string();
        }
        if filename.contains("addendum") {
            return "addendum".to_string();
        }

        // Check content patterns
        if content.contains("# ADR-") || content.contains("## Decision") {
            return "adr".to_string();
        }
        if content.contains("Implementation Blueprint") {
            return "blueprint".to_string();
        }

        "document".to_string()
    }

    fn extract_tags(&self, content: &str) -> Vec<String> {
        let mut tags = Vec::new();

        // Extract from common patterns
        if content.contains("Rust") || content.contains("rust") {
            tags.push("rust".to_string());
        }
        if content.contains("MCP") {
            tags.push("mcp".to_string());
        }
        if content.contains("gRPC") || content.contains("grpc") {
            tags.push("grpc".to_string());
        }
        if content.contains("Policy") || content.contains("policy") {
            tags.push("policy".to_string());
        }
        if content.contains("Security") || content.contains("security") {
            tags.push("security".to_string());
        }
        if content.contains("Model Host") {
            tags.push("model-host".to_string());
        }
        if content.contains("Zero Latency") {
            tags.push("zero-latency".to_string());
        }

        tags.sort();
        tags.dedup();
        tags
    }

    fn chunk_document(&self, content: &str, title: &str, doc_id: &str) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        let mut current_chunk_start = 0;
        let mut current_heading: Option<String> = None;
        let mut in_code_block = false;

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            // Skip if we're in a code block
            if in_code_block {
                continue;
            }

            // Check for headings
            if trimmed.starts_with('#') && !trimmed.starts_with("#[") {
                // Finalize previous chunk if we have content
                if line_idx > current_chunk_start {
                    let chunk_content = lines[current_chunk_start..line_idx].join("\n");
                    if !chunk_content.trim().is_empty() {
                        chunks.push(DocumentChunk {
                            chunk_id: format!("{}:{:05}", doc_id, chunks.len()),
                            document_id: doc_id.to_string(),
                            content: chunk_content.trim().to_string(),
                            start_byte: 0, // TODO: Calculate actual byte offset
                            end_byte: 0,   // TODO: Calculate actual byte offset
                            chunk_index: chunks.len(),
                            chunk_total: 0, // Will be updated at the end
                            chunk_type: ChunkType::Paragraph,
                            h_path: current_heading.clone().map(|h| vec![h]).unwrap_or_default(),
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        });
                    }
                }

                // Start new chunk
                current_heading = Some(trimmed.trim_start_matches('#').trim().to_string());
                current_chunk_start = line_idx;
            }
        }

        // Add final chunk
        if current_chunk_start < lines.len() {
            let chunk_content = lines[current_chunk_start..].join("\n");
            if !chunk_content.trim().is_empty() {
                chunks.push(DocumentChunk {
                    chunk_id: format!("{}:{:05}", doc_id, chunks.len()),
                    document_id: doc_id.to_string(),
                    content: chunk_content.trim().to_string(),
                    start_byte: 0, // TODO: Calculate actual byte offset
                    end_byte: 0,   // TODO: Calculate actual byte offset
                    chunk_index: chunks.len(),
                    chunk_total: 0, // Will be updated at the end
                    chunk_type: ChunkType::Paragraph,
                    h_path: current_heading.clone().map(|h| vec![h]).unwrap_or_default(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });
            }
        }

        // If no chunks were created, create one with the entire content
        if chunks.is_empty() && !content.trim().is_empty() {
            chunks.push(DocumentChunk {
                chunk_id: format!("{}:{:05}", doc_id, 0),
                document_id: doc_id.to_string(),
                content: content.trim().to_string(),
                start_byte: 0,
                end_byte: content.len(),
                chunk_index: 0,
                chunk_total: 1,
                chunk_type: ChunkType::Paragraph,
                h_path: vec![title.to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        // Update chunk_total for all chunks
        let total_chunks = chunks.len();
        for chunk in &mut chunks {
            chunk.chunk_total = total_chunks;
        }

        Ok(chunks)
    }
}
