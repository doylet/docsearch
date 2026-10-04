/// Text extraction for indexable files
///
/// Content handlers only ever receive text. Binary formats (PDF) are turned
/// into text here, when the file is read, so no handler does filesystem I/O.
use std::collections::HashMap;
use std::path::Path;
use zero_latency_core::{Result, ZeroLatencyError};

/// Extract the text of a PDF held in memory.
///
/// Returns `Ok(None)` when the PDF holds no extractable text (for example a
/// scanned, image-only PDF). A malformed PDF that makes `pdf-extract` panic
/// is reported as an error rather than unwinding into the caller.
pub fn extract_pdf_text(bytes: &[u8]) -> Result<Option<String>> {
    let extracted = std::panic::catch_unwind(|| pdf_extract::extract_text_from_mem(bytes))
        .map_err(|_| ZeroLatencyError::internal("PDF extraction panicked"))?
        .map_err(|e| ZeroLatencyError::internal(format!("Failed to extract PDF text: {}", e)))?;

    if extracted.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(extracted))
    }
}

/// Turns the bytes of a binary document into text; `Ok(None)` means no text.
type Extractor = fn(&[u8]) -> Result<Option<String>>;

/// Binary formats that need a text extractor, keyed by lowercase extension.
fn binary_extractor(path: &Path) -> Option<Extractor> {
    let extension = path.extension()?.to_str()?.to_lowercase();
    match extension.as_str() {
        "pdf" => Some(extract_pdf_text),
        _ => None,
    }
}

/// Read a file as text ready for content processing.
///
/// - Text files are read as UTF-8; an unreadable file is an error.
/// - Binary formats are size-checked, then extracted off the async runtime.
///   An oversized file, failed extraction or empty result logs a warning
///   naming the file and returns `Ok(None)` so the caller skips it.
pub async fn read_document_text(path: &Path, max_binary_size: u64) -> Result<Option<String>> {
    let Some(extract) = binary_extractor(path) else {
        let text = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ZeroLatencyError::internal(format!("Failed to read file: {}", e)))?;
        return Ok(Some(text));
    };

    let size = tokio::fs::metadata(path)
        .await
        .map_err(|e| ZeroLatencyError::internal(format!("Failed to read metadata: {}", e)))?
        .len();
    if size > max_binary_size {
        tracing::warn!(
            path = %path.display(),
            size,
            max_binary_size,
            "Skipping file larger than the maximum binary file size"
        );
        return Ok(None);
    }

    let bytes = tokio::fs::read(path)
        .await
        .map_err(|e| ZeroLatencyError::internal(format!("Failed to read file: {}", e)))?;

    let extracted = tokio::task::spawn_blocking(move || extract(&bytes))
        .await
        .map_err(|e| ZeroLatencyError::internal(format!("Extraction task failed: {}", e)))?;

    match extracted {
        Ok(Some(text)) => Ok(Some(text)),
        Ok(None) => {
            tracing::warn!(path = %path.display(), "Skipping file with no extractable text");
            Ok(None)
        }
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "Skipping file that failed extraction");
            Ok(None)
        }
    }
}

/// Build the custom metadata recorded for a document indexed from a file.
pub fn build_file_metadata(
    path: &Path,
    size: u64,
    modified: chrono::DateTime<chrono::Utc>,
) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    metadata.insert("file_path".to_string(), path.to_string_lossy().to_string());
    metadata.insert("file_size".to_string(), size.to_string());
    metadata.insert("file_type".to_string(), file_type(path));
    metadata.insert("last_modified".to_string(), modified.to_rfc3339());
    metadata.insert("indexed_at".to_string(), chrono::Utc::now().to_rfc3339());

    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        metadata.insert("file_name".to_string(), stem.to_string());
    }
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        metadata.insert("file_extension".to_string(), extension.to_lowercase());
    }

    metadata
}

/// MIME-like type derived from the file extension, e.g. `text/md`.
pub fn file_type(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!("text/{}", ext.to_lowercase()))
        .unwrap_or_else(|| "text/plain".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_file_metadata_records_required_keys() {
        let path = PathBuf::from("guides/setup.md");
        let modified = chrono::Utc::now();

        let metadata = build_file_metadata(&path, 42, modified);

        for key in [
            "file_path",
            "file_name",
            "file_extension",
            "file_size",
            "file_type",
            "last_modified",
            "indexed_at",
        ] {
            let value = metadata.get(key);
            assert!(
                value.is_some_and(|v| !v.is_empty()),
                "missing metadata key {key}"
            );
        }
        assert_eq!(metadata["file_name"], "setup");
        assert_eq!(metadata["file_extension"], "md");
        assert_eq!(metadata["file_size"], "42");
        assert!(chrono::DateTime::parse_from_rfc3339(&metadata["last_modified"]).is_ok());
        assert!(chrono::DateTime::parse_from_rfc3339(&metadata["indexed_at"]).is_ok());
    }

    #[test]
    fn test_build_file_metadata_without_extension() {
        let metadata = build_file_metadata(Path::new("docs/README"), 1, chrono::Utc::now());

        assert_eq!(metadata["file_name"], "README");
        assert!(!metadata.contains_key("file_extension"));
        assert_eq!(metadata["file_type"], "text/plain");
    }

    #[test]
    fn test_extract_pdf_text_rejects_garbage_without_panicking() {
        assert!(extract_pdf_text(b"not a pdf").is_err());
    }
}
