## ADDED Requirements

### Requirement: Real embedding model is tested
CI SHALL run the ONNX embedding tests against the real bge-small-en-v1.5 model in a separate job. That job SHALL restore the model directory from a cache keyed on the pinned model revision, fetch it with `doc-indexer --fetch-model` on a cache miss, and run the ignored ONNX tests. The main test job SHALL NOT need network access or model files.

#### Scenario: Cache hit
- **WHEN** the model-tests job runs and the cache holds the pinned revision
- **THEN** no model download happens, and the ONNX tests run and must pass

#### Scenario: Main job stays offline
- **WHEN** the main Rust test job runs `cargo test --workspace --all-features`
- **THEN** no test downloads model files or contacts `huggingface.co`

#### Scenario: Model behaviour regresses
- **WHEN** a change makes the related-text ranking scenario fail
- **THEN** the model-tests job fails
