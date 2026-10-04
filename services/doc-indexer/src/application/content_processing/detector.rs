use super::ContentType;
/// Content type detection service
///
/// Follows SRP by having a single responsibility: determining content type
/// from file path and content analysis
use std::path::Path;

/// Service responsible for detecting content types
///
/// This follows SRP by focusing solely on content type detection
pub struct ContentTypeDetector;

impl ContentTypeDetector {
    /// Detect content type from file extension and content
    pub fn detect_content_type(path: &Path, content: &str) -> ContentType {
        // First try to detect by file extension, then fall back to the content
        Self::detect_from_path(path).unwrap_or_else(|| Self::detect_by_content(content))
    }

    /// Detect content type from the file extension alone (case-insensitive).
    ///
    /// Returns `None` when the extension is missing or not recognised.
    pub fn detect_from_path(path: &Path) -> Option<ContentType> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        let content_type = match extension.as_str() {
            "md" | "markdown" => ContentType::Markdown,
            "txt" => ContentType::PlainText,
            "html" | "htm" => ContentType::Html,
            "rst" => ContentType::RestructuredText,
            "adoc" | "asciidoc" => ContentType::AsciiDoc,
            "org" => ContentType::OrgMode,
            "json" => ContentType::Json,
            "yaml" | "yml" => ContentType::Yaml,
            "toml" => ContentType::Toml,
            "rs" => ContentType::Rust,
            "js" | "ts" | "jsx" | "tsx" => ContentType::JavaScript,
            "py" => ContentType::Python,
            "sh" | "bash" | "zsh" | "fish" => ContentType::Shell,
            "pdf" => ContentType::Pdf,
            "conf" | "config" | "cfg" | "ini" => ContentType::Config,
            // Binary and unknown extensions
            "bin" | "exe" | "dll" | "so" | "dylib" | "o" | "obj" => ContentType::Unknown,
            _ => return None,
        };
        Some(content_type)
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
