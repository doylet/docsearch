## Why

GitHub Actions has failed on every push to `main` since at least 2025-11-28, so CI provides no signal: a broken change looks the same as a good one. Now that `restore-project-health` makes the test suite pass offline, CI is the last piece needed for a trustworthy baseline before feature work (PDF ingestion, the MCP server) lands.

The failures stack up, so fixing the first one only reveals the next:

1. `cargo fmt --check` fails, with about 500 formatting diffs.
2. `cargo clippy -D warnings` would fail, with about 440 warnings, roughly 70% of them dead code.
3. `--all-features` doesn't compile. The `tantivy` feature of `zero-latency-search` and the `cloud` feature of `doc-indexer` have drifted from the code around them.
4. `cargo-deny` runs with no `deny.toml`, so its default license policy rejects dependencies.
5. The build job uploads `target/release/docsearch`, a binary that doesn't exist (the binaries are `doc-indexer` and `mdx`).
6. The deploy jobs need Kubernetes secrets and GitHub environments that were never configured.
7. The frontend isn't checked in CI at all.

## What Changes

- Apply `cargo fmt` to the whole workspace in one commit, recorded in `.git-blame-ignore-revs`.
- Clear all non-dead-code clippy warnings (`cargo clippy --fix` plus hand fixes). Allow `dead_code` in CI until a dedicated cleanup removes the unused code.
- Test the features that compile (the defaults) instead of `--all-features`. The broken `tantivy` and `cloud` features are excluded and tracked, not silently ignored.
- Add a `deny.toml` with a license allowlist and advisory policy. Replace `cargo install cargo-audit` with a maintained action.
- Fix the build artifact paths.
- Add a frontend job: `npm ci`, type-check, build, and `npm audit --omit=dev --audit-level=high`.
- Make the Docker push and deploy jobs opt-in through a repository variable, so they don't fail on unconfigured infrastructure. **BREAKING:** pushes to `main` stop publishing `ghcr.io/.../docsearch:latest` until the variable is set. No image is being published today anyway.
- Drop the `beta` toolchain from the test matrix.
- Confirm `schema-validation.yml` passes now that a root `package-lock.json` exists.

## Capabilities

### New Capabilities
- `ci-pipeline`: What CI checks on every push and pull request, and the guarantee that it passes on `main`.

### Modified Capabilities
<!-- None. `build-health` (from restore-project-health) covers local guarantees; this change adds the CI side. -->

## Impact

- **Workflows:** `.github/workflows/ci-cd.yml`, `.github/workflows/schema-validation.yml`.
- **Code:** formatting across every crate, and clippy fixes across the workspace. No behaviour changes are intended.
- **New files:** `deny.toml`, `.git-blame-ignore-revs`.
- **Depends on:** `restore-project-health` merged first. The formatting commit touches nearly every file, so merge order matters for `finish-pdf-and-browse`, which will need a rebase.
- **Out of scope:** removing dead code; fixing or deleting the `tantivy` and `cloud` features; real deployment infrastructure (Linear project "Cloud Infrastructure"); running the `--ignored` smoke tests in CI (blocked on the collection-scoping bug).
