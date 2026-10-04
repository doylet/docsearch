# directory-browse Specification

## Purpose
How `GET /api/browse` lets a user pick a server-side directory to index: what it lists, how it is confined to configured browse roots (`DOC_INDEXER_BROWSE_ROOTS`, disabled when unset), and how it reports bad paths and large directories.
## Requirements
### Requirement: Browse lists directory contents within allowed roots
The system SHALL expose `GET /api/browse?path=<dir>`. For a directory inside a configured browse root, it SHALL return the directory's canonical path, its parent path, and its non-hidden sub-directories and indexable files. Directories SHALL come first, then files, each sorted alphabetically.

#### Scenario: Browse a sub-directory
- **WHEN** a client requests `/api/browse?path=<root>/docs`, where `docs` contains the directory `guides`, the files `a.md` and `b.pdf`, and `.git`
- **THEN** the response is `200`, with items `guides` (directory), then `a.md` and `b.pdf` (files)
- **AND** `.git` is not listed

#### Scenario: Non-indexable files are hidden
- **WHEN** the browsed directory contains `app.exe` and `notes.md`
- **THEN** only `notes.md` is listed

### Requirement: Browse cannot escape allowed roots
The system SHALL canonicalise the requested path and SHALL refuse, with HTTP `403`, any path that does not lie inside a canonicalised browse root. This includes paths reached through `..` segments or symbolic links.

#### Scenario: Path traversal
- **WHEN** a client requests `/api/browse?path=<root>/../../etc`
- **THEN** the response is `403`

#### Scenario: Symlink pointing outside a root
- **WHEN** `<root>/link` is a symbolic link to `/etc` and a client requests `/api/browse?path=<root>/link`
- **THEN** the response is `403`

#### Scenario: Absolute path outside every root
- **WHEN** a client requests `/api/browse?path=/`
- **THEN** the response is `403`

### Requirement: Browse without a path lists the roots
When no `path` is given, the system SHALL return the configured browse roots as directory items. It SHALL NOT fall back to any user's home directory.

#### Scenario: No path parameter
- **WHEN** a client requests `/api/browse` and the configured roots are `/data/docs` and `/data/wiki`
- **THEN** the response lists exactly those two directories
- **AND** `parent_path` is null

### Requirement: Browse handles bad paths and large directories
The system SHALL return `404` for a path that does not exist, `400` for a path that is not a directory, and SHALL cap the number of items returned per response, setting `truncated: true` when the cap is reached.

#### Scenario: Missing directory
- **WHEN** a client requests a path inside a root that does not exist
- **THEN** the response is `404`

#### Scenario: Truncated listing
- **WHEN** a directory contains more entries than the configured cap
- **THEN** the response contains exactly the cap's number of items
- **AND** `truncated` is true
