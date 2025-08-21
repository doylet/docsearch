use anyhow::{Context, Result};
use chrono::Utc;

use crate::chunking::{ChunkingConfig, ChunkingStrategy};
use crate::document::{DocumentChunk, ChunkType};

/// Advanced chunking processor that implements multiple strategies
pub struct AdvancedChunker {
    config: ChunkingConfig,
}

/// Represents a markdown structural element during parsing
#[derive(Debug, Clone)]
struct StructuralElement {
    content: String,
    element_type: ElementType,
    heading_level: Option<u8>,
    start_byte: usize,
    end_byte: usize,
    line_start: usize,
    line_end: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum ElementType {
    Heading(u8),
    Paragraph,
    CodeBlock,
    List,
    Table,
    Separator,
}

impl AdvancedChunker {
    pub fn new(config: ChunkingConfig) -> Result<Self> {
        config.validate()
            .context("Invalid chunking configuration")?;
        
        Ok(Self { config })
    }

    /// Main entry point for chunking a document
    pub fn chunk_document(
        &self,
        content: &str,
        doc_id: &str,
    ) -> Result<Vec<DocumentChunk>> {
        // Parse document structure
        let elements = self.parse_document_structure(content)?;
        
        // Apply chunking strategy
        let chunks = match self.config.strategy {
            ChunkingStrategy::ByHeading => self.chunk_by_heading(&elements, doc_id)?,
            ChunkingStrategy::BySize => self.chunk_by_size(&elements, doc_id)?,
            ChunkingStrategy::Hybrid => self.chunk_hybrid(&elements, doc_id)?,
            ChunkingStrategy::Semantic => self.chunk_semantic(&elements, doc_id)?,
        };

        // Post-process chunks for quality and consistency
        let processed_chunks = self.post_process_chunks(chunks)?;
        
        Ok(processed_chunks)
    }

    /// Parse document into structural elements
    fn parse_document_structure(&self, content: &str) -> Result<Vec<StructuralElement>> {
        let mut elements = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_byte_offset = 0;
        let mut in_code_block = false;
        let mut code_block_start = None;
        let mut current_element_start = 0;
        let mut current_element_lines = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_bytes = line.as_bytes().len() + 1; // +1 for newline
            
            // Handle code blocks
            if line.trim().starts_with("```") {
                if in_code_block {
                    // End code block
                    if let Some(start_idx) = code_block_start {
                        let code_content = lines[start_idx..=line_idx].join("\n");
                        let content_len = code_content.len();
                        elements.push(StructuralElement {
                            content: code_content,
                            element_type: ElementType::CodeBlock,
                            heading_level: None,
                            start_byte: current_byte_offset - content_len,
                            end_byte: current_byte_offset + line_bytes,
                            line_start: start_idx,
                            line_end: line_idx,
                        });
                    }
                    in_code_block = false;
                    code_block_start = None;
                } else {
                    // Start code block
                    in_code_block = true;
                    code_block_start = Some(line_idx);
                }
                current_byte_offset += line_bytes;
                continue;
            }

            if in_code_block {
                current_byte_offset += line_bytes;
                continue;
            }

            let trimmed = line.trim();
            
            // Handle headings
            if trimmed.starts_with('#') && !trimmed.starts_with("#[") {
                // Finalize previous element
                if !current_element_lines.is_empty() {
                    self.finalize_current_element(
                        &mut elements,
                        &current_element_lines,
                        current_element_start,
                        line_idx - 1,
                        current_byte_offset - line_bytes,
                    );
                    current_element_lines.clear();
                }

                // Process heading
                let level = trimmed.chars().take_while(|&c| c == '#').count() as u8;
                elements.push(StructuralElement {
                    content: line.to_string(),
                    element_type: ElementType::Heading(level),
                    heading_level: Some(level),
                    start_byte: current_byte_offset,
                    end_byte: current_byte_offset + line_bytes,
                    line_start: line_idx,
                    line_end: line_idx,
                });
                
                current_element_start = line_idx + 1;
            }
            // Handle tables
            else if trimmed.contains('|') && trimmed.len() > 2 {
                current_element_lines.push((line_idx, line.to_string()));
            }
            // Handle lists
            else if trimmed.starts_with('-') || trimmed.starts_with('*') || 
                    trimmed.starts_with('+') || 
                    (trimmed.len() > 2 && trimmed.chars().nth(0).unwrap_or(' ').is_ascii_digit() && trimmed.chars().nth(1) == Some('.')) {
                current_element_lines.push((line_idx, line.to_string()));
            }
            // Handle regular content
            else if !trimmed.is_empty() {
                current_element_lines.push((line_idx, line.to_string()));
            }
            // Handle separators (empty lines)
            else if current_element_lines.is_empty() {
                elements.push(StructuralElement {
                    content: String::new(),
                    element_type: ElementType::Separator,
                    heading_level: None,
                    start_byte: current_byte_offset,
                    end_byte: current_byte_offset + line_bytes,
                    line_start: line_idx,
                    line_end: line_idx,
                });
            } else {
                // Empty line within content - finalize current element
                self.finalize_current_element(
                    &mut elements,
                    &current_element_lines,
                    current_element_start,
                    line_idx - 1,
                    current_byte_offset,
                );
                current_element_lines.clear();
                current_element_start = line_idx + 1;
            }

            current_byte_offset += line_bytes;
        }

