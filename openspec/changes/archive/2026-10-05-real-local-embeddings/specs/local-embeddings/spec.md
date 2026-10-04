## ADDED Requirements

### Requirement: Local provider produces semantic embeddings
With the `embedded` feature and `embedding.provider = "local"`, embeddings SHALL come from the bge-small-en-v1.5 model run through ONNX Runtime. Each embedding SHALL be 384-dimensional and L2-normalised. Query embeddings SHALL carry the model's retrieval prefix (`Represent this sentence for searching relevant passages: `), and document embeddings SHALL NOT.

#### Scenario: Related text ranks higher
- **WHEN** the query "How do I deploy with rolling updates?" is compared with a passage about Kubernetes rolling updates and a passage about sourdough baking
- **THEN** the Kubernetes passage has the higher cosine similarity

#### Scenario: Shape and normalisation
- **WHEN** any non-empty text is embedded
- **THEN** the vector has 384 values and an L2 norm within 0.001 of 1.0

#### Scenario: Deterministic
- **WHEN** the same text is embedded twice
- **THEN** both vectors are identical

#### Scenario: Batch equals single
- **WHEN** three texts are embedded in one batch and also one at a time
- **THEN** each batch vector matches its single vector within 1e-5 per value

#### Scenario: Query prefix
- **WHEN** the same text is embedded as a query and as a document
- **THEN** the two vectors differ

### Requirement: Model files are obtained and verified
The local provider SHALL load `model.onnx` and `tokenizer.json` only after checking each file's SHA-256 against values pinned in the source for a fixed model revision. If `embedding.local_model_path` is set, files SHALL be read from that directory and SHALL NOT be downloaded. Otherwise they SHALL be read from `~/.zero-latency/models/bge-small-en-v1.5/` and downloaded there when missing. Downloads SHALL be written to a temporary file and moved into place only after the hash matches.

#### Scenario: First run downloads
- **WHEN** the default model directory is empty and the download source is reachable
- **THEN** both files are downloaded, verified and moved into place, and the provider starts

#### Scenario: Offline after first run
- **WHEN** both verified files are present and the network is unavailable
- **THEN** the provider starts without any network request

#### Scenario: Corrupt download
- **WHEN** a downloaded file's SHA-256 doesn't match the pinned value
- **THEN** the temporary file is deleted, no file is moved into place, and startup fails with an error naming the file and both hashes

#### Scenario: Corrupt cached file
- **WHEN** a file in the default directory fails verification
- **THEN** it is downloaded again once, and startup fails if the new copy also fails

#### Scenario: User-supplied directory
- **WHEN** `ZL_EMBEDDING_LOCAL_MODEL_PATH` points at a directory missing `model.onnx`
- **THEN** startup fails with an error naming the path, and no download is attempted

#### Scenario: Fetch for images
- **WHEN** `doc-indexer --fetch-model` runs
- **THEN** the model files are downloaded and verified, the directory is printed, and the process exits 0 without starting the server

### Requirement: Stale indexes are detected
The embedded vector store SHALL record the embedding model identifier and dimension that wrote its vectors. When the store is opened with a different model, or holds vectors with no recorded model, its vectors SHALL be removed, a warning SHALL state how many were removed, and `/api/status` SHALL report `reindex_required: true` until the next successful index.

#### Scenario: Legacy hash store
- **WHEN** `doc-indexer` starts with the local provider on a store containing vectors and no model record
- **THEN** those vectors are removed, a warning reports the count, and `/api/status` shows `reindex_required: true`

#### Scenario: Same model
- **WHEN** the store's recorded model matches the active provider
- **THEN** no vectors are removed and `reindex_required` is `false`

#### Scenario: Re-index clears the flag
- **WHEN** a stale store has been cleared and a path is then indexed successfully
- **THEN** `/api/status` shows `reindex_required: false`

### Requirement: Hash provider is test-only
`embedding.provider = "hash"` SHALL select the deterministic, non-semantic hash embedder, which needs no model files or network. It SHALL NOT be the default, and the documentation SHALL describe it as for testing only.

#### Scenario: Offline test container
- **WHEN** a test builds a service container with the hash provider and no model files exist
- **THEN** the container starts and indexing and search work without network access

#### Scenario: Local dimension fixed
- **WHEN** `embedding.provider = "local"` and `embedding.local_dimension` is set to a value other than 384
- **THEN** startup fails with a configuration error
