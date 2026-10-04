## Context

The workspace has three features that have never compiled on `main`: `zero-latency-search/tantivy`, `zero-latency-search/examples` and `doc-indexer/cloud`. The proposal calls out two of them. `examples` showed up when probing `--all-features`. CI (`ci-cd.yml`) and `cargo-deny` (`deny.toml`, `[graph] all-features = false`) both work around them by checking default features only.

A throwaway probe applied the fixes below. With them, `cargo clippy --workspace --all-targets --all-features` reached only mechanical lints and `cargo test --workspace --all-features` passed. The probe has been reverted. Findings:

- **tantivy:** the compile errors are API drift in tantivy 0.22. `INDEXED | STORED | FAST` on a text field becomes `STRING | STORED | FAST`. `index.writer` and `searcher.doc` need `::<TantivyDocument>`. `OwnedValue::as_text` becomes `Value::as_str`. `Path` needs a full path. Once it compiles, `test_tantivy_adapter` fails twice. First, `new()` checks `index_path.exists()` and calls `open_in_dir`, which fails on the empty directory the test (or a first run) gives it. Second, the reader uses the default `OnCommitWithDelay` reload policy, so a search right after `index_document` sees nothing.
- **cloud:** `container.rs` already builds `QdrantAdapter::new(config.vector.qdrant.clone())` and `OpenAIAdapter::new(config.embedding.openai.clone())` correctly. `concurrent_container.rs` has stale copies of both. `qdrant_adapter.rs` builds `VectorMetadata` without the `collection` field, and `openai_adapter.rs`'s batch test calls `generate_embeddings` where the trait method is `generate_batch_embeddings(Vec<&str>)`.
- **OpenAI stub:** `OpenAIAdapter::call_api` returns normalised vectors derived from a string hash. `reqwest` is already a `cloud` dependency but isn't used here. `prepare_text` truncates with `&text[..8000]`, which panics when byte 8000 falls inside a multi-byte character.
- **Qdrant stub:** in `qdrant_adapter.rs`, only `search` calls Qdrant. `insert`, `delete` and `update` log "not implemented" and return `Ok`. `count` returns 0 and `health_check` always returns `Healthy`. With `vector.backend = "qdrant"`, indexing reports success and stores nothing. The other adapters (memory, embedded) treat `delete`/`update`'s `document_id` as the vector's own id (`VectorDocument.id`). `search` reads the payload keys `document_id`, `chunk_index`, `content`, `title`, `heading_path` (joined with `/`), `url` and `custom_*`.
- **Unused cloud dependencies:** the live `qdrant_adapter.rs` talks to Qdrant's REST API through `reqwest::Client`. `qdrant-client` and `tonic` are imported only by `qdrant_adapter_old.rs` and `qdrant_adapter_new.rs`, which no `mod` declaration includes. No doc-indexer source uses `tantivy`. The `cloud` feature still lists all three.
- **examples:** `query_expansion/examples.rs` is `#![cfg(feature = "examples")]` behind `#[cfg(test)] pub mod examples;`. It calls `todo!()` for both search steps and has the wrong arity for `VectorSearchStep::new`.

## Goals / Non-Goals

**Goals:**
- Every Cargo feature in the workspace compiles, passes clippy with the CI lint set, and passes its tests.
- CI lints, tests and audits with all features, and no "excluded features" note remains.
- With `cloud` enabled, `provider = "openai"` produces real OpenAI embeddings, or fails with a clear error.
- With `cloud` enabled, `backend = "qdrant"` stores, updates, deletes and counts vectors in Qdrant, and reports Qdrant's actual health.

