//! Session and script-related FFI exports.
//!
//! Functions for loading scripts, evaluating commands, and managing discovery sessions.
#![allow(unsafe_code)]

use super::callbacks::{callback_registry, DiscoveryEventCallback};
use crate::cli::commands::SimulateCommand;
use crate::cli::OutputFormat;
use crate::discovery::{session::set_session_update_sink, SessionUpdate};
use crate::scripting::test_discovery::discover_tests;
use crate::scripting::test_runner::{TestRunner, TestSummary};
use crate::scripting::with_active_runtime;
use crate::scripting::RhaiRuntime;
use crate::traits::ScriptRuntime;
use serde::{Deserialize, Serialize};
use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::ptr;
use std::sync::Arc;

/// Load a script file.
///
/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_load_script(path: *const c_char) -> i32 {
    if path.is_null() {
        return -1;
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let script_name = Path::new(path_str)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("<unknown>");

    tracing::info!(
        service = "keyrx",
        event = "ffi_load_script",
        component = "ffi_exports",
        script = script_name,
        "Loading script"
    );

    // Load and run the script using the active runtime
    match with_active_runtime(|runtime| {
        runtime.load_file(path_str)?;
        runtime.run_script()
    }) {
        Some(Ok(())) => 0,
        Some(Err(err)) => {
            tracing::error!(
                service = "keyrx",
                event = "ffi_load_script_error",
                component = "ffi_exports",
                script = script_name,
                error = %err,
                "Script syntax/execution error"
            );
            -3
        }
        None => {
            tracing::error!(
                service = "keyrx",
                event = "ffi_load_script_error",
                component = "ffi_exports",
                script = script_name,
                "Engine not initialized"
            );
            -4
        }
    }
}

/// Script validation error detail.
#[derive(Serialize)]
struct ValidationError {
    line: Option<usize>,
    column: Option<usize>,
    message: String,
}

/// Script validation result.
#[derive(Serialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<ValidationError>,
}

/// Validate a Rhai script without executing it.
///
/// Returns JSON: `ok:{valid: bool, errors: [{line, column, message}]}`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_check_script(path: *const c_char) -> *mut c_char {
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

    let script = match std::fs::read_to_string(path_str) {
        Ok(s) => s,
        Err(err) => {
            let result = ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    line: None,
                    column: None,
                    message: format!("Failed to read file: {err}"),
                }],
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|e| format!("error:{e}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let engine = rhai::Engine::new();
    let result = match engine.compile(&script) {
        Ok(_) => ValidationResult {
            valid: true,
            errors: vec![],
        },
        Err(e) => {
            let position = e.position();
            let (line, column) = if position.is_none() {
                (None, None)
            } else {
                (position.line(), position.position())
            };
            ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    line,
                    column,
                    message: e.to_string(),
                }],
            }
        }
    };

    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Discovered test for FFI JSON output.
#[derive(Serialize)]
struct DiscoveredTestJson {
    name: String,
    file: String,
    line: Option<u32>,
}

