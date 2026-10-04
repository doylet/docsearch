## 1. Separate in-flight work

- [x] 1.1 Set aside the PDF, browse, metadata and frontend feature changes (committed to the branch `wip/finish-pdf-and-browse`), leaving only the docs reorganisation in the working tree
- [x] 1.2 Confirm `git status` shows only the docs moves, `.gitignore`, `README.md` and `scripts/start-services.sh`

## 2. Docs reorganisation

- [x] 2.1 Fix `README.md` so the duplicate `docs/PROJECT_OVERVIEW.md` link points to `docs/guides/PROJECT_OVERVIEW.md` (and remove the duplicate entry)
- [x] 2.2 Check every relative link in `README.md` and `docs/README.md` resolves
- [x] 2.3 Decide whether to commit `agents/` and `docs/ORGANIZATION.md`, or leave them for later
- [x] 2.4 Commit the docs reorganisation (explicit paths, no `git add -A`)

## 3. Test targets

- [x] 3.1 Delete `services/doc-indexer/tests/test_onnx_env.rs` and `tests/test_session_builder.rs`
- [x] 3.2 Check whether `smoke_cli_old.rs`, `smoke_cli_new.rs`, `test_minimal.rs` and `test_enhanced_service.py` are referenced anywhere; delete them if not
- [x] 3.3 Add `#[ignore = "requires running doc-indexer on :19000"]` to every test in `smoke_cli.rs` and `test_search_filtering_integration.rs`
- [x] 3.4 Confirm `cargo test --workspace` exits 0 with no services running
- [x] 3.5 Commit the test fixes

## 4. Frontend security

- [x] 4.1 Bump `next` and `eslint-config-next` in `apps/frontend/package.json` to the latest patched 16.x release
- [x] 4.2 Run `npm install` then `npm audit fix` at the repo root
- [x] 4.3 Confirm `npm audit --omit=dev --audit-level=high` exits 0 in `apps/frontend`
- [x] 4.4 Run `npx tsc --noEmit` and `npm run build` in `apps/frontend`
- [x] 4.5 Commit the dependency upgrade

## 5. Warnings

- [x] 5.1 Run `cargo fix --workspace --allow-dirty --all-targets` and review the diff for anything non-mechanical
- [x] 5.2 Confirm `cargo test --workspace` still passes, and record the before and after warning counts
- [x] 5.3 Commit the warning cleanup on its own

## 6. CI and wrap-up

- [x] 6.1 Check `.github/workflows/ci-cd.yml` runs `cargo test --workspace` and the frontend audit; add an `--ignored` job, or record it as follow-up
- [x] 6.2 Resolve the open question about `ai_guardrails_on_commit.yml` (deleted: its script was removed in d20f35f)
- [x] 6.3 Remove hand-rolled CI tests: the `test/` bash and Python scripts, the AI guardrails workflow, the `simple_validation.sh` step in `ci-cd.yml`, and the service-startup and docs-check scripts in `schema-validation.yml`
