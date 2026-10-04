## 1. Remove the `examples` feature

- [x] 1.1 Delete `crates/zero-latency-search/src/query_expansion/examples.rs`, and remove both the `#[cfg(test)]` attribute and `pub mod examples;` from `query_expansion/mod.rs`
- [x] 1.2 Remove `examples = []` from `crates/zero-latency-search/Cargo.toml`, and confirm nothing else references the feature (`grep -rn 'feature = "examples"'`)

## 2. Fix `zero-latency-search/tantivy`

- [x] 2.1 Port `bm25/tantivy_adapter.rs` to tantivy 0.22: `STRING | STORED | FAST` for `doc_id`/`collection`, `writer::<TantivyDocument>`, `searcher.doc::<TantivyDocument>`, `Value::as_str`, `std::path::Path`, drop the unused imports
- [x] 2.2 Replace the `exists()`/`open_in_dir`/`create_in_dir` branch with `create_dir_all` + `Index::open_or_create(MmapDirectory::open(..))`
- [x] 2.3 Call `self.reader.reload()` after the commit in `index_document` and in `delete_document`
- [x] 2.4 Add tests for the empty-directory, reopen-existing-index and delete-then-search scenarios in `specs/tantivy-bm25`. `cargo test -p zero-latency-search --features tantivy` passes

## 3. Make `doc-indexer/cloud` compile

- [x] 3.1 In `application/concurrent_container.rs`, build `QdrantAdapter::new(config.vector.qdrant.clone())` and `OpenAIAdapter::new(config.embedding.openai.clone())`, as `container.rs` does
- [x] 3.2 In `qdrant_adapter.rs`, read `collection` from the payload into `VectorMetadata.collection`
- [x] 3.3 Delete the unreferenced `persistence/vector/qdrant_adapter_old.rs` and `qdrant_adapter_new.rs`
- [x] 3.4 Set `cloud = ["reqwest"]` and remove the `qdrant-client` and `tonic` optional dependencies from `services/doc-indexer/Cargo.toml`
- [x] 3.5 `cargo check -p doc-indexer --all-targets` passes with `--features cloud`, with `--no-default-features --features cloud`, and with `--features full`

## 4. Offline HTTP mock helper

- [x] 4.1 Add a `#[cfg(test)]` helper under `services/doc-indexer/src/infrastructure/` that starts an `axum` router on `127.0.0.1:0`, records each request (method, path, headers, JSON body), and returns its base URL. No new dependencies

## 5. Real OpenAI embeddings

- [x] 5.1 Hold a `reqwest::Client` in `OpenAIAdapter`, built in `new()` with `timeout_seconds`. Base URL defaults to `https://api.openai.com/v1`
- [x] 5.2 Implement `call_api`: POST `{model, input}` to `/embeddings` with a bearer key, sort `data` by `index`, and error when the count doesn't match the inputs
- [x] 5.3 Retry 429, 5xx and connection errors up to `max_retries` with exponential backoff (base delay settable so tests use ~1 ms). Other statuses fail at once with `external_service("openai", status + error message)`
- [x] 5.4 Truncate in `prepare_text` on a char boundary. Remove `simple_hash`
- [x] 5.5 Rewrite the adapter's tests against the mock helper to cover every scenario in `specs/cloud-backends` for OpenAI (single, order, >100 split, count mismatch, 429 retry, 401 no retry, multi-byte truncation)

## 6. Real Qdrant writes

