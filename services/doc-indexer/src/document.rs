use anyhow::Result;
use chrono::{DateTime, Utc};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub path: String,
    pub title: String,
    pub content: String,
    pub content_hash: String,
    pub metadata: DocumentMetadata,
    pub chunks: Vec<DocumentChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub file_type: String,
    pub size: u64,
    pub modified_at: DateTime<Utc>,
    pub section: String, // e.g., "adr", "model-host/artefacts"
    pub doc_type: String, // e.g., "blueprint", "adr", "whitepaper"
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub heading: Option<String>,
    pub chunk_type: ChunkType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Heading,
    Paragraph,
    CodeBlock,
    List,
    Table,
}

pub struct DocumentProcessor {}

impl DocumentProcessor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process_document(&self, path: &Path, content: &str) -> Result<Document> {
        let id = self.generate_document_id(path);
        let path_str = path.to_string_lossy().to_string();
        
        // Extract title from first heading or filename
        let title = self.extract_title(content, path);
        
        // Get file metadata
        let metadata = std::fs::metadata(path)?;
        let modified_at = metadata.modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let modified_at = DateTime::from_timestamp(modified_at as i64, 0)
            .unwrap_or_else(Utc::now);
        
        // Extract document metadata
        let file_metadata = std::fs::metadata(path)?;
        let doc_metadata = DocumentMetadata {
            file_type: "markdown".to_string(),
            size: file_metadata.len(),
            modified_at,
            section: "General".to_string(),
            doc_type: "markdown".to_string(),
            tags: vec![],
        };
        
        // Process content into chunks - simplified for now
        let chunks = vec![DocumentChunk {
            id: Uuid::new_v4().to_string(),
            document_id: id.clone(),
            content: content.to_string(),
            start_line: 1,
            end_line: content.lines().count(),
            chunk_type: ChunkType::Paragraph,
            heading: None,
        }];
        
        Ok(Document {
            id,
            path: path_str,
            title,
            content: content.to_string(),
            chunks,
            metadata: doc_metadata,
            content_hash: format!("{:x}", md5::compute(content)),
        })
    }

    pub fn generate_document_id(&self, path: &Path) -> String {
        let path_str = path.to_string_lossy();
        format!("{:x}", md5::compute(path_str.as_bytes()))
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
            modified_at,
            section,
            doc_type,
            tags,
        })
    }

    fn extract_section_from_path(&self, file_path: &std::path::Path) -> String {
        let components: Vec<_> = file_path.components()
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

    fn chunk_document(&self, content: &str, title: &str) -> Result<Vec<DocumentChunk>> {
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
                            id: Uuid::new_v4().to_string(),
                            document_id: "".to_string(), // Will be set later
                            content: chunk_content.trim().to_string(),
                            start_line: current_chunk_start + 1,
                            end_line: line_idx,
                            heading: current_heading.clone(),
                            chunk_type: ChunkType::Paragraph,
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
                    id: Uuid::new_v4().to_string(),
                    document_id: "".to_string(),
                    content: chunk_content.trim().to_string(),
                    start_line: current_chunk_start + 1,
                    end_line: lines.len(),
                    heading: current_heading,
                    chunk_type: ChunkType::Paragraph,
                });
            }
        }

        // If no chunks were created, create one with the entire content
        if chunks.is_empty() && !content.trim().is_empty() {
            chunks.push(DocumentChunk {
                id: Uuid::new_v4().to_string(),
                document_id: "".to_string(),
                content: content.trim().to_string(),
                start_line: 1,
                end_line: lines.len(),
                heading: Some(title.to_string()),
                chunk_type: ChunkType::Paragraph,
            });
        }

        Ok(chunks)
    }
}
