## 1. Pin the model

- [x] 1.1 Where `huggingface.co` is reachable, record the bge-small-en-v1.5 commit revision and the size and SHA-256 of `onnx/model.onnx` and `tokenizer.json` (from the HF tree API's `lfs.oid`, confirmed by hashing a download). Put them in constants in a new `persistence/embeddings/model_files.rs`
- [x] 1.2 Inspect the ONNX graph's input and output names and dtypes (`input_ids`, `attention_mask`, `token_type_ids` → `last_hidden_state`), and record them in the design if they differ

## 2. Keep the hash embedder for tests

- [x] 2.1 Rename `local_adapter.rs` to `hash_adapter.rs` (`HashEmbeddingAdapter`, `HashEmbeddingConfig`) with the logic unchanged, and fix re-exports
- [x] 2.2 Add `EmbeddingProvider::Hash` (`"hash"`) to doc-indexer config and to `zero-latency-config` validation. Wire it in both containers
- [x] 2.3 Add `Config::for_tests()` (memory backend + hash provider) and switch every test that builds a `ServiceContainer` to it. `cargo test --workspace --all-features` passes with no model files present

## 3. Trait and indexing changes

- [x] 3.1 Add `generate_query_embedding` (defaulting to `generate_embedding`) and `model_id` (defaulting to `<model_name>@<dimension>`) to `EmbeddingGenerator`
- [x] 3.2 Make both containers' search `EmbeddingServiceAdapter`s call `generate_query_embedding`
- [x] 3.3 Make `DocumentIndexingService::index_document_with_collection` embed all chunks with one `generate_batch_embeddings` call. Existing indexing tests pass

## 4. Model files

- [x] 4.1 Implement `ModelStore`: resolve the directory (`local_model_path` or `~/.zero-latency/models/bge-small-en-v1.5`), verify size and SHA-256, download missing files from `<base_url>/resolve/<revision>/…` with streaming hash into `.part`, then rename. Re-download a corrupt cached file once. Never download into a user-supplied path
- [x] 4.2 Add `embedding.local_model_path` and `embedding.local_model_url` to `AppConfig` and doc-indexer config, and map them in `from_app_config`. Make `embedded` enable `reqwest`, and add `sha2` as a direct dependency
- [x] 4.3 Add mock-server tests (`mock_http.rs`, made available to `embedded` builds as well as `cloud`) for every scenario in "Model files are obtained and verified", using small fake files with real hashes

## 5. ONNX adapter

- [x] 5.1 Implement `OnnxEmbeddingAdapter`: load the tokenizer (truncate 512, pad to longest) and the `ort` session from the verified files, run batches of up to 32 in `spawn_blocking`, CLS-pool, L2-normalise. Override `generate_query_embedding` with the bge prefix. `model_id` = `bge-small-en-v1.5@<revision>`
- [x] 5.2 Make `provider = local` build `OnnxEmbeddingAdapter` in both containers. Reject `local_dimension != 384`
- [x] 5.3 Add `#[ignore]` ONNX tests, which read the model from `ZL_EMBEDDING_LOCAL_MODEL_PATH` or skip with a message, covering every scenario in "Local provider produces semantic embeddings"
- [x] 5.4 Measure and record in the design: model load time, startup hash time, chunks/s when indexing, per-query latency, and resident memory with the model loaded. Add the `.verified` cache if startup hashing is over about 1 s

## 6. Stale index handling

- [x] 6.1 Add the `index_meta` table to the embedded store, with read/write of `embedding_model` and `embedding_dimension`
- [x] 6.2 On container start, compare against the active generator's `model_id`. On a mismatch or legacy vectors, delete vectors in a transaction, write the meta, log a `warn` with the count, and set `reindex_required`
- [x] 6.3 Expose `reindex_required` on `/api/status` (and in the OpenAPI schema), and clear it after a successful index. *Note: `api/public/openapi.yaml` is an empty stub (`paths: {}`), so the field is documented in the README instead*
- [x] 6.4 Test the legacy-hash-store, same-model and re-index-clears-flag scenarios against a temp-dir store with the hash provider and a fake model id

## 7. CLI, Docker and CI

- [x] 7.1 Add `doc-indexer --fetch-model`: resolve and download, verify, print the directory, exit 0. Test it against the mock server
- [x] 7.2 Make the Dockerfile run `--fetch-model` into `/app/models/bge-small-en-v1.5` in the build stage, copy it into the runtime image, and set `ZL_EMBEDDING_LOCAL_MODEL_PATH`
- [x] 7.3 Add a `model-tests` CI job: cache `~/.zero-latency/models` keyed on the revision, run `--fetch-model`, then `cargo test -p doc-indexer -- --ignored onnx`

## 8. Docs and verification

- [x] 8.1 Update the README (local embeddings, first-run download and size, the offline path, re-indexing after upgrade, Qdrant users recreate the collection) and `--env-example`. Add a CHANGELOG entry marked BREAKING with the re-index and rollback steps
- [x] 8.2 `cargo fmt`, all-features clippy, `cargo deny check` (new direct deps), and `cargo test` for all-features, default and no-default all pass offline
- [x] 8.3 Run the ignored ONNX tests with the real model, then do a manual smoke test: start on a copy of a legacy store, confirm the warning and `reindex_required`, re-index the smoke docs, and confirm the Kubernetes query ranks `deploy.md` first. *Result: the ignored tests pass (6/6), and the legacy-store warning, `reindex_required` and re-index behave as specified. `/api/search` ranked `deploy.md` second at first because of the result ranker, which is fixed under 8.4. After that `deploy.md` ranks first, consistently*
- [x] 8.4 Make vector similarity dominate `MultiFactorResultRanker` (weight 0.8, heuristics scaled to [0, 1]) and choose its derived terms deterministically, with regression tests that fail on the old ranker
