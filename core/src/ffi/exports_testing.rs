//! Test discovery and execution FFI exports.
//!
//! Functions for discovering tests, running tests, and simulating key sequences.
#![allow(unsafe_code)]

use crate::cli::commands::SimulateCommand;
use crate::cli::OutputFormat;
use crate::scripting::test_discovery::discover_tests;
use crate::scripting::test_runner::{TestRunner, TestSummary};
use crate::scripting::RhaiRuntime;
use crate::traits::ScriptRuntime;
use serde::{Deserialize, Serialize};
use std::ffi::{c_char, CStr, CString};
use std::ptr;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::keyrx_free_string;
    use std::ffi::CString;
    use std::ptr;

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
}
