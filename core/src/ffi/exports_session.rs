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

// ─── Session Management FFI Exports ────────────────────────────────────────

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

// ─── Benchmark FFI Export ─────────────────────────────────────────────────

/// Benchmark result for FFI JSON output.
#[derive(Serialize)]
struct BenchmarkResultJson {
    #[serde(rename = "minNs")]
    min_ns: u64,
    #[serde(rename = "maxNs")]
    max_ns: u64,
    #[serde(rename = "meanNs")]
    mean_ns: u64,
    #[serde(rename = "p99Ns")]
    p99_ns: u64,
    iterations: usize,
    #[serde(rename = "hasWarning")]
    has_warning: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<String>,
}

/// Run latency benchmark on the engine.
///
/// Returns JSON: `ok:{minNs, maxNs, meanNs, p99Ns, iterations, hasWarning, warning?}`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `script_path` must be a valid null-terminated UTF-8 string or null.
#[no_mangle]
pub unsafe extern "C" fn keyrx_run_benchmark(
    iterations: u32,
    script_path: *const c_char,
) -> *mut c_char {
    use crate::cli::commands::BenchCommand;
    use crate::cli::OutputFormat;

    let iterations = iterations as usize;
    if iterations == 0 {
        return CString::new("error:iterations must be > 0")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let script_path_opt = if script_path.is_null() {
        None
    } else {
        match CStr::from_ptr(script_path).to_str() {
            Ok(s) if !s.is_empty() => Some(std::path::PathBuf::from(s)),
            _ => None,
        }
    };

    let cmd = BenchCommand::new(iterations, script_path_opt, OutputFormat::Json);

    // Use tokio runtime for async execution
    let rt = match tokio::runtime::Builder::new_current_thread().build() {
        Ok(rt) => rt,
        Err(err) => {
            return CString::new(format!("error:Failed to create runtime: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let bench_result = match rt.block_on(cmd.execute()) {
        Ok(r) => r,
        Err(err) => {
            return CString::new(format!("error:Benchmark failed: {err}"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let result = BenchmarkResultJson {
        min_ns: bench_result.min_ns,
        max_ns: bench_result.max_ns,
        mean_ns: bench_result.mean_ns,
        p99_ns: bench_result.p99_ns,
        iterations: bench_result.iterations,
        has_warning: bench_result.warning.is_some(),
        warning: bench_result.warning,
    };

    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
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

    // ─── Session Management Tests ──────────────────────────────────────────

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

    #[test]
    fn run_benchmark_basic() {
        let ptr = unsafe { keyrx_run_benchmark(100, ptr::null()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"), "got: {raw}");
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();

        assert!(result["minNs"].as_u64().is_some());
        assert!(result["maxNs"].as_u64().is_some());
        assert!(result["meanNs"].as_u64().is_some());
        assert!(result["p99Ns"].as_u64().is_some());
        assert!(result["iterations"].as_u64().is_some());
        assert_eq!(result["iterations"], 100);
    }
}
