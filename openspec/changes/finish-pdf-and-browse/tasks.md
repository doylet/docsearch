## 1. Setup

- [x] 1.1 Restore the set-aside feature work (moved from `restore-project-health` task 6.3): rebase `wip/finish-pdf-and-browse` (commit `3df6525`) onto `main`, or onto `restore-project-health` if that isn't merged yet; resolve conflicts with the `cargo fix` and test commits; rename the branch to `finish-pdf-and-browse`; if `fix-ci-pipeline` has merged, run `cargo fmt --all` after the rebase and commit it
- [x] 1.2 Confirm `cargo test --workspace --lib --bins` passes after the rebase (the `test_pdf_support` target stays broken until task 2.6)
- [x] 1.3 Add PDF fixtures under `services/doc-indexer/tests/fixtures/pdf/`: a text PDF, a corrupt PDF and an image-only PDF

## 2. PDF ingestion

- [x] 2.1 Add `extract_pdf_text(bytes) -> Result<Option<String>>` using `pdf_extract::extract_text_from_mem`, wrapped in `catch_unwind`
- [x] 2.2 Add `content_processing::extraction::read_document_text(path, max_size)`: read bytes of `.pdf` files, enforce the maximum size, run extraction in `spawn_blocking`; call it from both `DocumentIndexingService` (production path) and `StandardFileSystemService::read_file_content`
- [x] 2.3 Make `PdfHandler::process` a text normaliser (trim lines, drop blanks) with no filesystem access
- [x] 2.4 Make `DocumentIndexingService` (and `IndexingService`) skip and count files whose extraction fails or is empty, logging a warning with the path; report the count as `documents_skipped` in `IndexPathResponse`
- [x] 2.5 Make the detector match the `.pdf` extension case-insensitively
- [x] 2.6 Rewrite `tests/test_pdf_support.rs` against `ContentTypeDetector::detect_content_type` and the fixtures, covering every `pdf-ingestion` scenario

## 3. Directory browse

- [x] 3.1 Add `browse_roots` (`DOC_INDEXER_BROWSE_ROOTS`) and `browse_max_entries` to doc-indexer config, and resolve the open question about the default
- [x] 3.2 Move the browse handler into its own module; canonicalise roots at startup and requests per call; return `403`, `404` or `400` as the spec says
- [x] 3.3 Remove the hardcoded `/Users/thomasdoyle` default and the "common paths" list; with no `path`, list the roots
- [x] 3.4 Use `entry.file_type().await`, filter files through `ContentTypeDetector`, and apply the entry cap and `truncated`
- [x] 3.5 Add handler tests (using `tempfile`) for listing, hidden files, traversal, symlink escape, the missing path, and truncation

## 4. Remove progress tracking

- [x] 4.1 Remove `IndexingProgress`, `IndexingStatus`, `ProgressQuery`, `progress_tracker`, the `/api/progress` route and `IndexPathResponse.progress_id`
- [ ] 4.2 Record async progress as follow-up in the Linear "Ingestion" project

## 5. Search-result metadata and glob fix

- [x] 5.1 Simplify the metadata mapping in `enhanced_search.rs` so `doc_id`, `uri` and `title` are always added (one code path)
- [x] 5.2 Extract a shared `build_file_metadata` helper, use it from `DocumentIndexingService` and `IndexingService`, and add a unit test asserting it records every required metadata key
- [x] 5.3 Add a unit test for `match_recursive` with a non-ASCII path and the pattern `*.md`

## 6. Frontend

- [x] 6.1 Update `app/indexing/page.tsx` to use the roots-first browse response and stop relying on `progress_id`
- [x] 6.2 Update `SearchResults.tsx` to show file name, extension, size and dates from `metadata`, parsing numbers on the client
- [x] 6.3 Regenerate API types if they're generated from OpenAPI (`api/openapitools.json`) and add `/api/browse` to the spec
- [x] 6.4 Run `npx tsc --noEmit` and `npm run build`

## 7. Verify and ship

- [x] 7.1 Run `cargo test --workspace` and `cargo clippy -p doc-indexer`
- [x] 7.2 Manual check: index a folder containing a PDF, search for its text, and confirm the result shows PDF metadata
- [x] 7.3 Manual check: browse outside the roots returns `403` from the UI and from curl (curl verified; UI error mapping checked by build, not driven in a browser)
- [x] 7.4 Update `docs/API_REFERENCE.md` for `/api/browse` and the search-result metadata
- [x] 7.5 Commit the PDF work, the browse work, and the metadata/glob work as separate commits
