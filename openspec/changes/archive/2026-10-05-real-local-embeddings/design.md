## Context

- `EmbeddingProvider::Local` builds `LocalEmbeddingAdapter` (`persistence/embeddings/local_adapter.rs`). It derives a 384-dimension vector from the text with a seeded deterministic algorithm. It is fast, offline and reproducible, but not semantic.
- The `embedded` feature already declares `ort = 2.0.0-rc.10` (with `download-binaries`, so ONNX Runtime is fetched at build time), `tokenizers = 0.15` and `ndarray = 0.15`. No code uses them. `tokenizers`' `http`/`hf-hub` feature is off.
- Both containers (`container.rs`, `concurrent_container.rs`) pick the generator from `config.embedding.provider`. Indexing (`DocumentIndexingService::index_document_with_collection`) calls `generate_embedding` once per chunk. Search goes through an `EmbeddingServiceAdapter` in each container, which also calls `generate_embedding`. The two paths can't be told apart today.
- The embedded store (`embedded_adapter.rs`) has a single `vectors(id, embedding, metadata, created_at)` table and no record of how its vectors were made. A typical local store, the author's included, is full of hash vectors.
- Several tests build a `ServiceContainer` from `Config::default()` (provider `local`): `test_pdf_support.rs`, the HTTP server tests and others. Making `local` mean "download a 130 MB model" would put the network into `cargo test`.
- Config arrives through `AppConfig` (`ZL_EMBEDDING_*`, `[embedding]`), which `resolve-broken-features` added.
- This session's sandbox can't resolve `huggingface.co`, so the exact revision and file hashes are pinned during implementation (task 1.1), not here.

## Goals / Non-Goals

**Goals:**
- `provider = "local"` gives semantic embeddings from bge-small-en-v1.5, with no API key and offline after the first run.
- Model files are verified before use. A partial or tampered download is never loaded.
- A store written by a different embedding model is never silently searched.
- `cargo test` (default features and `--all-features`) stays offline and model-free. The real model is still tested in CI.

**Non-Goals:**
- Choosing a model at runtime from a list, or arbitrary user models. The adapter is written for one pinned model, and the code is structured so adding a second is a small change.
- GPU or CoreML execution providers. CPU only.
- Automatic re-indexing. A stale store is detected and cleared, and the user triggers the re-index.
- Model tracking for Qdrant collections. Documented as "recreate the collection".
- Replacing `ort` or `tokenizers`, or upgrading `ort` past `2.0.0-rc.10`.

## Decisions

**Model: bge-small-en-v1.5 in ONNX form, CLS pooling, query prefix.** 384 dimensions, which matches every existing dimension default, so config and stores don't need resizing, and it retrieves noticeably better than MiniLM. Use the ONNX export in the model repo's `onnx/` folder plus `tokenizer.json`. Following the model card:
- **Inputs:** `input_ids`, `attention_mask` and `token_type_ids` (zeros), as `i64`. Truncate at 512 tokens and pad to the longest in the batch.
- **Pooling:** take the CLS token (index 0) of `last_hidden_state`, then L2-normalise.
- **Queries** are prefixed with `Represent this sentence for searching relevant passages: `. Documents aren't.

*Alternatives:* `fastembed` (wraps `ort` and `tokenizers` and handles downloads). Rejected: it pins its own `ort` release candidate, which risks two ONNX Runtime builds, and it hides the download and verification behaviour we want to control and test. Mean pooling, rejected because bge is trained for CLS.

**Query and document embeddings are separate trait calls.** Add `async fn generate_query_embedding(&self, text: &str) -> Result<Vec<f32>>` to `EmbeddingGenerator`, with a default implementation that calls `generate_embedding`, so the OpenAI and hash providers don't change. Both containers' search adapters call it. The ONNX adapter overrides it to add the prefix. *Alternative:* a flag on `generate_embedding`. Rejected because it changes every implementation and call site.

**Indexing uses batches.** `index_document_with_collection` calls `generate_batch_embeddings` once per document (all chunks) instead of once per chunk. The ONNX adapter splits batches into groups of 32 to bound memory. This matters for a real model; it's free for the others.

**Inference runs on the blocking pool behind one shared session.** `ort::Session` is wrapped in `Arc<Mutex<Session>>`. Each batch runs in `tokio::task::spawn_blocking`, locks the session, and runs tokenize → `run` → pool → normalise. Intra-op threads default to ONNX Runtime's choice. One session at a time is simpler, and the model is small enough that queuing beats loading several copies. Revisit if load tests show contention.

**Model files: resolve, download if missing, verify.** A `ModelStore` (`persistence/embeddings/model_files.rs`) resolves the directory in this order:
1. `embedding.local_model_path` (`ZL_EMBEDDING_LOCAL_MODEL_PATH`) if set. Files must already be there; it's never downloaded into.
2. Otherwise `~/.zero-latency/models/bge-small-en-v1.5/`.