**Non-Goals:**
- Wiring `TantivyAdapter` into the running search pipeline. The non-tantivy BM25 path stays the default.
- Integration tests against a live Qdrant or the real OpenAI API. CI stays offline. Mock servers check the requests we send, not Qdrant's or OpenAI's behaviour.
- Qdrant collection management beyond create-if-missing (aliases, recreating on a dimension change, payload indexes).
- Redesigning the cloud adapters for multi-tenant hosting. That belongs to the Linear "Cloud Infrastructure" project.
- Checking that the vector store's dimension matches the embedding model's (1536/3072 for OpenAI, 384 for local). It's an existing gap that this change doesn't widen. Noted under Risks.
- Touching `doc-indexer`'s `embedded` feature or its `tantivy` entry, beyond keeping it compiling.

## Decisions

**Fix `tantivy` in place rather than delete it.** The port needs about ten lines, plus two real bug fixes that its own test catches. Open the index with `Index::open_or_create(MmapDirectory::open(path)?, schema)` after `create_dir_all`, which handles missing, empty and existing directories the same way. Call `self.reader.reload()` after every `commit()` (in index and delete), so a write is visible as soon as the call returns. *Alternative:* `ReloadPolicy::Manual` on the reader builder. Same effect, but leaving the default policy keeps the reader up to date with commits from other writers too.

**Make `concurrent_container.rs` match `container.rs`.** Pass the config structs straight through instead of rebuilding them field by field. That's the pattern that already compiles, so the two containers can't drift apart again on these fields. *Alternative:* add `vector_size` to `QdrantConfig`. Rejected because nothing reads it: the REST adapter takes the dimension from the vectors it receives.

**Set `VectorMetadata.collection` from the payload.** `qdrant_adapter.rs` already reads payload keys such as `title` and `url`. Read `collection` the same way (`Option<String>`), and write it on upsert. A missing key becomes `None`.

