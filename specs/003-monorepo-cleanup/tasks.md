# Tasks: Consolidate Monorepo Structure

**Input**: Design documents from `/specs/003-monorepo-cleanup/`
**Prerequisites**: plan.md ✅, spec.md ✅, implementation-steps.md ✅

**Tests**: No test tasks - this is infrastructure migration, verified through build success and functional testing

**Organization**: Tasks grouped by user story (P1: Source consolidation, P1: Docker config, P2: Documentation)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: User story (US1, US2, US3)
- File paths included in descriptions

---

## Phase 1: Setup (Pre-Migration)

**Purpose**: Ensure clean state before migration

- [ ] T001 Stop all Docker containers with `docker-compose down`
- [ ] T002 Verify git status is clean or commit any uncommitted changes in current branch
- [ ] T003 [P] Generate file comparison report between `/frontend/` and `/apps/frontend/` using `diff -r`
- [ ] T004 [P] Document which files in `/frontend/` are newer (identify latest versions to preserve)
- [ ] T005 Create backup of `/frontend/` directory as tar.gz in `/tmp/` for rollback safety

**Checkpoint**: Clean state confirmed, differences documented, backup created

---

## Phase 2: Foundational (Not Applicable)

**This phase typically contains blocking infrastructure that MUST complete before user stories.**

**Status**: ⚠️ **SKIPPED** - No foundational blockers for this migration

**Reason**: This is a pure directory reorganization. There is no shared infrastructure to build before consolidating files. We proceed directly to User Story implementation.

---

## Phase 3: User Story 1 - Single Source of Truth (Priority: P1) 🎯

**Goal**: All frontend source code exists exclusively in `apps/frontend/` with zero duplicates

**Independent Test**: Edit file in `apps/frontend/`, rebuild, verify change appears in running app

### Implementation for User Story 1

- [ ] T006 [P] [US1] Copy latest source from `frontend/app/` to `apps/frontend/app/` using rsync with --delete flag
- [ ] T007 [P] [US1] Copy latest source from `frontend/application/` to `apps/frontend/application/` using rsync with --delete flag
- [ ] T008 [P] [US1] Copy latest source from `frontend/domain/` to `apps/frontend/domain/` using rsync with --delete flag
- [ ] T009 [P] [US1] Copy latest source from `frontend/infrastructure/` to `apps/frontend/infrastructure/` using rsync with --delete flag
- [ ] T010 [P] [US1] Copy latest source from `frontend/presentation/` to `apps/frontend/presentation/` using rsync with --delete flag
- [ ] T011 [P] [US1] Copy `frontend/public/` directory to `apps/frontend/public/` using rsync
- [ ] T012 [US1] Resolve all symlinks in `apps/frontend/` to real files using bash loop with readlink and cp
- [ ] T013 [US1] Copy all config files from `frontend/` to `apps/frontend/` (package.json, tsconfig.json, next.config.ts, Dockerfile, etc.)
- [ ] T014 [US1] Verify zero symlinks remain in `apps/frontend/` using `find apps/frontend/ -type l`
- [ ] T015 [US1] Verify all required files present in `apps/frontend/` (app/indexing/page.tsx, infrastructure/api/RestApiSearchRepository.ts with POST fix, etc.)

**Checkpoint**: Single source location established - `apps/frontend/` contains all latest code, no symlinks

---

## Phase 4: User Story 2 - Docker Build Consistency (Priority: P1) 🎯

**Goal**: Docker builds always use `apps/frontend/` as source, builds succeed on first try

**Independent Test**: Run `docker-compose build --no-cache frontend`, verify no errors, all features work

### Implementation for User Story 2

- [ ] T016 [US2] Update `apps/frontend/Dockerfile` COPY commands to reference `apps/frontend/` paths (COPY apps/frontend/package*.json, COPY apps/frontend/ ., etc.)
- [ ] T017 [US2] Update `docker-compose.yml` frontend service build context to `.` (repo root) and dockerfile to `apps/frontend/Dockerfile`
- [ ] T018 [US2] Update `docker-compose.yml` frontend service volume mounts to reference `./apps/frontend:/app`
- [ ] T019 [P] [US2] Update `apps/frontend/Dockerfile.dev` COPY commands if dev Dockerfile exists
- [ ] T020 [P] [US2] Update `.dockerignore` to reflect new paths if needed
- [ ] T021 [US2] Build frontend container with `docker-compose build --no-cache frontend` to verify build succeeds
- [ ] T022 [US2] Start containers with `docker-compose up -d` and verify both services become healthy within 10 seconds
- [ ] T023 [US2] Test search API with curl POST to http://localhost:8081/api/search and verify results returned
- [ ] T024 [US2] Test indexing API with curl POST to http://localhost:8081/api/index and verify documents processed
- [ ] T025 [US2] Open http://localhost:3000/ in browser and verify search page loads with correct styling
- [ ] T026 [US2] Open http://localhost:3000/indexing in browser and verify indexing page loads with correct styling
- [ ] T027 [US2] Test navigation between pages (click "Index Documents" link) and verify routing works
- [ ] T028 [US2] Verify hot reload: edit `apps/frontend/app/page.tsx`, save, verify change appears in browser within 30 seconds

