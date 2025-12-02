# Implementation Plan: Frontend MVP for Zero-Latency Document Search

**Branch**: `001-frontend-mvp` | **Date**: December 1, 2025 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-frontend-mvp/spec.md`

## Summary

Build a Next.js frontend application that enables users to search indexed documentation, filter by collection, and index new documents through a clean hexagonal architecture. The MVP delivers three prioritized user journeys: (P1) semantic search with relevance scoring, (P2) collection-filtered search, and (P3) file/directory indexing. The application integrates with the existing Rust backend service via REST API (localhost:8081) and follows Clean Architecture patterns established in the project constitution.

## Technical Context

**Language/Version**: TypeScript 5.x, Next.js 16.0.5 with App Router, React 19.2.0
**Primary Dependencies**: @tanstack/react-query 5.x, Zustand 5.x, Lucide React (icons), Tailwind CSS 4.x
**Storage**: None (stateless frontend, all persistence via backend API)
**Testing**: Vitest (unit), React Testing Library (component), Playwright (E2E) - to be configured
**Target Platform**: macOS desktop browsers (Chrome, Safari, Firefox), Node.js 18+ for development
**Project Type**: Web application (frontend only, backend already exists)
**Performance Goals**: <300ms UI updates after API response, <5s end-to-end search, <30s indexing initiation
**Constraints**: <200ms backend API response (backend responsibility), zero unhandled promise rejections, 100% error handling coverage
**Scale/Scope**: Single-user local deployment, 3 pages (search, index, settings), ~40 functional requirements, MVP focused

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### ✅ Core Principle I: Clean Architecture (NON-NEGOTIABLE)

**Assessment**: PASS - Architecture explicitly specified in requirements FR-027 through FR-035

- **Application Layer**: React hooks (useSearch, useIndexing, useCollections), React Query for data fetching
- **Domain Layer**: Entity interfaces (Document, Collection, SearchResult, IndexOperation), Repository interfaces (SearchRepository, IndexRepository, CollectionRepository)
- **Infrastructure Layer**: REST API client adapters, repository implementations (RestApiSearchRepository, etc.)

**Rationale**: Follows three-layer separation. Domain defines contracts, application orchestrates use cases, infrastructure handles API communication. No business logic in infrastructure, no HTTP details in domain.

**Enforcement**: Manual code review will verify layer boundaries before merge. TypeScript path aliases enforce import restrictions.

### ✅ Core Principle II: SOLID Principles Compliance

**Assessment**: PASS - Architecture designed for SOLID compliance

- **Single Responsibility**: Each use case handles one operation (search, index, collections)
- **Open/Closed**: Repository interfaces allow new API implementations without changing domain
- **Liskov Substitution**: Any SearchRepository implementation works with SearchDocumentsUseCase
- **Interface Segregation**: Separate repositories for search, indexing, collections (not one monolithic API interface)
- **Dependency Inversion**: Use cases depend on repository interfaces (abstractions), not concrete API clients

**Rationale**: TypeScript interfaces naturally enforce dependency inversion. Repository pattern ensures substitutability.

**Enforcement**: Code review validates no use case depends on concrete infrastructure types. All dependencies injected via React Context.

### ✅ Core Principle III: Shared Domain Crates

**Assessment**: PASS (with adaptation for TypeScript ecosystem)

Frontend does not create "crates" (Rust concept), but follows equivalent pattern:

- Domain layer defines shared types (Document, Collection) matching backend contracts
- Infrastructure layer imports backend API types from contracts (future: shared TypeScript types package)
- No duplicate entity definitions across layers

**Rationale**: TypeScript project structure allows domain/ folder to serve as "shared" layer. Future work: extract @zero-latency/types package for backend-frontend type sharing.

**Enforcement**: Code review ensures entity definitions reference backend API contracts. No divergent type definitions.

### ✅ Core Principle IV: Dependency Injection via ServiceContainer

**Assessment**: PASS - Uses React Context for DI (functional equivalent)

- DependencyContainer (Zustand store) holds use case and repository instances
- All React components receive dependencies via useDependencyContainer() hook
- No direct instantiation of repositories or use cases in components
- Builder pattern in DependencyContainer initialization

**Rationale**: React Context provides DI for React components. Zustand manages singleton instances. Functional equivalent of Rust ServiceContainer pattern.

**Enforcement**: Code review validates no `new` calls in components or hooks. All dependencies injected via Context.

### ⚠️ Core Principle V: Feature Flag Architecture

**Assessment**: NOT APPLICABLE - Frontend has no feature flag requirements

Frontend is single-variant deployment (not embedded vs cloud). All features always available.

**Rationale**: Feature flags address backend deployment scenarios (embedded/cloud/full). Frontend communicates with whatever backend variant is running.

**Enforcement**: N/A

### ✅ Core Principle VI: Test Coverage Standards

**Assessment**: PASS - Test requirements specified (to be implemented)

- **Unit Tests**: Use cases, repository implementations, utility functions (target >80%)
- **Component Tests**: React components with mocked dependencies (target >80%)
- **Contract Tests**: API client validates backend contract (100% endpoint coverage)
- **End-to-End Tests**: Complete user journeys per acceptance scenarios (P1, P2, P3 stories)

**Rationale**: Testing framework not yet configured but constitution standards apply. Vitest + React Testing Library + Playwright provide comprehensive coverage.

**Enforcement**: CI gates to be added. Coverage thresholds enforced via vitest.config.ts.

### Constitution Compliance Summary

**Status**: ✅ **COMPLIANT** (5/5 applicable principles)

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Clean Architecture | ✅ PASS | Three-layer separation explicitly designed |
| II. SOLID Principles | ✅ PASS | Repository pattern, dependency inversion via interfaces |
| III. Shared Domain | ✅ PASS | Domain types reference backend contracts |
| IV. Dependency Injection | ✅ PASS | React Context + Zustand = functional equivalent of ServiceContainer |
| V. Feature Flags | ⚠️ N/A | Not applicable to frontend (single-variant deployment) |
| VI. Test Coverage | ✅ PASS | Standards defined, implementation pending |

**No violations requiring justification in Complexity Tracking.**

## Project Structure

### Documentation (this feature)

```text
specs/001-frontend-mvp/
├── plan.md              # This file (/speckit.plan command output)
├── spec.md              # Feature specification (input)
├── research.md          # Phase 0 output (NOT YET CREATED)
├── data-model.md        # Phase 1 output (NOT YET CREATED)
├── quickstart.md        # Phase 1 output (NOT YET CREATED)
├── contracts/           # Phase 1 output (NOT YET CREATED)
│   └── api-client.ts    # REST API contract definitions
├── tasks.md             # Phase 2 output (/speckit.tasks command - NOT YET CREATED)
└── checklists/
    └── requirements.md  # Specification validation (completed)
