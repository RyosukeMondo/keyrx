//! MCP tool handler functions.
//!
//! Each function delegates to the existing service layer via `AppState`,
//! maintaining SSOT with the REST and WebSocket RPC interfaces.

use serde_json::{json, Value};
use std::sync::Arc;

use crate::config::ProfileTemplate;
use crate::web::AppState;

/// List all profiles with their metadata.
pub async fn list_profiles(state: &AppState) -> Result<String, String> {
    let profiles = state
        .profile_service
        .list_profiles()
        .await
        .map_err(|e| format!("Failed to list profiles: {}", e))?;

    let result: Vec<Value> = profiles
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "layerCount": p.layer_count,
                "active": p.active,
            })
        })
        .collect();

    serde_json::to_string_pretty(&json!({ "profiles": result }))
        .map_err(|e| format!("Serialization failed: {}", e))
}

/// Get the Rhai DSL source code for a profile.
pub async fn get_profile_config(state: &AppState, name: &str) -> Result<String, String> {
    let source = state
        .profile_service
        .get_profile_config(name)
        .await
        .map_err(|e| format!("Failed to get profile config: {}", e))?;

    serde_json::to_string_pretty(&json!({ "name": name, "source": source }))
        .map_err(|e| format!("Serialization failed: {}", e))
}

/// Set the Rhai DSL source code for a profile.
pub async fn set_profile_config(
    state: &AppState,
    name: &str,
    source: &str,
) -> Result<String, String> {
    state
        .profile_service
        .set_profile_config(name, source)
        .await
        .map_err(|e| format!("Failed to set profile config: {}", e))?;

    Ok(json!({"success": true, "name": name}).to_string())
}

/// Activate a profile (compile + reload).
pub async fn activate_profile(state: &AppState, name: &str) -> Result<String, String> {
    let result = state
        .profile_service
        .activate_profile(name)
        .await
        .map_err(|e| format!("Failed to activate profile: {}", e))?;

    // Update shared daemon state if available (Windows hot-reload)
    if let Some(daemon_state) = &state.daemon_state {
        daemon_state.set_active_profile(Some(name.to_string()));
        daemon_state.request_reload();
    }

    serde_json::to_string_pretty(&json!({
        "success": result.success,
        "compileTimeMs": result.compile_time_ms,
        "reloadTimeMs": result.reload_time_ms,
        "error": result.error,
    }))
    .map_err(|e| format!("Serialization failed: {}", e))
}

/// Create a new profile from a template.
pub async fn create_profile(
    state: &AppState,
    name: &str,
    template_str: Option<&str>,
) -> Result<String, String> {
    let template = match template_str.unwrap_or("blank") {
        "blank" => ProfileTemplate::Blank,
        "simple_remap" => ProfileTemplate::SimpleRemap,
        "capslock_escape" => ProfileTemplate::CapslockEscape,
        "vim_navigation" => ProfileTemplate::VimNavigation,
        "gaming" => ProfileTemplate::Gaming,
        other => return Err(format!("Invalid template: '{}'. Valid: blank, simple_remap, capslock_escape, vim_navigation, gaming", other)),
    };

    let info = state
        .profile_service
        .create_profile(name, template)
        .await
        .map_err(|e| format!("Failed to create profile: {}", e))?;

    serde_json::to_string_pretty(&json!({
        "name": info.name,
        "layerCount": info.layer_count,
        "active": info.active,
    }))
    .map_err(|e| format!("Serialization failed: {}", e))
}

