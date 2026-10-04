//! `POST /api/search` collection scoping: `filters.collection_name` limits the
//! search to one collection; without it every collection is searched.
#![cfg(feature = "embedded")]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use doc_indexer::application::services::document_service::DocumentIndexingService;
use doc_indexer::application::ServiceContainer;
use doc_indexer::config::Config;
use doc_indexer::infrastructure::api::http::handlers::{create_router, AppState};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::Arc;
use tower::ServiceExt;

/// A router over a container holding `alpha.md` in `alpha` and `beta.md` in `beta`
async fn router_with_two_collections() -> (axum::Router, tempfile::TempDir) {
    let container = Arc::new(ServiceContainer::new(Config::for_tests()).await.unwrap());
    let temp = tempfile::tempdir().unwrap();
    let indexer = DocumentIndexingService::new(&container);
    for name in ["alpha", "beta"] {
        let dir = temp.path().join(name);
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{name}.md")),
            format!("# {name}\n\nShared words about searching documents."),
        )
        .unwrap();
        indexer
            .index_documents_from_path_with_filters_and_collection(
                dir.to_str().unwrap(),
                true,
                None,
                name,
            )
            .await
            .unwrap();
    }
    let state = AppState::new_async(container).await.unwrap();
    (create_router(state), temp)
}

async fn search_titles(router: axum::Router, body: Value) -> BTreeSet<String> {
    let response = router
        .oneshot(
            Request::post("/api/search")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&bytes).unwrap();
    response["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["title"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn without_collection_searches_every_collection() {
    let (router, _temp) = router_with_two_collections().await;

    let titles = search_titles(router, json!({ "query": "searching documents" })).await;

    assert_eq!(
        titles,
        BTreeSet::from(["alpha.md".to_string(), "beta.md".to_string()])
    );
}

#[tokio::test]
async fn collection_name_filter_scopes_the_search() {
    let (router, _temp) = router_with_two_collections().await;

    let titles = search_titles(
        router,
        json!({ "query": "searching documents", "filters": { "collection_name": "beta" } }),
    )
    .await;

    assert_eq!(titles, BTreeSet::from(["beta.md".to_string()]));
}
