## ADDED Requirements

### Requirement: PDF files are detected as indexable
The system SHALL classify files with a `.pdf` extension (case-insensitive) as `ContentType::Pdf`, and SHALL treat that content type as indexable.

#### Scenario: Lowercase extension
- **WHEN** the detector receives the path `docs/guide.pdf`
- **THEN** it returns `ContentType::Pdf`
- **AND** `is_indexable()` returns true

#### Scenario: Uppercase extension
- **WHEN** the detector receives the path `docs/GUIDE.PDF`
- **THEN** it returns `ContentType::Pdf`

### Requirement: PDF text is extracted before content processing
The system SHALL extract the text of a PDF when reading the file, and SHALL pass only extracted text, never a file path, to content handlers and to the stored document body.

#### Scenario: Text PDF is indexed
- **WHEN** a directory containing a PDF with the text "zero latency retrieval" is indexed
- **THEN** a search for "zero latency retrieval" returns a result whose `file_extension` metadata is `pdf`
- **AND** the stored document content contains the extracted text, not the file's path

#### Scenario: Extraction normalises whitespace
- **WHEN** a PDF's extracted text contains blank lines and lines with leading or trailing spaces
- **THEN** the indexed content has those lines trimmed and the blank lines removed

### Requirement: Unreadable PDFs do not abort indexing
The system SHALL skip a PDF that fails extraction, panics during extraction, yields no text, or exceeds the configured maximum size. It SHALL log a warning naming the file, and SHALL continue indexing the remaining files.

#### Scenario: Corrupt PDF in a directory
- **WHEN** a directory containing one corrupt PDF and two valid Markdown files is indexed
- **THEN** the indexing request succeeds
- **AND** two documents are indexed
- **AND** a warning naming the corrupt PDF is logged

#### Scenario: Image-only PDF
- **WHEN** a PDF with no extractable text is indexed
- **THEN** no document is created for it
- **AND** the file is counted as skipped

#### Scenario: Oversized PDF
- **WHEN** a PDF larger than the configured maximum size is indexed
- **THEN** it is skipped without attempting extraction
