//! Session management FFI exports.
//!
//! Functions for benchmarking and diagnostics.
#![allow(unsafe_code)]

use serde::Serialize;
use std::ffi::{c_char, CStr, CString};
use std::ptr;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::keycodes::KeyCode;
    use crate::engine::RemapAction;
    use crate::ffi::{keyrx_check_script, keyrx_eval, keyrx_free_string, keyrx_load_script};
    use crate::scripting::{clear_active_runtime, set_active_runtime, RhaiRuntime};
    use crate::traits::ScriptRuntime;
    use serial_test::serial;

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
}