**Qdrant writes over the same REST client.** Point id = `VectorDocument.id` (a UUID, which Qdrant accepts), so `delete`/`update` have the same meaning as in the other adapters. The payload is the exact inverse of `from_qdrant_rest_result`: `document_id`, `chunk_index`, `content`, `title`, `heading_path` as a JSON array (a `/`-joined string can't round-trip a heading that contains `/`, but the reader still accepts the old string form), `url`, `collection`, and each `custom` entry as `custom_<key>`. Round-tripping is unit-tested. All requests, `search_in_collection` included (it used to skip the `api-key` header), go through one helper.
- `insert` → `PUT /collections/{c}/points?wait=true`, batched (256 points per request). If the collection is missing, create it on first insert with `size` = the first vector's length and `distance: Cosine`, so `QdrantConfig` needs no dimension field.
- `delete` / `update` → `GET /collections/{c}/points/{id}` to learn whether the point exists (404 → `Ok(false)`), then `POST /points/delete` or `PUT /points/vectors` with `wait=true`.
- `search` / `search_in_collection` → `POST /points/search` on the configured Qdrant collection. `search_in_collection(name)` is a *logical* collection, as in the memory and embedded adapters: it adds a payload filter `collection == name`, and for `"default"` it also matches points with no collection. The original adapter used `name` as a physical Qdrant collection, so the search pipeline, which always calls `search_in_collection`, got a 404. The smoke test found this. A missing Qdrant collection, meaning nothing has been indexed yet, returns no results rather than an error.
- `count` → `POST /collections/{c}/points/count` with `exact: true`. A missing collection is 0.
- `health_check` → `GET /healthz`. Anything other than 2xx is `Unhealthy`, with the reason.
- Every request sends the `api-key` header when `api_key` is set. Non-2xx responses become `ZeroLatencyError::external_service("qdrant", <status + body>)`.

*Alternative:* bring back `qdrant-client` (gRPC). Rejected because search already uses REST, and mixing transports would add `tonic` back for no benefit.

**A real OpenAI client on `reqwest`, no SDK.** In `OpenAIAdapter::new`, build a `reqwest::Client` with `timeout_seconds` and a bearer `api_key`. `call_api` POSTs `{model, input: [..]}` to `{base_url or https://api.openai.com/v1}/embeddings` and parses `data[].embedding`, sorted by `data[].index`. It retries up to `max_retries` times with exponential backoff on 429, 5xx and connection errors, and returns `ZeroLatencyError::external_service` (or the closest existing variant) with the status and the API's error message for anything else, or once retries run out. It checks that the number of embeddings returned equals the number of inputs. `prepare_text` truncates on a char boundary (`char_indices().nth(MAX_CHARS)`). Remove `simple_hash`. *Alternatives:* the `async-openai` crate, rejected as a heavy dependency for one endpoint; or keep the mock and refuse at startup, rejected because you chose a working provider.

**Test both clients offline with `axum` mock servers.** `axum` is already a doc-indexer dependency. A small test helper binds `127.0.0.1:0`, serves canned routes, records requests, and returns a base URL to put in `base_url` (OpenAI) or `url` (Qdrant). Qdrant tests cover: insert creating a missing collection with the right size, payload round-trip through search, delete/update returning `false` for an unknown id, count, unhealthy on 503, and the `api-key` header. For OpenAI, tests cover: the happy path and order by `index`, batches over 100 inputs split across requests, a retry after 429 that then succeeds, a 401 surfacing as an error without retry, and truncation of multi-byte text without panicking. The existing tests that asserted on mock vectors are rewritten to use this server. *Alternative:* `wiremock`, a new dev-dependency for what about 30 lines of `axum` can do.

**Builds without `embedded` compile too.** The README documents `--no-default-features --features cloud`, but `--no-default-features` failed before this change, whatever else was enabled. The placeholder `LocalEmbeddingConfig` and `EmbeddedConfig` in `config.rs` were missing fields that `Config::from_env` sets, and `concurrent_container.rs` used `LocalEmbeddingAdapter` without the `embedded` gate. Add the fields and gate the branch as `container.rs` does. Tests that build a `ServiceContainer` from `Config::default()`, which means local embeddings, run only with `embedded`. Those are the HTTP router tests and the three end-to-end tests in `test_pdf_support.rs`. Without `embedded`, no embedding provider is available, and refusing that config is correct. This was found during implementation and is checked by tasks 3.5 and 9.2.

**Select backend and provider through `AppConfig`.** Found in the smoke test: `main.rs` loads `AppConfig` and calls `Config::from_app_config`, which starts from `Config::default()` (embedded + local) and copies only server, docs path, logging and timeout fields. `AppConfig` has no vector or embedding settings, and `Config::from_env()` (the `DOC_INDEXER_*` loader) has no callers, so no build can reach Qdrant or OpenAI. Even `DOC_INDEXER_PORT` is ignored.
- Add `vector` and `embedding` sections to `AppConfig` in `zero-latency-config`, with flat fields (`qdrant_url`, `openai_model`, …) and `#[serde(default)]`, so a partial `zero-latency.toml` or a single env var is enough. `backend` and `provider` stay strings in the shared crate, because the enums live in doc-indexer. `validate_config` rejects unknown values.
- Map env keys as `ZL_<SECTION>_<FIELD>`, splitting at the first `_` only. Today's `split("_")` turns every underscore into nesting, so `ZL_SERVER_DOCS_PATH` became `server.docs.path` and was silently ignored. The ADR-040 names (`ZL_TEST_PORT_BASE` and so on) assume first-underscore splitting, and single-word fields (`ZL_SERVER_PORT`) map the same either way.
- If `embedding.openai_api_key` is unset, fall back to `OPENAI_API_KEY`, the convention every OpenAI tool follows.
- `Config::from_app_config` maps both sections, parses `backend` and `provider` with the existing `FromStr` impls, and returns `Result`. `main` also runs `Config::validate()`, so `provider = openai` without a key fails at startup.
- Delete `Config::from_env()` and rewrite `env_example()` with the `ZL_` names. The `DOC_INDEXER_*` names were documented but never worked, so nothing that ran before breaks.

- `ConfigResolver::load()` (the default `zero-latency.toml` lookup) used to extract the file into a complete `AppConfig` and silently skip it if any section was missing, so a file with only `[vector]` would have been ignored. It now merges the TOML directly, as `load_from_file` already did. A side effect is that a malformed default config file now fails startup instead of being ignored.

*Alternative:* overlay `vector` and `embedding` from `Config::from_env()` in `main`. Rejected, because it keeps two config systems with different prefixes and precedence rules.

**Trim `cloud` to what it uses.** `cloud = ["reqwest"]`. Delete `qdrant_adapter_old.rs` and `qdrant_adapter_new.rs`, and remove the `qdrant-client` and `tonic` optional dependencies. `embedded` keeps its `tantivy` entry. Whether doc-indexer needs that at all belongs in `remove-dead-code`.

**Delete the `examples` feature.** It gates a stub that panics by design. A working version would need real BM25 and vector backends, and `crates/zero-latency-search/examples/hybrid_search.rs` already exists as a runnable example. Remove the feature, the file, and the `#[cfg(test)] pub mod examples;` pair from `query_expansion/mod.rs`. The attribute has to go too, or it attaches to the next `pub use`.

**CI switches to `--all-features` everywhere.** `cargo clippy --workspace --all-targets --all-features -- -D warnings -A dead_code`, `cargo test --workspace --all-features`, and `[graph] all-features = true` in `deny.toml`. Leave the release build (`cargo build --release --workspace`) on default features, because that's what ships. *Alternative:* a feature matrix (`cargo hack --each-feature`). Better at catching feature-combination bugs, but it costs much more CI time for a workspace this size. Revisit if a feature-specific break gets through.

## Risks / Trade-offs

- [Enabling all features in `cargo-deny` surfaces new advisories (reqwest 0.12's tree, tantivy 0.22 in `zero-latency-search`)] → Run `cargo deny check` locally before changing CI. Upgrade if a semver-compatible fix exists. Otherwise add an ignore with a reason and a review date, as `deny.toml` requires.
- [OpenAI dimension (1536/3072) differs from the configured vector store dimension] → Out of scope. `dimension()` already reports the model's true size, so a mismatch fails at insert rather than silently. Mention it in the provider docs.
- [The real client needs a key, so `provider = "openai"` with an empty key now fails at startup] → This already happens: `config.rs` validation rejects an empty `OPENAI_API_KEY`. No change in behaviour.
- [`--all-features` CI is slower (tantivy, reqwest 0.12)] → `Swatinem/rust-cache` covers repeat runs. The release build stays on default features.
- [Retry and backoff make test runs slow] → The base delay defaults to 500 ms and can be changed with `OpenAIAdapter::with_retry_base_delay`. Tests use 1 ms. It's a builder method rather than a config field, so `OpenAIConfig` and the env loading stay as they are.
- [Mock servers can drift from the real Qdrant/OpenAI APIs] → Keep the request and response shapes to documented, stable endpoints. Add a manual check task: run against a local `qdrant/qdrant` container and a real key once before archiving.
- [Collection created on first insert with the wrong dimension (for example, after switching embedding model)] → Qdrant rejects later inserts with a dimension error, and we surface it with the status and body. Recreating the collection stays manual.

- [Found during implementation: `cargo deny check` was already failing on default features because of RUSTSEC-2026-0187 (stack overflow in `lopdf` 0.34, via `pdf-extract` 0.7)] → Upgrade `pdf-extract` to 0.12 (`lopdf` 0.42), which needs no code changes. That brings in `ttf-parser` 0.25, which is unmaintained (RUSTSEC-2026-0192) with no replacement release. It's ignored with a reason and review date, like the other unmaintained crates.

## Migration Plan

Nothing to migrate: default builds and the shipped binary are unchanged. Roll back by reverting the commit. CI then returns to default features with the excluded-features note.

## Open Questions

- Does `ZeroLatencyError` have an external-service variant suited to HTTP failures? The tantivy errors show `ExternalService { service, message }`, so probably yes. Confirm the constructor name when implementing.
