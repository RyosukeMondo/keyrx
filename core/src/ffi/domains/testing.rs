//! Testing domain FFI implementation.
//!
//! Implements the FfiExportable trait for test discovery, execution, and simulation.
#![allow(unsafe_code)]

use crate::cli::commands::SimulateCommand;
use crate::cli::OutputFormat;
use crate::ffi::context::FfiContext;
use crate::ffi::error::{FfiError, FfiResult};
use crate::ffi::traits::FfiExportable;
use crate::scripting::test_discovery::discover_tests as discover_test_functions;
use crate::scripting::test_runner::{TestRunner, TestSummary};
use crate::scripting::RhaiRuntime;
use crate::traits::ScriptRuntime;
// use keyrx_ffi_macros::ffi_export; // TODO: Uncomment when exports_*.rs files are removed (task 20)
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Testing domain FFI implementation.
pub struct TestingFfi;

impl FfiExportable for TestingFfi {
    const DOMAIN: &'static str = "testing";

    fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
        if ctx.has_domain(Self::DOMAIN) {
            return Err(FfiError::invalid_input(
                "testing domain already initialized",
            ));
        }

        // No persistent state needed for testing domain
        Ok(())
    }

    fn cleanup(_ctx: &mut FfiContext) {
        // No cleanup needed
    }
}

/// Discovered test for FFI JSON output.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub struct DiscoveredTestJson {
    name: String,
    file: String,
    line: Option<u32>,
}

/// Test result for FFI JSON output.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
struct TestResultJson {
    name: String,
    passed: bool,
    error: Option<String>,
    #[serde(rename = "durationMs")]
    duration_ms: f64,
}

/// Test run result for FFI JSON output.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub struct TestRunResult {
    total: usize,
    passed: usize,
    failed: usize,
    #[serde(rename = "durationMs")]
    duration_ms: f64,
    results: Vec<TestResultJson>,
}

/// Key input for simulation FFI.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
struct SimKeyInput {
    code: String,
    #[serde(rename = "holdMs", default)]
    hold_ms: Option<u64>,
}

/// Simulation mapping result for FFI JSON output.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
struct SimMapping {
    input: String,
    output: String,
    decision: String,
}

/// Simulation result for FFI JSON output.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub struct SimFfiResult {
    mappings: Vec<SimMapping>,
    #[serde(rename = "activeLayers")]
    active_layers: Vec<String>,
    pending: Vec<String>,
}

/// Discover test functions in a Rhai script.
///
/// Returns JSON: `[{name, file, line}, ...]`
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn discover_tests(path: &str) -> FfiResult<Vec<DiscoveredTestJson>> {
    let script = std::fs::read_to_string(path)
        .map_err(|e| FfiError::not_found(format!("Failed to read file: {}", e)))?;

    let engine = rhai::Engine::new();
    let ast = engine
        .compile(&script)
        .map_err(|e| FfiError::invalid_input(format!("Compile error: {}", e)))?;

    let tests = discover_test_functions(&ast);
    let json_tests: Vec<DiscoveredTestJson> = tests
        .into_iter()
        .map(|t| DiscoveredTestJson {
            name: t.name,
            file: path.to_string(),
            line: t.line_number,
        })
        .collect();

    Ok(json_tests)
}

