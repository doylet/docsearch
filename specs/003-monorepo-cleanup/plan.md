# Implementation Plan: Consolidate Monorepo Structure

**Branch**: `003-monorepo-cleanup` | **Date**: 2025-12-05 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-monorepo-cleanup/spec.md`

## Summary

**Problem**: Duplicate frontend source code exists in both `/frontend/` and `/apps/frontend/` directories. The hybrid symlink/copy structure causes developer confusion where edits to `/frontend/` don't appear in Docker builds that use `/apps/frontend/`. This is a P1 blocker preventing all development work.

**Solution**: Consolidate all frontend source code into `apps/frontend/` as the single source of truth. Delete the `/frontend/` directory entirely. Update all Docker configurations, documentation, and scripts to reference the new location.

**Technical Approach**:
1. Copy latest changes from `/frontend/` to `/apps/frontend/` (overwrite with most recent versions)
2. Resolve all symlinks in `/apps/frontend/` to real files
3. Update Docker build contexts and COPY commands to use `apps/frontend/`
4. Update docker-compose.yml volume mounts
5. Update README, Makefile, and documentation
6. Delete `/frontend/` directory
7. Verify builds and functionality

## Technical Context

**Language/Version**: TypeScript 5.x / Next.js 15 (frontend), Rust 1.75 (backend)
**Primary Dependencies**: Next.js, React, Tailwind CSS, Docker, docker-compose
**Storage**: Browser localStorage (for future features), Backend uses SQLite/Qdrant
**Testing**: Vitest for frontend unit tests, Playwright for E2E tests
**Target Platform**: Docker containers (development), Browser (runtime)
**Project Type**: Web application with monorepo structure
**Performance Goals**: <30 seconds rebuild time, <2 second hot reload
**Constraints**:
- Must preserve all existing functionality (search, indexing)
- Must maintain Clean Architecture structure
- Zero downtime migration (dev environment)
- Build time cannot regress
**Scale/Scope**:
- ~50 frontend source files to consolidate
- 2 Docker services (backend, frontend)
- ~1000 LOC in documentation to update

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### ✅ Clean Architecture (NON-NEGOTIABLE)

**Status**: COMPLIANT - No changes to architecture layers

The monorepo consolidation is purely a directory reorganization. All Clean Architecture layers remain intact:
- Application Layer: `apps/frontend/application/` (hooks, providers, use cases)
- Domain Layer: `apps/frontend/domain/` (entities, repositories, use cases)
- Infrastructure Layer: `apps/frontend/infrastructure/` (API clients, config)
- Presentation Layer: `apps/frontend/presentation/` (components, pages)

No layer boundaries are affected by moving the parent directory.

### ✅ SOLID Principles Compliance

**Status**: COMPLIANT - No changes to component design

Directory consolidation does not affect SOLID principles. All existing components maintain:
- Single Responsibility (one reason to change)
- Open/Closed (extension via composition)
- Liskov Substitution (implementations via interfaces)
- Interface Segregation (focused interfaces)
- Dependency Inversion (depend on abstractions)

### ✅ Shared Domain Crates

**Status**: COMPLIANT - Frontend follows similar patterns

Frontend uses shared infrastructure patterns:
- `infrastructure/api/` - API client abstractions
- `infrastructure/config/` - Configuration management
- `domain/repositories/` - Repository interfaces

Backend Rust crates (`zero-latency-*`) are unaffected by frontend directory changes.

### ✅ Dependency Injection via ServiceContainer

**Status**: COMPLIANT - Frontend uses DependencyContainer

Frontend follows similar DI patterns:
- `application/providers/DependencyContainer.ts` - Manages dependencies
- React hooks inject repositories via container
- No direct instantiation in business logic

Directory move preserves all DI patterns.

### ✅ Test Coverage Standards

**Status**: COMPLIANT - Tests move with source code

- Unit tests: `apps/frontend/` will contain all Vitest tests
- Integration tests: Playwright config moves with tests
- Coverage requirements unchanged (>80%)
- CI pipeline continues to enforce coverage gates

### ✅ Development Workflow

**Status**: COMPLIANT - Using feature branch `003-monorepo-cleanup`

- Feature branch created following `###-feature-name` pattern
- Pull request will be required for merge to main
- Code review will verify constitution compliance
- Main branch remains protected

### Constitution Compliance Summary

**Overall Status**: ✅ **FULLY COMPLIANT**

This is a **pure infrastructure change** with zero impact on:
- Architecture patterns
- Design principles
- Testing strategy
- Development workflow