```

### Source Code (repository root)

```text
frontend/                          # Next.js 16.0.5 application
├── app/                          # Next.js App Router pages
│   ├── layout.tsx               # Root layout with providers (EXISTS - partial)
│   ├── page.tsx                 # Search page (P1) (EXISTS - partial)
│   ├── index/
│   │   └── page.tsx            # Indexing page (P3) (NEEDS CREATION)
│   └── collections/
│       └── page.tsx            # Collection management (P2 enhancement, optional)
│
├── domain/                       # Domain Layer (Clean Architecture)
│   ├── entities/                # NEEDS CREATION
│   │   ├── Document.ts         # Document entity
│   │   ├── Collection.ts       # Collection entity
│   │   ├── SearchResult.ts     # Search result aggregate
│   │   └── IndexOperation.ts   # Indexing operation entity
│   ├── repositories/            # NEEDS CREATION
│   │   ├── SearchRepository.ts      # Search repository interface
│   │   ├── IndexRepository.ts       # Indexing repository interface
│   │   └── CollectionRepository.ts  # Collection repository interface
│   └── usecases/                # NEEDS CREATION
│       ├── SearchDocumentsUseCase.ts       # P1 use case
│       ├── IndexDocumentsUseCase.ts        # P3 use case
│       └── ManageCollectionsUseCase.ts     # P2 use case
│
├── application/                  # Application Layer
│   ├── hooks/                   # React Query hooks (EXISTS - partial)
│   │   ├── useSearch.ts        # EXISTS (manually created by user)
│   │   ├── useCollections.ts   # EXISTS (manually created by user)
│   │   └── useIndexing.ts      # EXISTS (manually created by user)
│   └── providers/               # React Context providers (EXISTS - status unknown)
│       ├── DependencyContainer.ts  # Zustand DI container (partial)
│       └── QueryProvider.tsx       # React Query setup (partial)
│
├── infrastructure/               # Infrastructure Layer
│   ├── api/                     # NEEDS CREATION
│   │   ├── RestApiClient.ts                 # Base REST API client
│   │   ├── RestApiSearchRepository.ts       # Search API adapter
│   │   ├── RestApiIndexRepository.ts        # Indexing API adapter
│   │   └── RestApiCollectionRepository.ts   # Collection API adapter
│   └── config/                  # NEEDS CREATION
│       └── apiConfig.ts         # API endpoint configuration (reads .env.local)
│
├── presentation/                 # Presentation Layer (UI components)
│   └── components/              # NEEDS CREATION
│       ├── SearchInterface.tsx      # Search UI (P1)
│       ├── SearchResults.tsx        # Results display (P1)
│       ├── CollectionSelector.tsx   # Collection dropdown (P2)
│       ├── IndexingInterface.tsx    # Indexing UI (P3)
│       └── ErrorBoundary.tsx        # Error handling wrapper
│
├── .env.local                   # API configuration (EXISTS)
├── package.json                 # Dependencies (EXISTS - complete)
├── tsconfig.json                # TypeScript config (EXISTS)
├── next.config.ts               # Next.js config (EXISTS)
└── tailwind.config.ts           # Tailwind config (EXISTS - generated)

