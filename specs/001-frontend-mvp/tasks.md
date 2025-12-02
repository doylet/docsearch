# Tasks: Frontend MVP for Zero-Latency Document Search

**Input**: Design documents from `/specs/001-frontend-mvp/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ⏳, data-model.md ⏳, contracts/ ⏳

**Tests**: Tests are NOT explicitly requested in specification, therefore test tasks are EXCLUDED from this breakdown per template guidance.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Web application structure: `frontend/` directory at repository root
All paths relative to `/Users/thomasdoyle/Daintree/projects/rust/docsearch/frontend/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and configuration (Next.js already initialized, dependencies installed)

- [X] T001 Configure TypeScript path aliases for clean imports in tsconfig.json (@/domain, @/application, @/infrastructure, @/presentation)
- [X] T002 [P] Create frontend/vitest.config.ts for unit testing framework
- [X] T003 [P] Create frontend/playwright.config.ts for E2E testing framework
- [X] T004 [P] Update frontend/.gitignore with test coverage and build artifacts

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core domain, application, and infrastructure layers that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Domain Layer (Entity Definitions)

- [X] T005 [P] Create Document entity in frontend/domain/entities/Document.ts (id, title, path, content, score, collection)
- [X] T006 [P] Create Collection entity in frontend/domain/entities/Collection.ts (name, description, document_count, created_at)
- [X] T007 [P] Create SearchResult entity in frontend/domain/entities/SearchResult.ts (document, score, highlights)
- [X] T008 [P] Create IndexOperation entity in frontend/domain/entities/IndexOperation.ts (path, collection, status, documents_processed, errors)

### Domain Layer (Repository Interfaces)

- [X] T009 [P] Create SearchRepository interface in frontend/domain/repositories/SearchRepository.ts (search method with options)
- [X] T010 [P] Create IndexRepository interface in frontend/domain/repositories/IndexRepository.ts (indexPath, indexFile methods)
- [X] T011 [P] Create CollectionRepository interface in frontend/domain/repositories/CollectionRepository.ts (list, get methods)

### Domain Layer (Use Cases)

- [X] T012 Create SearchDocumentsUseCase in frontend/domain/usecases/SearchDocumentsUseCase.ts (depends on T009)
- [X] T013 [P] Create IndexDocumentsUseCase in frontend/domain/usecases/IndexDocumentsUseCase.ts (depends on T010)
- [X] T014 [P] Create ManageCollectionsUseCase in frontend/domain/usecases/ManageCollectionsUseCase.ts (depends on T011)

### Infrastructure Layer (API Client & Adapters)

- [X] T015 Create API configuration in frontend/infrastructure/config/apiConfig.ts (reads NEXT_PUBLIC_API_URL from .env.local)
- [X] T016 Create base RestApiClient in frontend/infrastructure/api/RestApiClient.ts (HTTP client with error handling, depends on T015)
- [X] T017 Create RestApiSearchRepository in frontend/infrastructure/api/RestApiSearchRepository.ts (implements SearchRepository, depends on T009, T016)
- [X] T018 [P] Create RestApiIndexRepository in frontend/infrastructure/api/RestApiIndexRepository.ts (implements IndexRepository, depends on T010, T016)
- [X] T019 [P] Create RestApiCollectionRepository in frontend/infrastructure/api/RestApiCollectionRepository.ts (implements CollectionRepository, depends on T011, T016)

### Application Layer (Dependency Injection)

- [X] T020 Complete DependencyContainer in frontend/application/providers/DependencyContainer.ts (Zustand store, wire use cases with repositories, depends on T012-T014, T017-T019)
- [X] T021 Verify QueryProvider in frontend/application/providers/QueryProvider.tsx (React Query setup, may already exist)
- [X] T022 Update frontend/app/layout.tsx to wrap children with DependencyContainer and QueryProvider (depends on T020, T021)

**Checkpoint**: Foundation ready - domain entities defined, repository interfaces established, use cases implemented, infrastructure adapters created, DI container configured. User story implementation can now begin.

---

## Phase 3: User Story 1 - Semantic Document Search (Priority: P1) 🎯 MVP

**Goal**: Enable users to search indexed documentation using natural language queries and view relevant results with titles, paths, snippets, and relevance scores.

**Independent Test**: Start backend with pre-indexed documents, open frontend, enter search query (e.g., "authentication methods"), verify results appear with all required fields.

### Implementation for User Story 1

