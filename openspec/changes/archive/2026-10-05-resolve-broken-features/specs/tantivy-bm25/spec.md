## ADDED Requirements

### Requirement: Index opens in any directory state
With the `tantivy` feature, `TantivyAdapter::new` SHALL open an existing index at `index_path`, or create one if the path is missing or holds an empty directory.

#### Scenario: Missing directory
- **WHEN** `index_path` does not exist
- **THEN** the directory and a new index are created

#### Scenario: Empty directory
- **WHEN** `index_path` is an existing, empty directory
- **THEN** a new index is created in it, without error

#### Scenario: Existing index
- **WHEN** `index_path` holds an index from an earlier run containing a document
- **THEN** the adapter opens it and that document is searchable

### Requirement: Writes are visible when they return
A document passed to `index_document` SHALL be returned by a matching `search` as soon as `index_document` returns. A document passed to `delete_document` SHALL no longer be returned as soon as `delete_document` returns.

#### Scenario: Index then search
- **WHEN** a document titled "Test Document" is indexed and `search("test document")` is called immediately afterwards
- **THEN** the results include that document's id

#### Scenario: Delete then search
- **WHEN** an indexed document is deleted and the same search runs immediately afterwards
- **THEN** the results don't include it
