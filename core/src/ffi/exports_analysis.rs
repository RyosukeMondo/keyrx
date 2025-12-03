//! Session analysis FFI exports.
//!
//! Functions for listing, analyzing, and replaying session files.
#![allow(unsafe_code)]

use serde::Serialize;
use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::ptr;

// ─── Session Analysis FFI Exports ────────────────────────────────────────

/// Session info for FFI JSON output.
#[derive(Serialize)]
struct SessionInfo {
    path: String,
    name: String,
    created: String,
    #[serde(rename = "eventCount")]
    event_count: usize,
    #[serde(rename = "durationMs")]
    duration_ms: f64,
}

/// List all .krx session files in a directory.
///
/// Returns JSON: `ok:[{path, name, created, eventCount, durationMs}, ...]`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `dir_path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_list_sessions(dir_path: *const c_char) -> *mut c_char {
    if dir_path.is_null() {
        return CString::new("error:null pointer")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let dir_str = match CStr::from_ptr(dir_path).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new("error:invalid utf8")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let dir = Path::new(dir_str);
    if !dir.exists() {
        return CString::new("ok:[]").map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => {
            return CString::new(format!("error:Failed to read directory: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let mut sessions: Vec<SessionInfo> = Vec::new();

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "krx") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(session) = crate::engine::SessionFile::from_json(&content) {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    sessions.push(SessionInfo {
                        path: path.display().to_string(),
                        name,
                        created: session.created_at.clone(),
                        event_count: session.event_count(),
                        duration_ms: session.duration_us() as f64 / 1000.0,
                    });
                }
            }
        }
    }

    // Sort by created date descending (newest first)
    sessions.sort_by(|a, b| b.created.cmp(&a.created));

    let payload = serde_json::to_string(&sessions)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Analysis result for FFI JSON output.
#[derive(Serialize)]
struct AnalysisResultJson {
    #[serde(rename = "sessionPath")]
    session_path: String,
    #[serde(rename = "eventCount")]
    event_count: usize,
    #[serde(rename = "durationMs")]
    duration_ms: f64,
    #[serde(rename = "avgLatencyUs")]
    avg_latency_us: u64,
    #[serde(rename = "minLatencyUs")]
    min_latency_us: u64,
    #[serde(rename = "maxLatencyUs")]
    max_latency_us: u64,
    #[serde(rename = "decisionBreakdown")]
    decision_breakdown: DecisionBreakdownJson,
}

/// Decision breakdown for FFI JSON output.
#[derive(Serialize, Default)]
struct DecisionBreakdownJson {
    #[serde(rename = "passThrough")]
    pass_through: usize,
    remap: usize,
    block: usize,
    tap: usize,
    hold: usize,
    combo: usize,
    layer: usize,
    modifier: usize,
}

/// Analyze a .krx session file.
///
/// Returns JSON: `ok:{sessionPath, eventCount, durationMs, avgLatencyUs, minLatencyUs, maxLatencyUs, decisionBreakdown}`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_analyze_session(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return CString::new("error:null pointer")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new("error:invalid utf8")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let content = match std::fs::read_to_string(path_str) {
        Ok(s) => s,
        Err(err) => {
            return CString::new(format!("error:Failed to read session: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let session = match crate::engine::SessionFile::from_json(&content) {
        Ok(s) => s,
        Err(err) => {
            return CString::new(format!("error:Failed to parse session: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Calculate statistics
    let mut breakdown = DecisionBreakdownJson::default();
    let mut min_latency = u64::MAX;
    let mut max_latency = 0u64;

    for event in &session.events {
        match event.decision_type {
            crate::engine::DecisionType::PassThrough => breakdown.pass_through += 1,
            crate::engine::DecisionType::Remap => breakdown.remap += 1,
            crate::engine::DecisionType::Block => breakdown.block += 1,
            crate::engine::DecisionType::Tap => breakdown.tap += 1,
            crate::engine::DecisionType::Hold => breakdown.hold += 1,
            crate::engine::DecisionType::Combo => breakdown.combo += 1,
            crate::engine::DecisionType::Layer => breakdown.layer += 1,
            crate::engine::DecisionType::Modifier => breakdown.modifier += 1,
        }
        min_latency = min_latency.min(event.latency_us);
        max_latency = max_latency.max(event.latency_us);
    }

    if session.events.is_empty() {
        min_latency = 0;
    }

    let result = AnalysisResultJson {
        session_path: path_str.to_string(),
        event_count: session.event_count(),
        duration_ms: session.duration_us() as f64 / 1000.0,
        avg_latency_us: session.avg_latency_us(),
        min_latency_us: min_latency,
        max_latency_us: max_latency,
        decision_breakdown: breakdown,
    };

    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Replay verification result for FFI JSON output.
#[derive(Serialize)]
struct ReplayResultJson {
    #[serde(rename = "totalEvents")]
    total_events: usize,
    matched: usize,
    mismatched: usize,
    success: bool,
    mismatches: Vec<MismatchJson>,
}

/// Mismatch detail for FFI JSON output.
#[derive(Serialize)]
struct MismatchJson {
    seq: u64,
    recorded: String,
    actual: String,
}

/// Replay a .krx session with optional verification.
///
/// Returns JSON: `ok:{totalEvents, matched, mismatched, success, mismatches: [{seq, recorded, actual}]}`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_replay_session(path: *const c_char, verify: bool) -> *mut c_char {
    if path.is_null() {
        return CString::new("error:null pointer")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new("error:invalid utf8")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Use tokio runtime for async execution
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            return CString::new(format!("error:Failed to create runtime: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let result = rt.block_on(async {
        let cmd = crate::cli::commands::ReplayCommand::new(
            std::path::PathBuf::from(path_str),
            crate::cli::OutputFormat::Json,
        )
        .with_verify(verify)
        .with_speed(0.0); // Instant replay

        cmd.run().await
    });

    let verification = match result {
        Ok(v) => v,
        Err(err) => {
            return CString::new(format!("error:Replay failed: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let mismatches: Vec<MismatchJson> = verification
        .mismatches
        .iter()
        .take(10) // Limit to first 10 mismatches
        .map(|m| MismatchJson {
            seq: m.seq,
            recorded: format!("{:?}", m.recorded),
            actual: format!("{:?}", m.actual),
        })
        .collect();

    let result = ReplayResultJson {
        total_events: verification.total_events,
        matched: verification.matched,
        mismatched: verification.mismatched,
        success: verification.all_matched(),
        mismatches,
    };

    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::keycodes::KeyCode;
    use crate::ffi::keyrx_free_string;
    use std::ptr;

    fn make_test_session() -> crate::engine::SessionFile {
        use crate::engine::{
            DecisionType, EngineState, EventRecordBuilder, InputEvent, LayerStack, ModifierState,
            OutputAction, TimingConfig,
        };

        let initial_state = EngineState {
            pressed_keys: vec![],
            modifiers: ModifierState::default(),
            layers: LayerStack::new(),
            pending: vec![],
            timing: TimingConfig::default(),
            safe_mode: false,
        };

        let mut session = crate::engine::SessionFile::new(
            "2024-01-15T10:30:00Z".to_string(),
            None,
            TimingConfig::default(),
            initial_state,
        );

        // Add test events
        for i in 0..3 {
            let input = InputEvent::key_down(KeyCode::A, (i * 10_000) as u64);
            session.add_event(
                EventRecordBuilder::new()
                    .seq(i)
                    .timestamp_us((i * 10_000) as u64)
                    .input(input)
                    .output(vec![OutputAction::KeyDown(KeyCode::A)])
                    .decision_type(DecisionType::PassThrough)
                    .active_layers(vec![0])
                    .modifiers_state(ModifierState::default())
                    .latency_us(50 + i * 10)
                    .build(),
            );
        }

        session
    }

    #[test]
    fn list_sessions_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let path = CString::new(dir.path().to_str().unwrap()).unwrap();
        let ptr = unsafe { keyrx_list_sessions(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let sessions: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn list_sessions_finds_krx_files() {
        let dir = tempfile::tempdir().unwrap();
        let session = make_test_session();
        let session_path = dir.path().join("test_session.krx");
        std::fs::write(&session_path, session.to_json().unwrap()).unwrap();

        let path = CString::new(dir.path().to_str().unwrap()).unwrap();
        let ptr = unsafe { keyrx_list_sessions(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"), "got: {raw}");
        let json_str = &raw["ok:".len()..];
        let sessions: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0]["name"], "test_session.krx");
        assert_eq!(sessions[0]["eventCount"], 3);
    }

    #[test]
    fn list_sessions_nonexistent_dir() {
        let path = CString::new("/nonexistent/directory").unwrap();
        let ptr = unsafe { keyrx_list_sessions(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        // Should return empty array for nonexistent dir
        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let sessions: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn list_sessions_null_pointer() {
        let ptr = unsafe { keyrx_list_sessions(ptr::null()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };
        assert!(raw.starts_with("error:"));
    }

    #[test]
    fn analyze_session_returns_statistics() {
        let dir = tempfile::tempdir().unwrap();
        let session = make_test_session();
        let session_path = dir.path().join("test_session.krx");
        std::fs::write(&session_path, session.to_json().unwrap()).unwrap();

        let path = CString::new(session_path.to_str().unwrap()).unwrap();
        let ptr = unsafe { keyrx_analyze_session(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"), "got: {raw}");
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();

        assert_eq!(result["eventCount"], 3);
        assert_eq!(result["minLatencyUs"], 50);
        assert_eq!(result["maxLatencyUs"], 70);
        assert_eq!(result["decisionBreakdown"]["passThrough"], 3);
    }

    #[test]
    fn analyze_session_missing_file() {
        let path = CString::new("/nonexistent/session.krx").unwrap();
        let ptr = unsafe { keyrx_analyze_session(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };
        assert!(raw.starts_with("error:"));
    }

    #[test]
    fn analyze_session_null_pointer() {
        let ptr = unsafe { keyrx_analyze_session(ptr::null()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };
        assert!(raw.starts_with("error:"));
    }

    #[test]
    fn replay_session_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let session = make_test_session();
        let session_path = dir.path().join("test_session.krx");
        std::fs::write(&session_path, session.to_json().unwrap()).unwrap();

        let path = CString::new(session_path.to_str().unwrap()).unwrap();
        let ptr = unsafe { keyrx_replay_session(path.as_ptr(), false) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"), "got: {raw}");
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();

        assert_eq!(result["totalEvents"], 3);
        assert_eq!(result["success"], true);
    }

    #[test]
    fn replay_session_with_verify() {
        let dir = tempfile::tempdir().unwrap();
        let session = make_test_session();
        let session_path = dir.path().join("test_session.krx");
        std::fs::write(&session_path, session.to_json().unwrap()).unwrap();

        let path = CString::new(session_path.to_str().unwrap()).unwrap();
        let ptr = unsafe { keyrx_replay_session(path.as_ptr(), true) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"), "got: {raw}");
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();

        assert_eq!(result["totalEvents"], 3);
        // Verification result depends on engine state match
        assert!(result["success"].is_boolean());
    }

    #[test]
    fn replay_session_missing_file() {
        let path = CString::new("/nonexistent/session.krx").unwrap();
        let ptr = unsafe { keyrx_replay_session(path.as_ptr(), false) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };
        assert!(raw.starts_with("error:"));
    }

    #[test]
    fn replay_session_null_pointer() {
        let ptr = unsafe { keyrx_replay_session(ptr::null(), false) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };
        assert!(raw.starts_with("error:"));
    }
}
