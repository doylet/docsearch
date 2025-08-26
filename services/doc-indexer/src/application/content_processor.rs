/// Content processing and type detection for different file formats
///
/// This module provides content type detection and appropriate processing
/// for different file formats to extract meaningful semantic content.
use std::path::Path;
use zero_latency_core::Result;

/// Supported content types for indexing
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    /// Markdown files (.md, .markdown)
    Markdown,
    /// Plain text files (.txt)
    PlainText,
    /// HTML files (.html, .htm)
    Html,
    /// reStructuredText files (.rst)
    RestructuredText,
    /// AsciiDoc files (.adoc, .asciidoc)
    AsciiDoc,
    /// Org mode files (.org)
    OrgMode,
    /// JSON files (.json)
    Json,
    /// YAML files (.yaml, .yml)
    Yaml,
    /// TOML files (.toml)
    Toml,
    /// Rust source files (.rs)
    Rust,
    /// JavaScript files (.js, .ts)
    JavaScript,
    /// Python files (.py)
    Python,
    /// Shell scripts (.sh, .bash)
    Shell,
    /// Configuration files
    Config,
    /// Unknown or unsupported file type
    Unknown,
}

/// Content processor that handles different file types appropriately
pub struct ContentProcessor;

impl ContentProcessor {
    /// Detect content type from file extension and content
    pub fn detect_content_type(path: &Path, content: &str) -> ContentType {
        // First try to detect by file extension
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
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
                "sh" | "bash" | "zsh" => ContentType::Shell,
                "conf" | "config" | "ini" => ContentType::Config,
                _ => ContentType::Unknown,
            }
        } else {
            // Fallback to content analysis
            Self::detect_by_content(content)
        }
    }

    /// Detect content type by analyzing the content
    fn detect_by_content(content: &str) -> ContentType {
        let content_lower = content.to_lowercase();

        // Check for common patterns
        if content_lower.contains("<!doctype html") || content_lower.contains("<html") {
            ContentType::Html
        } else if content.starts_with("```") || content.contains("# ") || content.contains("## ") {
            ContentType::Markdown
        } else if content_lower.starts_with("{") && content_lower.ends_with("}") {
            ContentType::Json
        } else if content.starts_with("---") || content.contains(": ") {
            ContentType::Yaml
        } else {
            ContentType::PlainText
        }
    }

    /// Check if a file type should be indexed
    pub fn should_index(content_type: &ContentType) -> bool {
        matches!(
            content_type,
            ContentType::Markdown
                | ContentType::PlainText
                | ContentType::Html
                | ContentType::RestructuredText
                | ContentType::AsciiDoc
                | ContentType::OrgMode
                | ContentType::Json
                | ContentType::Yaml
                | ContentType::Toml
                | ContentType::Rust
                | ContentType::JavaScript
                | ContentType::Python
                | ContentType::Shell
                | ContentType::Config
        )
    }

    /// Process content based on its type to extract meaningful text
    pub fn process_content(content: &str, content_type: &ContentType) -> Result<String> {
        match content_type {
            ContentType::Html => Self::process_html(content),
            ContentType::Markdown => Self::process_markdown(content),
            ContentType::Json => Self::process_json(content),
            ContentType::Yaml => Self::process_yaml(content),
            ContentType::Toml => Self::process_toml(content),
            ContentType::Rust => Self::process_code(content, "rust"),
            ContentType::JavaScript => Self::process_code(content, "javascript"),
            ContentType::Python => Self::process_code(content, "python"),
            ContentType::Shell => Self::process_code(content, "shell"),
            _ => Ok(content.to_string()), // Plain text and others
        }
    }

    /// Process HTML content to extract text while preserving semantic structure
    fn process_html(content: &str) -> Result<String> {
        // For now, simple HTML processing - in production you'd use a proper HTML parser
        let mut processed = content
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<br />", "\n")
            .replace("</p>", "\n\n")
            .replace("</div>", "\n")
            .replace("</h1>", "\n\n")
            .replace("</h2>", "\n\n")
            .replace("</h3>", "\n\n")
            .replace("</h4>", "\n\n")
            .replace("</h5>", "\n\n")
            .replace("</h6>", "\n\n");

        // Remove HTML tags but preserve the text content
        processed = regex::Regex::new(r"<[^>]*>")
            .unwrap()
            .replace_all(&processed, "")
            .to_string();

        // Clean up whitespace
        processed = regex::Regex::new(r"\n\s*\n\s*\n")
            .unwrap()
            .replace_all(&processed, "\n\n")
            .to_string();

        Ok(processed.trim().to_string())
    }

    /// Process Markdown content to extract text while preserving structure
    fn process_markdown(content: &str) -> Result<String> {
        // Remove markdown syntax but preserve the semantic content
        let mut processed = content
            .lines()
            .map(|line| {
                // Remove markdown headers but keep the text
                if line.starts_with('#') {
                    line.trim_start_matches('#').trim().to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Remove markdown links but keep the text: [text](url) -> text
        processed = regex::Regex::new(r"\[([^\]]+)\]\([^)]+\)")
            .unwrap()
            .replace_all(&processed, "$1")
            .to_string();

        // Remove code blocks and inline code for cleaner text search
        processed = regex::Regex::new(r"```[^`]*```")
            .unwrap()
            .replace_all(&processed, "")
            .to_string();

        processed = regex::Regex::new(r"`[^`]+`")
            .unwrap()
            .replace_all(&processed, "")
            .to_string();

        Ok(processed)
    }

    /// Process JSON content to extract meaningful text
    fn process_json(content: &str) -> Result<String> {
        // Extract string values and keys for semantic search
        let lines: Vec<String> = content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('"') && trimmed.contains(':') {
                    // Extract key-value pairs
                    Some(trimmed.replace(['"', ',', ':', '{', '}'], " "))
                } else {
                    None
                }
            })
            .collect();

        Ok(lines.join("\n"))
    }

    /// Process YAML content to extract meaningful text
    fn process_yaml(content: &str) -> Result<String> {
        // Extract keys and string values
        let lines: Vec<String> = content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if !trimmed.starts_with('#') && trimmed.contains(':') {
                    Some(trimmed.replace(':', " "))
                } else {
                    None
                }
            })
            .collect();

        Ok(lines.join("\n"))
    }

    /// Process TOML content to extract meaningful text
    fn process_toml(content: &str) -> Result<String> {
        // Similar to YAML but with TOML syntax
        let lines: Vec<String> = content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if !trimmed.starts_with('#') && (trimmed.contains('=') || trimmed.starts_with('['))
                {
                    Some(trimmed.replace(['=', '[', ']'], " "))
                } else {
                    None
                }
            })
            .collect();

        Ok(lines.join("\n"))
    }

    /// Process source code to extract comments and meaningful identifiers
    fn process_code(content: &str, language: &str) -> Result<String> {
        let mut extracted = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Extract comments based on language
            match language {
                "rust" | "javascript" => {
                    if trimmed.starts_with("//")
                        || trimmed.starts_with("/*")
                        || trimmed.starts_with("*")
                    {
                        extracted.push(trimmed.trim_start_matches(['/', '*']).trim());
                    }
                }
                "python" | "shell" => {
                    if trimmed.starts_with('#') {
                        extracted.push(trimmed.trim_start_matches('#').trim());
                    }
                }
                _ => {
                    // Generic comment extraction
                    if trimmed.starts_with("//") || trimmed.starts_with('#') {
                        extracted.push(trimmed.trim_start_matches(['/', '#']).trim());
                    }
                }
            }
        }

        Ok(extracted.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_content_type_detection() {
        assert_eq!(
            ContentProcessor::detect_content_type(&PathBuf::from("test.md"), ""),
            ContentType::Markdown
        );

        assert_eq!(
            ContentProcessor::detect_content_type(&PathBuf::from("test.html"), ""),
            ContentType::Html
        );

        assert_eq!(
            ContentProcessor::detect_content_type(&PathBuf::from("test.rs"), ""),
            ContentType::Rust
        );
    }

    #[test]
    fn test_html_processing() {
        let html = "<html><body><h1>Title</h1><p>Paragraph content</p></body></html>";
        let processed = ContentProcessor::process_html(html).unwrap();
        assert!(processed.contains("Title"));
        assert!(processed.contains("Paragraph content"));
        assert!(!processed.contains("<h1>"));
    }
}