The consolidation fixes a technical debt issue (duplicate source directories) without compromising any constitutional principles. This aligns with the constitution's emphasis on maintainability and clear project structure.

## Project Structure

### Documentation (this feature)

```text
specs/003-monorepo-cleanup/
├── spec.md              # Feature specification (complete)
├── plan.md              # This file (implementation plan)
├── research.md          # Phase 0 - Investigation findings
├── migration.md         # Phase 1 - Step-by-step migration guide
└── checklists/
    └── requirements.md  # Specification quality checklist (complete)
```

### Source Code (repository root)

**Current State (PROBLEMATIC)**:
```text
/frontend/                          # ❌ Original location (edits here)
├── app/
│   ├── globals.css                # Most recent version
│   ├── layout.tsx
│   ├── page.tsx
│   └── indexing/
│       └── page.tsx               # Has recent fixes
├── application/
│   ├── hooks/
│   └── providers/
├── domain/
│   ├── entities/
│   ├── repositories/
│   └── usecases/
├── infrastructure/
│   ├── api/
│   │   └── RestApiSearchRepository.ts  # Has recent POST fix
│   └── config/
├── presentation/
│   └── components/
├── package.json                   # Real file
├── Dockerfile                     # Real file
└── [all other configs]

/apps/frontend/                     # ❌ Docker builds from here
├── app/
│   ├── globals.css                # Older version
│   ├── layout.tsx
│   └── page.tsx                   # Missing indexing page
├── application/                   # Copied from frontend/
├── domain/                        # Copied from frontend/
├── infrastructure/                # Copied from frontend/
├── presentation/                  # Copied from frontend/
├── Dockerfile -> ../../frontend/Dockerfile      # Symlink
├── package.json -> ../../frontend/package.json  # Symlink
└── [other configs as symlinks]
```

**Target State (CORRECT)**:
```text
/apps/frontend/                     # ✅ Single source of truth
├── app/
│   ├── globals.css                # Latest version
│   ├── layout.tsx
│   ├── page.tsx
│   └── indexing/
│       └── page.tsx               # Migrated from /frontend/
├── application/
│   ├── hooks/
│   │   ├── useCollections.ts
│   │   ├── useIndexing.ts
│   │   ├── useKeyboardShortcut.ts
│   │   └── useSearch.ts
│   └── providers/
│       ├── DependencyContainer.ts
│       └── QueryProvider.tsx
├── domain/
│   ├── entities/
│   │   ├── Collection.ts
│   │   ├── Document.ts
│   │   ├── IndexOperation.ts
│   │   └── SearchResult.ts
│   ├── repositories/
│   │   ├── CollectionRepository.ts
│   │   ├── IndexRepository.ts
│   │   └── SearchRepository.ts
│   └── usecases/
│       ├── IndexDocumentsUseCase.ts
│       ├── ManageCollectionsUseCase.ts
│       └── SearchDocumentsUseCase.ts
├── infrastructure/
│   ├── api/
│   │   ├── RestApiClient.ts
│   │   ├── RestApiCollectionRepository.ts
│   │   ├── RestApiIndexRepository.ts
│   │   └── RestApiSearchRepository.ts    # Latest POST version
│   └── config/
│       └── apiConfig.ts
├── presentation/
│   └── components/
│       ├── CollectionSelector.tsx
│       ├── EmptyState.tsx
│       ├── ErrorBoundary.tsx
│       ├── IndexingInterface.tsx
│       ├── LoadingSpinner.tsx
│       ├── Navigation.tsx
│       ├── SearchInterface.tsx
│       └── SearchResults.tsx
├── public/
├── .next/                         # Build output
├── package.json                   # Real file (not symlink)
├── package-lock.json              # Real file
├── Dockerfile                     # Real file
├── Dockerfile.dev                 # Real file
├── tsconfig.json                  # Real file
├── next.config.ts                 # Real file
├── tailwind.config.ts             # Real file
└── [all other configs as real files]

/frontend/                          # ❌ DELETED
```

**Structure Decision**:

We are consolidating to **Option 2: Web application** monorepo structure with backend and frontend under `apps/`:

```text
/apps/
├── backend/                       # Rust backend (unchanged)
│   └── package.json
└── frontend/                      # TypeScript/Next.js frontend (consolidated here)
    └── [full source structure above]
```

This aligns with modern monorepo practices where all applications live under `/apps/` directory. The root `/frontend/` was a legacy location that should have been moved during initial monorepo setup but was partially copied instead, creating the current duplication problem.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
