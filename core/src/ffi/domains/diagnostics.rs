//! Diagnostics domain FFI implementation.
//!
//! Implements the FfiExportable trait for diagnostics and benchmarking.
//! Handles system diagnostics and performance benchmarking.
#![allow(unsafe_code)]

use crate::cli::commands::{BenchCommand, CheckStatus, DiagnosticCheck};
use crate::cli::OutputFormat;
use crate::ffi::context::FfiContext;
use crate::ffi::error::{FfiError, FfiResult};
use crate::ffi::traits::FfiExportable;
// use keyrx_ffi_macros::ffi_export; // TODO: Uncomment when exports_*.rs files are removed (task 20)
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Diagnostics domain FFI implementation.
pub struct DiagnosticsFfi;

impl FfiExportable for DiagnosticsFfi {
    const DOMAIN: &'static str = "diagnostics";

    fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
        if ctx.has_domain(Self::DOMAIN) {
            return Err(FfiError::invalid_input(
                "diagnostics domain already initialized",
            ));
        }

        // No persistent state needed for diagnostics domain
        Ok(())
    }

    fn cleanup(_ctx: &mut FfiContext) {
        // No cleanup needed
    }
}

/// Benchmark result for FFI JSON output.
#[derive(Clone, Serialize, Deserialize, Debug, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub struct BenchmarkResultJson {
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

/// Diagnostic check for FFI JSON output.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
struct DiagnosticCheckJson {
    name: String,
    status: String,
    details: String,
    remediation: Option<String>,
}

/// Diagnostics result for FFI JSON output.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub struct DiagnosticsResultJson {
    checks: Vec<DiagnosticCheckJson>,
    passed: usize,
    failed: usize,
    warned: usize,
}

/// Run latency benchmark on the engine.
///
/// Returns: `{minNs, maxNs, meanNs, p99Ns, iterations, hasWarning, warning?}`
#[allow(improper_ctypes_definitions)]
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn run_benchmark(iterations: u32, script_path: Option<&str>) -> FfiResult<BenchmarkResultJson> {
    let iterations = iterations as usize;
    if iterations == 0 {
        return Err(FfiError::invalid_input("iterations must be > 0"));
    }

    let script_path_opt = script_path.map(PathBuf::from);

    let cmd = BenchCommand::new(iterations, script_path_opt, OutputFormat::Json);

    // Use tokio runtime for async execution
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(|e| FfiError::internal(format!("Failed to create runtime: {}", e)))?;

    let bench_result = rt
        .block_on(cmd.execute())
        .map_err(|e| FfiError::internal(format!("Benchmark failed: {}", e)))?;

    Ok(BenchmarkResultJson {
        min_ns: bench_result.min_ns,
        max_ns: bench_result.max_ns,
        mean_ns: bench_result.mean_ns,
        p99_ns: bench_result.p99_ns,
        iterations: bench_result.iterations,
        has_warning: bench_result.warning.is_some(),
        warning: bench_result.warning,
    })
}

/// Run system diagnostics.
///
/// Returns: `{checks: [{name, status, details, remediation}], passed, failed, warned}`
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn run_doctor() -> FfiResult<DiagnosticsResultJson> {
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

    Ok(DiagnosticsResultJson {
        checks: json_checks,
        passed,
        failed,
        warned,
    })
}

#[cfg(target_os = "linux")]
fn run_linux_diagnostics(checks: &mut Vec<DiagnosticCheck>) {
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
fn run_windows_diagnostics(checks: &mut Vec<DiagnosticCheck>) {
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

    #[test]
    fn run_benchmark_basic() {
        let result = run_benchmark(100, None);
        assert!(result.is_ok());
        let bench = result.unwrap();

        assert!(bench.min_ns > 0);
        assert!(bench.max_ns >= bench.min_ns);
        assert!(bench.mean_ns >= bench.min_ns);
        assert_eq!(bench.iterations, 100);
    }

    #[test]
    fn run_benchmark_zero_iterations_returns_error() {
        let result = run_benchmark(0, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("iterations must be > 0"));
    }

    #[test]
    fn run_doctor_returns_diagnostics() {
        let result = run_doctor();
        assert!(result.is_ok());
        let diag = result.unwrap();

        // Check structure
        assert!(!diag.checks.is_empty());

        // Should have at least the Rhai Engine check
        assert_eq!(diag.checks[0].name, "Rhai Engine");
        assert_eq!(diag.checks[0].status, "pass");
        assert!(diag.checks[0].details.contains("Scripting engine"));

        // Counts should add up
        let total_checks = diag.checks.len();
        assert_eq!(diag.passed + diag.failed + diag.warned, total_checks);
    }

    #[test]
    fn run_doctor_check_structure() {
        let result = run_doctor();
        assert!(result.is_ok());
        let diag = result.unwrap();

        // Each check should have the required fields
        for check in &diag.checks {
            assert!(!check.name.is_empty());
            assert!(!check.status.is_empty());
            assert!(!check.details.is_empty());
            // remediation can be none or string

            // Status must be one of the valid values
            assert!(
                check.status == "pass" || check.status == "fail" || check.status == "warn",
                "invalid status: {}",
                check.status
            );
        }
    }
}