        // Finalize any remaining element
        if !current_element_lines.is_empty() {
            self.finalize_current_element(
                &mut elements,
                &current_element_lines,
                current_element_start,
                lines.len() - 1,
                current_byte_offset,
            );
        }

        Ok(elements)
    }

    fn finalize_current_element(
        &self,
        elements: &mut Vec<StructuralElement>,
        current_lines: &[(usize, String)],
        start_line: usize,
        end_line: usize,
        end_byte: usize,
    ) {
        if current_lines.is_empty() {
            return;
        }

        let content = current_lines.iter()
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let element_type = self.determine_element_type(&content);
        let start_byte = end_byte - content.len();

        elements.push(StructuralElement {
            content,
            element_type,
            heading_level: None,
            start_byte,
            end_byte,
            line_start: start_line,
            line_end: end_line,
        });
    }

    fn determine_element_type(&self, content: &str) -> ElementType {
        let trimmed = content.trim();
        
        // Check for table
        if trimmed.lines().any(|line| line.contains('|')) {
            return ElementType::Table;
        }
        
        // Check for list
        if trimmed.lines().any(|line| {
            let t = line.trim();
            t.starts_with('-') || t.starts_with('*') || t.starts_with('+') ||
            (t.len() > 2 && t.chars().nth(0).unwrap_or(' ').is_ascii_digit() && t.chars().nth(1) == Some('.'))
        }) {
            return ElementType::List;
        }
        
        ElementType::Paragraph
    }

    /// Chunk by markdown headings
    fn chunk_by_heading(
        &self,
        elements: &[StructuralElement],
        doc_id: &str,
    ) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();
        let mut heading_stack: Vec<String> = Vec::new();
        let mut current_section_elements: Vec<&StructuralElement> = Vec::new();

