## Why

Three Cargo features don't compile, so CI tests default features only, and `.github/workflows/ci-cd.yml` and `deny.toml` (from `fix-ci-pipeline`) both note the exclusions. A feature that can't be built is worse than none: it suggests a capability that doesn't exist, and `--all-features` (the usual CI and docs.rs default) fails.

- **`zero-latency-search/tantivy`:** `src/bm25/tantivy_adapter.rs` is out of date with the tantivy 0.22 API (`TextOptions: From<IndexedFlag>`, `IndexWriter<_>` and `searcher.doc` inference, `OwnedValue::as_text`, a missing `Path` import). Behind those, its only test fails: `new()` can't open an existing but empty directory, and searches don't see a commit until the reader reloads.
- **`doc-indexer/cloud`:** `application/concurrent_container.rs` uses names that no longer exist (`OpenAIEmbeddingService`, `QdrantConfig.vector_size`), `qdrant_adapter.rs` misses `VectorMetadata.collection`, and a test calls `generate_embeddings`. `container.rs` already wires the same adapters correctly. Worse, `OpenAIAdapter` never calls OpenAI: it returns hash-based placeholder vectors, so `provider = "openai"` would quietly index meaningless embeddings. `QdrantAdapter` is half there: search calls Qdrant, but `insert`/`delete`/`update` return `Ok` without writing, `count` is always 0 and `health_check` always reports healthy.
- **`zero-latency-search/examples`:** gates `query_expansion/examples.rs`, a 115-line stub that passes `todo!()` in place of its adapters and calls `VectorSearchStep::new` with the wrong arity. It has never compiled, and it isn't an example anyone can run.

The same errors were on `main` before `restore-project-health`, so nothing currently builds with any of these features.

## What Changes

- **`tantivy`: fix.** Port `tantivy_adapter.rs` to tantivy 0.22, open the index with `open_or_create`, and reload the reader after each commit so writes are searchable straight away.
- **`cloud`: fix.** Align `concurrent_container.rs` with `container.rs`, fill `VectorMetadata.collection`, and fix the test. Replace the placeholder in `OpenAIAdapter` with a real `POST /v1/embeddings` client on `reqwest`, and implement Qdrant `insert`/`delete`/`update`/`count`/`health_check` over the REST API the adapter already uses for search. Test both offline against local mock servers. Remove `qdrant-client`, `tonic` and `tantivy` from the `cloud` feature: the live Qdrant adapter speaks REST through `reqwest`, and only the unreferenced `qdrant_adapter_old.rs`/`qdrant_adapter_new.rs` use the gRPC client. Delete those two files.
- **Config: make `cloud` reachable.** The binary builds its config from `AppConfig` (`ZL_*` env vars or `zero-latency.toml`), which has no vector or embedding settings, so every build runs on the embedded store with local embeddings. `Config::from_env()`, which reads the `DOC_INDEXER_*` names the README documents, is never called. Add `vector` and `embedding` sections to `AppConfig`, map them in `Config::from_app_config`, and delete the unused `DOC_INDEXER_*` loader.
- **`examples`: delete** the feature and `query_expansion/examples.rs`. The runnable examples under `crates/zero-latency-search/examples/` are unaffected.
- **CI:** clippy, tests and `cargo-deny` run with `--all-features` / `all-features = true`. Remove the "excluded features" comments from `ci-cd.yml` and `deny.toml`.

## Capabilities

### New Capabilities
- `cloud-backends`: with the `cloud` feature, the OpenAI embedding provider and the Qdrant vector backend call their real services, and fail loudly instead of returning placeholder results.
- `tantivy-bm25`: with the `tantivy` feature, the Tantivy BM25 adapter opens or creates its index in any directory, and documents are searchable as soon as indexing returns.

### Modified Capabilities
- `ci-pipeline`: "Tests run on supported features" becomes "every feature is built and tested": CI lints and tests the workspace with `--all-features`, and the dependency audit covers every feature.

## Impact

- **Code:** `crates/zero-latency-search/src/bm25/tantivy_adapter.rs`, `src/query_expansion/{mod.rs,examples.rs}`; in `services/doc-indexer/src/`: `application/concurrent_container.rs`, `infrastructure/persistence/embeddings/openai_adapter.rs`, `infrastructure/persistence/vector/{qdrant_adapter.rs,qdrant_adapter_old.rs,qdrant_adapter_new.rs}`, plus clippy cleanup in the newly compiled `cloud` code.
- **Dependencies:** `qdrant-client` and `tonic` leave the tree. `deny.toml` starts auditing optional dependencies (`reqwest` 0.12 for `cloud`, `tantivy` for `zero-latency-search`), which may surface new advisories to fix or document.
- **Behaviour:** with `cloud` enabled, `provider = "openai"` calls the OpenAI API and needs a valid key, and `backend = "qdrant"` actually writes to Qdrant, creating the collection on first insert. Default builds are unchanged.
- **Config:** backend and provider are chosen with `ZL_VECTOR_*` / `ZL_EMBEDDING_*` env vars or `[vector]` / `[embedding]` in `zero-latency.toml`. `OPENAI_API_KEY` is still honoured. The `DOC_INDEXER_*` names never took effect, and they're removed from `--env-example` and the README. `ZL_` keys now split at the first `_` only (`ZL_SERVER_DOCS_PATH` → `server.docs_path`), so multi-word settings that couldn't be set from the environment now can.
- **Coordination:** the Linear "Cloud Infrastructure" project may still redesign these adapters. This change only makes them build and work as written.
- **Order:** after `fix-ci-pipeline`. Lands before `remove-dead-code`, which keeps `cloud`/`tantivy` code until this change settles it.