/// Discover test functions in a Rhai script.
///
/// Returns JSON: `ok:[{name, file, line}, ...]`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_discover_tests(path: *const c_char) -> *mut c_char {
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

    let script = match std::fs::read_to_string(path_str) {
        Ok(s) => s,
        Err(err) => {
            return CString::new(format!("error:Failed to read file: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let engine = rhai::Engine::new();
    let ast = match engine.compile(&script) {
        Ok(ast) => ast,
        Err(err) => {
            return CString::new(format!("error:Compile error: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let tests = discover_tests(&ast);
    let json_tests: Vec<DiscoveredTestJson> = tests
        .into_iter()
        .map(|t| DiscoveredTestJson {
            name: t.name,
            file: path_str.to_string(),
            line: t.line_number,
        })
        .collect();

    let payload = serde_json::to_string(&json_tests)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Test result for FFI JSON output.
#[derive(Serialize)]
struct TestResultJson {
    name: String,
    passed: bool,
    error: Option<String>,
    #[serde(rename = "durationMs")]
    duration_ms: f64,
}

/// Test run result for FFI JSON output.
#[derive(Serialize)]
struct TestRunResult {
    total: usize,
    passed: usize,
    failed: usize,
    #[serde(rename = "durationMs")]
    duration_ms: f64,
    results: Vec<TestResultJson>,
}

/// Run tests in a Rhai script with optional filter.
///
/// Returns JSON: `ok:{total, passed, failed, durationMs, results: [{name, passed, error, durationMs}]}`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `path` and `filter` must be valid null-terminated UTF-8 strings (filter can be null).
#[no_mangle]
pub unsafe extern "C" fn keyrx_run_tests(
    path: *const c_char,
    filter: *const c_char,
) -> *mut c_char {
    if path.is_null() {
        return CString::new("error:null path").map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new("error:invalid utf8 in path")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let filter_str = if filter.is_null() {
        None
    } else {
        match CStr::from_ptr(filter).to_str() {
            Ok(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    };

    // Read and compile script
    let script = match std::fs::read_to_string(path_str) {
        Ok(s) => s,
        Err(err) => {
            return CString::new(format!("error:Failed to read file: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let engine = rhai::Engine::new();
    let ast = match engine.compile(&script) {
        Ok(ast) => ast,
        Err(err) => {
            return CString::new(format!("error:Compile error: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Discover tests
    let discovered = discover_tests(&ast);
    if discovered.is_empty() {
        let result = TestRunResult {
            total: 0,
            passed: 0,
            failed: 0,
            duration_ms: 0.0,
            results: vec![],
        };
        let payload = serde_json::to_string(&result)
            .map(|json| format!("ok:{json}"))
            .unwrap_or_else(|err| format!("error:{err}"));
        return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    // Create runtime and load script
    let mut runtime = match RhaiRuntime::new() {
        Ok(r) => r,
        Err(err) => {
            return CString::new(format!("error:Failed to create runtime: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    if let Err(err) = runtime.load_file(path_str) {
        return CString::new(format!("error:Failed to load script: {err}"))
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    // Run tests
    let runner = TestRunner::new();
    let results = match filter_str {
        Some(pattern) => runner.run_filtered(&mut runtime, &discovered, pattern),
        None => runner.run_tests(&mut runtime, &discovered),
    };

    let summary = TestSummary::from_results(&results);
    let json_results: Vec<TestResultJson> = results
        .into_iter()
        .map(|r| TestResultJson {
            name: r.name,
            passed: r.passed,
            error: if r.passed { None } else { Some(r.message) },
            duration_ms: r.duration_us as f64 / 1000.0,
        })
        .collect();

    let result = TestRunResult {
        total: summary.total,
        passed: summary.passed,
        failed: summary.failed,
        duration_ms: summary.duration_us as f64 / 1000.0,
        results: json_results,
    };

    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Key input for simulation FFI.
#[derive(Deserialize)]
struct SimKeyInput {
    code: String,
    #[serde(rename = "holdMs", default)]
    hold_ms: Option<u64>,
}

/// Simulation mapping result for FFI JSON output.
#[derive(Serialize)]
struct SimMapping {
    input: String,
    output: String,
    decision: String,
}

/// Simulation result for FFI JSON output.
#[derive(Serialize)]
struct SimFfiResult {
    mappings: Vec<SimMapping>,
    #[serde(rename = "activeLayers")]
    active_layers: Vec<String>,
    pending: Vec<String>,
}

/// Simulate key sequences through the engine.
///
/// # Arguments
/// * `keys_json` - JSON array of key inputs: `[{code: "A", holdMs: 100}, ...]`
/// * `script_path` - Optional path to Rhai script (can be null)
/// * `combo_mode` - If true, keys are pressed simultaneously; otherwise sequentially
///
/// Returns JSON: `ok:{mappings: [{input, output, decision}], activeLayers, pending}`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `keys_json` and `script_path` must be valid null-terminated UTF-8 strings or null.
#[no_mangle]
pub unsafe extern "C" fn keyrx_simulate(
    keys_json: *const c_char,
    script_path: *const c_char,
    combo_mode: bool,
) -> *mut c_char {
    if keys_json.is_null() {
        return CString::new("error:null keys_json")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let keys_str = match CStr::from_ptr(keys_json).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new("error:invalid utf8 in keys_json")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let keys: Vec<SimKeyInput> = match serde_json::from_str(keys_str) {
        Ok(k) => k,
        Err(err) => {
            return CString::new(format!("error:Invalid keys JSON: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    if keys.is_empty() {
        let result = SimFfiResult {
            mappings: vec![],
            active_layers: vec!["base".to_string()],
            pending: vec![],
        };
        let payload = serde_json::to_string(&result)
            .map(|json| format!("ok:{json}"))
            .unwrap_or_else(|err| format!("error:{err}"));
        return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    // Build input string for SimulateCommand
    let input_keys: Vec<String> = keys
        .iter()
        .map(|k| {
            if let Some(hold) = k.hold_ms {
                format!("{}:hold:{}", k.code, hold)
            } else {
                k.code.clone()
            }
        })
        .collect();
    let input_str = input_keys.join(",");

    // Get script path if provided
    let script_path_opt = if script_path.is_null() {
        None
    } else {
        match CStr::from_ptr(script_path).to_str() {
            Ok(s) if !s.is_empty() => Some(std::path::PathBuf::from(s)),
            _ => None,
        }
    };

    // Create and run simulation
    let cmd =
        SimulateCommand::new(input_str, script_path_opt, OutputFormat::Json).with_combo(combo_mode);

    // Use tokio runtime for async execution
    let rt = match tokio::runtime::Builder::new_current_thread().build() {
        Ok(rt) => rt,
        Err(err) => {
            return CString::new(format!("error:Failed to create runtime: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let output = match rt.block_on(cmd.execute()) {
        Ok(o) => o,
        Err(err) => {
            return CString::new(format!("error:Simulation failed: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Convert to FFI result format
    let mappings: Vec<SimMapping> = output
        .results
        .iter()
        .map(|r| {
            let decision = if r.output == "BLOCKED" {
                "block"
            } else if r.output == r.input {
                "pass"
            } else if r.output == "NO_OUTPUT" {
                "pending"
            } else {
                "remap"
            };
            SimMapping {
                input: r.input.clone(),
                output: r.output.clone(),
                decision: decision.to_string(),
            }
        })
        .collect();

    let pending: Vec<String> = output.pending.iter().map(|p| format!("{:?}", p)).collect();

    let result = SimFfiResult {
        mappings,
        active_layers: output.active_layers,
        pending,
    };

    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Evaluate a console command against the active runtime.
///
/// Responses are prefixed with `ok:` or `error:`. Callers must free the returned pointer with
/// `keyrx_free_string`.
///
/// # Safety
/// `command` must be a valid, null-terminated UTF-8 string pointer or null.
#[no_mangle]
pub unsafe extern "C" fn keyrx_eval(command: *const c_char) -> *mut c_char {
    if command.is_null() {
        return CString::new("error: null command")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let cmd_str = match CStr::from_ptr(command).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new("error: invalid utf8")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let response = match with_active_runtime(|runtime| runtime.execute(cmd_str)) {
        Some(Ok(_)) => "ok:".to_string(),
        Some(Err(err)) => format!("error: {err}"),
        None => "error: engine not initialized".to_string(),
    };

    CString::new(response).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Register a callback for discovery progress updates.
/// The provided pointer/length pair references a JSON payload that is only valid for the duration of the callback.
#[no_mangle]
pub extern "C" fn keyrx_on_discovery_progress(callback: Option<DiscoveryEventCallback>) {
    callback_registry().set_progress(callback);
    refresh_discovery_sink();
}

/// Register a callback for duplicate key warnings during discovery.
/// The provided pointer/length pair references a JSON payload that is only valid for the duration of the callback.
#[no_mangle]
pub extern "C" fn keyrx_on_discovery_duplicate(callback: Option<DiscoveryEventCallback>) {
    callback_registry().set_duplicate(callback);
    refresh_discovery_sink();
}

/// Register a callback for discovery summaries (completed, cancelled, or bypassed).
/// The provided pointer/length pair references a JSON payload that is only valid for the duration of the callback.
#[no_mangle]
pub extern "C" fn keyrx_on_discovery_summary(callback: Option<DiscoveryEventCallback>) {
    callback_registry().set_summary(callback);
    refresh_discovery_sink();
}

fn refresh_discovery_sink() {
    if callback_registry().has_any_discovery_callback() {
        set_session_update_sink(Some(discovery_sink()));
    } else {
        set_session_update_sink(None);
    }
}

fn discovery_sink() -> Arc<dyn Fn(&SessionUpdate) + Send + Sync + 'static> {
    Arc::new(|update| {
        let registry = callback_registry();
        match update {
            SessionUpdate::Ignored => {}
            SessionUpdate::Progress(progress) => {
                registry.invoke_discovery(registry.progress(), progress, "progress");
            }
            SessionUpdate::Duplicate(dup) => {
                registry.invoke_discovery(registry.duplicate(), dup, "duplicate");
            }
            SessionUpdate::Finished(summary) => {
                registry.invoke_discovery(registry.summary(), summary, "summary");
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{
        session::publish_session_update, DeviceId, DiscoveryProgress, DiscoverySummary,
        ExpectedPosition, PhysicalKey, SessionStatus,
    };
    use crate::drivers::keycodes::KeyCode;
    use crate::engine::RemapAction;
    use crate::ffi::keyrx_free_string;
    use crate::scripting::{clear_active_runtime, set_active_runtime, RhaiRuntime};
    use serial_test::serial;
    use std::collections::HashMap;
    use std::ptr;
    use std::slice;
    use std::sync::{Mutex, OnceLock};

    fn progress_store() -> &'static Mutex<Vec<Vec<u8>>> {
        static STORE: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
        STORE.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn summary_store() -> &'static Mutex<Vec<Vec<u8>>> {
        static STORE: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
        STORE.get_or_init(|| Mutex::new(Vec::new()))
    }

    unsafe extern "C" fn record_progress(ptr: *const u8, len: usize) {
        let slice = slice::from_raw_parts(ptr, len);
        progress_store().lock().unwrap().push(slice.to_vec());
    }

    unsafe extern "C" fn record_summary(ptr: *const u8, len: usize) {
        let slice = slice::from_raw_parts(ptr, len);
        summary_store().lock().unwrap().push(slice.to_vec());
    }

    #[test]
    #[serial]
    fn load_script_returns_error_without_runtime() {
        clear_active_runtime();
        let path = CString::new("script.rhai").expect("CString should not contain nulls");
        let result = unsafe { keyrx_load_script(path.as_ptr()) };
        assert_eq!(result, -4); // Engine not initialized
    }

    #[test]
    #[serial]
    fn load_script_returns_error_for_missing_file() {
        clear_active_runtime();
        let mut runtime = RhaiRuntime::new().expect("runtime should initialize");
        set_active_runtime(&mut runtime);

        let path = CString::new("/nonexistent/path/script.rhai")
            .expect("CString should not contain nulls");
        let result = unsafe { keyrx_load_script(path.as_ptr()) };
        assert_eq!(result, -3); // File not found / syntax error

        clear_active_runtime();
    }

    #[test]
    #[serial]
    fn load_script_loads_valid_script() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        clear_active_runtime();
        let mut runtime = RhaiRuntime::new().expect("runtime should initialize");
        set_active_runtime(&mut runtime);

        // Create a temporary script file
        let mut temp_file = NamedTempFile::new().expect("temp file should create");
        writeln!(temp_file, r#"remap("A", "B");"#).expect("write should succeed");
        let temp_path = temp_file
            .path()
            .to_str()
            .expect("path should be valid UTF-8");

        let path = CString::new(temp_path).expect("CString should not contain nulls");
        let result = unsafe { keyrx_load_script(path.as_ptr()) };
        assert_eq!(result, 0); // Success

        // Verify the mapping was registered
        assert!(matches!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        ));

        clear_active_runtime();
    }

    #[test]
    fn load_script_rejects_null_pointer() {
        let result = unsafe { keyrx_load_script(ptr::null()) };
        assert_eq!(result, -1);
    }

    #[test]
    fn load_script_rejects_invalid_utf8() {
        static INVALID_UTF8: [u8; 2] = [0xFF, 0x00];
        let result = unsafe { keyrx_load_script(INVALID_UTF8.as_ptr() as *const c_char) };
        assert_eq!(result, -2);
    }

    #[test]
    fn check_script_valid_syntax() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "let x = 1 + 2;").unwrap();

        let path = CString::new(temp.path().to_str().unwrap()).unwrap();
        let ptr = unsafe { keyrx_check_script(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["valid"], true);
        assert!(result["errors"].as_array().unwrap().is_empty());
    }

    #[test]
    fn check_script_invalid_syntax() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "let x = (1 + 2").unwrap(); // Missing closing paren

        let path = CString::new(temp.path().to_str().unwrap()).unwrap();
        let ptr = unsafe { keyrx_check_script(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["valid"], false);
        assert!(!result["errors"].as_array().unwrap().is_empty());

        let error = &result["errors"][0];
        assert!(error["message"].as_str().unwrap().len() > 0);
    }

    #[test]
    fn check_script_missing_file() {
        let path = CString::new("/nonexistent/script.rhai").unwrap();
        let ptr = unsafe { keyrx_check_script(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["valid"], false);
        assert!(result["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Failed to read file"));
    }

    #[test]
    fn check_script_null_pointer() {
        let ptr = unsafe { keyrx_check_script(ptr::null()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };
        assert!(raw.starts_with("error:"));
    }

    #[test]
    fn discover_tests_finds_test_functions() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::with_suffix(".rhai").unwrap();
        writeln!(
            temp,
            r#"
            fn test_alpha() {{ let x = 1; }}
            fn test_beta() {{ let y = 2; }}
            fn helper() {{ }}
        "#
        )
        .unwrap();

        let path = CString::new(temp.path().to_str().unwrap()).unwrap();
        let ptr = unsafe { keyrx_discover_tests(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let tests: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap();
        assert_eq!(tests.len(), 2);
        assert!(tests.iter().any(|t| t["name"] == "test_alpha"));
        assert!(tests.iter().any(|t| t["name"] == "test_beta"));
    }

    #[test]
    fn discover_tests_empty_script() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::with_suffix(".rhai").unwrap();
        writeln!(temp, "fn helper() {{ }}").unwrap();

        let path = CString::new(temp.path().to_str().unwrap()).unwrap();
        let ptr = unsafe { keyrx_discover_tests(path.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let tests: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap();
        assert!(tests.is_empty());
    }

    #[test]
    fn run_tests_passing() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::with_suffix(".rhai").unwrap();
        writeln!(temp, "fn test_pass() {{ let x = 1 + 1; }}").unwrap();

        let path = CString::new(temp.path().to_str().unwrap()).unwrap();
        let ptr = unsafe { keyrx_run_tests(path.as_ptr(), ptr::null()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"), "got: {raw}");
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["total"], 1);
        assert_eq!(result["passed"], 1);
        assert_eq!(result["failed"], 0);
    }

    #[test]
    fn run_tests_with_filter() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::with_suffix(".rhai").unwrap();
        writeln!(
            temp,
            r#"
            fn test_alpha() {{ let x = 1; }}
            fn test_beta() {{ let y = 2; }}
        "#
        )
        .unwrap();

        let path = CString::new(temp.path().to_str().unwrap()).unwrap();
        let filter = CString::new("test_alpha*").unwrap();
        let ptr = unsafe { keyrx_run_tests(path.as_ptr(), filter.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        // Only test_alpha should run due to filter
        assert_eq!(result["total"], 1);
        assert_eq!(result["results"][0]["name"], "test_alpha");
    }

    #[test]
    fn simulate_empty_keys() {
        let keys = CString::new("[]").unwrap();
        let ptr = unsafe { keyrx_simulate(keys.as_ptr(), ptr::null(), false) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert!(result["mappings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn simulate_basic_key() {
        let keys = CString::new(r#"[{"code": "A"}]"#).unwrap();
        let ptr = unsafe { keyrx_simulate(keys.as_ptr(), ptr::null(), false) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"), "got: {raw}");
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        // Without a script, key should pass through
        assert!(!result["mappings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn simulate_with_script() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::with_suffix(".rhai").unwrap();
        writeln!(temp, r#"remap("A", "B");"#).unwrap();

        let keys = CString::new(r#"[{"code": "A"}]"#).unwrap();
        let path = CString::new(temp.path().to_str().unwrap()).unwrap();
        let ptr = unsafe { keyrx_simulate(keys.as_ptr(), path.as_ptr(), false) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"), "got: {raw}");
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let mappings = result["mappings"].as_array().unwrap();

        // Find the A key press result
        let a_press = mappings.iter().find(|m| m["input"] == "A");
        assert!(a_press.is_some(), "should have A input in mappings");
    }

    #[test]
    #[serial]
    fn eval_returns_error_when_runtime_missing() {
        clear_active_runtime();
        let cmd = CString::new(r#"remap("A","B");"#).unwrap();
        let ptr = unsafe { keyrx_eval(cmd.as_ptr()) };
        assert!(!ptr.is_null());
        let response = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("response should be utf8");
        assert!(
            response.starts_with("error: engine not initialized"),
            "unexpected response: {response}"
        );
        unsafe { keyrx_free_string(ptr) };
    }

    #[test]
    #[serial]
    fn eval_executes_against_shared_runtime() {
        clear_active_runtime();
        let mut runtime = RhaiRuntime::new().expect("runtime should initialize");
        set_active_runtime(&mut runtime);

        let cmd = CString::new(r#"remap("A","B");"#).unwrap();
        let ptr = unsafe { keyrx_eval(cmd.as_ptr()) };
        assert!(!ptr.is_null());
        let response = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("response should be utf8");
        assert!(
            response.starts_with("ok:"),
            "unexpected response: {response}"
        );
        unsafe { keyrx_free_string(ptr) };

        assert!(matches!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        ));
        clear_active_runtime();
    }

    #[test]
    fn discovery_callbacks_emit_json_payloads() {
        progress_store().lock().unwrap().clear();
        summary_store().lock().unwrap().clear();

        keyrx_on_discovery_progress(Some(record_progress));
        keyrx_on_discovery_summary(Some(record_summary));

        let progress = SessionUpdate::Progress(DiscoveryProgress {
            captured: 1,
            total: 3,
            next: Some(ExpectedPosition { row: 0, col: 1 }),
        });
        publish_session_update(&progress);

        let mut keymap = HashMap::new();
        keymap.insert(10, PhysicalKey::new(10, 0, 0));

        let mut aliases = HashMap::new();
        aliases.insert("r0_c0".to_string(), 10);

        let summary = SessionUpdate::Finished(DiscoverySummary {
            device_id: DeviceId::new(0x1, 0x2),
            status: SessionStatus::Completed,
            message: None,
            rows: 1,
            cols_per_row: vec![1],
            captured: 1,
            total: 1,
            next: None,
            unmapped: vec![],
            duplicates: vec![],
            keymap,
            aliases,
        });
        publish_session_update(&summary);

        let progress_payloads = progress_store().lock().unwrap();
        assert_eq!(progress_payloads.len(), 1);
        let progress_json: DiscoveryProgress =
            serde_json::from_slice(&progress_payloads[0]).expect("valid progress json");
        assert_eq!(progress_json.captured, 1);
        assert_eq!(progress_json.total, 3);

        let summary_payloads = summary_store().lock().unwrap();
        assert_eq!(summary_payloads.len(), 1);
        let summary_json: DiscoverySummary =
            serde_json::from_slice(&summary_payloads[0]).expect("valid summary json");
        assert_eq!(summary_json.status, SessionStatus::Completed);
        assert_eq!(summary_json.device_id.vendor_id, 0x1);

        keyrx_on_discovery_progress(None);
        keyrx_on_discovery_summary(None);
    }
}
