## Context

A health review on 2026-10-04 found:

- `cargo check --workspace --all-targets` succeeds with about 250 warnings, mostly in `doc-indexer`.
- Unit tests (`--lib --bins`) all pass: 110 passed, 0 failed.
- `cargo test --workspace` aborts at compile time. `test_onnx_env` and `test_session_builder` import `ort::{Environment, LoggingLevel, SessionBuilder, ExecutionProvider}`, which were removed in `ort` 2.0 (the crate is pinned at `2.0.0-rc.10`). `test_pdf_support` calls a non-existent `ContentTypeDetector::detect_from_extension`.
- `test_search_filtering_integration` fails with "connection refused" because it expects an external server on `localhost:8081`.
- `smoke_cli` spawns its own server, but fails when run as a suite. *(Found during implementation.)* `TestConfigHelper::get_unique_port` kept a per-instance counter, so every test got port 19000. Tests only killed their server on success, so a panic leaked the process. On top of that, search ignores the `collection` parameter, so results from the shared local data store make ranking assertions order-dependent.
- `npm audit --omit=dev` reports four advisories: Next.js ≤16.3.2 (critical), and `sharp`, `postcss` and `nanoid` (high).
- The working tree mixes three unrelated streams of work: the docs reorganisation, PDF support, and browse/progress endpoints.

## Goals / Non-Goals

**Goals:**
- `cargo test --workspace` compiles and passes on a clean machine with no running services.
- No critical or high advisories in production frontend dependencies.
- The docs reorganisation is committed separately from feature work.
- Warning count goes down substantially through mechanical fixes only.

**Non-Goals:**
- Finishing PDF ingestion or the browse endpoint (see `finish-pdf-and-browse`).
- Reaching zero warnings. Dead-code warnings that need design judgement stay for now.
- Upgrading Rust dependencies across major versions.
- Upgrading Next.js to a new major version.

## Decisions

**Delete the ONNX probe tests rather than port them.** They were one-off diagnostics for ONNX Runtime environment setup and exercise no project code. Porting them to the `ort` 2.0 API gives no coverage. The real embedding path is already exercised by the `embedded` feature's unit tests. *Alternative:* port to `ort::session::Session::builder()`. Rejected because it's maintenance with no value.

**Use `#[ignore]` for server-dependent tests rather than a cargo feature.** `#[ignore = "requires running doc-indexer on :19000"]` is self-documenting, and `cargo test -- --ignored` is the standard way to opt in. *Alternative:* an `integration` feature flag. Rejected because features change compilation and complicate the build matrix.

**Fix the smoke harness, but keep the tests opt-in.** Make the port allocation process-wide and skip ports already in use. Wrap spawned servers in a `ChildGuard` that kills them on drop. That makes `--ignored` runs reliable apart from the collection-scoping bug. The tests stay ignored by default because they write to the shared local data store, and because the collection-scoping bug makes them order-dependent. Fixing scoping is a product change and a follow-up, not part of this change.

**Handle `test_pdf_support.rs` by isolating, not deleting.** The file belongs to the in-flight PDF work. It is parked with the rest of the feature work on the branch `wip/finish-pdf-and-browse`, rather than in a stash, because `cargo fix` edits the same files. `finish-pdf-and-browse` task 1.1 rebases it back.

**Separate commits by stream.** Make one commit each for: the docs move, the test fixes, the frontend dependency bump, and `cargo fix`. Each is independently revertible.

**Upgrade Next.js within 16.x.** Use the latest 16.x that clears the advisory (at least 16.3.3). Bump `eslint-config-next` to match. Run `npm audit fix` for transitive advisories, and avoid `--force` unless it stays within the current major versions.

## Risks / Trade-offs

- [`cargo fix` touches many files and creates a noisy diff] → Commit it on its own, with no hand edits mixed in.
- [A Next.js minor upgrade could change behaviour] → Run `tsc --noEmit` and `next build`, then smoke-test the search and indexing pages manually.
- [Ignored tests rot unnoticed] → Add a CI job that starts `doc-indexer` and runs `cargo test -- --ignored`, or record it as follow-up if CI can't host the server today.
- [Uncommitted feature work gets swept into health commits] → Stage paths explicitly. Never use `git add -A` in this change.

## Open Questions

- **CI has been red on every push since at least 2025-11-28** (found during task 6.1). The `CI/CD Pipeline` "Test Suite" job fails at `cargo fmt --check` within about 8 seconds: there are about 500 formatting diffs. Even with formatting fixed, the job would then fail `cargo clippy -D warnings` (about 330 warnings, mostly dead code). The `ai_guardrails_on_commit.yml` workflow fails instantly because its script `ai/scripts/check_envelope.py` no longer exists. It was removed in `d20f35f`. Making CI green is out of this change's scope as written.
- **Follow-up bug:** `/api/search` with `collection: "X"` returns documents from other collections, such as `zero_latency_docs`. Seen by hand on 2026-10-04. It needs its own change.

- Is the `ai_guardrails_on_commit.yml` workflow still wanted? The custom AI pre-commit hooks were removed in commit `d20f35f`.