- [X] T023 [P] [US1] Create SearchInterface component in frontend/presentation/components/SearchInterface.tsx (search input, query state management)
- [X] T024 [P] [US1] Create SearchResults component in frontend/presentation/components/SearchResults.tsx (result list display with title, path, snippet, score)
- [X] T025 [P] [US1] Create LoadingSpinner component in frontend/presentation/components/LoadingSpinner.tsx (loading state indicator)
- [X] T026 [P] [US1] Create EmptyState component in frontend/presentation/components/EmptyState.tsx (no results, empty query states)
- [X] T027 [US1] Create or update useSearch hook in frontend/application/hooks/useSearch.ts to use SearchDocumentsUseCase from DI container (verify file exists first, depends on T012, T020)
- [X] T028 [US1] Update frontend/app/page.tsx to integrate SearchInterface and SearchResults components (depends on T023, T024, T025, T026, T027)
- [X] T029 [US1] Add error handling and error boundary in frontend/presentation/components/ErrorBoundary.tsx for search failures
- [X] T030 [US1] Verify query debouncing in useSearch hook (300ms delay for better UX)
- [X] T031 [US1] Verify 2-character minimum query length enforcement in SearchInterface

**Checkpoint**: At this point, User Story 1 should be fully functional - users can search and view results with all required information. Test independently before proceeding.

---

## Phase 4: User Story 2 - Collection-Filtered Search (Priority: P2)

**Goal**: Enable users to filter search results by collection to get more targeted results from specific document sets.

**Independent Test**: Create multiple collections via CLI, index documents into each, use frontend collection dropdown to filter search results, verify results come from selected collection only.

### Implementation for User Story 2

- [X] T032 [P] [US2] Create CollectionSelector component in frontend/presentation/components/CollectionSelector.tsx (dropdown with "All Collections" plus collection list)
- [X] T033 [US2] Create or update useCollections hook in frontend/application/hooks/useCollections.ts to use ManageCollectionsUseCase from DI container (verify file exists first, depends on T014, T020)
- [X] T034 [US2] Add collection state management to SearchInterface component in frontend/presentation/components/SearchInterface.tsx (integrate CollectionSelector, depends on T032)
- [X] T035 [US2] Update useSearch hook to accept collection filter parameter in frontend/application/hooks/useSearch.ts (depends on T027)
- [X] T036 [US2] Update SearchResults component to display collection name for each result in frontend/presentation/components/SearchResults.tsx (depends on T024)
- [X] T037 [US2] Add error handling for collection fetch failures (graceful fallback to "All Collections" only)
- [X] T038 [US2] Update frontend/app/page.tsx to wire collection selector with search (depends on T034, T035)

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Users can search across all collections or filter by specific collection. Test both scenarios.

---

## Phase 5: User Story 3 - File and Directory Indexing (Priority: P3)

**Goal**: Enable users to add new documents to the search system by selecting files or directories from their local filesystem.

**Independent Test**: Open indexing interface, select local directory, specify collection, click "Index Documents", verify progress display and success message with statistics.

### Implementation for User Story 3

- [X] T039 [P] [US3] Create IndexingInterface component in frontend/presentation/components/IndexingInterface.tsx (directory picker, collection selector)
- [X] T040 [P] [US3] Create IndexingProgress component in frontend/presentation/components/IndexingProgress.tsx (progress bar, documents processed, errors, time elapsed)
- [X] T041 [P] [US3] Create IndexingSummary component in frontend/presentation/components/IndexingSummary.tsx (success/failure summary with statistics)
- [X] T042 [US3] Create or update useIndexPath hook in frontend/application/hooks/useIndexing.ts to use IndexDocumentsUseCase from DI container (verify file exists first, depends on T013, T020)
- [X] T043 [US3] Create or update useIndexFile hook in frontend/application/hooks/useIndexing.ts to use IndexDocumentsUseCase from DI container (verify file exists first, depends on T013, T020)
- [X] T044 [US3] Create frontend/app/index/page.tsx for indexing page (depends on T039, T040, T041, T042, T043)
- [X] T045 [US3] Add navigation link to indexing page in frontend/app/layout.tsx or navigation component
- [X] T046 [US3] Implement directory picker dialog (browser native file API or library)
- [X] T047 [US3] Add default collection fallback logic (use "default" if no collection selected)
- [X] T048 [US3] Add error handling for indexing failures (display per-file errors while continuing)

**Checkpoint**: All three user stories should now be independently functional. Users can search (US1), filter by collection (US2), and index new documents (US3).

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories or enhance overall quality