**Checkpoint**: Docker builds work correctly, all features functional, hot reload working

---

## Phase 5: User Story 3 - Path Reference Updates (Priority: P2)

**Goal**: All documentation and scripts reference `apps/frontend/`, instructions work first time

**Independent Test**: Follow README from scratch, run all Makefile targets, verify no manual path adjustments needed

### Implementation for User Story 3

- [ ] T029 [P] [US3] Update `README.md` - replace all `cd frontend` with `cd apps/frontend`
- [ ] T030 [P] [US3] Update `README.md` - replace all `./frontend/` paths with `./apps/frontend/`
- [ ] T031 [P] [US3] Update `Makefile` targets - replace `cd frontend` with `cd apps/frontend` in all targets
- [ ] T032 [P] [US3] Update `Makefile` targets - replace `./frontend` with `./apps/frontend` in all paths
- [ ] T033 [P] [US3] Update `MONOREPO.md` if exists - document final structure with only `/apps/frontend/`
- [ ] T034 [P] [US3] Search codebase for remaining hardcoded paths with `grep -r "cd frontend" . --exclude-dir={node_modules,.next,target,.git}`
- [ ] T035 [P] [US3] Search codebase for remaining hardcoded paths with `grep -r "./frontend/" . --exclude-dir={node_modules,.next,target,.git}`
- [ ] T036 [US3] Fix any remaining hardcoded paths found in previous searches
- [ ] T037 [US3] Test README instructions: follow setup steps from a fresh terminal and verify they work
- [ ] T038 [US3] Test Makefile targets: run each make target and verify no path errors occur

**Checkpoint**: All documentation accurate, instructions work without manual adjustments

---

## Phase 6: Cleanup & Verification

**Purpose**: Remove `/frontend/` directory and final validation

- [ ] T039 Verify Tasks T001-T038 are all complete and checkpoints passed
- [ ] T040 Create final backup of `/frontend/` directory as tar.gz with timestamp in `/tmp/`
- [ ] T041 Delete `/frontend/` directory using `git rm -rf frontend/`
- [ ] T042 Verify deletion: `ls -la | grep frontend` should show only `apps/` (no `/frontend/` directory)
- [ ] T043 [P] Run full clean build: `docker-compose down -v && docker-compose build --no-cache`
- [ ] T044 [P] Verify build completes without errors and both services start successfully
- [ ] T045 Test search functionality end-to-end in browser
- [ ] T046 Test indexing functionality end-to-end in browser
- [ ] T047 Test navigation between search and indexing pages
- [ ] T048 Verify input text is readable (dark text on light background from globals.css fix)
- [ ] T049 Measure build time and confirm it's ≤ previous build times (no regression)
- [ ] T050 Verify hot reload still works: edit file, save, see change within 30 seconds

**Checkpoint**: `/frontend/` deleted, all features working, build successful

---

## Phase 7: Documentation & Commit

**Purpose**: Document changes and commit to branch

- [ ] T051 Review all changes with `git status` and `git diff`
- [ ] T052 Stage all changes with `git add apps/frontend/ docker-compose.yml README.md Makefile MONOREPO.md`
- [ ] T053 Create migration documentation in `specs/003-monorepo-cleanup/migration.md` documenting before/after state, steps taken, verification results
- [ ] T054 Commit changes with descriptive message including problem statement, changes made, verification status
- [ ] T055 Verify commit with `git log --oneline -1` and `git show --stat`
- [ ] T056 Push branch to remote: `git push origin 003-monorepo-cleanup`
- [ ] T057 Create pull request with summary of changes and link to specification
- [ ] T058 Request code review from team

**Checkpoint**: Changes committed, documented, ready for review and merge

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup)
  ↓
Phase 2 (Foundational) - SKIPPED
  ↓
Phase 3 (US1: Single Source) ← MUST COMPLETE FIRST
  ↓
Phase 4 (US2: Docker Config) ← Depends on US1 completion
  ↓
Phase 5 (US3: Documentation) ← Can start after US2, but better after testing
  ↓
Phase 6 (Cleanup) ← Depends on US1, US2, US3 all complete
  ↓
