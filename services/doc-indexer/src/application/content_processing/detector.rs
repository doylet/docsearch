/// Content type detection service
/// 
/// Follows SRP by having a single responsibility: determining content type
/// from file path and content analysis

use std::path::Path;
use super::ContentType;

/// Service responsible for detecting content types
/// 
/// This follows SRP by focusing solely on content type detection
pub struct ContentTypeDetector;

impl ContentTypeDetector {
    /// Detect content type from file extension and content
    pub fn detect_content_type(path: &Path, content: &str) -> ContentType {
        // First try to detect by file extension
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "md" | "markdown" => return ContentType::Markdown,
                "txt" => return ContentType::PlainText,
                "html" | "htm" => return ContentType::Html,
                "rst" => return ContentType::RestructuredText,
                "adoc" | "asciidoc" => return ContentType::AsciiDoc,
                "org" => return ContentType::OrgMode,
                "json" => return ContentType::Json,
                "yaml" | "yml" => return ContentType::Yaml,
                "toml" => return ContentType::Toml,
                "rs" => return ContentType::Rust,
                "js" | "ts" | "jsx" | "tsx" => return ContentType::JavaScript,
                "py" => return ContentType::Python,
                "sh" | "bash" | "zsh" | "fish" => return ContentType::Shell,
                "conf" | "config" | "cfg" | "ini" => return ContentType::Config,
                // Binary and unknown extensions
                "bin" | "exe" | "dll" | "so" | "dylib" | "o" | "obj" => return ContentType::Unknown,
                _ => {}
            }
        }

        // Fallback to content-based detection
        Self::detect_by_content(content)
    }

    /// Detect content type by analyzing content
    fn detect_by_content(content: &str) -> ContentType {
        let content_lower = content.to_lowercase();
        
        // Check for binary content (non-UTF8 or binary indicators)
        if content.contains('\0') || content_lower.contains("binary content") {
            return ContentType::Unknown;
        }
        
        // Check for HTML
        if content_lower.contains("<html") || content_lower.contains("<!doctype html") {
            return ContentType::Html;
        }
        
        // Check for JSON
        if content.trim().starts_with('{') && content.trim().ends_with('}') {
            return ContentType::Json;
        }
        
        // Check for Markdown (headers)
        if content.lines().any(|line| line.trim().starts_with('#')) {
            return ContentType::Markdown;
        }

        // Default to plain text for text content
        ContentType::PlainText
    }
}
