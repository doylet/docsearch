## Why

The project has been dormant since December 2025. The core still compiles and all 110 unit tests pass, but `cargo test --workspace` fails because three integration-test targets no longer compile. The frontend also carries a critical Next.js advisory and three high-severity advisories. A half-finished docs reorganisation is sitting uncommitted in the working tree. All of this needs fixing before new feature work (PDF ingestion, the MCP server) can start from a trustworthy baseline.

## What Changes

- Commit the pending docs reorganisation (root `.md` files moved under `docs/guides/`, `docs/milestones/`, `docs/archive/`, `docs/issues/`; `start-services.sh` moved to `scripts/`) and fix the broken `docs/PROJECT_OVERVIEW.md` link in `README.md`.
- Remove the obsolete ONNX probe tests `tests/test_onnx_env.rs` and `tests/test_session_builder.rs`, which target the pre-2.0 `ort` API (`ort::Environment`, `SessionBuilder`, and so on).
- Keep `tests/test_pdf_support.rs` out of the workspace build until the `finish-pdf-and-browse` change lands, so this change doesn't depend on that one.
- Gate server-dependent integration tests (`smoke_cli`, `test_search_filtering_integration`) behind `#[ignore]` so `cargo test --workspace` passes without a running server. Run them explicitly with `--ignored`.
- Upgrade `next` and `eslint-config-next` to a patched 16.x release and resolve the `sharp`, `postcss` and `nanoid` advisories.
- Apply `cargo fix` for mechanical warnings (unused imports and variables) across the workspace.
- Delete the stale `tests/smoke_cli_old.rs`, `tests/smoke_cli_new.rs` and stray Python test files if they're unused.

## Capabilities

### New Capabilities
- `build-health`: Baseline guarantees for the repository: the workspace test suite compiles and passes offline, and production frontend dependencies carry no known critical or high advisories.

### Modified Capabilities
<!-- None: no runtime behaviour changes. -->

## Impact

- **Code**: `services/doc-indexer/tests/*`, mechanical edits across all crates from `cargo fix`.
- **Dependencies**: `apps/frontend/package.json`, `package-lock.json` (Next.js minor bump within 16.x).
- **Docs**: `README.md`, `docs/**` (file moves only).
- **CI**: `.github/workflows/ci-cd.yml` may need `--ignored` tests moved into a separate job that starts the server.
- No API or runtime behaviour changes.
