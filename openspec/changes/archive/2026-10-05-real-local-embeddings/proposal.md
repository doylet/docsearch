## Why

The default embedding provider doesn't understand text. `LocalEmbeddingAdapter` turns each text into a deterministic pseudo-random vector with a simple algorithm, so "similar" vectors share hash patterns, not meaning. Every build that doesn't use OpenAI (the default, the macOS app, the Docker image) ranks search results more or less arbitrarily. The `embedded` feature has carried `ort` (ONNX Runtime) and `tokenizers` for a real model since it was created, but nothing uses them.

## What Changes

- **Real local embeddings:** `embedding.provider = "local"` runs **bge-small-en-v1.5** (BAAI, MIT, 384 dimensions) through ONNX Runtime: tokenize, run the model, take the CLS vector, normalise. Inference runs off the async runtime. Queries get bge's retrieval instruction prefix; documents don't.
- **Model files on first use:** with no model present, `doc-indexer` downloads the ONNX model and tokenizer from a pinned Hugging Face revision into `~/.zero-latency/models/bge-small-en-v1.5/`, checks each file's SHA-256, and works offline from then on. `ZL_EMBEDDING_LOCAL_MODEL_PATH` points at pre-supplied files instead, for Docker and air-gapped installs, and in that case nothing is downloaded. `doc-indexer --fetch-model` downloads and exits, so images can bake the model in at build time.
- **Stale index detection:** the embedded store records which embedding model wrote its vectors. If you open a store written by a different model, including today's hash vectors, its vectors are discarded, a warning says how many, and `/api/status` reports that a re-index is needed. The source documents aren't touched. **BREAKING** for existing local installs: search returns nothing until you re-index.
- **The hash embedder stays, for tests only:** it's renamed `HashEmbeddingAdapter` behind a new `embedding.provider = "hash"`, documented as test-only. Tests that build a service container use it, so `cargo test` and CI never need the network or the model. The real model gets its own `#[ignore]`d tests, which a CI job runs with a cached model directory.
- **Config:** `embedding.local_dimension` is no longer free-form. The model fixes it at 384, and any other value is rejected. Adds `embedding.local_model_path`.
- **Docker:** the image runs `--fetch-model` at build time and sets `ZL_EMBEDDING_LOCAL_MODEL_PATH`, so containers start offline.

## Capabilities

### New Capabilities
- `local-embeddings`: the local provider produces semantic embeddings from bge-small-en-v1.5, how its model files are obtained and verified, how stale indexes are detected, and the test-only hash provider.

### Modified Capabilities
- `ci-pipeline`: adds a requirement that the real model is exercised in CI from a cached download, while the main test job stays offline.

## Impact

- **Code:** `services/doc-indexer/src/infrastructure/persistence/embeddings/` (a new `onnx_adapter.rs` and model download/verify module, with `local_adapter.rs` renamed to `hash_adapter.rs`), `config.rs` (`EmbeddingProvider::Hash`, model path), both containers, `main.rs` (`--fetch-model`), `embedded_adapter.rs` (an `index_meta` table and model check), `/api/status`. Shared config: `zero-latency-config` gains `embedding.local_model_path` and accepts `hash` as a provider.
- **Dependencies:** `embedded` additionally enables `reqwest` (download) and `sha2` (already in the tree transitively). `ort` and `tokenizers` move from declared-but-unused to used. Builds that pull `ort`'s prebuilt ONNX Runtime don't change.
- **Disk and network:** about 130 MB in `~/.zero-latency/models` on first run, plus one download from `huggingface.co`. The Docker image grows by the same.
- **Performance:** indexing becomes CPU-bound in model inference, batched, and each query adds one model call. Both are measured during implementation and recorded in the design.
- **Users:** existing embedded stores must be re-indexed once. Qdrant collections built with hash vectors need recreating; that's documented, not automated.
- **Order:** after `resolve-broken-features`, which introduced `ZL_EMBEDDING_*` config and is implemented but not yet archived.
