## Context

Findings from 2026-10-04, measured on the `restore-project-health` branch:

| Check | State |
|---|---|
| `cargo fmt --check` | About 500 diffs. CI's first failing step, about 8 seconds into the "Test Suite" job |
| `cargo clippy --workspace --all-targets` (default features) | About 440 warnings: about 300 dead code (`never constructed`, `never used`, `never read`), 39 `needless_borrow`, 18 `new_without_default`, the rest assorted |
| `--all-features` | Does not compile. `zero-latency-search/tantivy`: 7 errors against the current `tantivy` API (`TextOptions: From<IndexedFlag>`, `IndexWriter<_>` inference, a missing `Path` import). `doc-indexer/cloud`: 5 errors (`VectorMetadata.collection`, `QdrantConfig.vector_size`, `OpenAIEmbeddingService`, `generate_embeddings`). Both errors are identical on `main` |
| `cargo test --workspace` | Passes offline (from `restore-project-health`) |
| cargo-deny | No `deny.toml`; default policy |
| Build artifact | `target/release/docsearch` does not exist |
| Docker and deploy | No repo secrets, no GitHub environments. `KUBE_CONFIG_*` and `SLACK_WEBHOOK_URL` are unset |
| `schema-validation.yml` | Last failure: `setup-node` `cache: npm` found no lockfile. A root `package-lock.json` is tracked now |
| Frontend | Not in CI |

## Goals / Non-Goals

**Goals:**
- CI is green on `main` and on pull requests, and every check that runs is one that can fail meaningfully.
- Formatting and non-dead-code lints are enforced from here on.
- Broken feature flags are visible as tracked debt, not hidden behind a skipped step.
- The frontend's type-check, build and dependency audit run in CI.

**Non-Goals:**
- Deleting dead code. It's a large, judgement-heavy job and gets its own change.
- Making the `tantivy` or `cloud` features compile.
- Standing up deployment infrastructure or publishing images.
- Running the server-dependent `--ignored` tests in CI.

## Decisions

**Format in one isolated commit and record it in `.git-blame-ignore-revs`.** A single mechanical commit keeps review trivial and lets `git blame` skip it. *Alternative:* format files gradually as they're touched. Rejected because CI would stay red and diffs would mix style with logic.

**Allow dead code in CI for now, and deny everything else.** Run `cargo clippy --workspace --all-targets -- -D warnings -A dead_code`. Put the allowance on the CI command line, not in source or `Cargo.toml`, so local `cargo check` still shows dead-code warnings as a reminder, and removing the flag later is a one-line change. *Alternative:* `#![allow(dead_code)]` per crate. Rejected because it hides the debt locally too. *Alternative:* delete the dead code first. Rejected for scope: it's about 300 sites, and some of it (for example `load_testing`, `operations/production`) may be meant for future use.

**Fix the remaining lints mechanically where possible.** `cargo clippy --fix` handles `needless_borrow` and similar. `new_without_default` is fixed by deriving or implementing `Default`. Anything that changes a public API signature gets `#[allow]` with a comment, not a redesign.

**Test the default features, and exclude broken features explicitly.** Replace `--all-features` with the default feature set. Add a comment block in `ci-cd.yml` naming `zero-latency-search/tantivy` and `doc-indexer/cloud` as excluded because they don't compile, with a pointer to the follow-up. *Alternative:* fix both features now. Rejected: `cloud` touches Qdrant and OpenAI adapters, which the Linear "Cloud Infrastructure" work may redesign. *Alternative:* delete the features. That's a reasonable follow-up decision, but not one to make inside a CI fix.

**Use maintained actions for audits.** Use `EmbarkStudios/cargo-deny-action@v2` with a checked-in `deny.toml` covering advisories, licenses (an allowlist from the current dependency tree: MIT, Apache-2.0, BSD-2/3-Clause, ISC, Unicode-3.0, Zlib, MPL-2.0 and whatever else the tree needs, confirmed by running it), bans (warn on duplicates) and sources. Advisories through cargo-deny replace the separate `cargo install cargo-audit` step, which takes about 3 minutes and checks the same RustSec database.

**Gate image publishing and deploys behind a repository variable.** Make the `docker` push and `deploy-*` jobs conditional on `vars.DEPLOY_ENABLED == 'true'`. The Docker *build*, without pushing, still runs on pull requests, so the Dockerfile is kept honest. *Alternative:* delete the deploy jobs. Rejected because they document intent for the cloud work and cost nothing while disabled.

**Use the stable toolchain only.** The `beta` matrix entry doubles runtime and produces failures nobody acts on.

**Make the frontend its own job.** Use `actions/setup-node` with `cache: npm` on the root lockfile, then `npm ci`, then in `apps/frontend`: `npx tsc --noEmit`, `npm run build`, `npm audit --omit=dev --audit-level=high`. ESLint is left out until its config is confirmed working under eslint 9 (an open question).

## Risks / Trade-offs

- [The formatting commit conflicts with in-flight branches (`wip/finish-pdf-and-browse`)] → Merge this change before rebasing that branch, and run `cargo fmt` on it after the rebase. Formatting conflicts resolve mechanically.
- [`-A dead_code` becomes permanent] → Record the dead-code cleanup as a follow-up OpenSpec change in this change's tasks. The flag sits in one visible place.
- [The `deny.toml` license allowlist is too broad] → Build it from cargo-deny's actual findings, not a generic list, and review any copyleft entries explicitly.
- [Stopping image publishing breaks someone pulling `:latest`] → Nothing has published successfully since CI went red, because `docker` depends on `build`, which depends on `test`. No working consumer exists.
- [The ONNX Runtime `download-binaries` feature fails or is slow on CI runners] → Cache `~/.cache/ort` or the equivalent alongside cargo. If downloads are unreliable, record it and consider testing `doc-indexer` without default features in CI.

## Migration Plan

1. Merge `restore-project-health`.
2. Land this change in four commits: formatting; clippy fixes; `deny.toml`; workflow edits.
3. Open a pull request, confirm every job is green, and merge.
4. Rebase `wip/finish-pdf-and-browse` onto the new `main` and run `cargo fmt`.
5. Rollback: revert the workflow commit. The code commits are behaviour-neutral.

## Open Questions

- Does the frontend's eslint 9 flat config work? If it does, add `npm run lint` to the frontend job.
- Should `develop` and `sprint/**` stay in the push triggers? Neither branch appears active.
- Should broken features be fixed or deleted? Raise it as its own change after this one.
