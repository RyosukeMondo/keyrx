//! Recording control FFI exports.
//!
//! Functions for managing session recording: start, stop, and query recording state.
#![allow(unsafe_code)]

use serde::Serialize;
use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::{Mutex, OnceLock};

// ─── Recording Control FFI Exports ────────────────────────────────────────

/// Global state for recording control.
static RECORDING_STATE: OnceLock<Mutex<RecordingState>> = OnceLock::new();

/// Get the recording state slot.
pub fn recording_state_slot() -> &'static Mutex<RecordingState> {
    RECORDING_STATE.get_or_init(|| Mutex::new(RecordingState::default()))
}

/// Thread-safe recording state.
#[derive(Default)]
pub struct RecordingState {
    /// Whether recording is currently active.
    pub is_recording: bool,
    /// Path where the session will be saved.
    pub output_path: Option<std::path::PathBuf>,
    /// Active recorder (when engine is running).
    pub recorder: Option<crate::engine::EventRecorder>,
    /// Last saved session path.
    pub last_session_path: Option<String>,
}

/// Recording start result for FFI JSON output.
#[derive(Serialize)]
struct RecordingStartResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(rename = "outputPath", skip_serializing_if = "Option::is_none")]
    output_path: Option<String>,
}

/// Recording stop result for FFI JSON output.
#[derive(Serialize)]
struct RecordingStopResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(rename = "sessionPath", skip_serializing_if = "Option::is_none")]
    session_path: Option<String>,
    #[serde(rename = "eventCount", skip_serializing_if = "Option::is_none")]
    event_count: Option<usize>,
}

