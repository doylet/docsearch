//! PDF ingestion tests covering the `pdf-ingestion` spec scenarios.

use doc_indexer::application::content_processing::extraction::read_document_text;
use doc_indexer::application::content_processing::handlers::PdfHandler;
use doc_indexer::application::content_processing::{
    ContentHandler, ContentProcessorRegistry, ContentType, ContentTypeDetector,
};
#[cfg(feature = "embedded")]
use doc_indexer::application::services::document_service::DocumentIndexingService;
#[cfg(feature = "embedded")]
use doc_indexer::application::ServiceContainer;
#[cfg(feature = "embedded")]
use doc_indexer::config::Config;
use doc_indexer::{FileSystemService, StandardFileSystemService};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const MAX_SIZE: u64 = 50 * 1024 * 1024;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pdf")
        .join(name)
}

/// Copy fixtures (and extra Markdown files) into a fresh directory.
#[cfg(feature = "embedded")]
fn directory_with(pdfs: &[&str], markdown: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for name in pdfs {
        std::fs::copy(fixture(name), dir.path().join(name)).unwrap();
    }
    for name in markdown {
        std::fs::write(
            dir.path().join(name),
            format!("# {name}\n\nSome markdown content."),
        )
        .unwrap();
    }
    dir
}

/// Needs the hash embedder, so only in builds with `embedded`.
#[cfg(feature = "embedded")]
async fn indexing_service() -> DocumentIndexingService {
    let config = Config::for_tests();
    let container = ServiceContainer::new(config).await.unwrap();
    DocumentIndexingService::new(&container)
}

/// Captures formatted tracing output for the current thread.
#[derive(Clone, Default)]
struct LogCapture(Arc<Mutex<Vec<u8>>>);

impl LogCapture {
    fn contents(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).to_string()
    }
}

impl Write for LogCapture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn capture_warnings() -> (LogCapture, tracing::subscriber::DefaultGuard) {
    let capture = LogCapture::default();
    let writer = capture.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    let guard = tracing::subscriber::set_default(subscriber);
    (capture, guard)
}

// Requirement: PDF files are detected as indexable

#[test]
fn lowercase_pdf_extension_is_detected_and_indexable() {
    let content_type = ContentTypeDetector::detect_content_type(Path::new("docs/guide.pdf"), "");
    assert_eq!(content_type, ContentType::Pdf);
    assert!(content_type.should_index());
}

#[test]
fn uppercase_pdf_extension_is_detected() {
    let content_type = ContentTypeDetector::detect_content_type(Path::new("docs/GUIDE.PDF"), "");
    assert_eq!(content_type, ContentType::Pdf);
}

#[test]
fn registry_has_pdf_handler() {
    let registry = ContentProcessorRegistry::new();
    assert_eq!(
        registry.get_handler(&ContentType::Pdf).content_type(),
        ContentType::Pdf
    );
}

// Requirement: PDF text is extracted before content processing

#[tokio::test]
async fn text_pdf_is_extracted_not_passed_as_path() {
    let path = fixture("text.pdf");

    let text = read_document_text(&path, MAX_SIZE).await.unwrap().unwrap();
    assert!(
        text.contains("zero latency retrieval"),
        "extracted: {text:?}"
    );

    let via_service = StandardFileSystemService::new()
        .read_file_content(&path)
        .await
        .unwrap();
    assert!(via_service.contains("zero latency retrieval"));
    assert!(!via_service.contains(path.to_str().unwrap()));
}

#[cfg(feature = "embedded")]
#[tokio::test]
async fn text_pdf_is_indexed_and_searchable_with_pdf_metadata() {
    let dir = directory_with(&["text.pdf"], &[]);
    let service = indexing_service().await;

    let stats = service
        .index_documents_from_path_with_filters_and_collection(
            dir.path().to_str().unwrap(),
            true,
            None,
            "zero_latency_docs",
        )
        .await
        .unwrap();
    assert_eq!(stats.documents_processed, 1);
    assert_eq!(stats.documents_skipped, 0);

    let response = service
        .search_documents_in_collection("zero latency retrieval", "zero_latency_docs", 5)
        .await
        .unwrap();
    let result = response
        .results
        .iter()
        .find(|r| r.custom_metadata.get("file_extension").map(String::as_str) == Some("pdf"))
        .expect("a result with file_extension = pdf");

    assert_eq!(result.collection.as_deref(), Some("zero_latency_docs"));
    for key in ["file_name", "file_size", "last_modified", "indexed_at"] {
        assert!(result.custom_metadata.contains_key(key), "missing {key}");
    }
    assert!(result.content.contains("zero latency retrieval"));
    assert!(!result.content.contains(dir.path().to_str().unwrap()));
}

#[test]
fn pdf_handler_normalises_whitespace() {
    let processed = PdfHandler
        .process("  first line  \n\n   \nsecond line\t\n\n")
        .unwrap();
    assert_eq!(processed, "first line\nsecond line");
}

// Requirement: Unreadable PDFs do not abort indexing

#[cfg(feature = "embedded")]
#[tokio::test]
async fn corrupt_pdf_in_directory_is_skipped_with_warning() {
    let dir = directory_with(&["corrupt.pdf"], &["one.md", "two.md"]);
    let service = indexing_service().await;
    let (logs, _guard) = capture_warnings();

    let stats = service
        .index_documents_from_path_with_filters_and_collection(
            dir.path().to_str().unwrap(),
            true,
            None,
            "zero_latency_docs",
        )
        .await
        .unwrap();

    assert_eq!(stats.documents_processed, 2);
    assert_eq!(stats.documents_skipped, 1);
    assert!(
        logs.contents().contains("corrupt.pdf"),
        "warning should name the file: {}",
        logs.contents()
    );
}

#[cfg(feature = "embedded")]
#[tokio::test]
async fn image_only_pdf_creates_no_document_and_counts_as_skipped() {
    assert_eq!(
        read_document_text(&fixture("image_only.pdf"), MAX_SIZE)
            .await
            .unwrap(),
        None
    );

    let dir = directory_with(&["image_only.pdf"], &[]);
    let stats = indexing_service()
        .await
        .index_documents_from_path_with_filters_and_collection(
            dir.path().to_str().unwrap(),
            true,
            None,
            "zero_latency_docs",
        )
        .await
        .unwrap();

    assert_eq!(stats.documents_processed, 0);
    assert_eq!(stats.documents_skipped, 1);
}

#[tokio::test]
async fn oversized_pdf_is_skipped_before_extraction() {
    let (logs, _guard) = capture_warnings();

    let result = read_document_text(&fixture("text.pdf"), 10).await.unwrap();

    assert_eq!(result, None);
    let logs = logs.contents();
    assert!(logs.contains("larger than the maximum"), "{logs}");
    assert!(
        !logs.contains("failed extraction") && !logs.contains("no extractable text"),
        "extraction should not run: {logs}"
    );
}
