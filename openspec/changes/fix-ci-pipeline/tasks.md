## 1. Setup

- [ ] 1.1 Confirm `restore-project-health` is merged to `main`, then branch `fix-ci-pipeline` from `main`
- [ ] 1.2 Record a baseline: `cargo fmt --check` diff count and the clippy warning count by lint

## 2. Formatting

- [ ] 2.1 Run `cargo fmt --all` and confirm `cargo test --workspace` still passes
- [ ] 2.2 Commit the formatting on its own
- [ ] 2.3 Add that commit's hash to `.git-blame-ignore-revs` and commit

## 3. Clippy

- [ ] 3.1 Run `cargo clippy --fix --workspace --all-targets --allow-dirty` and review the diff for non-mechanical changes
- [ ] 3.2 Fix the remaining non-dead-code warnings by hand (`new_without_default` and others); use a commented `#[allow]` where a fix would change a public API
- [ ] 3.3 Confirm `cargo clippy --workspace --all-targets -- -D warnings -A dead_code` exits 0, and that `cargo test --workspace` passes
- [ ] 3.4 Commit the clippy fixes

## 4. Dependency policy

- [ ] 4.1 Install cargo-deny locally and run `cargo deny init`, then `cargo deny check`
- [ ] 4.2 Write `deny.toml`: license allowlist taken from the actual findings (flag any copyleft licenses for review), advisories set to deny, duplicate versions set to warn
- [ ] 4.3 Resolve or explicitly ignore (with a reason and an expiry comment) any current RustSec advisories
- [ ] 4.4 Confirm `cargo deny check` exits 0 and commit

## 5. Workflows

- [ ] 5.1 `ci-cd.yml` test job: stable toolchain only; remove `--all-features` from clippy, test and doc; clippy gets `-- -D warnings -A dead_code`; add a comment naming the excluded features `zero-latency-search/tantivy` and `doc-indexer/cloud`
- [ ] 5.2 `ci-cd.yml` security job: drop `cargo install cargo-audit` and `cargo audit`; upgrade to `EmbarkStudios/cargo-deny-action@v2` using `deny.toml`
- [ ] 5.3 `ci-cd.yml` build job: upload `target/release/doc-indexer` and `target/release/mdx`
- [ ] 5.4 `ci-cd.yml`: build the Docker image without pushing on pull requests; push and deploy jobs run only when `vars.DEPLOY_ENABLED == 'true'`
- [ ] 5.5 Add a `frontend` job: setup-node with npm cache, `npm ci`, then `tsc --noEmit`, `npm run build` and `npm audit --omit=dev --audit-level=high` in `apps/frontend`
- [ ] 5.6 Check the ONNX Runtime download works on the runner; add a cache for it if it's slow
- [ ] 5.7 Resolve the open question on the `develop` and `sprint/**` triggers
- [ ] 5.8 Remove any remaining steps that swallow their own failure (`|| true`, `|| echo`) where the check should be able to fail, or document why they stay
- [ ] 5.9 Commit the workflow changes

## 6. Verify

- [ ] 6.1 Open a pull request and confirm every job in both workflows is green, or skipped by design
- [ ] 6.2 Push a throwaway commit with a deliberate formatting error to the pull request, confirm CI fails, then drop the commit
- [ ] 6.3 Merge, and confirm the `main` push run is green

## 7. Follow-ups

- [ ] 7.1 Propose an OpenSpec change for dead-code removal (which ends with removing `-A dead_code` from CI)
- [ ] 7.2 Propose an OpenSpec change to fix or delete the `tantivy` and `cloud` features