        for element in elements {
            match &element.element_type {
                ElementType::Heading(level) => {
                    // Finalize current section if we have content
                    if !current_section_elements.is_empty() {
                        let chunk = self.create_chunk_from_elements(
                            &current_section_elements,
                            &heading_stack,
                            doc_id,
                            chunks.len(),
                        )?;
                        chunks.push(chunk);
                        current_section_elements.clear();
                    }

                    // Update heading stack
                    let heading_text = element.content.trim_start_matches('#').trim().to_string();
                    
                    // Remove headings at same or deeper level
                    let current_level = *level as usize;
                    heading_stack.truncate(current_level.saturating_sub(1));
                    
                    // Add current heading if within configured depth
                    if *level <= self.config.max_heading_depth {
                        heading_stack.push(heading_text);
                    }

                    // Include heading in content if configured
                    if self.config.include_heading_context {
                        current_section_elements.push(element);
                    }
                }
                ElementType::CodeBlock => {
                    if self.config.preserve_code_blocks {
                        // Create separate chunk for code block
                        if !current_section_elements.is_empty() {
                            let chunk = self.create_chunk_from_elements(
                                &current_section_elements,
                                &heading_stack,
                                doc_id,
                                chunks.len(),
                            )?;
                            chunks.push(chunk);
                            current_section_elements.clear();
                        }
                        
                        let code_chunk = self.create_chunk_from_elements(
                            &[element],
                            &heading_stack,
                            doc_id,
                            chunks.len(),
                        )?;
                        chunks.push(code_chunk);
                    } else {
                        current_section_elements.push(element);
                    }
                }
                ElementType::Table => {
                    if self.config.preserve_tables {
                        // Similar to code blocks
                        if !current_section_elements.is_empty() {
                            let chunk = self.create_chunk_from_elements(
                                &current_section_elements,
                                &heading_stack,
                                doc_id,
                                chunks.len(),
                            )?;
                            chunks.push(chunk);
                            current_section_elements.clear();
                        }
                        
                        let table_chunk = self.create_chunk_from_elements(
                            &[element],
                            &heading_stack,
                            doc_id,
                            chunks.len(),
                        )?;
                        chunks.push(table_chunk);
                    } else {
                        current_section_elements.push(element);
                    }
                }
                ElementType::Separator => {
                    // Skip separators unless they break large sections
                    continue;
                }
                _ => {
                    current_section_elements.push(element);
                    
                    // Check if current section is getting too large
                    let current_size = self.calculate_elements_size(&current_section_elements);
                    if current_size > self.config.max_chunk_size {
                        // Split the section
                        let chunk = self.create_chunk_from_elements(
                            &current_section_elements,
                            &heading_stack,
                            doc_id,
                            chunks.len(),
                        )?;
                        chunks.push(chunk);
                        current_section_elements.clear();
                    }
                }
            }
        }

        // Finalize any remaining content
        if !current_section_elements.is_empty() {
            let chunk = self.create_chunk_from_elements(
                &current_section_elements,
                &heading_stack,
                doc_id,
                chunks.len(),
            )?;
            chunks.push(chunk);
        }

