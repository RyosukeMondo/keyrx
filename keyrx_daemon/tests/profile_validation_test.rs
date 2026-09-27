//! Config validation reports where the error is.
//!
//! Regressions: the UI's editor validated via `POST /api/profiles/validate`,
//! which did not exist, and `/profiles/:name/validate` always reported line 1.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request};
use serde_json::{json, Value};
use tower::ServiceExt;

use keyrx_daemon::web::{create_router, AppState};

async fn post(app: &Arc<AppState>, path: &str, body: Value) -> (u16, Value) {
    let request = Request::builder()
        .method(Method::POST)
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = create_router(Arc::clone(app))
        .oneshot(request)
        .await
        .unwrap();
    let status = response.status().as_u16();
    let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn app() -> (Arc<AppState>, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let app = Arc::new(AppState::new_for_testing(dir.path().to_path_buf()));
    (app, dir)
}

#[tokio::test]
async fn valid_source_is_valid() {
    let (app, _dir) = app();
    let source = "device_start(\"*\");\n  map(\"VK_A\", \"VK_B\");\ndevice_end();\n";
    let (status, body) = post(&app, "/api/profiles/validate", json!({ "config": source })).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["valid"], true);
    assert_eq!(body["errors"], json!([]));
}

#[tokio::test]
async fn invalid_source_reports_its_line_and_column() {
    let (app, _dir) = app();
    let source = "device_start(\"*\");\n\n  bogus_fn(\"x\");\ndevice_end();\n";
    let (status, body) = post(&app, "/api/profiles/validate", json!({ "config": source })).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["valid"], false);
    let error = &body["errors"][0];
    assert_eq!(error["line"], 3, "{body}");
    assert_eq!(error["column"], 3, "{body}");
    let message = error["message"].as_str().unwrap();
    assert!(message.contains("bogus_fn"), "{message}");
    assert!(!message.contains(".rhai"), "temp path leaked: {message}");
}
