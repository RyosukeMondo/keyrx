//! Debug endpoints: full daemon state snapshot, raw profile source, runtime
//! log level, and suspend/resume control.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::error::DaemonError;
use crate::web::AppState;

use super::health::get_hook_status;

/// GET /api/debug/state - Comprehensive daemon state snapshot for debugging
pub(super) async fn get_debug_state(State(state): State<Arc<AppState>>) -> Json<Value> {
    let status = state.daemon_query.get_status();
    let mapping_count = status
        .active_profile
        .as_ref()
        .map(|n| count_mappings_for_profile(&state, n))
        .unwrap_or(0);
    let daemon_info = serde_json::json!({
        "running": status.daemon_running, "uptime_secs": status.uptime_secs,
        "active_profile": status.active_profile, "device_count": status.device_count, "input_overflows": status.input_overflows,
        "mapping_count": mapping_count,
        "suspended": state.daemon_state.as_ref().map(|d| d.is_suspended()),
    });
    let config_info = build_config_info(&state).await;
    let profiles_info = build_profiles_info(&state).await;
    let hook = get_hook_status();
    Json(serde_json::json!({
        "daemon": daemon_info,
        "config": config_info,
        "profiles": profiles_info,
        "hook": { "installed": hook.installed, "remapped_keys_count": hook.remapped_keys_count },
        "ws_info": { "daemon_state_available": state.daemon_state.is_some() },
    }))
}

/// Count mappings for a profile by reading its .krx file from disk.
fn count_mappings_for_profile(state: &AppState, name: &str) -> usize {
    let profiles_dir = state.profile_service.profile_manager().profiles_dir();
    let krx_path = profiles_dir.join(format!("{name}.krx"));
    let Ok(data) = std::fs::read(&krx_path) else {
        return 0;
    };
    let Ok(archived) = keyrx_compiler::serialize::deserialize(&data) else {
        return 0;
    };
    use rkyv::Deserialize;
    let config: keyrx_core::config::ConfigRoot =
        archived.deserialize(&mut rkyv::Infallible).unwrap();
    config.devices.iter().map(|d| d.mappings.len()).sum()
}

/// Build config info for the active profile.
async fn build_config_info(state: &AppState) -> Value {
    let active_name = state.profile_service.get_active_profile().await;
    let Some(name) = active_name else {
        return serde_json::json!({
            "active_profile": null,
            "source": null,
            "file_size_bytes": null,
            "last_modified": null,
        });
    };

    let profiles_dir = state.profile_service.profile_manager().profiles_dir();
    let rhai_path = profiles_dir.join(format!("{name}.rhai"));

    let source = state.profile_service.get_profile_config(&name).await.ok();
    let (file_size, last_modified) = std::fs::metadata(&rhai_path)
        .ok()
        .map(|m| {
            let size = m.len();
            let modified = m
                .modified()
                .ok()
                .and_then(|t| {
                    let d = t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok()?;
                    chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                })
                .map(|dt| dt.to_rfc3339());
            (Some(size), modified)
        })
        .unwrap_or((None, None));

    serde_json::json!({
        "active_profile": name,
        "source": source,
        "file_size_bytes": file_size,
        "last_modified": last_modified,
    })
}

/// Build profiles listing with file sizes.
async fn build_profiles_info(state: &AppState) -> Value {
    let profiles_dir = state.profile_service.profile_manager().profiles_dir();
    let Ok(entries) = std::fs::read_dir(&profiles_dir) else {
        return serde_json::json!([]);
    };

    let mut profiles = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rhai") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        profiles.push(serde_json::json!({
            "name": name,
            "file_size_bytes": file_size,
        }));
    }

    serde_json::json!(profiles)
}

/// GET /api/debug/config/:name - Raw .rhai source code for a profile
pub(super) async fn get_debug_config(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, DaemonError> {
    let source = state
        .profile_service
        .get_profile_config(&name)
        .await
        .map_err(|e| {
            DaemonError::from(crate::error::ConfigError::ParseError {
                path: std::path::PathBuf::from(format!("{name}.rhai")),
                reason: format!("Profile not found: {e}"),
            })
        })?;

    Ok(Json(serde_json::json!({
        "name": name,
        "source": source,
    })))
}

/// Request body for changing log level.
#[derive(Deserialize)]
pub(super) struct LogLevelRequest {
    level: String,
}

/// POST /api/debug/log-level - Change runtime log level
pub(super) async fn set_debug_log_level(
    Json(payload): Json<LogLevelRequest>,
) -> Result<Json<Value>, DaemonError> {
    let level = match payload.level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        other => {
            return Err(DaemonError::from(crate::error::ConfigError::ParseError {
                path: std::path::PathBuf::from("log-level"),
                reason: format!(
                    "Invalid log level '{other}'. \
                         Use: trace, debug, info, warn, error"
                ),
            }));
        }
    };

    log::set_max_level(level);
    log::info!("Log level changed to: {}", payload.level);

    Ok(Json(serde_json::json!({
        "level": payload.level.to_lowercase(),
        "applied": true,
    })))
}

/// Request body for suspend/resume.
#[derive(Deserialize)]
pub(super) struct SuspendRequest {
    suspended: bool,
}

/// POST /api/debug/suspend - Suspend or resume key remapping
pub(super) async fn set_suspend_state(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SuspendRequest>,
) -> Result<Json<Value>, DaemonError> {
    match state.daemon_state.as_ref() {
        Some(daemon) => {
            daemon.set_suspended(payload.suspended);
            log::info!(
                "Daemon {} via REST API",
                if payload.suspended {
                    "suspended"
                } else {
                    "resumed"
                }
            );
            Ok(Json(serde_json::json!({
                "suspended": payload.suspended,
                "applied": true,
            })))
        }
        None => Err(DaemonError::from(crate::error::ConfigError::ParseError {
            path: std::path::PathBuf::from("suspend"),
            reason: "Daemon state not available".to_string(),
        })),
    }
}
