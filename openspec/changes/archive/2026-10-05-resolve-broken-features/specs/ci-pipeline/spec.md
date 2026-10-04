## MODIFIED Requirements

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
