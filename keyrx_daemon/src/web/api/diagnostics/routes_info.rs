//! Registered-route listing, frontend bundle status, and the combined
//! "full diagnostics" endpoint that aggregates both with basic diagnostics.

use axum::Json;
use serde_json::Value;

use crate::error::DaemonError;

use super::health::get_diagnostics;

/// GET /api/diagnostics/full - Get all diagnostics including routes and frontend status
pub(super) async fn get_full_diagnostics() -> Result<Json<Value>, DaemonError> {
    tokio::task::spawn_blocking(move || {
        let basic_diag = tokio::runtime::Handle::current()
            .block_on(get_diagnostics())
            .map_err(|e| {
                DaemonError::from(crate::error::ConfigError::ParseError {
                    path: std::path::PathBuf::from("full-diagnostics"),
                    reason: format!("Failed to get basic diagnostics: {}", e),
                })
            })?;

        let routes_info = get_routes_list();
        let frontend_status = get_frontend_info();

        Ok::<Json<Value>, DaemonError>(Json(serde_json::json!({
            "basic": basic_diag.0,
            "routes": routes_info,
            "frontend": frontend_status,
        })))
    })
    .await
    .map_err(|e| {
        DaemonError::from(crate::error::ConfigError::ParseError {
            path: std::path::PathBuf::from("full-diagnostics"),
            reason: format!("Task join error: {}", e),
        })
    })?
}

/// GET /api/diagnostics/routes - Get information about registered API routes
pub(super) async fn get_routes_info() -> Json<Value> {
    Json(serde_json::json!(get_routes_list()))
}

/// GET /api/diagnostics/frontend - Get frontend bundle status
pub(super) async fn get_frontend_status() -> Json<Value> {
    Json(serde_json::json!(get_frontend_info()))
}

/// Get list of registered API routes
fn get_routes_list() -> Value {
    let api = vec![
        "health",
        "version",
        "status",
        "metrics/latency",
        "metrics/events",
        "daemon/state",
        "diagnostics",
        "diagnostics/full",
        "diagnostics/routes",
        "diagnostics/frontend",
        "diagnostics/build",
        "devices",
        "devices/:device_id",
        "devices/:device_id/layout",
        "profiles",
        "profiles/:name",
        "profiles/:name/config",
        "profiles/:name/activate",
        "profiles/active",
        "config/:profile/layers",
        "config/:profile/layers/:layer",
        "layouts",
        "layouts/:layout_name",
        "simulator/press",
        "simulator/release",
        "simulator/sequence",
        "macros/start",
        "macros/stop",
        "macros/events",
        "macros/save",
        "debug/state",
        "debug/config/:name",
        "debug/log-level",
        "debug/suspend",
        "keyboard/labels",
    ];
    let api_routes: Vec<String> = api.into_iter().map(|r| format!("/api/{r}")).collect();
    serde_json::json!({
        "api_routes": api_routes,
        "websocket_routes": ["/ws", "/ws-rpc"],
        "frontend_routes": ["/", "/home", "/devices", "/profiles",
            "/profiles/:name/config", "/config", "/metrics", "/simulator"],
        "note": "Frontend routes are handled by React Router via SPA fallback"
    })
}

/// Get frontend bundle information
fn get_frontend_info() -> Value {
    let ui_dist_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../keyrx_ui/dist");
    serde_json::json!({
        "ui_embedded": true,
        "ui_dist_path": ui_dist_path.to_string_lossy(),
        "ui_dist_exists": ui_dist_path.exists(),
        "spa_fallback": "Enabled - all non-API routes serve index.html",
        "bundle_files": ["/index.html", "/assets/index-*.js",
            "/assets/vendor-*.js", "/assets/index-*.css"],
    })
}
