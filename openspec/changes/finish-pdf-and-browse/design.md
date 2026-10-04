## Context

The current uncommitted implementation:

- **PDF**: `StandardFileSystemService::read_file_content` returns the *path string* for `.pdf` files. `PdfHandler::process(&str)` then treats that string as a path and calls `pdf_extract::extract_text`. `ContentType::Pdf` and detection by extension are wired in. `tests/test_pdf_support.rs` calls `ContentTypeDetector::detect_from_extension`, which doesn't exist; the real API is `ContentTypeDetector::detect_content_type(path, content)`.
- **Browse**: the handler in `infrastructure/api/http/handlers.rs` reads any directory with `tokio::fs::read_dir`. It defaults to `/Users/thomasdoyle`, hardcodes "common paths", skips dotfiles, and uses a blocking `is_dir()` inside async code.
- **Progress**: `Arc<Mutex<HashMap>>` in `AppState`. It's written at the start and end of a synchronous request handler and never cleaned up.
- **Metadata**: `IndexingService` populates `custom` metadata. `EnhancedSearchService` maps it into `EnhancedSearchResult.metadata` as JSON strings.
- **Two indexing paths**: `POST /api/index` runs through `DocumentIndexingService` (`services/document_service.rs`), which reads files with `fs::read_to_string` and records only `collection` in custom metadata. `IndexingService` and `StandardFileSystemService` are used only by tests. Changes made only there would never reach the running server.

The HTTP server binds locally today, but the roadmap is a cloud-hosted service (see `add-mcp-server`). An unrestricted filesystem listing would be a serious hole there.

## Goals / Non-Goals

**Goals:**
- PDFs are indexed and searchable like any other text document.
- Browsing never exposes paths outside the configured roots.
- Search results carry enough metadata for the UI to show the file name, type, size and dates.
- The PDF test suite compiles and passes using fixture files.

**Non-Goals:**
- OCR for scanned or image-only PDFs.
- Async indexing jobs and progress streaming.
- Other binary formats (DOCX, etc.). The design should make adding them easy, but they're not part of this change.
- Authentication on `/api/browse`. That arrives with the multi-tenant cloud work.

## Decisions

**Extract PDF text in the file-reading layer, not in a `ContentHandler`.** `read_file_content` matches on `.pdf`, reads the bytes, and runs `pdf_extract::extract_text_from_mem` inside `tokio::task::spawn_blocking`, because extraction is CPU-bound. It returns the extracted text. `PdfHandler` then becomes a plain text normaliser (trim lines, drop blanks), like the other handlers. *Alternative:* keep the path-passing approach. Rejected because it breaks the `ContentHandler` contract (input is content), makes handlers do I/O, and stores the path as the document body if any step is skipped.

**Share extraction and metadata between both indexing paths.** A new `content_processing::extraction` module provides `read_document_text(path, max_size) -> Result<Option<String>>`. It reads text files directly, dispatches binary extensions to an extractor, and returns `Ok(None)` for a skip (failed, empty or oversized extraction). A shared `build_file_metadata(path, size, modified)` helper builds the custom metadata map. `DocumentIndexingService` (the production path) and `StandardFileSystemService` both call these helpers, so neither path duplicates the logic. *Alternative:* route `/api/index` through `IndexingService`. Rejected for this change because it also touches collection handling, filters and chunking.

**Introduce a small `BinaryExtractor` seam.** Use a match on extension inside the file service that dispatches to an extractor function (`pdf` → `extract_pdf_text`). It's a seam for DOCX later without a plugin system now.

**Treat failed or empty extraction as a skip, not an error.** If extraction fails or yields only whitespace (for example, a scanned PDF), log a warning with the path, skip the file, and count it in the indexing response as skipped. One bad PDF must not abort a directory index.

**Restrict browsing with an allowlist of canonical roots.** Config `browse_roots: Vec<PathBuf>` comes from `DOC_INDEXER_BROWSE_ROOTS` (separated by `:`). It defaults to the server's current working directory. Each request path is `canonicalize`d (which resolves `..` and symlinks) and must `starts_with` one canonical root, or the server returns `403`. With no `path`, the response lists the roots. `parent_path` is `null` at a root. *Alternative:* a feature flag to disable browsing. Rejected because the allowlist keeps the feature useful and safe.

**Return only directories and indexable files from browse.** Use `ContentTypeDetector` on file names, so the picker shows what will actually be indexed. Get file type from `entry.file_type().await` rather than blocking `Path::is_dir()`. Cap entries per response (default 1,000) and set `truncated: true` when the cap is hit.

**Drop `/api/progress` from this change.** It only ever observes completed work. Remove `IndexingProgress`, `progress_tracker` and `progress_id`, and keep the frontend's indexing page working without polling.

**Keep the `/api/search` response shape.** `/api/search` returns `zero_latency_search::SearchResult`, which already exposes `custom_metadata`, `collection`, `doc_id`, `uri` and `title`. Only the enhanced API (`EnhancedSearchService`) builds a `metadata` object. Renaming fields on `/api/search` would break the CLI, so the frontend maps `custom_metadata` instead.

**Return metadata values as JSON strings, unchanged.** Keep the current mapping. Numeric parsing (for `file_size`, for example) happens in the frontend. That avoids changing `custom_metadata: HashMap<String, String>` in core. Always include `doc_id`, `uri` and `title`, whether or not custom metadata exists (this removes the duplicated branch).

## Risks / Trade-offs

- [`pdf-extract` panics on some malformed PDFs] → Wrap the call in `std::panic::catch_unwind` inside `spawn_blocking`, and treat a panic as an extraction failure.
- [Large PDFs use a lot of memory] → Skip files above a configurable maximum size (default 50 MB) before extraction, and log it.
- [Symlink tricks to escape the browse roots] → Canonicalise both roots and requests before comparing. Add a test with a symlink pointing outside a root.
- [Frontend relies on the removed `progress_id`] → Update `indexing/page.tsx` in the same change, and confirm with `tsc --noEmit`.
- [The `pdf-extract` 0.7 dependency tree is heavy (lopdf and others)] → Accept it for now. Revisit if build time becomes a problem, since it's a candidate for a feature flag.

## Open Questions

- ~~Should `browse_roots` default to empty rather than the current working directory?~~ **Resolved:** it defaults to empty. Browsing is disabled until `DOC_INDEXER_BROWSE_ROOTS` is set; the no-path listing returns no roots, and any path gets `403`. A missing path is `404` only when it would lie inside a root, so the endpoint doesn't reveal what exists outside the roots.
- Should already-indexed PDFs (stored as path strings by the current prototype) be detected and re-indexed? It only matters if the prototype was run against a persistent collection.
