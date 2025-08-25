/// Content handler trait and implementations
/// 
/// This follows SOLID principles:
/// - SRP: Each handler has a single responsibility for one content type
/// - OCP: System is open for extension (new handlers) but closed for modification
/// - ISP: Focused interface with only necessary methods
/// - DIP: Depends on abstractions (trait) not concretions

use zero_latency_core::Result;
use super::ContentType;

/// Trait for handling specific content types
/// 
/// Each implementation handles processing for exactly one content type,
/// following the Single Responsibility Principle
pub trait ContentHandler: Send + Sync {
    /// Get the content type this handler processes
    fn content_type(&self) -> ContentType;
    
    /// Process content and extract meaningful text for indexing
    fn process(&self, content: &str) -> Result<String>;
    
    /// Check if this handler can process the given content type
    fn can_handle(&self, content_type: &ContentType) -> bool {
        &self.content_type() == content_type
    }
}

/// HTML content handler
pub struct HtmlHandler;

impl ContentHandler for HtmlHandler {
    fn content_type(&self) -> ContentType {
        ContentType::Html
    }
    
    fn process(&self, content: &str) -> Result<String> {
        // Process HTML content to extract text while preserving semantic structure
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
}

/// Markdown content handler
pub struct MarkdownHandler;

impl ContentHandler for MarkdownHandler {
    fn content_type(&self) -> ContentType {
        ContentType::Markdown
    }
    
    fn process(&self, content: &str) -> Result<String> {
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
}

/// JSON content handler
pub struct JsonHandler;

impl ContentHandler for JsonHandler {
    fn content_type(&self) -> ContentType {
        ContentType::Json
    }
    
    fn process(&self, content: &str) -> Result<String> {
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
}

/// YAML content handler
pub struct YamlHandler;

impl ContentHandler for YamlHandler {
    fn content_type(&self) -> ContentType {
        ContentType::Yaml
    }
    
    fn process(&self, content: &str) -> Result<String> {
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
}

/// TOML content handler
pub struct TomlHandler;

impl ContentHandler for TomlHandler {
    fn content_type(&self) -> ContentType {
        ContentType::Toml
    }
    
    fn process(&self, content: &str) -> Result<String> {
        // Similar to YAML but with TOML syntax
        let lines: Vec<String> = content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if !trimmed.starts_with('#') && (trimmed.contains('=') || trimmed.starts_with('[')) {
                    Some(trimmed.replace(['=', '[', ']'], " "))
                } else {
                    None
                }
            })
            .collect();

        Ok(lines.join("\n"))
    }
}

/// Source code content handler
pub struct SourceCodeHandler {
    content_type: ContentType,
    language: String,
}

impl SourceCodeHandler {
    pub fn new(content_type: ContentType, language: String) -> Self {
        Self {
            content_type,
            language,
        }
    }
}

impl ContentHandler for SourceCodeHandler {
    fn content_type(&self) -> ContentType {
        self.content_type.clone()
    }
    
    fn process(&self, content: &str) -> Result<String> {
        let mut extracted = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            
            // Extract comments based on language
            match self.language.as_str() {
                "rust" | "javascript" => {
                    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
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

/// Plain text content handler
pub struct PlainTextHandler;

impl ContentHandler for PlainTextHandler {
    fn content_type(&self) -> ContentType {
        ContentType::PlainText
    }
    
    fn process(&self, content: &str) -> Result<String> {
        Ok(content.to_string())
    }
}

/// Default handler for unknown content types
pub struct DefaultHandler;

impl ContentHandler for DefaultHandler {
    fn content_type(&self) -> ContentType {
        ContentType::Unknown
    }
    
    fn process(&self, content: &str) -> Result<String> {
        Ok(content.to_string())
    }
}
