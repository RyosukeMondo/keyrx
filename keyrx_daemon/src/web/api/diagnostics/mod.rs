//! Diagnostics endpoint for comprehensive system health information.
//!
//! Split by concern:
//! - `types` - Response/status DTOs shared by the handlers below
//! - `health` - Build info, basic diagnostics, IME status, config validation
//! - `routes_info` - Registered routes, frontend bundle status, "full" aggregate
//! - `debug` - Daemon state snapshot, raw profile source, log level, suspend
//! - `keyboard_labels` - Keyboard layout detection

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::web::AppState;

mod debug;
mod health;
mod keyboard_labels;
mod routes_info;
mod types;

use debug::{get_debug_config, get_debug_state, set_debug_log_level, set_suspend_state};
use health::{get_build_info, get_diagnostics, get_ime_status};
use keyboard_labels::get_keyboard_labels;
use routes_info::{get_frontend_status, get_full_diagnostics, get_routes_info};

#[cfg(test)]
use health::format_bytes;

pub use types::{
    BuildInfo, ConfigStatus, DiagnosticsResponse, HookStatus, KeyboardLabelsResponse, MemoryUsage,
    PlatformInfo,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/diagnostics", get(get_diagnostics))
        .route("/diagnostics/ime", get(get_ime_status))
        .route("/diagnostics/full", get(get_full_diagnostics))
        .route("/diagnostics/routes", get(get_routes_info))
        .route("/diagnostics/frontend", get(get_frontend_status))
        .route("/diagnostics/build", get(get_build_info))
        .route("/debug/state", get(get_debug_state))
        .route("/debug/config/:name", get(get_debug_config))
        .route("/debug/log-level", post(set_debug_log_level))
        .route("/debug/suspend", post(set_suspend_state))
        .route("/keyboard/labels", get(get_keyboard_labels))
}

#[cfg(test)]
mod tests;
