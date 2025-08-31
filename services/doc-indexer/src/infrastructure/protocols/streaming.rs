/// HTTP streaming support for real-time updates
///
/// This module provides Server-Sent Events (SSE) streaming capabilities
/// for long-running operations like document indexing and batch searches.
use axum::{extract::State, response::Sse, routing::get, Router};
use futures_util::Stream;
use serde_json::json;
use std::{convert::Infallible, time::Duration};
use tokio::time::interval;

use crate::infrastructure::api::http::handlers::AppState;

/// Create streaming router with SSE endpoints
pub fn create_streaming_router() -> Router<AppState> {
    Router::new()
        .route("/stream/index-progress", get(stream_index_progress))
        .route("/stream/search-results", get(stream_search_results))
        .route("/stream/health", get(stream_health_updates))
}

/// Stream indexing progress for long-running document indexing operations
async fn stream_index_progress(
    State(_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let stream = create_progress_stream();
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(5))
            .text("heartbeat"),
    )
}

/// Stream search results as they become available
async fn stream_search_results(
    State(_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let stream = create_search_stream();
    Sse::new(stream)
}

/// Stream health status updates
async fn stream_health_updates(
    State(_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let stream = create_health_stream();
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("ping"),
    )
}

/// Create a mock progress stream (to be replaced with real indexing progress)
fn create_progress_stream() -> impl Stream<Item = Result<axum::response::sse::Event, Infallible>> {
    let mut interval = interval(Duration::from_secs(1));
    let mut progress = 0;

    async_stream::stream! {
        while progress <= 100 {
            interval.tick().await;

            let event_data = json!({
                "type": "progress",
                "operation": "document_indexing",
                "progress": progress,
                "message": format!("Processing documents: {}%", progress)
            });

            let event = axum::response::sse::Event::default()
                .event("indexing_progress")
                .data(serde_json::to_string(&event_data).unwrap());

            yield Ok(event);

            progress += 10;
        }

        // Final completion event
        let completion_data = json!({
            "type": "completion",
            "operation": "document_indexing",
            "status": "completed",
            "message": "Document indexing completed successfully"
        });

        let completion_event = axum::response::sse::Event::default()
            .event("indexing_complete")
            .data(serde_json::to_string(&completion_data).unwrap());

        yield Ok(completion_event);
    }
}

/// Create a mock search results stream (to be replaced with real streaming search)
fn create_search_stream() -> impl Stream<Item = Result<axum::response::sse::Event, Infallible>> {
    let mut interval = interval(Duration::from_millis(500));
    let mut count = 0;

    async_stream::stream! {
        while count < 5 {
            interval.tick().await;

            let result_data = json!({
                "type": "search_result",
                "document_id": format!("doc_{}", count),
                "title": format!("Document {}", count),
                "score": 0.95 - (count as f64 * 0.1),
                "snippet": format!("This is a preview of document {} content...", count)
            });

            let event = axum::response::sse::Event::default()
                .event("search_result")
                .data(serde_json::to_string(&result_data).unwrap());

            yield Ok(event);
            count += 1;
        }

        // End of results marker
        let end_data = json!({
            "type": "search_complete",
            "total_results": count,
            "message": "Search completed"
        });

        let end_event = axum::response::sse::Event::default()
            .event("search_complete")
            .data(serde_json::to_string(&end_data).unwrap());

        yield Ok(end_event);
    }
}

/// Create health status stream
fn create_health_stream() -> impl Stream<Item = Result<axum::response::sse::Event, Infallible>> {
    let mut interval = interval(Duration::from_secs(5));

    async_stream::stream! {
        loop {
            interval.tick().await;

            let health_data = json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "status": "healthy",
                "memory_usage": "45%",
                "indexed_documents": 1234,
                "active_connections": 3
            });

            let event = axum::response::sse::Event::default()
                .event("health_update")
                .data(serde_json::to_string(&health_data).unwrap());

            yield Ok(event);
        }
    }
}
