## Why

`fix-ci-pipeline` made clippy enforce every lint except `dead_code`, which CI allows with `-A dead_code` on the command line. That allowance was meant to be temporary. Until it goes, unused code keeps piling up unnoticed, and about 260 unused items make the codebase harder to read and change: you can't tell which modules are live.

Measured on 2026-10-04 (`cargo clippy --workspace --all-targets`, default features), almost all of the dead code is in `doc-indexer`:

| Area | Sites |
|---|---|
| `infrastructure/operations` (including `production/`) | 39 |
| `infrastructure/load_testing` | 39 |
| `infrastructure/memory` | 27 |
| `infrastructure/batch_operations.rs` | 20 |
| `infrastructure/enhanced_api.rs` | 18 |
| `infrastructure/collection_management.rs` | 15 |
| `application/adapters.rs` | 14 |
| `infrastructure/enhanced_search.rs` | 13 |
| `application/interfaces.rs` | 11 |
| Other | about 64 |

Some of these modules may be scaffolding meant for future use (load testing, production operations). Others are superseded implementations.

## What Changes

- Triage each area as **delete** (superseded or abandoned), **wire up** (meant to be used, with a small gap), or **keep** (deliberate future work, kept with a targeted, commented `#[allow(dead_code)]` on the module).
- Delete the code marked for deletion, along with its tests and any dependencies only it uses.
- Remove `-A dead_code` from the clippy step in `.github/workflows/ci-cd.yml`, so CI denies dead code from then on.
- Revisit the `#[allow]`s added in `fix-ci-pipeline` that point at this cleanup, such as `only_used_in_recursion` in `load_testing/runner.rs`, where `timeout` is never applied.

## Capabilities

### New Capabilities
<!-- None. -->

### Modified Capabilities
- `ci-pipeline`: the lint requirement changes from "warnings are errors except `dead_code`" to "all warnings are errors".

## Impact

- **Code:** mostly deletions in `services/doc-indexer/src/infrastructure` and `application`. No behaviour change for the reachable server, CLI or API surface.
- **Dependencies:** some may become unused (for example, crates only `load_testing` or `operations/production` use). Re-run `cargo deny check` and remove them.
- **Workflows:** a one-line change to `ci-cd.yml`.
- **Risk:** deleting something that is reached dynamically or through a feature flag. Mitigation: check each deletion with `cargo check` on the default features and on the `embedded` feature, and keep `cloud`/`tantivy` code until `resolve-broken-features` decides their fate.
- **Order:** after `fix-ci-pipeline`. Prefer landing it before large feature branches, or after them, to limit conflicts.
