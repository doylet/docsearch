# Feature Specification: Consolidate Monorepo Structure

**Feature Branch**: `003-monorepo-cleanup`
**Created**: 2025-12-05
**Status**: Draft
**Input**: User description: "Move everything into apps/ and delete frontend/ to fix duplicate source code causing build confusion"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Single Source of Truth (Priority: P1)

As a developer, I want all frontend source code to exist in exactly one location (`apps/frontend/`), so that when I edit a file, the changes are immediately reflected in builds without confusion about which copy is being used.

**Why this priority**: This is blocking all development work. Currently editing `/frontend/app/page.tsx` while Docker builds from `/apps/frontend/app/page.tsx` causes constant confusion and wasted time. This must be fixed before any other work can proceed.

**Independent Test**: Can be fully tested by editing a source file in `apps/frontend/`, running `docker-compose build frontend`, and verifying the changes appear in the running container. Delivers immediate value by eliminating duplicate file confusion.

**Acceptance Scenarios**:

1. **Given** I edit `apps/frontend/app/page.tsx`, **When** I rebuild the frontend container, **Then** my changes appear in the running application
2. **Given** the migration is complete, **When** I search for `frontend/app/page.tsx`, **Then** only one file exists in `apps/frontend/app/page.tsx`
3. **Given** the old `/frontend/` directory is removed, **When** I run `docker-compose up`, **Then** services start successfully without errors
4. **Given** I am new to the codebase, **When** I read the project structure, **Then** there is one clear location for frontend code in `apps/`

---

### User Story 2 - Docker Build Consistency (Priority: P1)

As a developer, I want Docker builds to always use the source from `apps/frontend/`, so that I don't waste time debugging why my changes aren't appearing in the running application.

**Why this priority**: Critical blocker. The current hybrid symlink/copy approach means developers can't trust that their edits will be built. This breaks the basic development workflow.

**Independent Test**: Can be tested by modifying `docker-compose.yml` and Dockerfiles to point to `apps/frontend/`, then running a full build and verifying all files are correctly included.

**Acceptance Scenarios**:

1. **Given** the Dockerfile references `apps/frontend/`, **When** I run `docker-compose build frontend`, **Then** the build completes without "file not found" errors
2. **Given** I have made changes to any file in `apps/frontend/`, **When** I rebuild and restart containers, **Then** all changes are reflected in the running app
3. **Given** the build is running, **When** Docker copies source files, **Then** no symlinks cause build failures or warnings
4. **Given** I run `docker-compose up` after migration, **When** containers start, **Then** both backend and frontend are healthy within 10 seconds

---

### User Story 3 - Path Reference Updates (Priority: P2)

As a developer, I want all documentation, scripts, and configuration files to reference the correct `apps/` paths, so that instructions work the first time without manual path adjustments.

**Why this priority**: Important for maintainability. While the app will work with just P1 and P2, developers will be confused by outdated docs and scripts referencing the old `/frontend/` path.

**Independent Test**: Can be tested by following README instructions and running all Makefile targets to verify they work with new paths.

**Acceptance Scenarios**:

1. **Given** the README mentions frontend paths, **When** I follow the instructions, **Then** all paths reference `apps/frontend/` correctly
2. **Given** I run `make` targets, **When** executing build/test/deploy commands, **Then** all scripts work without manual path corrections
3. **Given** I review docker-compose.yml, **When** checking volume mounts, **Then** all mounts reference `apps/frontend/` paths
4. **Given** a new developer clones the repo, **When** they follow setup instructions, **Then** they never encounter references to the deleted `/frontend/` directory

---

### Edge Cases

- What happens to existing Git history referencing `/frontend/` paths?
- How do we handle any hardcoded paths in build artifacts or cache files?
- What if there are uncommitted changes in either `/frontend/` or `/apps/frontend/` during migration?
- How do we ensure symlinks don't cause issues during the file move operations?
- What happens if developers have the old `/frontend/` directory open in their IDE during migration?
- How do we handle any absolute paths in configuration files that reference `/frontend/`?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST have all frontend source code located exclusively in `apps/frontend/` directory
- **FR-002**: System MUST remove the `/frontend/` directory entirely after migrating source code
- **FR-003**: System MUST update all Dockerfile references to use `apps/frontend/` as the source path
- **FR-004**: System MUST update docker-compose.yml to reference `apps/frontend/` for all frontend volume mounts
- **FR-005**: System MUST preserve all existing functionality (search, indexing, navigation) after migration
- **FR-006**: System MUST maintain all existing source code structure (Clean Architecture folders: app/, application/, domain/, infrastructure/, presentation/)
- **FR-007**: System MUST update README.md to reference correct `apps/frontend/` paths in all instructions
- **FR-008**: System MUST update Makefile targets to use `apps/frontend/` paths
- **FR-009**: System MUST verify no symlinks remain that point to the deleted `/frontend/` directory
- **FR-010**: System MUST preserve Git history by using `git mv` operations where possible
- **FR-011**: System MUST ensure Docker builds complete successfully with the new directory structure
- **FR-012**: System MUST ensure all TypeScript imports continue to work with the new structure

### Key Entities

- **Frontend Source Directory**: The canonical location of all frontend source code, configuration files, and assets - will be `apps/frontend/` after migration
- **Docker Build Context**: The directory Docker uses as the base for COPY operations - must be updated to reference `apps/frontend/`
- **Symlink**: File system pointer that currently creates hybrid structure - all symlinks pointing to `/frontend/` must be resolved or removed

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can edit any file in `apps/frontend/` and see changes in under 30 seconds after rebuild (no confusion about which file to edit)
- **SC-002**: Zero duplicate source files exist after migration (verified by checking for both `frontend/app/page.tsx` and `apps/frontend/app/page.tsx`)
- **SC-003**: All Docker builds complete successfully on first try after migration without path-related errors
- **SC-004**: All existing features (search, indexing, navigation) work identically before and after migration
- **SC-005**: New developers can clone, build, and run the project by following README instructions without encountering old `/frontend/` references
- **SC-006**: Build time remains the same or improves (no performance regression from directory structure change)

## Assumptions

- Docker and docker-compose are already installed and working
- Current services can be safely stopped during migration
- The existing `apps/` directory structure is correct and should be preserved
- All developers are aware that `/frontend/` will be deleted and should commit any local changes first
- The monorepo structure with `apps/` is the desired long-term architecture

## Out of Scope

- Renaming `apps/` to a different directory name
- Restructuring the Clean Architecture folder layout (app/, domain/, etc.)
- Adding additional services to the monorepo
- Implementing new features or fixing existing bugs unrelated to the directory structure
- Changing build tools or Docker configuration beyond path updates
- Modifying the backend service structure

## Technical Constraints

- Git must preserve history during file moves
- Docker build context cannot reference paths outside the build directory
- Symlinks must be resolved to real files before Docker COPY operations
- All TypeScript path aliases (@/application, @/domain, etc.) must continue working
- Volume mounts in docker-compose must point to real directories, not symlinks

## Dependencies

- Existing docker-compose.yml configuration
- Current Dockerfile setup for frontend
- Makefile targets that reference frontend paths
- README.md setup instructions
- TypeScript configuration (tsconfig.json) with path mappings
