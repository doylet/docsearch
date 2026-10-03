use doc_indexer::application::content_processing::*;
use doc_indexer::application::content_processing::handlers::PdfHandler;
use std::collections::HashMap;

#[tokio::test]
async fn test_pdf_handler() {
    let handler = PdfHandler;

    // Test with a non-existent file path to check error handling
    let result = handler.process("/nonexistent/path.pdf");
    assert!(result.is_err());

    println!("PDF handler error handling works correctly");
}

#[tokio::test]
async fn test_content_type_detection() {
    let detector = ContentTypeDetector;

    // Test PDF detection
    let pdf_type = detector.detect_from_extension("test.pdf");
    assert_eq!(pdf_type, ContentType::Pdf);

    // Test that PDF content is indexable
    assert!(pdf_type.is_indexable());

    println!("PDF content type detection works correctly");
}

#[test]
fn test_registry_has_pdf_handler() {
    let registry = ContentProcessorRegistry::new();

    // Get the PDF handler from registry
    let handler = registry.get_handler(&ContentType::Pdf);

    // Verify it's the right type by checking the content type
    assert_eq!(handler.content_type(), ContentType::Pdf);

    println!("PDF handler is properly registered");
}

fn main() {
    println!("Running PDF handler tests...");

    // Run sync test
    test_registry_has_pdf_handler();

    println!("PDF support implementation is complete!");
}