- [X] T049 [P] Add page metadata and titles in all frontend/app/**/page.tsx files (SEO, browser tabs)
- [X] T050 [P] Verify responsive design for all components (1280x720 minimum as per spec)
- [X] T051 [P] Verify consistent styling and typography across all pages (FR-039: Tailwind classes, color scheme, spacing)
- [X] T052 [P] Add keyboard shortcuts for search (Cmd+K or Ctrl+K to focus search input)
- [X] T053 [P] Add aria-labels and accessibility attributes to all interactive components
- [X] T054 Verify error boundary catches all unhandled promise rejections (SC-006: zero unhandled rejections)
- [X] T055 Add loading states to all async operations (search, collection fetch, indexing)
- [X] T056 [P] Document API endpoint URLs in frontend/infrastructure/api/ comments
- [X] T057 [P] Add TypeScript strict mode validation (verify no 'any' types in domain/application layers)
- [X] T058 Performance validation: measure UI update time after API response (target <300ms per SC-007)
- [X] T059 Performance validation: measure end-to-end search time (target <5s per SC-001)
- [X] T060 [P] Update frontend/README.md with setup, development, and testing instructions
- [X] T061 Run quickstart validation: fresh checkout to running app in <5 minutes

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately (mostly configuration)
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
  - T005-T008: Domain entities (can run in parallel)
  - T009-T011: Repository interfaces (can run in parallel, after entities)
  - T012-T014: Use cases (sequential, depend on repository interfaces)
  - T015-T019: Infrastructure adapters (T015 first, then T016, then T017-T019 in parallel)
  - T020-T022: DI container setup (depends on all use cases and repositories)
- **User Stories (Phase 3-5)**: All depend on Foundational phase (T022) completion
  - User stories can proceed in parallel (if staffed) or sequentially in priority order
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after T022 (Foundation complete) - No dependencies on other stories
  - T023-T026: Components can run in parallel
  - T027-T031: Sequential wiring and integration

- **User Story 2 (P2)**: Can start after T022 (Foundation complete) - Extends US1 but independently testable
  - T032-T033: Can run in parallel
  - T034-T038: Sequential integration with US1 components

- **User Story 3 (P3)**: Can start after T022 (Foundation complete) - Independent of US1/US2
  - T039-T041: Components can run in parallel
  - T042-T048: Sequential wiring and integration

### Within Each User Story

- Components marked [P] can be developed in parallel (different files)
- Hooks and wiring tasks are typically sequential (depend on components)
- Page integration is final step for each story
- Each story completes with checkpoint validation

### Parallel Opportunities by Phase

**Phase 2 (Foundational)**:
- Parallel batch 1: T005, T006, T007, T008 (domain entities)
- Parallel batch 2: T009, T010, T011 (repository interfaces, after entities)
- Parallel batch 3: T013, T014 (use cases, after T012)
- Parallel batch 4: T017, T018, T019 (repository implementations, after T016)

**Phase 3 (User Story 1)**:
- Parallel batch: T023, T024, T025, T026 (all UI components)

**Phase 4 (User Story 2)**:
- Parallel batch: T032, T033 (component + hook)

**Phase 5 (User Story 3)**:
- Parallel batch: T039, T040, T041 (all UI components)
- Parallel batch: T042, T043 (indexing hooks, if not already complete)

**Phase 6 (Polish)**:
- Parallel batch: T049, T050, T051, T052, T053, T056, T057, T060 (independent improvements)

---

## Parallel Example: Foundational Phase