Phase 7 (Commit) ← Depends on successful cleanup verification
```

### Critical Path

**MUST BE SEQUENTIAL** (cannot parallelize):
1. Phase 1: Setup → 2. Phase 3: Consolidate source → 3. Phase 4: Update Docker → 4. Phase 6: Delete & verify → 5. Phase 7: Commit

**Why sequential**: Each phase depends on previous phase's output. Files must be copied before Docker can reference them. Docker must work before documentation can be verified. Everything must work before deletion and commit.

### Within-Phase Parallelization

**Phase 1 (Setup)**:
- T003 and T004 can run in parallel (different operations)

**Phase 3 (User Story 1)**:
- T006-T011 can ALL run in parallel (copying different directories)
- T012 must wait for T006-T011 (needs files to exist)
- T013 must wait for T012 (resolving symlinks first)

**Phase 4 (User Story 2)**:
- T019 and T020 can run in parallel with T016-T018 (different files)
- T021-T028 must be sequential (build before start, start before test)

**Phase 5 (User Story 3)**:
- T029-T035 can ALL run in parallel (editing different files)
- T036 must wait for T034-T035 (needs search results)

**Phase 6 (Cleanup)**:
- T043 and T044 can overlap (build initiates startup)
- T045-T048 can run in parallel (testing different features)

### Task Time Estimates

- **Phase 1**: 10 minutes (pre-flight checks)
- **Phase 3**: 15 minutes (file consolidation)
- **Phase 4**: 45 minutes (Docker updates + verification)
- **Phase 5**: 20 minutes (documentation updates)
- **Phase 6**: 30 minutes (cleanup + full verification)
- **Phase 7**: 15 minutes (commit + documentation)

**Total**: ~2 hours 15 minutes (including buffer)

---

## Parallel Execution Examples

### Phase 3: Copy All Directories Simultaneously

```bash
# Launch all directory copies in parallel:
rsync -av --delete frontend/app/ apps/frontend/app/ &
rsync -av --delete frontend/application/ apps/frontend/application/ &
rsync -av --delete frontend/domain/ apps/frontend/domain/ &
rsync -av --delete frontend/infrastructure/ apps/frontend/infrastructure/ &
rsync -av --delete frontend/presentation/ apps/frontend/presentation/ &
wait  # Wait for all copies to complete
```

### Phase 5: Update All Documentation Simultaneously

```bash
# Launch all sed replacements in parallel:
sed -i '' 's|cd frontend|cd apps/frontend|g' README.md &
sed -i '' 's|cd frontend|cd apps/frontend|g' Makefile &
sed -i '' 's|/frontend/|/apps/frontend/|g' MONOREPO.md &
wait  # Wait for all updates to complete
```

---

## Implementation Strategy

### ⚡ Recommended: Single-Pass Sequential (2 hours)

This is infrastructure work that MUST be done correctly the first time. Sequential execution reduces risk:

1. **Phase 1**: Setup and verification (10 min)
2. **Phase 3**: Consolidate source to apps/frontend/ (15 min)
3. **Phase 4**: Update Docker and verify builds (45 min) ← CRITICAL TESTING
4. **Phase 5**: Update documentation (20 min)
5. **Phase 6**: Delete frontend/ and final verification (30 min) ← CRITICAL TESTING
6. **Phase 7**: Commit and document (15 min)

**Why sequential**:
- Docker changes depend on files being in place
- Deletion must happen AFTER everything works
- Testing gates prevent broken state from being committed

### 🚫 Not Recommended: Parallel execution

While some tasks CAN run in parallel, the risk of missing a dependency or breaking the build is too high for infrastructure changes. The time savings (~20 minutes) don't justify the risk.

---

## Rollback Plan

If verification fails at any checkpoint:

```bash
# Restore from backup
tar -xzf /tmp/frontend-backup-*.tar.gz

# Revert Docker changes
git checkout docker-compose.yml apps/frontend/Dockerfile

# Revert documentation
git checkout README.md Makefile MONOREPO.md

# Rebuild with original structure
docker-compose down -v
docker-compose build
docker-compose up -d
```

---

## Success Verification Checklist

After completion, all success criteria must be met:

- ✅ **SC-001**: Edit `apps/frontend/app/page.tsx` → rebuild → see changes in <30s
- ✅ **SC-002**: `find . -path "./frontend/app" -type d` returns nothing (zero duplicates)
- ✅ **SC-003**: `docker-compose build frontend` succeeds on first try
- ✅ **SC-004**: Search, indexing, navigation all work identically to before migration
- ✅ **SC-005**: README instructions work without mentioning `/frontend/`
- ✅ **SC-006**: Build time measured and ≤ previous (use `time docker-compose build frontend`)

---

## Notes

- **[P] markers**: Indicate parallelizable tasks within a phase (not cross-phase)
- **[Story] labels**: Map tasks to user stories for traceability
- **No tests**: Infrastructure migration verified through build success + functional testing
- **Sequential critical path**: Cannot parallelize phases - each depends on previous
- **Checkpoints**: Each phase has verification checkpoint before proceeding
- **Rollback ready**: Backup created before any destructive operations
- **~2 hours total**: Time estimate includes testing and verification
- **Zero downtime**: Development environment only - no production impact
