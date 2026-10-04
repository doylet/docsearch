# ci-pipeline Specification

## Purpose
What CI checks on every push and pull request: formatting, lints (except dead code), tests on supported features, dependency audits, and the frontend build. It also guarantees that every check can fail and that CI passes on `main`. Image publishing and deploys are opt-in.
## Requirements
### Requirement: CI passes on the main branch
Every required CI job SHALL pass on the head of `main`, and every check that runs SHALL be able to fail. No step may swallow its own failure with `|| true`, `|| echo`, or similar.

#### Scenario: Push to main
- **WHEN** a commit is pushed to `main`
- **THEN** the CI/CD Pipeline and Schema Validation workflows finish with every job successful or skipped by design

#### Scenario: Failing checks are reported
- **WHEN** a pull request introduces a failing unit test
- **THEN** the Rust test job fails and the pull request shows a failed check

### Requirement: Formatting is enforced
CI SHALL fail when any Rust source file in the workspace is not formatted according to `cargo fmt`.

#### Scenario: Unformatted change
- **WHEN** a pull request adds a Rust file that `cargo fmt --check` would change
- **THEN** the formatting step fails

### Requirement: Lints are enforced except for dead code
CI SHALL run clippy on all workspace targets with warnings treated as errors, allowing only the `dead_code` lint.

#### Scenario: New lint violation
- **WHEN** a pull request introduces a clippy warning other than `dead_code`, for example a needless borrow
- **THEN** the clippy step fails

#### Scenario: Existing dead code
- **WHEN** clippy runs on the current workspace with unused items present
- **THEN** the clippy step passes

### Requirement: Tests run on supported features
CI SHALL lint and test the workspace with every Cargo feature enabled (`--all-features`). Every feature declared in a workspace crate SHALL compile, pass clippy with the CI lint set, and pass its tests. The workflow file SHALL NOT list features excluded from CI. The release build MAY use default features, because that is what ships.

#### Scenario: All-features test run
- **WHEN** the Rust test job runs
- **THEN** it executes `cargo test --workspace --all-features` and passes

#### Scenario: All-features lint run
- **WHEN** the Rust test job runs clippy
- **THEN** it executes `cargo clippy --workspace --all-targets --all-features -- -D warnings -A dead_code` and passes

#### Scenario: Feature-only breakage is caught
- **WHEN** a pull request breaks code that compiles only with a non-default feature, for example under `#[cfg(feature = "cloud")]`
- **THEN** the Rust test job fails

#### Scenario: No excluded features
- **WHEN** someone reads `ci-cd.yml`
- **THEN** it does not name any feature as excluded from CI

### Requirement: Dependencies are audited
CI SHALL check Rust dependencies for known advisories and license policy using a checked-in `deny.toml`, over the dependency graph with all features enabled, and SHALL check the frontend's production dependencies for critical or high advisories.

#### Scenario: Rust advisory found
- **WHEN** a dependency with an unignored RustSec advisory is added
- **THEN** the security job fails

#### Scenario: Advisory in an optional dependency
- **WHEN** a dependency enabled only by a non-default feature has an unignored RustSec advisory
- **THEN** the security job fails

#### Scenario: Disallowed license
- **WHEN** a dependency with a license not on the `deny.toml` allowlist is added
- **THEN** the security job fails

#### Scenario: Frontend advisory
- **WHEN** `npm audit --omit=dev --audit-level=high` reports an advisory
- **THEN** the frontend job fails

### Requirement: Frontend is checked
CI SHALL install frontend dependencies from the lockfile, type-check the frontend, and build it.

#### Scenario: Type error
- **WHEN** a pull request introduces a TypeScript type error in `apps/frontend`
- **THEN** the frontend job fails

### Requirement: Publishing and deploys are opt-in
CI SHALL push container images and run deploy jobs only when the repository variable `DEPLOY_ENABLED` is `true`. With the variable unset, those jobs SHALL be skipped, not failed. The Docker image SHALL still be built, without pushing, on pull requests.

#### Scenario: Variable unset
- **WHEN** a commit is pushed to `main` and `DEPLOY_ENABLED` is unset
- **THEN** the image push and deploy jobs are skipped
- **AND** the workflow does not fail because of them

#### Scenario: Dockerfile broken
- **WHEN** a pull request breaks the Dockerfile
- **THEN** the Docker build job fails

### Requirement: Build artifacts exist
The build job SHALL upload the release binaries the workspace actually produces.

#### Scenario: Release build
- **WHEN** the build job completes
- **THEN** the uploaded artifact contains the `doc-indexer` and `mdx` binaries

### Requirement: Real embedding model is tested
CI SHALL run the ONNX embedding tests against the real bge-small-en-v1.5 model in a separate job. That job SHALL restore the model directory from a cache keyed on the pinned model revision, fetch it with `doc-indexer --fetch-model` on a cache miss, and run the ignored ONNX tests. The main test job SHALL NOT need network access or model files.

#### Scenario: Cache hit
- **WHEN** the model-tests job runs and the cache holds the pinned revision
- **THEN** no model download happens, and the ONNX tests run and must pass

#### Scenario: Main job stays offline
- **WHEN** the main Rust test job runs `cargo test --workspace --all-features`
- **THEN** no test downloads model files or contacts `huggingface.co`

#### Scenario: Model behaviour regresses
- **WHEN** a change makes the related-text ranking scenario fail
- **THEN** the model-tests job fails
