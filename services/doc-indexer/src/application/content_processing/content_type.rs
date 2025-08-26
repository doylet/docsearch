/// Content type enumeration
///
/// Represents different types of content that can be processed by the system.
/// This is separate from processing logic to follow SRP.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl ContentType {
    /// Check if this content type should be indexed
    ///
    /// This follows SRP by separating indexing policy from content processing
    pub fn should_index(&self) -> bool {
        matches!(
            self,
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
}
