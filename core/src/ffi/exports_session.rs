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
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex, OnceLock};

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

// ─── Discovery FFI Exports ─────────────────────────────────────────────────

/// Global state for active discovery session.
static DISCOVERY_SESSION: OnceLock<Mutex<Option<DiscoverySessionState>>> = OnceLock::new();
static DISCOVERY_CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

fn discovery_session_slot() -> &'static Mutex<Option<DiscoverySessionState>> {
    DISCOVERY_SESSION.get_or_init(|| Mutex::new(None))
}

struct DiscoverySessionState {
    session: crate::discovery::DiscoverySession,
    device_path: std::path::PathBuf,
}

/// Discovery start result for FFI JSON output.
#[derive(Serialize)]
struct DiscoveryStartResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(rename = "totalKeys", skip_serializing_if = "Option::is_none")]
    total_keys: Option<usize>,
}

/// Start a discovery session for a device.
///
/// # Arguments
/// * `device_id` - Device identifier as "vendorId:productId" (e.g., "1234:5678")
/// * `rows` - Number of rows in the keyboard layout
/// * `cols_per_row_json` - JSON array of column counts per row (e.g., "[14, 14, 13, 12, 8]")
///
/// Returns JSON: `ok:{success: bool, error?: string, totalKeys?: number}`
///
/// Progress updates are delivered via the callbacks registered with:
/// - `keyrx_on_discovery_progress`
/// - `keyrx_on_discovery_duplicate`
/// - `keyrx_on_discovery_summary`
///
/// Call `keyrx_process_discovery_event` to process input events.
/// Call `keyrx_cancel_discovery` to cancel the session.
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `device_id` and `cols_per_row_json` must be valid null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn keyrx_start_discovery(
    device_id: *const c_char,
    rows: u8,
    cols_per_row_json: *const c_char,
) -> *mut c_char {
    if device_id.is_null() || cols_per_row_json.is_null() {
        let result = DiscoveryStartResult {
            success: false,
            error: Some("null pointer".to_string()),
            total_keys: None,
        };
        let payload = serde_json::to_string(&result)
            .map(|json| format!("ok:{json}"))
            .unwrap_or_else(|err| format!("error:{err}"));
        return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let device_id_str = match CStr::from_ptr(device_id).to_str() {
        Ok(s) => s,
        Err(_) => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some("invalid utf8 in device_id".to_string()),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let cols_str = match CStr::from_ptr(cols_per_row_json).to_str() {
        Ok(s) => s,
        Err(_) => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some("invalid utf8 in cols_per_row_json".to_string()),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Parse device ID (format: "vendorId:productId")
    let parts: Vec<&str> = device_id_str.split(':').collect();
    let (vendor_id, product_id) = if parts.len() == 2 {
        let v = u16::from_str_radix(parts[0], 16).unwrap_or(0);
        let p = u16::from_str_radix(parts[1], 16).unwrap_or(0);
        (v, p)
    } else {
        let result = DiscoveryStartResult {
            success: false,
            error: Some("device_id must be 'vendorId:productId' (hex)".to_string()),
            total_keys: None,
        };
        let payload = serde_json::to_string(&result)
            .map(|json| format!("ok:{json}"))
            .unwrap_or_else(|err| format!("error:{err}"));
        return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
    };

    // Parse cols_per_row JSON array
    let cols_per_row: Vec<u8> = match serde_json::from_str(cols_str) {
        Ok(cols) => cols,
        Err(err) => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some(format!("invalid cols_per_row JSON: {err}")),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Find device by vendor/product ID
    let devices = match crate::drivers::list_keyboards() {
        Ok(d) => d,
        Err(err) => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some(format!("failed to list devices: {err}")),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let device = devices
        .iter()
        .find(|d| d.vendor_id == vendor_id && d.product_id == product_id);
    let device_path = match device {
        Some(d) => d.path.clone(),
        None => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some(format!(
                    "device {:04x}:{:04x} not found",
                    vendor_id, product_id
                )),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Create discovery session
    let dev_id = crate::discovery::DeviceId::new(vendor_id, product_id);
    let session = match crate::discovery::DiscoverySession::new(dev_id, rows, cols_per_row) {
        Ok(s) => s,
        Err(err) => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some(format!("invalid layout: {err}")),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let total_keys = session.progress().total;

    // Store session state
    DISCOVERY_CANCEL_FLAG.store(false, Ordering::SeqCst);
    if let Ok(mut guard) = discovery_session_slot().lock() {
        *guard = Some(DiscoverySessionState {
            session,
            device_path,
        });
    }

    let result = DiscoveryStartResult {
        success: true,
        error: None,
        total_keys: Some(total_keys),
    };
    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Process an input event during discovery.
///
/// # Arguments
/// * `scan_code` - The scan code of the key event
/// * `pressed` - Whether the key was pressed (true) or released (false)
/// * `timestamp_us` - Timestamp in microseconds
///
/// Returns:
/// - 0: Event processed successfully
/// - 1: Discovery session completed
/// - -1: No active discovery session
/// - -2: Discovery was cancelled
///
/// Progress/duplicate/summary callbacks will be invoked as appropriate.
#[no_mangle]
pub extern "C" fn keyrx_process_discovery_event(
    scan_code: u16,
    pressed: bool,
    timestamp_us: u64,
) -> i32 {
    if DISCOVERY_CANCEL_FLAG.load(Ordering::SeqCst) {
        return -2;
    }

    let mut guard = match discovery_session_slot().lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };

    let state = match guard.as_mut() {
        Some(s) => s,
        None => return -1,
    };

    let device_path_str = state.device_path.to_str().map(String::from);
    let event = crate::engine::InputEvent::with_metadata(
        crate::engine::KeyCode::Unknown(scan_code),
        pressed,
        timestamp_us,
        device_path_str,
        false,
        false,
        scan_code,
    );

    match state.session.handle_event(&event) {
        crate::discovery::SessionUpdate::Finished(_) => {
            // Clear session after completion
            *guard = None;
            1
        }
        _ => 0,
    }
}

/// Cancel an ongoing discovery session.
///
/// Returns:
/// - 0: Discovery cancelled successfully
/// - -1: No active discovery session
#[no_mangle]
pub extern "C" fn keyrx_cancel_discovery() -> i32 {
    DISCOVERY_CANCEL_FLAG.store(true, Ordering::SeqCst);

    let mut guard = match discovery_session_slot().lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };

    match guard.take() {
        Some(mut state) => {
            let summary = state.session.cancel("cancelled by user");
            // Publish the cancellation through callbacks
            crate::discovery::session::publish_session_update(
                &crate::discovery::SessionUpdate::Finished(summary),
            );
            0
        }
        None => -1,
    }
}

/// Get the current discovery progress.
///
/// Returns JSON: `ok:{captured, total, next?: {row, col}}` or `error:...`
///
/// Caller must free with `keyrx_free_string`.
#[no_mangle]
pub extern "C" fn keyrx_get_discovery_progress() -> *mut c_char {
    let guard = match discovery_session_slot().lock() {
        Ok(g) => g,
        Err(_) => {
            return CString::new("error:lock poisoned")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    match guard.as_ref() {
        Some(state) => {
            let progress = state.session.progress();
            let payload = serde_json::to_string(&progress)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
        }
        None => CString::new("error:no active discovery session")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw),
    }
}

// ─── Diagnostics FFI Export ────────────────────────────────────────────────

/// Diagnostics result for FFI JSON output.
#[derive(Serialize)]
struct DiagnosticsResultJson {
    checks: Vec<DiagnosticCheckJson>,
    passed: usize,
    failed: usize,
    warned: usize,
}

/// Diagnostic check for FFI JSON output.
#[derive(Serialize)]
struct DiagnosticCheckJson {
    name: String,
    status: String,
    details: String,
    remediation: Option<String>,
}

/// Run system diagnostics.
///
/// Returns JSON: `ok:{checks: [{name, status, details, remediation}], passed, failed, warned}`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub extern "C" fn keyrx_run_doctor() -> *mut c_char {
    use crate::cli::commands::{CheckStatus, DiagnosticCheck};

    let mut checks: Vec<DiagnosticCheck> = Vec::new();

    // Rhai engine check - always available
    checks.push(DiagnosticCheck::pass(
        "Rhai Engine",
        "Scripting engine available (v1.16)",
    ));

    // Platform-specific checks
    #[cfg(target_os = "linux")]
    run_linux_diagnostics(&mut checks);

    #[cfg(target_os = "windows")]
    run_windows_diagnostics(&mut checks);

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    checks.push(DiagnosticCheck::warn(
        "Platform",
        "Unsupported platform",
        "KeyRx currently supports Linux and Windows only",
    ));

    // Convert to JSON format
    let json_checks: Vec<DiagnosticCheckJson> = checks
        .iter()
        .map(|c| DiagnosticCheckJson {
            name: c.name.clone(),
            status: match c.status {
                CheckStatus::Pass => "pass".to_string(),
                CheckStatus::Fail => "fail".to_string(),
                CheckStatus::Warn => "warn".to_string(),
            },
            details: c.message.clone(),
            remediation: c.remediation.clone(),
        })
        .collect();

    let passed = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Pass)
        .count();
    let failed = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
    let warned = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Warn)
        .count();

    let result = DiagnosticsResultJson {
        checks: json_checks,
        passed,
        failed,
        warned,
    };

    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

#[cfg(target_os = "linux")]
fn run_linux_diagnostics(checks: &mut Vec<crate::cli::commands::DiagnosticCheck>) {
    use crate::cli::commands::DiagnosticCheck;
    use std::fs::File;

    checks.push(DiagnosticCheck::pass("Platform", "Linux (evdev/uinput)"));

    // Check /dev/uinput exists
    let uinput_path = std::path::Path::new("/dev/uinput");
    if uinput_path.exists() {
        checks.push(DiagnosticCheck::pass(
            "/dev/uinput exists",
            "Device node found",
        ));
    } else {
        checks.push(DiagnosticCheck::fail(
            "/dev/uinput exists",
            "Device node not found",
            "Load uinput kernel module: sudo modprobe uinput",
        ));
    }

    // Check /dev/uinput is accessible
    match File::open("/dev/uinput") {
        Ok(_) => checks.push(DiagnosticCheck::pass(
            "/dev/uinput accessible",
            "Read access confirmed",
        )),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => checks.push(DiagnosticCheck::fail(
                "/dev/uinput accessible",
                "Device not found",
                "Load uinput kernel module: sudo modprobe uinput",
            )),
            std::io::ErrorKind::PermissionDenied => checks.push(DiagnosticCheck::fail(
                "/dev/uinput accessible",
                "Permission denied",
                "Add user to input group: sudo usermod -aG input $USER && newgrp input",
            )),
            _ => checks.push(DiagnosticCheck::fail(
                "/dev/uinput accessible",
                format!("Cannot access: {}", e),
                "Check device permissions and kernel module status",
            )),
        },
    }

    // Check user is in input group
    let groups = std::process::Command::new("groups")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    if groups.split_whitespace().any(|g| g == "input") {
        checks.push(DiagnosticCheck::pass(
            "User in input group",
            "Group membership confirmed",
        ));
    } else {
        checks.push(DiagnosticCheck::warn(
            "User in input group",
            "User not in input group",
            "Add user to input group: sudo usermod -aG input $USER && newgrp input",
        ));
    }
}

#[cfg(target_os = "windows")]
fn run_windows_diagnostics(checks: &mut Vec<crate::cli::commands::DiagnosticCheck>) {
    use crate::cli::commands::DiagnosticCheck;
    use windows::core::PCSTR;
    use windows::Win32::System::LibraryLoader::LoadLibraryA;

    checks.push(DiagnosticCheck::pass(
        "Platform",
        "Windows (WH_KEYBOARD_LL)",
    ));

    // Check if user32.dll is loadable (contains SetWindowsHookExW)
    let dll_name = b"user32.dll\0";
    let result = unsafe { LoadLibraryA(PCSTR::from_raw(dll_name.as_ptr())) };

    match result {
        Ok(_) => checks.push(DiagnosticCheck::pass(
            "Keyboard Hook API",
            "SetWindowsHookExW available via user32.dll",
        )),
        Err(_) => checks.push(DiagnosticCheck::fail(
            "Keyboard Hook API",
            "Cannot load user32.dll",
            "Ensure Windows is properly installed; user32.dll should always be present",
        )),
    }
}

// ─── Recording Control FFI Exports ────────────────────────────────────────

/// Global state for recording control.
static RECORDING_STATE: OnceLock<Mutex<RecordingState>> = OnceLock::new();

fn recording_state_slot() -> &'static Mutex<RecordingState> {
    RECORDING_STATE.get_or_init(|| Mutex::new(RecordingState::default()))
}

/// Thread-safe recording state.
#[derive(Default)]
struct RecordingState {
    /// Whether recording is currently active.
    is_recording: bool,
    /// Path where the session will be saved.
    output_path: Option<std::path::PathBuf>,
    /// Active recorder (when engine is running).
    recorder: Option<crate::engine::EventRecorder>,
    /// Last saved session path.
    last_session_path: Option<String>,
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
fn clear_recording_state() {
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

    // ─── Diagnostics Tests ──────────────────────────────────────────────────

    #[test]
    fn run_doctor_returns_diagnostics() {
        let ptr = keyrx_run_doctor();
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"), "got: {raw}");
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();

        // Check structure
        assert!(result["checks"].is_array());
        assert!(result["passed"].is_number());
        assert!(result["failed"].is_number());
        assert!(result["warned"].is_number());

        // Should have at least the Rhai Engine check
        let checks = result["checks"].as_array().unwrap();
        assert!(!checks.is_empty());

        // First check should be Rhai Engine
        assert_eq!(checks[0]["name"], "Rhai Engine");
        assert_eq!(checks[0]["status"], "pass");
        assert!(checks[0]["details"]
            .as_str()
            .unwrap()
            .contains("Scripting engine"));

        // Counts should add up
        let total_checks = checks.len();
        let passed = result["passed"].as_u64().unwrap() as usize;
        let failed = result["failed"].as_u64().unwrap() as usize;
        let warned = result["warned"].as_u64().unwrap() as usize;
        assert_eq!(passed + failed + warned, total_checks);
    }

    #[test]
    fn run_doctor_check_structure() {
        let ptr = keyrx_run_doctor();
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();

        // Each check should have the required fields
        for check in result["checks"].as_array().unwrap() {
            assert!(check["name"].is_string());
            assert!(check["status"].is_string());
            assert!(check["details"].is_string());
            // remediation can be null or string
            assert!(check["remediation"].is_null() || check["remediation"].is_string());

            // Status must be one of the valid values
            let status = check["status"].as_str().unwrap();
            assert!(
                status == "pass" || status == "fail" || status == "warn",
                "invalid status: {status}"
            );
        }
    }

    // ─── Discovery FFI Tests ────────────────────────────────────────────────

    fn clear_discovery_session() {
        if let Ok(mut guard) = super::discovery_session_slot().lock() {
            *guard = None;
        }
        super::DISCOVERY_CANCEL_FLAG.store(false, super::Ordering::SeqCst);
    }

    #[test]
    #[serial]
    fn start_discovery_null_pointers() {
        clear_discovery_session();

        let ptr = unsafe { keyrx_start_discovery(ptr::null(), 1, ptr::null()) };
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
    fn start_discovery_invalid_device_id_format() {
        clear_discovery_session();

        let device_id = CString::new("invalid").unwrap();
        let cols = CString::new("[3, 3]").unwrap();
        let ptr = unsafe { keyrx_start_discovery(device_id.as_ptr(), 2, cols.as_ptr()) };
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
            .contains("vendorId:productId"));
    }

    #[test]
    #[serial]
    fn start_discovery_invalid_cols_json() {
        clear_discovery_session();

        let device_id = CString::new("0001:0002").unwrap();
        let cols = CString::new("not valid json").unwrap();
        let ptr = unsafe { keyrx_start_discovery(device_id.as_ptr(), 2, cols.as_ptr()) };
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
            .contains("invalid cols_per_row JSON"));
    }

    #[test]
    #[serial]
    fn start_discovery_invalid_layout() {
        clear_discovery_session();

        // rows=2 but cols_per_row has 3 elements - mismatch
        let device_id = CString::new("0001:0002").unwrap();
        let cols = CString::new("[3, 3, 3]").unwrap();
        let ptr = unsafe { keyrx_start_discovery(device_id.as_ptr(), 2, cols.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["success"], false);
        // Either "device not found" or "invalid layout" depending on device availability
        assert!(result["error"].is_string());
    }

    #[test]
    #[serial]
    fn cancel_discovery_no_active_session() {
        clear_discovery_session();
        let result = keyrx_cancel_discovery();
        assert_eq!(result, -1);
    }

    #[test]
    #[serial]
    fn get_discovery_progress_no_active_session() {
        clear_discovery_session();
        let ptr = keyrx_get_discovery_progress();
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("error:"));
        assert!(raw.contains("no active discovery session"));
    }

    #[test]
    #[serial]
    fn process_discovery_event_no_active_session() {
        clear_discovery_session();
        let result = keyrx_process_discovery_event(30, true, 1000);
        assert_eq!(result, -1);
    }

    // ─── Recording Control FFI Tests ────────────────────────────────────────

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