```bash
# Domain entities (all different files, no dependencies):
T005: "Create Document entity in frontend/domain/entities/Document.ts"
T006: "Create Collection entity in frontend/domain/entities/Collection.ts"
T007: "Create SearchResult entity in frontend/domain/entities/SearchResult.ts"
T008: "Create IndexOperation entity in frontend/domain/entities/IndexOperation.ts"

# Repository interfaces (after entities complete):
T009: "Create SearchRepository interface in frontend/domain/repositories/SearchRepository.ts"
T010: "Create IndexRepository interface in frontend/domain/repositories/IndexRepository.ts"
T011: "Create CollectionRepository interface in frontend/domain/repositories/CollectionRepository.ts"

# Repository implementations (after T016 complete):
T017: "Create RestApiSearchRepository in frontend/infrastructure/api/RestApiSearchRepository.ts"
T018: "Create RestApiIndexRepository in frontend/infrastructure/api/RestApiIndexRepository.ts"
T019: "Create RestApiCollectionRepository in frontend/infrastructure/api/RestApiCollectionRepository.ts"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only - Recommended)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T022) - **CRITICAL GATE**
3. Complete Phase 3: User Story 1 (T023-T031)
4. **STOP and VALIDATE**: Test search functionality independently
5. Deploy/demo if ready (users can search existing indexed documents)

**Estimated effort**: ~60% of total work delivers core MVP value

### Incremental Delivery

1. Setup + Foundational (T001-T022) → Foundation ready (~40% complete)
2. Add User Story 1 (T023-T031) → Test independently → **Deploy/Demo (MVP!)** (~60% complete)
3. Add User Story 2 (T032-T038) → Test independently → Deploy/Demo (~80% complete)
4. Add User Story 3 (T039-T048) → Test independently → Deploy/Demo (~95% complete)
5. Polish (T049-T060) → Final quality improvements → Production ready (100%)

**Each story adds incremental value without breaking previous stories**

### Parallel Team Strategy

With 3 developers after Foundational phase (T022) complete:

- **Developer A**: User Story 1 (T023-T031) - Priority 1, most critical
- **Developer B**: User Story 2 (T032-T038) - Priority 2, enhances search
- **Developer C**: User Story 3 (T039-T048) - Priority 3, convenience feature

Stories complete independently and integrate without conflicts (different page routes, isolated components).

---

## Task Estimates by Phase

| Phase | Task Count | Parallel Opportunities | Sequential Dependencies | Estimated Effort |
|-------|------------|------------------------|-------------------------|------------------|
| Phase 1: Setup | 4 | 3 | 1 | 5% (config files) |
| Phase 2: Foundational | 18 | 12 | 6 | 35% (architecture backbone) |
| Phase 3: US1 (P1) | 9 | 4 | 5 | 25% (MVP core) |
| Phase 4: US2 (P2) | 7 | 2 | 5 | 15% (search enhancement) |
| Phase 5: US3 (P3) | 10 | 5 | 5 | 15% (convenience feature) |
| Phase 6: Polish | 13 | 10 | 3 | 5% (quality/docs) |
| **Total** | **61** | **36** | **25** | **100%** |

---

## Notes

### File Status Legend
- **EXISTS**: File already created (manually by user or previous work)
- **PARTIAL**: File exists but incomplete/needs updates
- **NEEDS CREATION**: File does not exist, must be created

### Existing Files (from conversation history)
- ✅ frontend/application/hooks/useSearch.ts (EXISTS - manually created)
- ✅ frontend/application/hooks/useCollections.ts (EXISTS - manually created)
- ✅ frontend/application/hooks/useIndexing.ts (EXISTS - manually created)
- ⚠️ frontend/application/providers/DependencyContainer.ts (PARTIAL - creation attempted)
- ⚠️ frontend/application/providers/QueryProvider.tsx (PARTIAL - creation attempted)
- ⚠️ frontend/app/layout.tsx (PARTIAL - exists but needs provider wiring)
- ⚠️ frontend/app/page.tsx (PARTIAL - search UI code written but not rendering)

### Critical Success Factors
- **Foundation First**: Do NOT skip Phase 2. All user stories depend on clean domain/infrastructure setup.
- **Independent Testing**: After each user story checkpoint, test that story in isolation before proceeding.
- **Layer Boundaries**: Verify TypeScript imports respect hexagonal architecture (domain never imports infrastructure).
- **Error Handling**: Every API call must have error handling (FR-005, FR-021, FR-037, SC-008 = 100% coverage).
- **Constitution Compliance**: Code reviews must verify Clean Architecture principles (see plan.md Constitution Check).

### Risk Mitigation
- **File Creation Issues**: Historical tool failures documented. If file creation fails, provide code to clipboard for manual creation.
- **API Contract Mismatch**: T016-T019 create REST API adapters that match docs/API_REFERENCE.md exactly. Any mismatch blocks user stories. Note: Files use "RestApi" prefix for clarity (not JSON-RPC 2.0 protocol).
- **Dependency Confusion**: Use TypeScript path aliases from T001 to enforce layer separation (@/domain, @/application, @/infrastructure).

### Quality Gates
- [ ] After Phase 2: Verify all domain entities, repository interfaces, and use cases compile without errors
- [ ] After Phase 2: Verify DependencyContainer provides all required dependencies to hooks
- [ ] After Phase 3: Verify User Story 1 acceptance scenarios pass (6 scenarios from spec.md)
- [ ] After Phase 4: Verify User Story 2 acceptance scenarios pass (5 scenarios from spec.md)
- [ ] After Phase 5: Verify User Story 3 acceptance scenarios pass (7 scenarios from spec.md)
- [ ] After Phase 6: Verify all 8 success criteria met (SC-001 through SC-008 from spec.md)

---

## Ready for Implementation

This task breakdown provides:
- ✅ 61 concrete tasks with exact file paths
- ✅ Clear dependencies and parallel opportunities
- ✅ User story organization for independent delivery
- ✅ Constitution-aligned architecture (hexagonal pattern)
- ✅ MVP-first strategy (P1 → P2 → P3)
- ✅ Quality checkpoints after each story

**Next Step**: Begin with Phase 1 (Setup), then complete Phase 2 (Foundational) before starting any user story work. Foundation is the critical path that blocks all value delivery.
