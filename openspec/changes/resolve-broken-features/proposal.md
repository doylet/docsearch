## Why

Two Cargo features don't compile, so CI tests default features only and names these two as excluded in `.github/workflows/ci-cd.yml` (from `fix-ci-pipeline`). A feature that can't be built is worse than none: it suggests a capability that doesn't exist, and `--all-features` (the usual CI and docs.rs default) fails.

- **`zero-latency-search/tantivy`:** 7 errors against tantivy 0.22 in `src/bm25/tantivy_adapter.rs` (`TextOptions: From<IndexedFlag>`, `IndexWriter<_>` inference, a missing `Path` import). The non-tantivy BM25 path is what actually runs.
- **`doc-indexer/cloud`:** 5 errors (`VectorMetadata.collection`, `QdrantConfig.vector_size`, `OpenAIEmbeddingService`, `generate_embeddings`). It pulls in `qdrant-client`, `tonic`, `reqwest` and `tantivy`. The adapters have drifted from the core types.

The same errors were on `main` before `restore-project-health`, so nothing currently builds with either feature.

## What Changes

Decide each feature on its own merits:

- **`tantivy`:** either fix the adapter against the current tantivy API and add it to CI, or delete the feature, `tantivy_adapter.rs`'s feature-gated code and the optional dependency. Lean: **delete**, unless BM25 quality needs tantivy's scoring. `doc-indexer`'s `embedded` feature has its own tantivy dependency, which this decision doesn't touch.
- **`cloud`:** either fix the Qdrant and OpenAI adapters, or delete them and leave cloud storage to the Linear "Cloud Infrastructure" project, which may redesign these adapters anyway. Lean: **delete**, and re-add during that project with a design that fits it.
- Whichever way each goes, finish with CI either building that feature or not having it. Remove the "excluded features" comment from `ci-cd.yml`, and switch CI to `--all-features` if every remaining feature compiles.

## Capabilities

### New Capabilities
<!-- None. -->

### Modified Capabilities
- `ci-pipeline`: the "tests run on supported features" requirement no longer lists excluded features. If every feature compiles, CI tests all of them.

## Impact

- **Code:** `crates/zero-latency-search/src/bm25/tantivy_adapter.rs`; the `#[cfg(feature = "cloud")]` code in `services/doc-indexer/src/` (`config.rs`, `application/container.rs`, `application/concurrent_container.rs`, `infrastructure/mod.rs`, `infrastructure/persistence/{embeddings,vector}/mod.rs`).
- **Dependencies:** deleting `cloud` removes `qdrant-client` and `tonic` from the tree, along with several of the advisory ignores in `deny.toml`, so re-check it.
- **Coordination:** confirm the direction with the Linear "Cloud Infrastructure" project before deleting `cloud`.
- **Order:** after `fix-ci-pipeline`. Independent of `remove-dead-code`, but landing this first shrinks that change's scope.
