## 1. Setup

- [x] 1.1 Confirm `restore-project-health` is merged to `main`, then branch `fix-ci-pipeline` from `main`
- [x] 1.2 Record a baseline: `cargo fmt --check` diff count and the clippy warning count by lint (2026-10-04: 515 fmt hunks in 59 files; 391 warnings — 259 dead_code, 39 needless_borrows_for_generic_args, 18 new_without_default, 75 other)

## 2. Formatting

- [x] 2.1 Run `cargo fmt --all` and confirm `cargo test --workspace` still passes
- [x] 2.2 Commit the formatting on its own
- [x] 2.3 Add that commit's hash to `.git-blame-ignore-revs` and commit

## 3. Clippy

- [x] 3.1 Run `cargo clippy --fix --workspace --all-targets --allow-dirty` and review the diff for non-mechanical changes
- [x] 3.2 Fix the remaining non-dead-code warnings by hand (`new_without_default` and others); use a commented `#[allow]` where a fix would change a public API
- [x] 3.3 Confirm `cargo clippy --workspace --all-targets -- -D warnings -A dead_code` exits 0, and that `cargo test --workspace` passes
- [x] 3.4 Commit the clippy fixes

## 4. Dependency policy

- [x] 4.1 Install cargo-deny locally and run `cargo deny init`, then `cargo deny check`
- [x] 4.2 Write `deny.toml`: license allowlist taken from the actual findings (flag any copyleft licenses for review), advisories set to deny, duplicate versions set to warn
- [x] 4.3 Resolve or explicitly ignore (with a reason and an expiry comment) any current RustSec advisories
- [x] 4.4 Confirm `cargo deny check` exits 0 and commit

## 5. Workflows

- [x] 5.1 `ci-cd.yml` test job: stable toolchain only; remove `--all-features` from clippy, test and doc; clippy gets `-- -D warnings -A dead_code`; add a comment naming the excluded features `zero-latency-search/tantivy` and `doc-indexer/cloud`
- [x] 5.2 `ci-cd.yml` security job: drop `cargo install cargo-audit` and `cargo audit`; upgrade to `EmbarkStudios/cargo-deny-action@v2` using `deny.toml`
- [x] 5.3 `ci-cd.yml` build job: upload `target/release/doc-indexer` and `target/release/mdx`
- [x] 5.4 `ci-cd.yml`: build the Docker image without pushing on pull requests; push and deploy jobs run only when `vars.DEPLOY_ENABLED == 'true'`
- [x] 5.5 Add a `frontend` job: setup-node with npm cache, `npm ci`, then `tsc --noEmit`, `npm run build` and `npm audit --omit=dev --audit-level=high` in `apps/frontend` (ESLint open question: the flat config loads under eslint 9 but reports 8 errors, so lint stays out for now)
- [x] 5.6 Check the ONNX Runtime download works on the runner; add a cache for it if it's slow (works; Test Suite 172s and Build 280s on a cold cache, so no ORT-specific cache needed)
- [x] 5.7 Resolve the open question on the `develop` and `sprint/**` triggers (resolved: dropped both, since the remote has only `main`; deploy-staging now runs on pushes to `main`)
- [x] 5.8 Remove any remaining steps that swallow their own failure (`|| true`, `|| echo`) where the check should be able to fail, or document why they stay (schema-validation: removed the `|| echo`s; breaking-change detection stays warn-only by design, with a comment saying why)
- [x] 5.9 Commit the workflow changes

## 6. Verify

- [x] 6.1 Open a pull request and confirm every job in both workflows is green, or skipped by design (PR #1 merged directly at your request; on `main` both workflows are green: CI/CD run 37173283726, Schema Validation run 37172791541)
- [x] 6.2 Push a throwaway commit with a deliberate formatting error to the pull request, confirm CI fails, then drop the commit (substituted: no synthetic commit, per the instruction to skip the PR flow. A real failure proved the same thing: run 37172268466 on c7ac1b2 failed at clippy on Rust 1.99 lints and blocked the downstream jobs)
- [x] 6.3 Merge, and confirm the `main` push run is green (green at 245f128 after two follow-up fixes: Rust 1.99 lints, Docker builder image)

## 7. Follow-ups

- [x] 7.1 Propose an OpenSpec change for dead-code removal (which ends with removing `-A dead_code` from CI) (`remove-dead-code`)
- [x] 7.2 Propose an OpenSpec change to fix or delete the `tantivy` and `cloud` features (`resolve-broken-features`)
