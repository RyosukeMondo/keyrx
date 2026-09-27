//! UI ↔ API contract for the monitoring endpoints.
//!
//! Calls the real REST handlers over a seeded read model and pins each
//! response body as a fixture under `keyrx_ui/src/test/contract/`. The UI
//! tests load the same files (typed with the typeshare-generated types), so a
//! backend shape change fails here until the fixtures are regenerated, and the
//! regenerated fixtures fail the UI tests until the UI handles them.
//!
//! Regenerate: `UPDATE_CONTRACT_FIXTURES=1 cargo test -p keyrx_daemon --test api_contract_test`

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request};
use serde_json::Value;
use tower::ServiceExt;

use keyrx_daemon::container::ServiceContainerBuilder;
use keyrx_daemon::daemon::{DaemonSharedState, DaemonTelemetry, TelemetryState};
use keyrx_daemon::services::DaemonQueryService;
use keyrx_daemon::web::events::KeyEventData;
use keyrx_daemon::web::{create_router, AppState};

/// Fixed value for fields that depend on the clock.
const FIXED_TIMESTAMP_US: u64 = 1_790_000_000_000_000;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../keyrx_ui/src/test/contract")
}

fn key_event(event_type: &str, input: &str, output: &str, mapped: bool, t: u64) -> KeyEventData {
    KeyEventData {
        timestamp: FIXED_TIMESTAMP_US + t,
        key_code: input.to_string(),
        event_type: event_type.to_string(),
        input: input.to_string(),
        output: output.to_string(),
        latency: 42 + t,
        device_id: Some("path-/dev/input/event3".to_string()),
        device_name: Some("path-/dev/input/event3".to_string()),
        mapping_type: mapped.then(|| "simple".to_string()),
        mapping_triggered: mapped,
    }
}

/// A read model with a loaded profile, two modifiers (one a layer) + one lock, three
/// key events and some latency samples.
fn seeded_app() -> (Arc<AppState>, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let telemetry = Arc::new(DaemonTelemetry::new());
    let mut state = TelemetryState::empty();
    state.set_modifier(0, true);
    state.set_modifier(0x0A, true); // hex labels: MD_0A, not MD_10
    state.set_lock(1, true);
    state.set_active_layer(Some(0x0A));
    telemetry.update_state(state);
    telemetry.push_event(key_event("press", "CapsLock", "Escape", true, 1));
    telemetry.push_event(key_event("release", "CapsLock", "Escape", true, 2));
    telemetry.push_event(key_event("press", "A", "A", false, 3));
    for us in [30, 40, 50, 60, 200] {
        telemetry.latency_recorder().record(us);
    }

    let shared = Arc::new(DaemonSharedState::new(
        Arc::new(AtomicBool::new(true)),
        Some("default".to_string()),
        dir.path().join("profiles/default.krx"),
        1,
    ));
    let query = Arc::new(DaemonQueryService::new(shared, telemetry));
    let container = ServiceContainerBuilder::new(dir.path().to_path_buf())
        .build()
        .expect("container");
    let app = AppState::from_container_with_daemon(container, query);
    (Arc::new(app), dir)
}

async fn call(app: &Arc<AppState>, method: Method, path: &str) -> Value {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap();
    let response = create_router(Arc::clone(app))
        .oneshot(request)
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{path}: {}",
        response.status()
    );
    let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).expect("JSON body")
}

/// Compares `body` with the checked-in fixture (or rewrites it on request).
fn assert_fixture(name: &str, body: &Value) {
    let path = fixture_dir().join(name);
    let rendered = format!("{}\n", serde_json::to_string_pretty(body).unwrap());
    if std::env::var_os("UPDATE_CONTRACT_FIXTURES").is_some() {
        std::fs::create_dir_all(fixture_dir()).unwrap();
        std::fs::write(&path, &rendered).unwrap();
        return;
    }
    let pinned = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        pinned,
        rendered,
        "{} is out of date with the API. If the change is intended, regenerate with \
         UPDATE_CONTRACT_FIXTURES=1 and update the UI to match.",
        path.display()
    );
}

#[tokio::test]
async fn latency_contract() {
    let (app, _dir) = seeded_app();
    let mut body = call(&app, Method::GET, "/api/metrics/latency").await;
    assert!(body["timestamp"].as_u64().unwrap() > 0);
    body["timestamp"] = FIXED_TIMESTAMP_US.into();
    assert_fixture("metrics_latency.json", &body);
}

#[tokio::test]
async fn event_log_contract() {
    let (app, _dir) = seeded_app();
    let body = call(&app, Method::GET, "/api/metrics/events").await;
    assert_fixture("metrics_events.json", &body);
}

#[tokio::test]
async fn clear_event_log_contract() {
    let (app, _dir) = seeded_app();
    let body = call(&app, Method::DELETE, "/api/metrics/events").await;
    assert_fixture("metrics_events_clear.json", &body);
    let after = call(&app, Method::GET, "/api/metrics/events").await;
    assert_eq!(after, Value::Array(vec![]));
}

#[tokio::test]
async fn daemon_state_contract() {
    let (app, _dir) = seeded_app();
    let body = call(&app, Method::GET, "/api/daemon/state").await;
    assert_fixture("daemon_state.json", &body);
}