/// Start recording to a session file.
///
/// Recording will begin when the engine starts processing events.
/// Call `keyrx_stop_recording()` to stop recording and save the session.
///
/// Returns JSON: `ok:{success: bool, error?: string, outputPath?: string}`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_start_recording(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        let result = RecordingStartResult {
            success: false,
            error: Some("null pointer".to_string()),
            output_path: None,
        };
        let payload = serde_json::to_string(&result)
            .map(|json| format!("ok:{json}"))
            .unwrap_or_else(|err| format!("error:{err}"));
        return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => {
            let result = RecordingStartResult {
                success: false,
                error: Some("invalid utf8".to_string()),
                output_path: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let output_path = std::path::PathBuf::from(path_str);

    // Verify parent directory exists
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            let result = RecordingStartResult {
                success: false,
                error: Some(format!(
                    "parent directory does not exist: {}",
                    parent.display()
                )),
                output_path: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    }

    let mut state = match recording_state_slot().lock() {
        Ok(s) => s,
        Err(_) => {
            let result = RecordingStartResult {
                success: false,
                error: Some("failed to acquire lock".to_string()),
                output_path: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    if state.is_recording {
        let result = RecordingStartResult {
            success: false,
            error: Some("recording already in progress".to_string()),
            output_path: None,
        };
        let payload = serde_json::to_string(&result)
            .map(|json| format!("ok:{json}"))
            .unwrap_or_else(|err| format!("error:{err}"));
        return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    state.is_recording = true;
    state.output_path = Some(output_path.clone());
    state.last_session_path = None;

    tracing::info!(
        service = "keyrx",
        event = "recording_started",
        component = "ffi_exports",
        path = %output_path.display(),
        "Recording started"
    );

    let result = RecordingStartResult {
        success: true,
        error: None,
        output_path: Some(output_path.display().to_string()),
    };
    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Stop recording and save the session file.
///
/// Returns JSON: `ok:{success: bool, error?: string, sessionPath?: string, eventCount?: number}`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub extern "C" fn keyrx_stop_recording() -> *mut c_char {
    let mut state = match recording_state_slot().lock() {
        Ok(s) => s,
        Err(_) => {
            let result = RecordingStopResult {
                success: false,
                error: Some("failed to acquire lock".to_string()),
                session_path: None,
                event_count: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    if !state.is_recording {
        let result = RecordingStopResult {
            success: false,
            error: Some("no recording in progress".to_string()),
            session_path: None,
            event_count: None,
        };
        let payload = serde_json::to_string(&result)
            .map(|json| format!("ok:{json}"))
            .unwrap_or_else(|err| format!("error:{err}"));
        return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    // If recorder is active, finish it and save
    let (session_path, event_count) = if let Some(recorder) = state.recorder.take() {
        let count = recorder.event_count();
        match recorder.finish() {
            Ok(session) => {
                let path = state.output_path.as_ref().map(|p| p.display().to_string());
                tracing::info!(
                    service = "keyrx",
                    event = "recording_saved",
                    component = "ffi_exports",
                    path = ?path,
                    events = count,
                    avg_latency_us = session.avg_latency_us(),
                    "Recording saved"
                );
                (path, Some(count))
            }
            Err(err) => {
                let result = RecordingStopResult {
                    success: false,
                    error: Some(format!("failed to save session: {err}")),
                    session_path: None,
                    event_count: Some(count),
                };
                state.is_recording = false;
                state.output_path = None;
                let payload = serde_json::to_string(&result)
                    .map(|json| format!("ok:{json}"))
                    .unwrap_or_else(|e| format!("error:{e}"));
                return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
            }
        }
    } else {
        // No recorder active - just return the path that would have been used
        (
            state.output_path.as_ref().map(|p| p.display().to_string()),
            Some(0),
        )
    };

    state.is_recording = false;
    state.last_session_path = session_path.clone();
    state.output_path = None;

    let result = RecordingStopResult {
        success: true,
        error: None,
        session_path,
        event_count,
    };
    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Check if recording is currently active.
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub extern "C" fn keyrx_is_recording() -> bool {
    recording_state_slot()
        .lock()
        .map(|s| s.is_recording)
        .unwrap_or(false)
}

/// Get the current recording output path (if recording is active).
///
/// Returns the path as a C string, or null if not recording.
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub extern "C" fn keyrx_get_recording_path() -> *mut c_char {
    let state = match recording_state_slot().lock() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    match &state.output_path {
        Some(path) if state.is_recording => CString::new(path.display().to_string())
            .map_or_else(|_| ptr::null_mut(), CString::into_raw),
        _ => ptr::null_mut(),
    }
}

/// Internal: Set the active recorder for the current recording session.
///
/// This is called by the engine when it starts processing with recording enabled.
pub fn set_active_recorder(recorder: crate::engine::EventRecorder) {
    if let Ok(mut state) = recording_state_slot().lock() {
        if state.is_recording && state.recorder.is_none() {
            state.recorder = Some(recorder);
        }
    }
}

/// Internal: Get a mutable reference to the active recorder.
///
/// This is called by the engine to record events.
pub fn with_active_recorder<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut crate::engine::EventRecorder) -> R,
{
    recording_state_slot()
        .lock()
        .ok()
        .and_then(|mut s| s.recorder.as_mut().map(f))
}

/// Internal: Check if recording should be started and get the output path.
pub fn get_recording_request() -> Option<std::path::PathBuf> {
    recording_state_slot().lock().ok().and_then(|s| {
        if s.is_recording && s.recorder.is_none() {
            s.output_path.clone()
        } else {
            None
        }
    })
}

/// Internal: Clear recording state (for testing).
#[cfg(test)]
pub fn clear_recording_state() {
    if let Ok(mut state) = recording_state_slot().lock() {
        state.is_recording = false;
        state.output_path = None;
        state.recorder = None;
        state.last_session_path = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::keyrx_free_string;
    use serial_test::serial;
    use std::ptr;

    #[test]
    #[serial]
    fn start_recording_null_pointer() {
        clear_recording_state();

        let ptr = unsafe { keyrx_start_recording(ptr::null()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["success"], false);
        assert!(result["error"].as_str().unwrap().contains("null pointer"));
    }

    #[test]
    #[serial]
    fn start_recording_invalid_parent_directory() {
        clear_recording_state();

        let path = CString::new("/nonexistent/directory/session.krx").unwrap();
        let ptr = unsafe { keyrx_start_recording(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["success"], false);
        assert!(result["error"]
            .as_str()
            .unwrap()
            .contains("parent directory does not exist"));
    }

    #[test]
    #[serial]
    fn start_recording_success() {
        clear_recording_state();

        let dir = tempfile::tempdir().expect("create temp dir");
        let session_path = dir.path().join("test_session.krx");
        let path = CString::new(session_path.display().to_string()).unwrap();

        let ptr = unsafe { keyrx_start_recording(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["success"], true);
        assert!(result["outputPath"].is_string());

        // Verify state
        assert!(keyrx_is_recording());

        // Clean up
        clear_recording_state();
    }

    #[test]
    #[serial]
    fn start_recording_already_in_progress() {
        clear_recording_state();

        let dir = tempfile::tempdir().expect("create temp dir");
        let session_path = dir.path().join("test_session.krx");
        let path = CString::new(session_path.display().to_string()).unwrap();

        // Start first recording
        let ptr = unsafe { keyrx_start_recording(path.as_ptr()) };
        unsafe { keyrx_free_string(ptr) };

        // Try to start another
        let ptr2 = unsafe { keyrx_start_recording(path.as_ptr()) };
        assert!(!ptr2.is_null());

        let raw = unsafe { CStr::from_ptr(ptr2).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr2) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["success"], false);
        assert!(result["error"]
            .as_str()
            .unwrap()
            .contains("already in progress"));

        // Clean up
        clear_recording_state();
    }

    #[test]
    #[serial]
    fn stop_recording_not_in_progress() {
        clear_recording_state();

        let ptr = keyrx_stop_recording();
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["success"], false);
        assert!(result["error"]
            .as_str()
            .unwrap()
            .contains("no recording in progress"));
    }

    #[test]
    #[serial]
    fn stop_recording_success_no_recorder() {
        clear_recording_state();

        let dir = tempfile::tempdir().expect("create temp dir");
        let session_path = dir.path().join("test_session.krx");
        let path = CString::new(session_path.display().to_string()).unwrap();

        // Start recording
        let ptr = unsafe { keyrx_start_recording(path.as_ptr()) };
        unsafe { keyrx_free_string(ptr) };

        // Stop without an active recorder (no engine running)
        let ptr2 = keyrx_stop_recording();
        assert!(!ptr2.is_null());

        let raw = unsafe { CStr::from_ptr(ptr2).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr2) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["eventCount"], 0);

        // Verify state
        assert!(!keyrx_is_recording());
    }

    #[test]
    #[serial]
    fn is_recording_false_when_not_started() {
        clear_recording_state();
        assert!(!keyrx_is_recording());
    }

    #[test]
    #[serial]
    fn get_recording_path_null_when_not_recording() {
        clear_recording_state();
        let ptr = keyrx_get_recording_path();
        assert!(ptr.is_null());
    }

    #[test]
    #[serial]
    fn get_recording_path_returns_path_when_recording() {
        clear_recording_state();

        let dir = tempfile::tempdir().expect("create temp dir");
        let session_path = dir.path().join("test_session.krx");
        let path = CString::new(session_path.display().to_string()).unwrap();

        // Start recording
        let ptr = unsafe { keyrx_start_recording(path.as_ptr()) };
        unsafe { keyrx_free_string(ptr) };

        // Get path
        let path_ptr = keyrx_get_recording_path();
        assert!(!path_ptr.is_null());

        let path_str = unsafe { CStr::from_ptr(path_ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(path_ptr) };

        assert!(path_str.contains("test_session.krx"));

        // Clean up
        clear_recording_state();
    }
}