Required files are `model.onnx` and `tokenizer.json`, each with a size and SHA-256 compiled in, from a pinned Hugging Face revision (`https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/<revision>/…`).
- **On startup**, a missing file in the default directory is downloaded with `reqwest` (streamed to `<name>.part`), hashed while streaming, then checked and renamed into place. It is never loaded unverified. A file that's present but wrong is treated as corrupt: in the default directory it's re-downloaded once, and under a user-supplied path it's a startup error naming the file and both hashes.
- **Every run** checks hashes before loading (about 130 MB of hashing on startup, a few hundred ms). That's the price of never loading a corrupt model.
- Progress and the destination are logged at `info`. The base URL can be overridden (`ZL_EMBEDDING_LOCAL_MODEL_URL`) for mirrors and for tests.

*Alternative:* `hf-hub` (via `tokenizers/http`). Rejected because it adds its own HTTP stack and cache layout, and doesn't do hash pinning.

**`--fetch-model` for images and CI.** `doc-indexer --fetch-model` resolves and downloads the model, verifies it, prints the directory and exits 0. The Dockerfile runs it during the build into `/app/models/bge-small-en-v1.5`, then sets `ZL_EMBEDDING_LOCAL_MODEL_PATH` to that directory, so containers never touch the network.

**Index metadata and stale-store handling.** The embedded store gets an `index_meta(key TEXT PRIMARY KEY, value TEXT)` table with `embedding_model` (for example `bge-small-en-v1.5@<revision>` or `hash-v1`) and `embedding_dimension`. `EmbeddingGenerator` gains a `fn model_id(&self) -> String` with a default implementation built from `model_name()` and `dimension()`. When the container opens the store with the active generator:
- If there's no `index_meta` and no vectors, the meta is written.
- If there's no `index_meta` but vectors exist, those are legacy hash vectors: they're treated as model `hash-v1`, which is a mismatch.
- If the meta matches, nothing happens.
- On a mismatch, `vectors` is deleted in a transaction, the meta is rewritten, and a `warn` reports how many vectors were dropped and says to re-index. The flag `reindex_required = true` is held in memory and surfaced on `/api/status` until the next successful index.

*Alternatives:*
- Refuse to start. Rejected because the macOS app would just fail to launch.
- Keep the stale vectors. Rejected because searching hash vectors with a real query vector returns confident nonsense.
- Re-index automatically from `docs_path`. Rejected because indexing can be slow and the user may index several paths. Out of scope.

**The hash provider stays, for tests.** Rename `local_adapter.rs` to `hash_adapter.rs` (`HashEmbeddingAdapter`, `HashEmbeddingConfig`), with the logic unchanged. Add `EmbeddingProvider::Hash` (`"hash"`), allowed in `AppConfig` validation and documented as "testing only, not semantic". Container-building tests set `provider = Hash` through a small test helper (`Config::for_tests()`). The ONNX adapter is `OnnxEmbeddingAdapter`. `LocalEmbeddingConfig` gains `model_path: Option<PathBuf>` and `model_url: Option<String>`. `dimension` stays for compatibility, but any value other than 384 is rejected while the provider is `local`.

**Testing the real model.** ONNX tests are `#[ignore]` with a clear reason. They load the model from `ZL_EMBEDDING_LOCAL_MODEL_PATH`, or skip with a message if it's unset. They check: the output is 384-dimensional and L2-normalised; the same text gives the same vector; a semantically related pair ("How do I deploy with rolling updates?" vs a Kubernetes passage) scores higher than an unrelated one (sourdough); batch results equal single results; the query prefix changes the query vector. `ModelStore` is tested offline against the `axum` mock server (`mock_http.rs`), with a tiny fake "model" file and its real SHA-256: download, `.part` cleanup on hash mismatch, corrupt-file re-download, user path never downloaded into. A new CI job, `model-tests`, caches `~/.zero-latency/models` (key: model revision), runs `doc-indexer --fetch-model`, then `cargo test -p doc-indexer -- --ignored onnx`. The main `test` job stays offline.

## Risks / Trade-offs

