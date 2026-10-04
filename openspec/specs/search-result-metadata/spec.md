# search-result-metadata Specification

## Purpose
What file metadata is recorded for each indexed document, how search results expose it with the document identity and collection, and how path filters handle non-ASCII paths.
## Requirements
### Requirement: Indexed documents record file metadata
The system SHALL record the following metadata for every document indexed from a file: `file_path`, `file_name` (without extension), `file_extension` (when present), `file_size`, `file_type`, `last_modified` (RFC 3339) and `indexed_at` (RFC 3339).

#### Scenario: Markdown file metadata
- **WHEN** the file `guides/setup.md` is indexed
- **THEN** the document's metadata includes `file_name` = `setup`, `file_extension` = `md`, and a non-empty `file_path`, `file_size`, `last_modified` and `indexed_at`

### Requirement: Search results include metadata and collection
Each search result SHALL include the document's recorded metadata plus `doc_id`, `uri` and `title`, and SHALL include the name of the collection the result came from. On `POST /api/search` the recorded metadata is the `custom_metadata` object and `doc_id`, `uri`, `title` and `collection` are top-level fields. On the enhanced search API they are combined in a single `metadata` object.

#### Scenario: Result from a file-indexed document
- **WHEN** a search matches a chunk of `guides/setup.md` in the collection `zero_latency_docs`
- **THEN** the result carries `doc_id`, `uri`, `title`, `file_name` and `file_extension`
- **AND** the result's `collection` is `zero_latency_docs`

#### Scenario: Result from a document with no file metadata
- **WHEN** a search matches a document that was indexed through the API with no custom metadata
- **THEN** the result still carries `doc_id`, `uri` and `title`

### Requirement: Glob filters handle non-ASCII paths
Path filters with wildcards SHALL match on character boundaries, and SHALL NOT panic on multi-byte UTF-8 paths.

#### Scenario: Wildcard against a non-ASCII path
- **WHEN** the filter pattern `*.md` is applied to the path `docs/café/naïve.md`
- **THEN** the path matches
- **AND** no panic occurs
