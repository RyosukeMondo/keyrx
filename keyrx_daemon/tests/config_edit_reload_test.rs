//! Editing the active profile's config over REST takes effect.
//!
//! Regression: `PUT /api/config` and `/api/config/key-mappings` wrote the
//! `.rhai` without compiling it or telling the daemon, and found the "active
//! profile" by calling the daemon's own IPC socket (falling back to a profile
//! literally named "default"). They now go through ConfigService →
//! ProfileService: compile, then reload if the daemon has the profile loaded.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request};
use tower::ServiceExt;

use keyrx_daemon::config::{ProfileManager, ProfileTemplate};
use keyrx_daemon::container::ServiceContainerBuilder;
use keyrx_daemon::daemon::{DaemonSharedState, DaemonTelemetry};
use keyrx_daemon::services::DaemonQueryService;
use keyrx_daemon::web::{create_router, AppState};

const CAPS_TO_LCTRL: &str =
    "device_start(\"*\");\n  map(\"VK_CapsLock\", \"VK_LCtrl\");\ndevice_end();\n";

struct Fixture {
    app: Arc<AppState>,
    shared: Arc<DaemonSharedState>,
    dir: tempfile::TempDir,
}

/// Profile "a" active (CapsLock→Escape); the daemon has `loaded` loaded.
fn fixture(loaded: &str) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let manager = ProfileManager::new(dir.path().to_path_buf()).unwrap();
    manager
        .create("a", ProfileTemplate::CapslockEscape)
        .unwrap();
    assert!(manager.activate("a").unwrap().success);

    let shared = Arc::new(DaemonSharedState::new(
        Arc::new(AtomicBool::new(true)),
        Some(loaded.to_string()),
        dir.path().join("profiles").join(format!("{loaded}.krx")),
        1,
    ));
    let query = Arc::new(DaemonQueryService::new(
        Arc::clone(&shared),
        Arc::new(DaemonTelemetry::new()),
    ));
    let container = ServiceContainerBuilder::new(dir.path().to_path_buf())
        .build()
        .unwrap();
    let app = Arc::new(AppState::from_container_with_daemon(container, query));
    Fixture { app, shared, dir }
}

async fn send(app: &Arc<AppState>, method: Method, path: &str, body: serde_json::Value) -> u16 {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = create_router(Arc::clone(app))
        .oneshot(request)
        .await
        .unwrap();
    response.status().as_u16()
}

fn krx(f: &Fixture) -> Vec<u8> {
    std::fs::read(f.dir.path().join("profiles/a.krx")).unwrap()
}

#[tokio::test]
async fn put_config_compiles_and_reloads_the_loaded_profile() {
    let f = fixture("a");
    let before = krx(&f);
    let status = send(
        &f.app,
        Method::PUT,
        "/api/config",
        serde_json::json!({ "content": CAPS_TO_LCTRL }),
    )
    .await;
    assert_eq!(status, 200);
    assert_ne!(krx(&f), before, ".krx was not recompiled");
    assert!(
        f.shared.take_reload_request(),
        "daemon was not asked to reload"
    );
}

#[tokio::test]
async fn key_mapping_edit_compiles_and_reloads_the_loaded_profile() {
    let f = fixture("a");
    let before = krx(&f);
    let status = send(
        &f.app,
        Method::POST,
        "/api/config/key-mappings",
        serde_json::json!({
            "layer": "base", "key": "VK_A", "action_type": "simple", "output": "VK_B"
        }),
    )
    .await;
    assert_eq!(status, 200);
    assert_ne!(krx(&f), before, ".krx was not recompiled");
    assert!(
        f.shared.take_reload_request(),
        "daemon was not asked to reload"
    );
}

#[tokio::test]
async fn editing_a_profile_the_daemon_has_not_loaded_does_not_reload() {
    let f = fixture("other");
    let status = send(
        &f.app,
        Method::PUT,
        "/api/config",
        serde_json::json!({ "content": CAPS_TO_LCTRL }),
    )
    .await;
    assert_eq!(status, 200);
    assert!(!f.shared.take_reload_request());
}

/// Invalid source is kept as a draft (the editor's text is not lost) but is
/// not applied: the compiled `.krx` stays the last good one and the daemon is
/// not asked to reload.
#[tokio::test]
async fn invalid_source_is_reported_and_not_applied() {
    let f = fixture("a");
    let before = krx(&f);
    let status = send(
        &f.app,
        Method::PUT,
        "/api/config",
        serde_json::json!({ "content": "device_start(\"*\");\n  map(\"VK_A\"" }),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(krx(&f), before, "a failed compile replaced the live .krx");
    assert!(!f.shared.take_reload_request());
}