/// Run tests in a Rhai script with optional filter.
///
/// Returns JSON: `{total, passed, failed, durationMs, results: [{name, passed, error, durationMs}]}`
#[allow(improper_ctypes_definitions)]
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn run_tests(path: &str, filter: Option<&str>) -> FfiResult<TestRunResult> {
    // Read and compile script
    let script = std::fs::read_to_string(path)
        .map_err(|e| FfiError::not_found(format!("Failed to read file: {}", e)))?;

    let engine = rhai::Engine::new();
    let ast = engine
        .compile(&script)
        .map_err(|e| FfiError::invalid_input(format!("Compile error: {}", e)))?;

    // Discover tests
    let discovered = discover_test_functions(&ast);
    if discovered.is_empty() {
        return Ok(TestRunResult {
            total: 0,
            passed: 0,
            failed: 0,
            duration_ms: 0.0,
            results: vec![],
        });
    }

    // Create runtime and load script
    let mut runtime = RhaiRuntime::new()
        .map_err(|e| FfiError::internal(format!("Failed to create runtime: {}", e)))?;

    runtime
        .load_file(path)
        .map_err(|e| FfiError::internal(format!("Failed to load script: {}", e)))?;

    // Run tests
    let runner = TestRunner::new();
    let results = match filter {
        Some(pattern) if !pattern.is_empty() => {
            runner.run_filtered(&mut runtime, &discovered, pattern)
        }
        _ => runner.run_tests(&mut runtime, &discovered),
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

    Ok(TestRunResult {
        total: summary.total,
        passed: summary.passed,
        failed: summary.failed,
        duration_ms: summary.duration_us as f64 / 1000.0,
        results: json_results,
    })
}

/// Simulate key sequences through the engine.
///
/// # Arguments
/// * `keys_json` - JSON array of key inputs: `[{code: "A", holdMs: 100}, ...]`
/// * `script_path` - Optional path to Rhai script
/// * `combo_mode` - If true, keys are pressed simultaneously; otherwise sequentially
///
/// Returns JSON: `{mappings: [{input, output, decision}], activeLayers, pending}`
#[allow(improper_ctypes_definitions)]
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn simulate(
    keys_json: &str,
    script_path: Option<&str>,
    combo_mode: bool,
) -> FfiResult<SimFfiResult> {
    let keys: Vec<SimKeyInput> = serde_json::from_str(keys_json)
        .map_err(|e| FfiError::invalid_input(format!("Invalid keys JSON: {}", e)))?;

    if keys.is_empty() {
        return Ok(SimFfiResult {
            mappings: vec![],
            active_layers: vec!["base".to_string()],
            pending: vec![],
        });
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
    let script_path_opt = script_path.map(PathBuf::from);

    // Create and run simulation
    let cmd =
        SimulateCommand::new(input_str, script_path_opt, OutputFormat::Json).with_combo(combo_mode);

    // Use tokio runtime for async execution
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(|e| FfiError::internal(format!("Failed to create runtime: {}", e)))?;

    let output = rt
        .block_on(cmd.execute())
        .map_err(|e| FfiError::internal(format!("Simulation failed: {}", e)))?;

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

    let pending = vec![format!("pending_count: {}", output.pending_count)];

    Ok(SimFfiResult {
        mappings,
        active_layers: output.active_layers,
        pending,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn discover_tests_finds_test_functions() {
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

        let result = discover_tests(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        let tests = result.unwrap();
        assert_eq!(tests.len(), 2);
        assert!(tests.iter().any(|t| t.name == "test_alpha"));
        assert!(tests.iter().any(|t| t.name == "test_beta"));
    }

    #[test]
    fn discover_tests_empty_script() {
        let mut temp = NamedTempFile::with_suffix(".rhai").unwrap();
        writeln!(temp, "fn helper() {{ }}").unwrap();

        let result = discover_tests(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        let tests = result.unwrap();
        assert!(tests.is_empty());
    }

    #[test]
    fn run_tests_passing() {
        let mut temp = NamedTempFile::with_suffix(".rhai").unwrap();
        writeln!(temp, "fn test_pass() {{ let x = 1 + 1; }}").unwrap();

        let result = run_tests(temp.path().to_str().unwrap(), None);
        assert!(result.is_ok());
        let test_result = result.unwrap();
        assert_eq!(test_result.total, 1);
        assert_eq!(test_result.passed, 1);
        assert_eq!(test_result.failed, 0);
    }

    #[test]
    fn run_tests_with_filter() {
        let mut temp = NamedTempFile::with_suffix(".rhai").unwrap();
        writeln!(
            temp,
            r#"
            fn test_alpha() {{ let x = 1; }}
            fn test_beta() {{ let y = 2; }}
        "#
        )
        .unwrap();

        let result = run_tests(temp.path().to_str().unwrap(), Some("test_alpha*"));
        assert!(result.is_ok());
        let test_result = result.unwrap();
        // Only test_alpha should run due to filter
        assert_eq!(test_result.total, 1);
        assert_eq!(test_result.results[0].name, "test_alpha");
    }

    #[test]
    fn simulate_empty_keys() {
        let result = simulate("[]", None, false);
        assert!(result.is_ok());
        let sim_result = result.unwrap();
        assert!(sim_result.mappings.is_empty());
    }

    #[test]
    fn simulate_basic_key() {
        let result = simulate(r#"[{"code": "A"}]"#, None, false);
        assert!(result.is_ok());
        let sim_result = result.unwrap();
        // Without a script, key should pass through
        assert!(!sim_result.mappings.is_empty());
    }

    #[test]
    fn simulate_with_script() {
        let mut temp = NamedTempFile::with_suffix(".rhai").unwrap();
        writeln!(temp, r#"remap("A", "B");"#).unwrap();

        let result = simulate(
            r#"[{"code": "A"}]"#,
            Some(temp.path().to_str().unwrap()),
            false,
        );
        assert!(result.is_ok());
        let sim_result = result.unwrap();
        // Find the A key press result
        let has_a = sim_result.mappings.iter().any(|m| m.input == "A");
        assert!(has_a, "should have A input in mappings");
    }
}