        Ok(chunks)
    }

    /// Chunk by size with smart boundaries
    fn chunk_by_size(
        &self,
        elements: &[StructuralElement],
        doc_id: &str,
    ) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();
        let mut current_elements: Vec<&StructuralElement> = Vec::new();
        let mut current_size = 0;
        let mut heading_context: Vec<String> = Vec::new();

        for element in elements {
            // Update heading context
            if let ElementType::Heading(level) = &element.element_type {
                let heading_text = element.content.trim_start_matches('#').trim().to_string();
                let current_level = *level as usize;
                heading_context.truncate(current_level.saturating_sub(1));
                heading_context.push(heading_text);
            }

            let element_size = element.content.len();
            
            // Check if adding this element would exceed max size
            if current_size + element_size > self.config.max_chunk_size && !current_elements.is_empty() {
                // Create chunk from current elements
                let chunk = self.create_chunk_from_elements(
                    &current_elements,
                    &heading_context,
                    doc_id,
                    chunks.len(),
                )?;
                chunks.push(chunk);
                
                // Start new chunk with overlap if configured
                current_elements.clear();
                current_size = 0;
                
                if self.config.chunk_overlap > 0 {
                    // Add overlap from previous chunk (simplified)
                    current_size = self.config.chunk_overlap.min(element_size);
                }
            }

            current_elements.push(element);
            current_size += element_size;
        }

        // Finalize remaining elements
        if !current_elements.is_empty() {
            let chunk = self.create_chunk_from_elements(
                &current_elements,
                &heading_context,
                doc_id,
                chunks.len(),
            )?;
            chunks.push(chunk);
        }

        Ok(chunks)
    }

    /// Hybrid chunking: primarily by headings, but split large sections
    fn chunk_hybrid(
        &self,
        elements: &[StructuralElement],
        doc_id: &str,
    ) -> Result<Vec<DocumentChunk>> {
        // Start with heading-based chunking
        let initial_chunks = self.chunk_by_heading(elements, doc_id)?;
        
        // Split any chunks that are too large
        let mut final_chunks = Vec::new();
        
        for chunk in initial_chunks {
            if chunk.content.len() <= self.config.max_chunk_size {
                final_chunks.push(chunk);
            } else {
                // Re-parse this chunk's content and apply size-based chunking
                let chunk_elements = self.parse_document_structure(&chunk.content)?;
                let sub_chunks = self.chunk_by_size(&chunk_elements, doc_id)?;
                
                // Update chunk indices and IDs
                for (_, mut sub_chunk) in sub_chunks.into_iter().enumerate() {
                    sub_chunk.chunk_id = format!("{}:{:05}", doc_id, final_chunks.len());
                    sub_chunk.chunk_index = final_chunks.len();
                    sub_chunk.h_path = chunk.h_path.clone(); // Preserve original heading context
                    final_chunks.push(sub_chunk);
                }
            }
        }

        Ok(final_chunks)
    }

    /// Semantic chunking (placeholder for future AI-based chunking)
    fn chunk_semantic(
        &self,
        elements: &[StructuralElement],
        doc_id: &str,
    ) -> Result<Vec<DocumentChunk>> {
        // For now, fall back to hybrid approach
        // Future: implement semantic similarity-based chunking
        self.chunk_hybrid(elements, doc_id)
    }

    /// Create a DocumentChunk from structural elements
    fn create_chunk_from_elements(
        &self,
        elements: &[&StructuralElement],
        heading_context: &[String],
        doc_id: &str,
        chunk_index: usize,
    ) -> Result<DocumentChunk> {
        if elements.is_empty() {
            return Err(anyhow::anyhow!("Cannot create chunk from empty elements"));
        }

        let content = elements.iter()
            .map(|e| e.content.as_str())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        let chunk_type = self.determine_chunk_type(elements);
        let start_byte = elements.first().map(|e| e.start_byte).unwrap_or(0);
        let end_byte = elements.last().map(|e| e.end_byte).unwrap_or(content.len());

        let chunk_id = format!("{}:{:05}", doc_id, chunk_index);
        let now = Utc::now();

        Ok(DocumentChunk {
            chunk_id,
            document_id: doc_id.to_string(),
            content,
            start_byte,
            end_byte,
            chunk_index,
            chunk_total: 0, // Will be updated in post-processing
            chunk_type,
            h_path: heading_context.to_vec(),
            created_at: now,
            updated_at: now,
        })
    }

    fn determine_chunk_type(&self, elements: &[&StructuralElement]) -> ChunkType {
        // Determine chunk type based on primary element type
        let element_types: Vec<_> = elements.iter().map(|e| &e.element_type).collect();
        
        if element_types.iter().any(|t| matches!(t, ElementType::CodeBlock)) {
            ChunkType::CodeBlock
        } else if element_types.iter().any(|t| matches!(t, ElementType::Table)) {
            ChunkType::Table
        } else if element_types.iter().any(|t| matches!(t, ElementType::List)) {
            ChunkType::List
        } else if element_types.iter().any(|t| matches!(t, ElementType::Heading(_))) {
            ChunkType::Heading
        } else {
            ChunkType::Paragraph
        }
    }

    fn calculate_elements_size(&self, elements: &[&StructuralElement]) -> usize {
        elements.iter().map(|e| e.content.len()).sum()
    }

    /// Post-process chunks for quality and consistency
    fn post_process_chunks(&self, mut chunks: Vec<DocumentChunk>) -> Result<Vec<DocumentChunk>> {
        let total_chunks = chunks.len();
        
        // Update chunk_total for all chunks
        for chunk in &mut chunks {
            chunk.chunk_total = total_chunks;
        }

        // Filter out chunks that are too small (unless they're the only chunk)
        if chunks.len() > 1 {
            chunks.retain(|chunk| chunk.content.len() >= self.config.min_chunk_size);
        }

        // Ensure we have at least one chunk
        if chunks.is_empty() {
            return Err(anyhow::anyhow!("No valid chunks created"));
        }

        Ok(chunks)
    }
}
