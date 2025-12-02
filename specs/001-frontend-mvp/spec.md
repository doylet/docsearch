# Feature Specification: Frontend MVP for Zero-Latency Document Search

**Feature Branch**: `001-frontend-mvp`
**Created**: December 1, 2025
**Status**: Draft
**Input**: NextJS frontend application that allows users to 1) choose files/directories to index 2) search the documentation service 3) augment generative AI chat - following clean hexagonal architecture with domain/application/infrastructure layers, developed to MVP standard for local OS operation

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Semantic Document Search (Priority: P1)

A user wants to search their indexed documentation using natural language queries and view relevant results with context.

**Why this priority**: Search is the core value proposition of the system. Without search functionality, the application has no purpose. This delivers immediate value as users can find information in their existing indexed documents.

**Independent Test**: Can be fully tested by starting the backend service with pre-indexed documents, opening the frontend, entering a search query (e.g., "authentication methods"), and verifying that relevant results appear with titles, paths, and similarity scores. Delivers standalone value even without indexing or collection management capabilities.

**Acceptance Scenarios**:

1. **Given** the backend service is running with indexed documents, **When** user enters a search query with at least 2 characters, **Then** search results appear showing document titles, paths, content snippets, and relevance scores
2. **Given** the user has entered a search query, **When** results are displayed, **Then** each result shows a document title, file path, snippet of matching content, and a similarity score between 0 and 1
3. **Given** multiple results are available, **When** displayed, **Then** results are ordered by relevance score (highest first)
4. **Given** the search query is empty or has only 1 character, **When** displayed, **Then** no search is performed and the UI shows a neutral state (no results message)
5. **Given** the backend returns an error, **When** the error occurs, **Then** the UI displays a user-friendly error message without crashing
6. **Given** the search returns no results, **When** displayed, **Then** the UI shows a "No results found" message with the search query quoted

---

### User Story 2 - Collection-Filtered Search (Priority: P2)

A user wants to filter search results by collection to get more targeted and relevant results from specific document sets.

**Why this priority**: Enhances search precision for users with multiple document collections. Particularly valuable for power users organizing different document domains (e.g., API docs vs tutorials vs internal docs). Builds on P1 search functionality.

**Independent Test**: Can be tested by creating multiple collections via CLI, indexing different documents into each collection, then using the frontend collection dropdown to filter search results. Delivers value by improving search relevance when users have organized collections.

**Acceptance Scenarios**:

1. **Given** multiple collections exist in the backend, **When** the user opens the search interface, **Then** a collection dropdown appears showing "All Collections" plus all available collection names
2. **Given** the user selects a specific collection from the dropdown, **When** performing a search, **Then** results only come from the selected collection
3. **Given** the user selects "All Collections", **When** performing a search, **Then** results come from all collections across the system
4. **Given** a collection is selected, **When** results are displayed, **Then** each result indicates which collection it belongs to
5. **Given** the backend has no collections or collection endpoint fails, **When** the dropdown loads, **Then** only "All Collections" appears and search proceeds without collection filtering

---

### User Story 3 - File and Directory Indexing (Priority: P3)

A user wants to add new documents to the search system by selecting files or directories from their local filesystem.

**Why this priority**: Enables users to expand their searchable corpus without using the CLI. While valuable for self-service, users can initially rely on CLI-based indexing. This is primarily a convenience feature that improves the user experience but isn't essential for MVP operation.

**Independent Test**: Can be tested by opening the indexing interface, selecting a local directory or file, specifying a collection name, and verifying that documents are processed and become searchable. Delivers value by enabling non-technical users to manage their document corpus.

**Acceptance Scenarios**:

1. **Given** the user is on the indexing page, **When** the page loads, **Then** a file/directory picker interface appears with a collection selection dropdown
2. **Given** the user clicks "Select Directory", **When** the system dialog opens, **Then** the user can browse and select any accessible local directory
3. **Given** the user has selected a directory and collection, **When** clicking "Index Documents", **Then** the system displays progress showing number of documents processed
4. **Given** indexing is in progress, **When** displayed, **Then** the UI shows real-time status (documents processed, errors if any, time elapsed)
5. **Given** indexing completes successfully, **When** finished, **Then** the UI displays a success message with summary statistics (total documents, time taken, success rate)
6. **Given** indexing encounters errors, **When** errors occur, **Then** the UI displays specific error messages for failed files while continuing with successful ones
7. **Given** the user doesn't select a collection, **When** attempting to index, **Then** documents are indexed to a "default" collection automatically

---

### Edge Cases

