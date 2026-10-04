## Why

The working tree holds about 900 lines of uncommitted feature work: PDF ingestion, a directory-browse endpoint for the indexing UI, an indexing-progress endpoint, richer search-result metadata, and a UTF-8 fix in glob filtering. It's useful but not shippable as it stands:

- The PDF test doesn't compile.
- PDF handling passes a file *path* through a function that should carry file *content*.
- `/api/browse` defaults to a hardcoded home directory and will list any path on the host.
- `/api/progress` reports on indexing that has already finished, because indexing runs synchronously.

This change finishes the work and makes it safe before it's committed.

## What Changes

- **PDF ingestion**: index `.pdf` files by extracting their text. Extraction moves to the file-reading layer, so content handlers always receive text, never a path.
- **Directory browse**: `GET /api/browse?path=` lists sub-directories and indexable files, restricted to a configured set of allowed root directories. It drops the hardcoded `/Users/thomasdoyle` paths. **BREAKING** for the frontend: if no path is given, the response lists the allowed roots instead of a home directory.
- **Search-result metadata**: search results carry document metadata (`file_path`, `file_extension`, `file_size`, `last_modified`, `indexed_at`, `doc_id`, `uri`, `title`) and the collection name, replacing the `metadata: None` TODO.
- **Glob filter fix**: wildcard matching steps over character boundaries, so non-ASCII paths no longer panic.
- **Removed from scope**: the in-memory `/api/progress` endpoint and the `progress_id` field on `IndexPathResponse`. With synchronous indexing it only ever reports "completed", and its entries are never evicted. Real progress reporting belongs with async job queues (Linear project "Ingestion: Cloud Storage and Processing").

## Capabilities

### New Capabilities
- `pdf-ingestion`: Detecting, extracting and indexing text from PDF documents.
- `directory-browse`: Browsing allowed server-side directories so a user can pick an indexing path.
- `search-result-metadata`: Document metadata and collection returned with each search result.

### Modified Capabilities
<!-- None: openspec/specs/ has no existing capability specs. -->

## Impact

- **Rust**: `services/doc-indexer` (content processing, `adapters.rs`, `indexing_service.rs`, `filter_service.rs`, `enhanced_search.rs`, HTTP handlers, config); `crates/zero-latency-api` (the `BROWSE` endpoint constant).
- **Dependencies**: adds `pdf-extract = "0.7"` to `doc-indexer`.
- **API**: new `GET /api/browse`; search responses gain a populated `metadata` object and `collection`.
- **Frontend**: `apps/frontend/app/indexing/page.tsx` (path picker) and `SearchResults.tsx` (metadata display); adds `date-fns`.
- **Config**: new `browse_roots` setting (environment variable `DOC_INDEXER_BROWSE_ROOTS`).
- **Depends on**: `restore-project-health` merged first, so the test suite is green.
