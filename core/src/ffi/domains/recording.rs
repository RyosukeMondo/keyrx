//! Recording domain FFI implementation.
//!
//! Implements the FfiExportable trait for session recording.
//! Handles recording start, stop, and state management.
#![allow(unsafe_code)]

use crate::ffi::context::FfiContext;
use crate::ffi::error::{FfiError, FfiResult};
use crate::ffi::traits::FfiExportable;
use keyrx_ffi_macros::ffi_export;
use serde::Serialize;
use std::path::PathBuf;

/// Recording domain FFI implementation.
pub struct RecordingFfi;

/// Recording state for FFI.
#[derive(Debug, Default)]
pub struct RecordingState {
    /// Whether recording is currently active
    pub is_recording: bool,
    /// Path where the session will be saved
    pub output_path: Option<PathBuf>,
    /// Active recorder (when engine is running)
    pub recorder: Option<crate::engine::EventRecorder>,
    /// Last saved session path
    pub last_session_path: Option<String>,
}

impl FfiExportable for RecordingFfi {
    const DOMAIN: &'static str = "recording";

    fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
        if ctx.has_domain(Self::DOMAIN) {
            return Err(FfiError::invalid_input(
                "recording domain already initialized",
            ));
        }

        // Initialize recording domain state
        ctx.set_domain(Self::DOMAIN, RecordingState::default());

        Ok(())
    }

    fn cleanup(ctx: &mut FfiContext) {
        // Stop any active recording before cleanup
        if let Some(mut state_guard) = ctx.get_domain_mut::<RecordingState>(Self::DOMAIN) {
            if let Some(state) = state_guard.downcast_mut::<RecordingState>() {
                if state.is_recording {
                    if let Some(recorder) = state.recorder.take() {
                        let _ = recorder.finish();
                    }
                    state.is_recording = false;
                    state.output_path = None;
                }
            }
        }

        // Remove domain state
        ctx.remove_domain(Self::DOMAIN);
    }
}

impl RecordingFfi {
    /// Get the recording request if recording should be started.
    pub fn get_recording_request(ctx: &FfiContext) -> Option<PathBuf> {
        ctx.get_domain::<RecordingState>(Self::DOMAIN)
            .and_then(|state_guard| {
                state_guard
                    .downcast_ref::<RecordingState>()
                    .and_then(|state| {
                        if state.is_recording && state.recorder.is_none() {
                            state.output_path.clone()
                        } else {
                            None
                        }
                    })
            })
    }

    /// Set the active recorder for the current recording session.
    pub fn set_active_recorder(ctx: &mut FfiContext, recorder: crate::engine::EventRecorder) {
        if let Some(mut state_guard) = ctx.get_domain_mut::<RecordingState>(Self::DOMAIN) {
            if let Some(state) = state_guard.downcast_mut::<RecordingState>() {
                if state.is_recording && state.recorder.is_none() {
                    state.recorder = Some(recorder);
                }
            }
        }
    }

    /// Execute a function with the active recorder.
    pub fn with_active_recorder<F, R>(ctx: &mut FfiContext, f: F) -> Option<R>
    where
        F: FnOnce(&mut crate::engine::EventRecorder) -> R,
    {
        ctx.get_domain_mut::<RecordingState>(Self::DOMAIN)
            .and_then(|mut state_guard| {
                state_guard
                    .downcast_mut::<RecordingState>()
                    .and_then(|state| state.recorder.as_mut().map(f))
            })
    }
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
/// Call `stop_recording()` to stop recording and save the session.
///
/// Returns: `{success: bool, error?: string, outputPath?: string}`
#[ffi_export]
pub fn start_recording(ctx: &mut FfiContext, path: &str) -> FfiResult<RecordingStartResult> {
    let output_path = PathBuf::from(path);

    // Verify parent directory exists
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Ok(RecordingStartResult {
                success: false,
                error: Some(format!(
                    "parent directory does not exist: {}",
                    parent.display()
                )),
                output_path: None,
            });
        }
    }

    if let Some(mut state_guard) = ctx.get_domain_mut::<RecordingState>(RecordingFfi::DOMAIN) {
        if let Some(state) = state_guard.downcast_mut::<RecordingState>() {
            if state.is_recording {
                return Ok(RecordingStartResult {
                    success: false,
                    error: Some("recording already in progress".to_string()),
                    output_path: None,
                });
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

            return Ok(RecordingStartResult {
                success: true,
                error: None,
                output_path: Some(output_path.display().to_string()),
            });
        }
    }

    Err(FfiError::internal("Failed to access recording state"))
}