tests/                           # Test suite (NEEDS CREATION)
├── unit/
│   ├── domain/                 # Use case tests
│   └── infrastructure/         # Repository tests
├── integration/
│   └── api/                    # API contract tests
└── e2e/
    └── user-journeys/          # Playwright E2E tests (P1, P2, P3)
```

**Structure Decision**: Web application structure (Option 2 adapted for frontend-only project). Backend already exists as separate Rust service, so frontend/ is self-contained. Follows hexagonal architecture with domain/application/infrastructure layers per constitution. Next.js App Router used for routing (app/ directory). Presentation layer separated from application logic per Clean Architecture.

**Existing vs Missing**:
- **EXISTS**: Next.js scaffolding, dependencies, 3 hooks (manually created by user), layout.tsx (partial), page.tsx (partial)
- **PARTIAL**: DependencyContainer.ts, QueryProvider.tsx (creation attempted but status unknown)
- **NEEDS CREATION**: All domain entities, repository interfaces, use cases, infrastructure adapters, presentation components, indexing page

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

**No violations** - All constitution principles either satisfied or not applicable. No complexity justification required.

---

## Phase 0: Research & Clarification

**Objective**: Resolve all NEEDS CLARIFICATION items from Technical Context and validate assumptions against constitution.

### Research Topics

#### R1: Next.js 16 App Router Best Practices
- **Question**: How to structure hexagonal architecture in Next.js App Router with proper separation of concerns?
- **Rationale**: Next.js 16 introduced App Router with Server Components. Need patterns for client-side data fetching that align with Clean Architecture.
- **Success Criteria**: Document recommended project structure with domain/application/infrastructure layers compatible with App Router constraints.

#### R2: React Query + Zustand Integration Patterns
- **Question**: What is the idiomatic way to combine React Query (data fetching) with Zustand (DI container)?
- **Rationale**: React Query handles async operations, Zustand stores use case instances. Need clear pattern to avoid redundancy.
- **Success Criteria**: Document DI container pattern that provides use cases to hooks, with React Query handling HTTP lifecycle.

#### R3: TypeScript Interface Design for Repository Pattern
- **Question**: How to design TypeScript interfaces that enforce Liskov Substitution and Interface Segregation?
- **Rationale**: Repository pattern requires strict interface contracts. TypeScript interfaces are structural (duck typing), not nominal.
- **Success Criteria**: Document interface design patterns with examples showing substitutability verification.

#### R4: REST API Error Handling Patterns
- **Question**: How to handle backend errors gracefully with user-friendly messages?
- **Rationale**: Backend returns various error types (connection, 404, 500, validation). Need consistent error handling across all API calls.
- **Success Criteria**: Document error handling pattern with type-safe error mapping and UI display strategies.

#### R5: Testing Strategy for Hexagonal Architecture
- **Question**: How to test each layer independently with proper mocking?
- **Rationale**: Constitution requires >80% unit/integration coverage. Need test structure that validates layer boundaries.
- **Success Criteria**: Document test setup with examples for domain (use case tests), infrastructure (API mocks), and E2E (Playwright).

### Research Outputs

**Deliverable**: `specs/001-frontend-mvp/research.md`

Expected sections:
- Next.js 16 App Router + Hexagonal Architecture patterns
- React Query + Zustand DI integration
- TypeScript repository interface design
- REST API error handling approach
- Testing strategy per layer

---

## Phase 1: Design & Contracts

**Prerequisites**: Phase 0 research complete, all NEEDS CLARIFICATION resolved

### D1: Data Model Definition

**Deliverable**: `specs/001-frontend-mvp/data-model.md`

**Content**:
- **Document entity**: Fields (id, title, path, content, score, collection), validation rules
- **Collection entity**: Fields (name, description, document_count, created_at), validation rules
- **SearchResult aggregate**: Relationship between Document and search metadata
- **IndexOperation entity**: State machine (pending → in_progress → completed/failed), fields
- **Type relationships**: How entities compose and interact

**Success Criteria**: Complete entity definitions with TypeScript interface sketches (not implementation), validation rules documented, relationships mapped.

### D2: API Contract Specification

**Deliverable**: `specs/001-frontend-mvp/contracts/api-client.ts` (TypeScript interface definitions)

**Content**:
- REST API endpoint contracts matching backend API reference:
  - `GET /search?q={query}&collection={name}&limit={n}` → SearchResult[]
  - `GET /collections` → Collection[]
  - `POST /api/index` → IndexOperation
- Request/response type definitions
- Error response types
- HTTP status code mappings
- Backend behavior documentation:
  - Duplicate document indexing behavior (update vs skip)
  - Collection filtering semantics
  - Error response format specifications

**Success Criteria**: Complete TypeScript interfaces covering all backend endpoints used by P1/P2/P3 stories. Matches docs/API_REFERENCE.md exactly.

### D3: Dependency Injection Architecture

**Deliverable**: Section in `data-model.md` covering DI design

**Content**:
- DependencyContainer structure (Zustand store)
- Use case instantiation pattern
- Repository instantiation pattern
- Component access pattern (useDependencyContainer hook)
- Lifecycle management (singleton vs transient)

**Success Criteria**: Clear diagram showing dependency flow from components → hooks → use cases → repositories → API client.

### D4: Quickstart Guide

**Deliverable**: `specs/001-frontend-mvp/quickstart.md`

**Content**:
- Prerequisites (Node.js 18+, backend running on localhost:8081)
- Installation steps (`npm install`, `.env.local` setup)
- Running dev server (`npm run dev`)
- Running tests (`npm test`, `npm run e2e`)
- Building for production (`npm run build`)
- Troubleshooting common issues (backend not running, port conflicts)

**Success Criteria**: A developer can go from fresh checkout to running application in <5 minutes following this guide.

---

## Phase 2: Implementation Roadmap

**Note**: Detailed task breakdown will be generated by `/speckit.tasks` command. This section provides high-level implementation sequence.

### Implementation Sequence (by Priority)

#### Stage 1: Foundation (Blocks all user stories)
1. Complete domain layer (entities, repository interfaces, use cases)
2. Complete infrastructure layer (API client, repository implementations)
3. Complete DependencyContainer setup
4. Verify layer boundaries (import restrictions via tsconfig paths)

#### Stage 2: P1 - Semantic Search (MVP Core)
1. SearchInterface component with input and state management
2. SearchResults component with result display
3. Error handling and loading states
4. E2E test for P1 acceptance scenarios

#### Stage 3: P2 - Collection Filtering (Enhanced Precision)
1. CollectionSelector component (dropdown)
2. Integration with useCollections hook
3. Search filtering by collection
4. E2E test for P2 acceptance scenarios

#### Stage 4: P3 - Indexing (User Convenience)
1. IndexingInterface component (directory picker)
2. Progress tracking UI
3. Integration with useIndexing hook
4. E2E test for P3 acceptance scenarios

#### Stage 5: Polish & Testing
1. Error boundaries and fallback UI
2. Unit tests (>80% coverage)
3. Integration tests (API contracts)
4. Performance validation (success criteria)

### Dependencies & Blockers

**Critical Path**:
- Foundation (Stage 1) blocks all user stories
- P1 must complete before P2 (collection filtering extends search)
- P3 independent of P1/P2 (can develop in parallel after Stage 1)

**External Dependencies**:
- Backend service must be running on localhost:8081
- Backend must implement API as documented in docs/API_REFERENCE.md
- Collections must exist in backend for P2 testing (can be created via CLI)

**Risk Factors**:
- File creation tool failures (historical issue) → Mitigation: User manual creation or code generation to clipboard
- API contract mismatches → Mitigation: Contract tests in Stage 1
- React 19 compatibility issues → Mitigation: Next.js 16 officially supports React 19

---

## Success Validation

### Constitution Compliance Re-Check (Post-Phase 1)

After Phase 1 design complete, re-validate:

- [ ] Clean Architecture: Layer boundaries enforced via TypeScript paths
- [ ] SOLID Principles: Interfaces properly defined with substitutability
- [ ] Shared Domain: Entity types match backend contracts
- [ ] Dependency Injection: DependencyContainer pattern documented
- [ ] Test Coverage: Testing strategy documented with examples

### Specification Success Criteria Mapping

| Success Criterion | Validation Method | Target |
|-------------------|-------------------|--------|
| SC-001: Search completion <5s | E2E test with timer | <5000ms |
| SC-002: Indexing initiation <30s | E2E test with timer | <30000ms |
| SC-003: Setup time ≤2min | Quickstart guide walkthrough | ≤120s |
| SC-004: Data accuracy | Contract tests vs backend | 100% match |
| SC-005: Platform compatibility | Manual test on macOS | Pass |
| SC-006: Zero unhandled rejections | E2E test error monitoring | 0 errors |
| SC-007: UI responsiveness <300ms | Performance.now() measurement | <300ms |
| SC-008: Error handling 100% | Contract tests error scenarios | 100% coverage |

### Ready for `/speckit.tasks` Command

- [x] Phase 0 research topics identified
- [x] Phase 1 deliverables specified
- [x] Implementation sequence defined
- [x] Constitution compliance validated
- [x] Success criteria mapped

**Next Command**: `/speckit.tasks` to generate detailed task breakdown organized by user story and phase.