- [`ort` 2.0.0-rc.10 API churn: tensor and output extraction APIs changed between release candidates] → Pin the exact version (already pinned) and keep all `ort` calls in `onnx_adapter.rs`.
- [Hugging Face unreachable on first run (offline laptop, corporate proxy, this sandbox)] → The startup error names the URL, the destination directory and both remedies (`--fetch-model` elsewhere and copy, or `ZL_EMBEDDING_LOCAL_MODEL_PATH`). `reqwest` honours `HTTPS_PROXY`.
- [Existing users lose search results until they re-index] → A loud `warn`, `/api/status.reindex_required`, and a CHANGELOG entry marked BREAKING with the one-line re-index command.
- [Hashing 130 MB on every start slows startup] → Measure. If it's over about 1 s, cache `(size, mtime, sha)` in `<dir>/.verified` and re-hash only when size or mtime changes.
- [The CPU cost of indexing large corpora goes up a lot compared with hashing] → Batching plus the blocking pool. Throughput (chunks/s) is measured and recorded here during implementation.
- [Memory: ONNX Runtime plus a loaded model adds noticeable resident memory] → Measure resident memory before and after loading the model, and record it here and in the README.
- [Tests drift to depend on the hash provider's arbitrary ranking] → The hash provider's docs and the test helper say it's not semantic. Ranking-quality tests exist only for the real model.
- [Qdrant users keep hash vectors] → Documented in the README and CHANGELOG: delete the collection and re-index.

## Migration Plan

1. Ship. On first start, the model downloads and the stale embedded store is cleared, with a warning.
2. The user re-indexes (`POST /api/index` or the app's index action).
3. **Rollback:** revert the release, delete `~/.zero-latency/vectors.db`, and re-index. The old binary has no model check, so it would otherwise search bge vectors with hash queries. Note this in the CHANGELOG.

## Implementation Notes

- `ort` 2.0.0-rc.10 depends on `ndarray` 0.16, while `embedded` declares 0.15, so the adapter doesn't use `ndarray`. Tensors are built from `(shape, Vec<i64>)` and the output is read with `try_extract_tensor::<f32>()`.
- The adapter passes `token_type_ids` only if the graph declares it, and reads `last_hidden_state` (falling back to the first output). It checks the output is `[batch, seq, 384]`.
- `--fetch-model` takes an optional directory, because a user-supplied model path is never downloaded into. `--fetch-model DIR` downloads into DIR (used by the Dockerfile). A bare `--fetch-model` uses `ZL_EMBEDDING_LOCAL_MODEL_PATH` (verify only) or the default directory.
- The vectors are deleted and the flag set only when vectors were actually removed. A mismatched store with no vectors just records the new model.
- The hash provider's `model_id` is `hash-v1@<dimension>:<seed>`, because the seed changes the vectors.

## Result Ranking

This was added during implementation, after the smoke test (task 8.3). `MultiFactorResultRanker` re-scores results after the vector search. Its heuristic signals (content relevance, title boost, metadata, exact match) never see the query: it derives "query terms" from the results themselves. Those signals made up 60% of the score, and the title boost (1.0–3.0) and the exact-match bonus (added unweighted, up to 0.2) could each exceed their nominal share. With hash vectors this hardly mattered. With bge it let a short heading-like chunk outrank a clearly closer passage. The terms were also chosen in `HashMap` order, so the same query could rank differently on each run: `deploy.md` scored 0.652 in one run and 0.700 in another.

**Changes:**
- Vector similarity weight is 0.8. The heuristics share 0.2: content 0.06, title 0.04, metadata 0.04, exact match 0.04, recency 0.02.
- The title boost and the exact-match bonus are scaled to [0, 1] before weighting, so similarity is always at least 80% of the final score.
- Derived terms are chosen by frequency, then alphabetically.

This affects every provider, OpenAI included. Passing the real query through to the ranker would make the heuristics useful again. That's out of scope here.

## Measurements

These were taken on 2026-10-04 on the author's macOS laptop, with a release build, ONNX Runtime's default threads and the pinned revision `5c38ec7c…`.

| What | Result |
|---|---|
| First-run download (both files, 127.6 MB) | 51 s |
| Startup verification (size and SHA-256 of both files) | 422 ms. This is under the ~1 s threshold, so the `.verified` cache isn't needed |
| Model load (tokenizer and ONNX session) | 123 ms |
| Indexing throughput | ~10 chunks/s: 164 Markdown files from `docs/` (1.7 MB) gave 177 chunks in 17.2 s. Most chunks hit the 512-token cap, because the indexer's chunks are up to 50 sentences |
| Query latency, end to end through `/api/search` (including embedding) | p50 11 ms, max 13 ms (10 runs) |
| Resident memory with the model loaded | 270–330 MB, against ~12 MB for the hash provider |

Graph signature (task 1.2): the inputs are `input_ids`, `attention_mask` and `token_type_ids`, all `Int64` `[batch, seq]`, and the output is `last_hidden_state`, `Float32` `[batch, seq, 384]`. This matches the design.

## Open Questions

- Pinned revision and SHA-256 hashes: filled in by task 1.1, where `huggingface.co` is reachable.
- Whether the macOS app bundle should also ship the model (no first-run download). Left to the packaging scripts, outside this change.
