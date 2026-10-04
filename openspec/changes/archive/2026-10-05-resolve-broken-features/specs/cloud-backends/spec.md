## ADDED Requirements

### Requirement: Backend and provider are chosen by configuration
The `doc-indexer` binary SHALL take its vector backend (`embedded`, `memory`, `qdrant`) and embedding provider (`local`, `openai`), with their settings, from the shared `AppConfig`: `ZL_VECTOR_*` and `ZL_EMBEDDING_*` environment variables, or the `[vector]` and `[embedding]` tables in the config file. Environment variables take precedence over the file. When `embedding.openai_api_key` is unset, `OPENAI_API_KEY` SHALL be used. An unknown backend or provider, or `openai` without a key, SHALL stop startup with an error.

#### Scenario: Qdrant selected by environment
- **WHEN** `doc-indexer` starts with `ZL_VECTOR_BACKEND=qdrant` and `ZL_VECTOR_QDRANT_URL=http://localhost:6333`
- **THEN** indexed documents are stored in Qdrant at that URL

#### Scenario: Multi-word setting from environment
- **WHEN** `ZL_VECTOR_QDRANT_COLLECTION=team_docs` is set
- **THEN** the Qdrant adapter uses the `team_docs` collection

#### Scenario: OpenAI key fallback
- **WHEN** `ZL_EMBEDDING_PROVIDER=openai` and only `OPENAI_API_KEY` is set
- **THEN** the OpenAI adapter uses that key

#### Scenario: Invalid selection
- **WHEN** `ZL_VECTOR_BACKEND=pinecone` is set, or `ZL_EMBEDDING_PROVIDER=openai` with no key
- **THEN** `doc-indexer` exits at startup with a configuration error naming the problem

#### Scenario: Defaults unchanged
- **WHEN** no vector or embedding settings are given
- **THEN** `doc-indexer` uses the embedded store and local embeddings, as before


### Requirement: OpenAI embeddings come from the OpenAI API
With the `cloud` feature and `embedding.provider = "openai"`, the embedding generator SHALL get embeddings by calling `POST {base_url}/embeddings`, where `base_url` defaults to `https://api.openai.com/v1`, sending the configured model and a bearer API key. It SHALL NOT return placeholder or locally computed vectors.

#### Scenario: Single embedding
- **WHEN** `generate_embedding("hello")` is called and the API returns one embedding
- **THEN** the returned vector is exactly the API's `data[0].embedding`
- **AND** the request carried `Authorization: Bearer <api_key>` and the configured `model`

#### Scenario: Batch order follows the API index
- **WHEN** a batch of three texts is embedded and the API returns `data` out of order
- **THEN** the results are ordered by each item's `index`, matching the input order

#### Scenario: Large batch is split
- **WHEN** more than 100 texts are embedded
- **THEN** the adapter sends several requests of at most 100 inputs each and returns one embedding per input, in order

#### Scenario: Count mismatch
- **WHEN** the API returns a different number of embeddings than inputs sent
- **THEN** the call fails with an error

### Requirement: OpenAI failures are retried or surfaced
The OpenAI adapter SHALL retry rate-limit (429), server (5xx) and connection errors up to `max_retries` times with backoff. It SHALL return an error that includes the HTTP status and the API's error message for any other failure, or once retries run out.

#### Scenario: Transient rate limit
- **WHEN** the API returns 429 once and then 200
- **THEN** the call succeeds after one retry

#### Scenario: Bad key
- **WHEN** the API returns 401
- **THEN** the call fails immediately, without retrying, with an error mentioning the status

### Requirement: Long input is truncated safely
The OpenAI adapter SHALL truncate input text longer than its character limit at a character boundary, and SHALL NOT panic on multi-byte UTF-8.

#### Scenario: Multi-byte text over the limit
- **WHEN** a text of more than 8000 characters made of multi-byte characters is embedded
- **THEN** the request is sent with the text truncated, and no panic occurs

### Requirement: Qdrant stores what is inserted
With the `cloud` feature and `vector.backend = "qdrant"`, `insert` SHALL upsert each vector to the configured collection over Qdrant's REST API, using `VectorDocument.id` as the point id and storing every `VectorMetadata` field in the payload. If the collection doesn't exist, the first insert SHALL create it, sized to the vectors' dimension, with cosine distance.

#### Scenario: Insert then search
- **WHEN** a vector with title, content, heading path, url, collection and custom metadata is inserted, and a search returns that point
- **THEN** the returned metadata equals the inserted metadata

#### Scenario: Missing collection
- **WHEN** the first insert runs against a Qdrant instance without the configured collection
- **THEN** the adapter creates the collection with `size` equal to the vector length and `distance` `Cosine`, then upserts the points

#### Scenario: Search within a logical collection
- **WHEN** documents indexed into collections `guides` and `notes` share the configured Qdrant collection, and `search_in_collection("guides", …)` is called
- **THEN** only points whose payload `collection` is `guides` are returned

#### Scenario: Search before anything is indexed
- **WHEN** a search runs and the configured Qdrant collection doesn't exist yet
- **THEN** it returns no results, without an error

#### Scenario: Qdrant rejects the write
- **WHEN** Qdrant responds to an upsert with a non-2xx status
- **THEN** `insert` fails with an error that includes the status and response body

### Requirement: Qdrant delete, update, count and health reflect the server
`delete` and `update` SHALL act on the point whose id is the given `document_id`, and SHALL return `false` when no such point exists. `count` SHALL return the exact number of points in the collection, or 0 if the collection doesn't exist. `health_check` SHALL report unhealthy when Qdrant's health endpoint doesn't return success. When `api_key` is configured, every request SHALL send it in the `api-key` header.

#### Scenario: Delete an existing point
- **WHEN** `delete(id)` is called for a point that exists
- **THEN** the point is deleted in Qdrant and the call returns `true`

#### Scenario: Update an unknown point
- **WHEN** `update(id, vector)` is called for an id that doesn't exist
- **THEN** no write is sent and the call returns `false`

#### Scenario: Count
- **WHEN** the collection holds 3 points
- **THEN** `count()` returns 3

#### Scenario: Qdrant down
- **WHEN** Qdrant's health endpoint returns 503 or can't be reached
- **THEN** `health_check()` reports unhealthy

#### Scenario: API key sent
- **WHEN** `api_key` is set and any adapter method makes a request
- **THEN** the request carries the `api-key` header with that value
