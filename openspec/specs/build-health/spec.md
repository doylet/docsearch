# build-health Specification

## Purpose
Baseline guarantees for the repository: the workspace test suite compiles and passes offline, server-dependent tests are opt-in, frontend production dependencies carry no severe advisories, and documentation links resolve.
## Requirements
### Requirement: Workspace tests compile and pass offline
The repository SHALL compile every test target in the Cargo workspace, and `cargo test --workspace` SHALL pass on a machine with no doc-indexer server, vector database or network services running.

#### Scenario: Clean offline test run
- **WHEN** a developer runs `cargo test --workspace` on a fresh checkout with no services running
- **THEN** every test target compiles
- **AND** the command exits with status 0

#### Scenario: No tests target removed library APIs
- **WHEN** the workspace is built with `--all-targets`
- **THEN** no target fails with an unresolved import or a missing method

### Requirement: Server-dependent tests are opt-in
Tests that need a running doc-indexer server SHALL be marked ignored with a reason, and SHALL run only when explicitly requested.

#### Scenario: Default run skips server tests
- **WHEN** `cargo test -p doc-indexer` runs with no server on port 19000
- **THEN** the smoke and filtering integration tests are reported as ignored, not failed

#### Scenario: Explicit run executes server tests
- **WHEN** a developer starts doc-indexer on port 19000 and runs `cargo test -p doc-indexer -- --ignored`
- **THEN** the server-dependent tests execute against the running server

### Requirement: Frontend production dependencies are free of severe advisories
The frontend's production dependency tree SHALL have no known critical or high severity advisories at the time of merge.

#### Scenario: Audit passes
- **WHEN** `npm audit --omit=dev --audit-level=high` runs in `apps/frontend`
- **THEN** it exits with status 0

#### Scenario: Frontend still builds
- **WHEN** `npx tsc --noEmit` and `npm run build` run in `apps/frontend` after the upgrade
- **THEN** both succeed

### Requirement: Documentation links resolve
Every relative link in the root `README.md` SHALL point to a file or directory that exists in the repository.

#### Scenario: README link check
- **WHEN** each relative Markdown link in `README.md` is resolved against the repository root
- **THEN** every target path exists