- What happens when the backend service is not running or unreachable?
  - UI displays connection error message with instructions to start the backend service
  - Search and indexing operations fail gracefully with user-friendly error messages
  - Application remains functional (doesn't crash) and retries can be attempted

- What happens when a search query contains special characters or very long text?
  - Special characters are properly URL-encoded when sent to backend
  - Very long queries (>500 characters) are accepted but may return fewer results based on semantic meaning
  - No SQL injection or XSS vulnerabilities due to proper input sanitization

- What happens when indexing a very large directory (thousands of files)?
  - Progress indicator shows incremental updates (not frozen)
  - User can navigate away from indexing page without canceling operation
  - Backend handles batch processing and memory management (not a frontend concern)

- What happens when the same document is indexed multiple times?
  - Backend determines behavior (update vs skip) based on its configuration
  - Frontend displays whatever status the backend returns
  - No duplicate entries appear in search results (backend responsibility)

- What happens when search returns more than 100 results?
  - Frontend displays the top results returned by backend (backend handles limits)
  - For MVP, pagination is not implemented (backend default limit applies)
  - User can refine search query to get more targeted results

## Requirements *(mandatory)*

### Functional Requirements

**Search Functionality:**

- **FR-001**: System MUST allow users to enter search queries of 2 or more characters
- **FR-002**: System MUST display search results showing document title, file path, content snippet, and relevance score
- **FR-003**: System MUST order search results by relevance score (highest first)
- **FR-004**: System MUST show "No results found" message when search yields zero results
- **FR-005**: System MUST display user-friendly error messages when backend search fails
- **FR-006**: System MUST disable search requests when query is empty or has only 1 character

**Collection Management:**

- **FR-007**: System MUST display available collections in a dropdown selector
- **FR-008**: System MUST include "All Collections" option as default in collection dropdown
- **FR-009**: System MUST filter search results by selected collection when a specific collection is chosen
- **FR-010**: System MUST search across all collections when "All Collections" is selected
- **FR-011**: System MUST indicate which collection each result belongs to
- **FR-012**: System MUST handle missing or failed collection data gracefully (default to all-collection search)

**Indexing Functionality:**

- **FR-013**: System MUST provide file picker interface for selecting directories to index
- **FR-014**: System MUST provide collection selection for indexing operations
- **FR-015**: System MUST default to "default" collection when no collection is specified for indexing
- **FR-016**: System MUST display indexing progress (documents processed, errors, time elapsed)
- **FR-017**: System MUST show summary statistics when indexing completes (total documents, time taken, success rate)
- **FR-018**: System MUST display specific error messages for failed files while continuing to process successful ones
- **FR-019**: System MUST allow users to initiate indexing operations via UI button click

**API Integration:**

- **FR-020**: System MUST communicate with backend via REST API at http://localhost:8081
- **FR-021**: System MUST handle backend connection failures with clear error messages
- **FR-022**: System MUST properly encode search queries for URL transmission
- **FR-023**: System MUST parse backend JSON responses correctly for all API endpoints
- **FR-024**: System MUST use GET /search for search requests with query parameters
- **FR-025**: System MUST use POST /api/index for indexing operations with request body
- **FR-026**: System MUST use GET /collections for retrieving collection list

**Architecture & Code Organization:**

- **FR-027**: System MUST implement hexagonal (ports and adapters) architecture pattern
- **FR-028**: System MUST organize code into domain, application, and infrastructure layers
- **FR-029**: Domain layer MUST contain entity definitions (Document, Collection, SearchResult, IndexOperation)
- **FR-030**: Domain layer MUST define repository interfaces (SearchRepository, IndexRepository, CollectionRepository)
- **FR-031**: Application layer MUST contain use case implementations (SearchDocumentsUseCase, IndexDocumentsUseCase, ManageCollectionsUseCase)
- **FR-032**: Application layer MUST contain React hooks (useSearch, useIndexing, useCollections)
- **FR-033**: Infrastructure layer MUST contain JSON-RPC API client adapters
- **FR-034**: Infrastructure layer MUST implement repository interfaces using API client
- **FR-035**: System MUST use dependency injection via React Context for use case and repository wiring

**User Experience:**

- **FR-036**: System MUST provide responsive UI that works on desktop screen sizes (1280x720 minimum)
- **FR-037**: System MUST use loading indicators during asynchronous operations (search, indexing, collection fetch)
- **FR-038**: System MUST maintain UI responsiveness during all operations (no frozen states)
- **FR-039**: System MUST use consistent styling and typography across all pages
- **FR-040**: System MUST provide clear navigation between search and indexing interfaces

### Key Entities

- **Document**: Represents an indexed document with properties: id (string), title (string), path (string), content (string, optional), score (number, 0-1 range), collection (string)

- **Collection**: Represents a document collection with properties: name (string), description (optional string), document_count (number), created_at (timestamp)

- **SearchResult**: Represents a search result with properties: document (Document), score (number), highlights (array of strings, optional)

- **IndexOperation**: Represents an indexing operation with properties: path (string), collection (string), status (enum: pending, in_progress, completed, failed), documents_processed (number), errors (array of error messages)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can complete a search query and view results in under 5 seconds (including backend processing time)
- **SC-002**: Users can select a directory and initiate indexing in under 30 seconds
- **SC-003**: Application setup requires no more than 2 minutes (start backend, start frontend, verify connectivity)
- **SC-004**: Search interface displays results that match backend API response exactly (no data loss or transformation errors)
- **SC-005**: Application runs successfully on macOS with Node.js 18+ and backend service running on localhost:8081
- **SC-006**: Zero unhandled promise rejections or React errors during normal operation (search, collection selection, indexing)
- **SC-007**: All API responses under 200ms result in UI updates within 300ms (perceived responsiveness)
- **SC-008**: Application displays meaningful error messages for 100% of backend error scenarios (connection failed, invalid request, not found)

### Assumptions

- Backend service (doc-indexer) is running on localhost:8081 and is reachable
- Backend implements the REST API as documented in docs/API_REFERENCE.md
- User has basic familiarity with file system navigation
- User has Node.js 18+ installed for running the Next.js development server
- User is running macOS for initial MVP (cross-platform support deferred)
- Backend handles all business logic for indexing, embeddings, and vector search
- Backend enforces collection-level isolation and validation
- Backend manages all persistence (vector database, metadata storage)