/// Validate a profile's Rhai config by attempting compilation.
pub async fn validate_profile(state: &AppState, name: &str) -> Result<String, String> {
    use crate::config::profile_compiler::ProfileCompiler;

    let pm = Arc::clone(state.profile_service.profile_manager());
    let name = name.to_string();

    tokio::task::spawn_blocking(move || {
        let profile = pm
            .get(&name)
            .ok_or_else(|| format!("Profile '{}' not found", name))?;

        let compiler = ProfileCompiler::new();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temp_krx = std::env::temp_dir().join(format!("{}_{}.krx", name, ts));
        let result = compiler.compile_profile(&profile.rhai_path, &temp_krx);
        let _ = std::fs::remove_file(&temp_krx);

        match result {
            Ok(_) => Ok(json!({"valid": true, "errors": []}).to_string()),
            Err(e) => Ok(json!({"valid": false, "errors": [e.to_string()]}).to_string()),
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Simulate events using a built-in scenario.
pub fn simulate(state: &AppState, scenario: Option<&str>) -> Result<String, String> {
    match scenario {
        Some(name) => {
            let result = state
                .simulation_service
                .run_scenario(name)
                .map_err(|e| format!("Simulation failed: {}", e))?;
            serde_json::to_string_pretty(&result)
                .map_err(|e| format!("Serialization failed: {}", e))
        }
        None => {
            let results = state
                .simulation_service
                .run_all_scenarios()
                .map_err(|e| format!("Simulation failed: {}", e))?;
            serde_json::to_string_pretty(&results)
                .map_err(|e| format!("Serialization failed: {}", e))
        }
    }
}

/// Get daemon status (running, version, active profile, devices).
pub fn get_status(state: &AppState) -> Result<String, String> {
    let (daemon_running, uptime_secs, active_profile, device_count) =
        if let Some(query) = &state.daemon_query {
            let s = query.get_status();
            (
                s.daemon_running,
                Some(s.uptime_secs),
                s.active_profile,
                Some(s.device_count),
            )
        } else if let Some(ds) = &state.daemon_state {
            (
                ds.is_running(),
                Some(ds.uptime_secs()),
                ds.get_active_profile(),
                Some(ds.get_device_count()),
            )
        } else {
            (false, None, None, None)
        };

    Ok(json!({
        "version": crate::version::VERSION,
        "daemonRunning": daemon_running,
        "uptimeSecs": uptime_secs,
        "activeProfile": active_profile,
        "deviceCount": device_count,
    })
    .to_string())
}

/// Get daemon runtime state (modifiers, locks, layers).
pub fn get_state(state: &AppState) -> Result<String, String> {
    if let Some(ds) = &state.daemon_state {
        Ok(json!({
            "running": ds.is_running(),
            "activeProfile": ds.get_active_profile(),
            "deviceCount": ds.get_device_count(),
        })
        .to_string())
    } else {
        Ok(json!({"running": false, "activeProfile": null}).to_string())
    }
}

/// List connected input devices.
pub async fn list_devices(state: &AppState) -> Result<String, String> {
    let devices = state
        .device_service
        .list_devices()
        .await
        .map_err(|e| format!("Failed to list devices: {}", e))?;

    let result: Vec<Value> = devices
        .iter()
        .map(|d| {
            json!({
                "id": d.id,
                "name": d.name,
                "path": d.path,
                "active": d.active,
                "layout": d.layout,
            })
        })
        .collect();

    serde_json::to_string_pretty(&json!({ "devices": result }))
        .map_err(|e| format!("Serialization failed: {}", e))
}

/// Get system diagnostics (version, build info, platform).
pub fn get_diagnostics() -> Result<String, String> {
    Ok(json!({
        "version": crate::version::VERSION,
        "buildDate": crate::version::BUILD_DATE,
        "gitHash": crate::version::GIT_HASH,
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    })
    .to_string())
}

/// Get latency statistics.
pub fn get_latency(state: &AppState) -> Result<String, String> {
    if let Some(query) = &state.daemon_query {
        let snap = query.get_latency_snapshot();
        Ok(json!({
            "minUs": snap.min_us,
            "avgUs": snap.avg_us,
            "maxUs": snap.max_us,
            "p95Us": snap.p95_us,
            "p99Us": snap.p99_us,
        })
        .to_string())
    } else {
        Ok(json!({
            "minUs": 0, "avgUs": 0, "maxUs": 0, "p95Us": 0, "p99Us": 0,
            "note": "Latency data unavailable (no daemon query service)"
        })
        .to_string())
    }
}