- [x] 6.1 Add a shared request helper that sends the `api-key` header when it's configured, and maps non-2xx responses to `external_service("qdrant", status + body)`
- [x] 6.2 Implement `insert`: build the payload as the inverse of `from_qdrant_rest_result` (including `collection` and `custom_*`), and `PUT /collections/{c}/points?wait=true` in batches of 256. On a missing collection, create it with the first vector's size and `Cosine`, then retry
- [x] 6.3 Implement `delete` and `update`: look up the point with `GET /points/{id}` (404 → `false`), then `POST /points/delete` or `PUT /points/vectors` with `wait=true`
- [x] 6.4 Implement `count` (`POST /points/count`, `exact: true`; a missing collection is 0) and `health_check` (`GET /healthz`; non-2xx or unreachable → unhealthy)
- [x] 6.5 Add mock-server tests for every Qdrant scenario in `specs/cloud-backends` (payload round-trip, collection creation, rejected write, delete/update existing and unknown, count, 503 health, `api-key` header)

## 7. Lints and audit with all features

- [x] 7.1 `cargo clippy --workspace --all-targets --all-features -- -D warnings -A dead_code` passes. Fix the newly visible lints in the cloud code (unused `ZeroLatencyError` imports, needless `&format!`, `Uuid` clones, empty lines after doc comments, `from_*` taking `self`, useless `vec!`)
- [x] 7.2 Set `[graph] all-features = true` in `deny.toml` and update its comment. Run `cargo deny check`, then upgrade or add a reasoned, dated ignore for every new advisory. Remove ignores that no longer match anything
- [x] 7.3 `cargo fmt --all -- --check` passes

## 8. CI and docs

- [x] 8.1 In `.github/workflows/ci-cd.yml`, add `--all-features` to the clippy and test steps, keep the release build on default features, and replace the "excluded features" comment with a one-line note that CI checks all features
- [x] 8.2 In the README's `cloud` feature section, say that OpenAI embeddings call the API (dimension depends on the model, so it must match the vector store), and that the Qdrant collection is created on first insert
- [x] 8.3 Add a CHANGELOG entry: the tantivy and cloud features build and work, the examples feature is removed, and CI checks all features

## 9. Configuration: select backend and provider

- [x] 9.1 In `zero-latency-config`, add `VectorSettings` and `EmbeddingSettings` (flat fields, defaults matching today's behaviour) as `#[serde(default)]` sections of `AppConfig`
- [x] 9.2 Replace `Env::prefixed("ZL_").split("_")` with one shared provider that splits at the first `_` only. Fall back to `OPENAI_API_KEY` when `embedding.openai_api_key` is unset
- [x] 9.3 Make `validate_config` reject an unknown `vector.backend` or `embedding.provider`
- [x] 9.4 Test the env mapping (including multi-word fields), file + env precedence, key fallback and validation with `figment::Jail`, so env vars don't leak between parallel tests
- [x] 9.5 Make `Config::from_app_config` map both sections, parse backend and provider, and return `Result`. Have `main` call it and then `Config::validate()`. Add unit tests for Qdrant/OpenAI mapping and an invalid backend
- [x] 9.6 Delete the unused `Config::from_env()` and rewrite `Config::env_example()` and the README's Qdrant/OpenAI setup with the `ZL_` names

## 10. Verification

- [x] 10.1 `cargo test --workspace --all-features` and `cargo test --workspace` (default features) both pass
- [x] 10.2 `cargo check --workspace --no-default-features` passes (no feature combination a README command uses is broken)
- [x] 10.3 Manual smoke test, recorded in the PR: run `doc-indexer` with `--features cloud` against `docker run qdrant/qdrant` and a real `OPENAI_API_KEY`, index a small folder, search it, and check `count`/health. If no key is available, note that in the PR and skip only the OpenAI half
  - Result (2026-10-04): Qdrant half passed against `qdrant/qdrant:latest` using `ZL_VECTOR_*` config and local embeddings. Indexing 2 Markdown files created `smoke_test` (384 dims, cosine) with 2 points. Search returned both, with metadata intact. `/api/status` counted 2, a search before indexing returned empty, and real Qdrant accepted the `default` + `is_empty` filter. The first run found the logical-vs-physical collection bug, fixed in `search_points`. OpenAI half **skipped**: no `OPENAI_API_KEY` available. Covered only by mock-server tests.
