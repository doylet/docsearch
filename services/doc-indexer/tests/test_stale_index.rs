//! Stale-index detection, covering the "Stale indexes are detected" scenarios.
//!
//! Each test uses its own temp-dir embedded store with the hash provider, so no
//! model files or network are needed.
#![cfg(feature = "embedded")]

use doc_indexer::application::services::document_service::DocumentIndexingService;
use doc_indexer::application::ServiceContainer;
use doc_indexer::config::{Config, VectorBackend};
use doc_indexer::infrastructure::{EmbeddedConfig, EmbeddedVectorStore};
use std::path::Path;
use zero_latency_core::Uuid;
use zero_latency_vector::{VectorDocument, VectorMetadata, VectorRepository};

fn config(dir: &Path) -> Config {
    let mut config = Config::for_tests();
    config.vector.backend = VectorBackend::Embedded;
    config.vector.embedded = EmbeddedConfig {
        db_path: dir.join("vectors.db"),
        dimension: 384,
        cache_size: 100,
        enable_string_interning: false,
        enable_smart_caching: false,
    };
    config
}

async fn open_store(config: &Config) -> EmbeddedVectorStore {
    EmbeddedVectorStore::new(config.vector.embedded.clone())
        .await
        .unwrap()
}

fn vector() -> VectorDocument {
    VectorDocument {
        id: Uuid::new_v4(),
        embedding: vec![0.1; 384],
        metadata: VectorMetadata {
            document_id: Uuid::new_v4(),
            chunk_index: 0,
            content: "old hash-embedded content".to_string(),
            title: "old.md".to_string(),
            heading_path: vec![],
            url: None,
            collection: Some("zero_latency_docs".to_string()),
            custom: Default::default(),
        },
    }
}

#[tokio::test]
async fn legacy_store_is_cleared_and_flagged() {
    let temp = tempfile::tempdir().unwrap();
    let config = config(temp.path());
    // A store from before model tracking: vectors, no index_meta
    open_store(&config)
        .await
        .insert(vec![vector(), vector(), vector()])
        .await
        .unwrap();

    let container = ServiceContainer::new(config).await.unwrap();

    assert!(container.index_state().reindex_required());
    assert_eq!(container.vector_repository().count().await.unwrap(), 0);
}

#[tokio::test]
async fn store_from_another_model_is_cleared() {
    let temp = tempfile::tempdir().unwrap();
    let config = config(temp.path());
    let store = open_store(&config).await;
    store.reconcile_model("fake-model@1", 384).await.unwrap();
    store.insert(vec![vector()]).await.unwrap();
    drop(store);

    let container = ServiceContainer::new(config).await.unwrap();

    assert!(container.index_state().reindex_required());
    assert_eq!(container.vector_repository().count().await.unwrap(), 0);
}

#[tokio::test]
async fn same_model_keeps_vectors() {
    let temp = tempfile::tempdir().unwrap();
    let config = config(temp.path());
    // First start records the hash provider's model id
    drop(ServiceContainer::new(config.clone()).await.unwrap());
    open_store(&config)
        .await
        .insert(vec![vector(), vector()])
        .await
        .unwrap();

    let container = ServiceContainer::new(config).await.unwrap();

    assert!(!container.index_state().reindex_required());
    assert_eq!(container.vector_repository().count().await.unwrap(), 2);
}

#[tokio::test]
async fn reindex_clears_the_flag() {
    let temp = tempfile::tempdir().unwrap();
    let config = config(temp.path());
    open_store(&config)
        .await
        .insert(vec![vector()])
        .await
        .unwrap();
    let container = ServiceContainer::new(config).await.unwrap();
    assert!(container.index_state().reindex_required());

    let docs = temp.path().join("docs");
    std::fs::create_dir(&docs).unwrap();
    std::fs::write(
        docs.join("deploy.md"),
        "# Deploy\n\nRolling updates replace pods gradually.",
    )
    .unwrap();
    let service = DocumentIndexingService::new(&container);
    let stats = service
        .index_documents_from_path(docs.to_str().unwrap(), true)
        .await
        .unwrap();

    assert_eq!(stats.documents_processed, 1);
    assert!(!container.index_state().reindex_required());
}
