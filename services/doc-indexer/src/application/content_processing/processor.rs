/// SOLID-compliant content processor
/// 
/// This processor follows all SOLID principles:
/// - SRP: Single responsibility for orchestrating content processing
/// - OCP: Open for extension via handler registry
/// - LSP: Handlers are substitutable via ContentHandler trait
/// - ISP: Focused interfaces for specific concerns
/// - DIP: Depends on abstractions (traits) not concretions

use std::path::Path;
use zero_latency_core::Result;

use super::{ContentType, ContentTypeDetector, ContentProcessorRegistry};

/// Main content processor that orchestrates type detection and processing
/// 
/// This follows SRP by focusing on orchestration rather than implementation details
#[derive(Clone)]
pub struct ContentProcessor {
    registry: ContentProcessorRegistry,
}

impl ContentProcessor {
    /// Create a new content processor with default configuration
    pub fn new() -> Self {
        Self {
            registry: ContentProcessorRegistry::new(),
        }
    }

    /// Create a content processor with a custom registry
    /// 
    /// This follows DIP by depending on abstractions
    pub fn with_registry(registry: ContentProcessorRegistry) -> Self {
        Self { registry }
    }

    /// Detect content type from file path and content
    /// 
    /// Delegates to specialized detector (SRP)
    pub fn detect_content_type(&self, path: &Path, content: &str) -> ContentType {
        ContentTypeDetector::detect_content_type(path, content)
    }

    /// Check if content should be indexed based on its type
    /// 
    /// Delegates to content type (SRP)
    pub fn should_index(&self, content_type: &ContentType) -> bool {
        content_type.should_index()
    }

    /// Process content based on its type
    /// 
    /// Delegates to appropriate handler via registry (OCP, DIP)
    pub fn process_content(&self, content: &str, content_type: &ContentType) -> Result<String> {
        self.registry.process_content(content, content_type)
    }

    /// Complete processing pipeline: detect type, check indexability, and process
    pub fn process_document(&self, path: &Path, content: &str) -> Result<Option<String>> {
        let content_type = self.detect_content_type(path, content);
        
        if !self.should_index(&content_type) {
            return Ok(None);
        }

        let processed = self.process_content(content, &content_type)?;
        Ok(Some(processed))
    }
}

impl Default for ContentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_content_processor_pipeline() {
        let processor = ContentProcessor::new();
        
        // Test markdown processing
        let result = processor
            .process_document(&PathBuf::from("test.md"), "# Test\nContent")
            .unwrap();
        
        assert!(result.is_some());
        let processed = result.unwrap();
        assert!(processed.contains("Test"));
        assert!(processed.contains("Content"));
    }

    #[test]
    fn test_unsupported_content_filtering() {
        let processor = ContentProcessor::new();
        
        // Test that unknown types are filtered out
        let result = processor
            .process_document(&PathBuf::from("test.bin"), "binary content")
            .unwrap();
        
        // Should return None for non-indexable content
        assert!(result.is_none());
    }
}