/// Stop recording and save the session file.
///
/// Returns: `{success: bool, error?: string, sessionPath?: string, eventCount?: number}`
#[ffi_export]
pub fn stop_recording(ctx: &mut FfiContext) -> FfiResult<RecordingStopResult> {
    if let Some(mut state_guard) = ctx.get_domain_mut::<RecordingState>(RecordingFfi::DOMAIN) {
        if let Some(state) = state_guard.downcast_mut::<RecordingState>() {
            if !state.is_recording {
                return Ok(RecordingStopResult {
                    success: false,
                    error: Some("no recording in progress".to_string()),
                    session_path: None,
                    event_count: None,
                });
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
                        state.is_recording = false;
                        state.output_path = None;
                        return Ok(RecordingStopResult {
                            success: false,
                            error: Some(format!("failed to save session: {err}")),
                            session_path: None,
                            event_count: Some(count),
                        });
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

            return Ok(RecordingStopResult {
                success: true,
                error: None,
                session_path,
                event_count,
            });
        }
    }

    Err(FfiError::internal("Failed to access recording state"))
}

/// Check if recording is currently active.
#[ffi_export]
pub fn is_recording(ctx: &FfiContext) -> FfiResult<bool> {
    if let Some(state_guard) = ctx.get_domain::<RecordingState>(RecordingFfi::DOMAIN) {
        if let Some(state) = state_guard.downcast_ref::<RecordingState>() {
            return Ok(state.is_recording);
        }
    }
    Ok(false)
}

/// Get the current recording output path (if recording is active).
///
/// Returns the path as a string, or None if not recording.
#[ffi_export]
pub fn get_recording_path(ctx: &FfiContext) -> FfiResult<Option<String>> {
    if let Some(state_guard) = ctx.get_domain::<RecordingState>(RecordingFfi::DOMAIN) {
        if let Some(state) = state_guard.downcast_ref::<RecordingState>() {
            if state.is_recording {
                return Ok(state.output_path.as_ref().map(|p| p.display().to_string()));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn setup_context() -> FfiContext {
        let mut ctx = FfiContext::new();
        RecordingFfi::init(&mut ctx).expect("init should succeed");
        ctx
    }

    #[test]
    #[serial]
    fn start_recording_invalid_parent_directory() {
        let mut ctx = setup_context();
        let result = start_recording(&mut ctx, "/nonexistent/directory/session.krx");
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.success, false);
        assert!(res
            .error
            .unwrap()
            .contains("parent directory does not exist"));
    }

    #[test]
    #[serial]
    fn start_recording_success() {
        let mut ctx = setup_context();
        let dir = tempfile::tempdir().expect("create temp dir");
        let session_path = dir.path().join("test_session.krx");

        let result = start_recording(&mut ctx, &session_path.display().to_string());
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.success, true);
        assert!(res.output_path.is_some());

        // Verify state
        let is_rec = is_recording(&ctx).unwrap();
        assert!(is_rec);
    }

    #[test]
    #[serial]
    fn start_recording_already_in_progress() {
        let mut ctx = setup_context();
        let dir = tempfile::tempdir().expect("create temp dir");
        let session_path = dir.path().join("test_session.krx");

        // Start first recording
        let _ = start_recording(&mut ctx, &session_path.display().to_string());

        // Try to start another
        let result = start_recording(&mut ctx, &session_path.display().to_string());
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.success, false);
        assert!(res.error.unwrap().contains("already in progress"));
    }

    #[test]
    #[serial]
    fn stop_recording_not_in_progress() {
        let mut ctx = setup_context();
        let result = stop_recording(&mut ctx);
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.success, false);
        assert!(res.error.unwrap().contains("no recording in progress"));
    }

    #[test]
    #[serial]
    fn stop_recording_success_no_recorder() {
        let mut ctx = setup_context();
        let dir = tempfile::tempdir().expect("create temp dir");
        let session_path = dir.path().join("test_session.krx");

        // Start recording
        let _ = start_recording(&mut ctx, &session_path.display().to_string());

        // Stop without an active recorder (no engine running)
        let result = stop_recording(&mut ctx);
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.success, true);
        assert_eq!(res.event_count, Some(0));

        // Verify state
        let is_rec = is_recording(&ctx).unwrap();
        assert!(!is_rec);
    }

    #[test]
    #[serial]
    fn is_recording_false_when_not_started() {
        let ctx = setup_context();
        let result = is_recording(&ctx);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    #[serial]
    fn get_recording_path_none_when_not_recording() {
        let ctx = setup_context();
        let result = get_recording_path(&ctx);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    #[serial]
    fn get_recording_path_returns_path_when_recording() {
        let mut ctx = setup_context();
        let dir = tempfile::tempdir().expect("create temp dir");
        let session_path = dir.path().join("test_session.krx");

        // Start recording
        let _ = start_recording(&mut ctx, &session_path.display().to_string());

        // Get path
        let result = get_recording_path(&ctx);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.is_some());
        assert!(path.unwrap().contains("test_session.krx"));
    }
}
