/// In-process HTTP mock server for adapter tests
///
/// Starts an axum server on 127.0.0.1 with an ephemeral port, records every
/// request it receives, and answers with whatever the test's responder returns.
/// Keeps the cloud adapter tests offline without a mocking dependency.
use std::sync::{Arc, Mutex};

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json, Router,
};
use serde_json::Value;

/// A request as the mock server received it
#[derive(Debug, Clone)]
pub struct RecordedRequest {
    pub method: Method,
    /// Path including any query string, e.g. `/collections/docs/points?wait=true`
    pub path: String,
    pub headers: HeaderMap,
    /// JSON body, or `Value::Null` when the body is empty or not JSON
    pub body: Value,
}

impl RecordedRequest {
    /// Path without the query string
    pub fn route(&self) -> &str {
        self.path.split('?').next().unwrap_or(&self.path)
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }
}

type Responder = dyn Fn(&RecordedRequest, usize) -> Response + Send + Sync;

struct Inner {
    requests: Mutex<Vec<RecordedRequest>>,
    responder: Box<Responder>,
}

pub struct MockServer {
    pub base_url: String,
    inner: Arc<Inner>,
}

impl MockServer {
    /// Start a server whose `responder` gets each request and the number of
    /// requests already received for the same method and route.
    pub async fn start<F>(responder: F) -> Self
    where
        F: Fn(&RecordedRequest, usize) -> (StatusCode, Value) + Send + Sync + 'static,
    {
        Self::start_with(Box::new(move |request, seen| {
            let (status, body) = responder(request, seen);
            (status, Json(body)).into_response()
        }))
        .await
    }

    /// Like `start`, but the responder returns a raw byte body (e.g. a file download)
    pub async fn start_raw<F>(responder: F) -> Self
    where
        F: Fn(&RecordedRequest, usize) -> (StatusCode, Vec<u8>) + Send + Sync + 'static,
    {
        Self::start_with(Box::new(move |request, seen| {
            responder(request, seen).into_response()
        }))
        .await
    }

    async fn start_with(responder: Box<Responder>) -> Self {
        let inner = Arc::new(Inner {
            requests: Mutex::new(Vec::new()),
            responder,
        });
        let app = Router::new().fallback(handle).with_state(inner.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self { base_url, inner }
    }

    /// Every request received so far, in order
    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.inner.requests.lock().unwrap().clone()
    }

    /// Requests received for one method and route (path without query)
    pub fn requests_to(&self, method: Method, route: &str) -> Vec<RecordedRequest> {
        self.requests()
            .into_iter()
            .filter(|r| r.method == method && r.route() == route)
            .collect()
    }
}

async fn handle(
    State(inner): State<Arc<Inner>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let request = RecordedRequest {
        method,
        path: uri
            .path_and_query()
            .map(|p| p.to_string())
            .unwrap_or_else(|| uri.path().to_string()),
        headers,
        body: serde_json::from_slice(&body).unwrap_or(Value::Null),
    };

    let seen = {
        let mut requests = inner.requests.lock().unwrap();
        let seen = requests
            .iter()
            .filter(|r| r.method == request.method && r.route() == request.route())
            .count();
        requests.push(request.clone());
        seen
    };

    (inner.responder)(&request, seen)
}
