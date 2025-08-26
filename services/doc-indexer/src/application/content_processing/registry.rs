/// Content processor registry
///
/// Implements OCP by being open for extension (new handlers) but closed for modification
/// Follows DIP by depending on abstractions (ContentHandler trait) not concretions
use std::collections::HashMap;
use std::sync::Arc;
use zero_latency_core::Result;

use super::handlers::*;
use super::{ContentHandler, ContentType};

/// Registry for content handlers
///
/// This follows OCP - open for extension via new handlers,
/// closed for modification of existing registry logic
#[derive(Clone)]
pub struct ContentProcessorRegistry {
    handlers: HashMap<ContentType, Arc<dyn ContentHandler>>,
    default_handler: Arc<dyn ContentHandler>,
}

impl ContentProcessorRegistry {
    /// Create a new registry with default handlers
    pub fn new() -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
            default_handler: Arc::new(DefaultHandler),
        };

        // Register all default handlers
        registry.register_default_handlers();
        registry
    }

    /// Register a handler for a specific content type
    ///
    /// This enables extension without modification (OCP)
    pub fn register_handler(&mut self, handler: Arc<dyn ContentHandler>) {
        let content_type = handler.content_type();
        self.handlers.insert(content_type, handler);
    }

    /// Get a handler for the given content type
    pub fn get_handler(&self, content_type: &ContentType) -> &Arc<dyn ContentHandler> {
        self.handlers
            .get(content_type)
            .unwrap_or(&self.default_handler)
    }

    /// Process content using the appropriate handler
    pub fn process_content(&self, content: &str, content_type: &ContentType) -> Result<String> {
        let handler = self.get_handler(content_type);
        handler.process(content)
    }

    /// Register all default content handlers
    fn register_default_handlers(&mut self) {
        // Register all built-in handlers
        self.register_handler(Arc::new(HtmlHandler));
        self.register_handler(Arc::new(MarkdownHandler));
        self.register_handler(Arc::new(JsonHandler));
        self.register_handler(Arc::new(YamlHandler));
        self.register_handler(Arc::new(TomlHandler));
        self.register_handler(Arc::new(PlainTextHandler));

        // Register source code handlers
        self.register_handler(Arc::new(SourceCodeHandler::new(
            ContentType::Rust,
            "rust".to_string(),
        )));
        self.register_handler(Arc::new(SourceCodeHandler::new(
            ContentType::JavaScript,
            "javascript".to_string(),
        )));
        self.register_handler(Arc::new(SourceCodeHandler::new(
            ContentType::Python,
            "python".to_string(),
        )));
        self.register_handler(Arc::new(SourceCodeHandler::new(
            ContentType::Shell,
            "shell".to_string(),
        )));
    }
}

impl Default for ContentProcessorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_extension() {
        let mut registry = ContentProcessorRegistry::new();

        // Should be extensible with new handlers
        struct CustomHandler;
        impl ContentHandler for CustomHandler {
            fn content_type(&self) -> ContentType {
                ContentType::Config
            }

            fn process(&self, content: &str) -> Result<String> {
                Ok(format!("Custom: {}", content))
            }
        }

        registry.register_handler(Arc::new(CustomHandler));

        let result = registry
            .process_content("test", &ContentType::Config)
            .unwrap();
        assert!(result.starts_with("Custom:"));
    }
}
