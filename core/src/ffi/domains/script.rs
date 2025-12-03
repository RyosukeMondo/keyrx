//! Script domain FFI implementation.
//!
//! Implements the FfiExportable trait for script management.
//! Handles script loading, validation, and evaluation.
#![allow(unsafe_code)]

use crate::ffi::context::FfiContext;
use crate::ffi::error::{FfiError, FfiResult};
use crate::ffi::traits::FfiExportable;
use crate::scripting::with_active_runtime;
use crate::traits::ScriptRuntime;
use keyrx_ffi_macros::ffi_export;
use serde::Serialize;
use std::path::Path;

/// Script domain FFI implementation.
pub struct ScriptFfi;

impl FfiExportable for ScriptFfi {
    const DOMAIN: &'static str = "script";

    fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
        if ctx.has_domain(Self::DOMAIN) {
            return Err(FfiError::invalid_input("script domain already initialized"));
        }

        // No persistent state needed for script domain
        Ok(())
    }

    fn cleanup(_ctx: &mut FfiContext) {
        // No cleanup needed
    }
}

/// Script validation error detail.
#[derive(Serialize)]
pub struct ValidationError {
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
}

/// Script validation result.
#[derive(Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

/// Load a script file.
///
/// # Arguments
/// * `path` - Path to the script file
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(FfiError)` if file not found, invalid UTF-8, syntax error, or engine not initialized
#[ffi_export]
pub fn load_script(path: &str) -> FfiResult<()> {
    let script_name = Path::new(path)
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
        runtime.load_file(path)?;
        runtime.run_script()
    }) {
        Some(Ok(())) => Ok(()),
        Some(Err(err)) => {
            tracing::error!(
                service = "keyrx",
                event = "ffi_load_script_error",
                component = "ffi_exports",
                script = script_name,
                error = %err,
                "Script syntax/execution error"
            );
            Err(FfiError::invalid_input(&format!("Script error: {}", err)))
        }
        None => {
            tracing::error!(
                service = "keyrx",
                event = "ffi_load_script_error",
                component = "ffi_exports",
                script = script_name,
                "Engine not initialized"
            );
            Err(FfiError::internal("Engine not initialized"))
        }
    }
}

/// Validate a Rhai script without executing it.
///
/// Returns: `{valid: bool, errors: [{line, column, message}]}`
#[ffi_export]
pub fn check_script(path: &str) -> FfiResult<ValidationResult> {
    let script = std::fs::read_to_string(path)
        .map_err(|e| FfiError::not_found(&format!("Failed to read file: {}", e)))?;

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

    Ok(result)
}

/// Evaluate a console command against the active runtime.
///
/// # Arguments
/// * `command` - Command string to evaluate
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(FfiError)` if engine not initialized or command fails
#[ffi_export]
pub fn eval(command: &str) -> FfiResult<()> {
    match with_active_runtime(|runtime| runtime.execute(command)) {
        Some(Ok(_)) => Ok(()),
        Some(Err(err)) => Err(FfiError::invalid_input(&format!("Eval error: {}", err))),
        None => Err(FfiError::internal("Engine not initialized")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::keycodes::KeyCode;
    use crate::engine::RemapAction;
    use crate::scripting::{clear_active_runtime, set_active_runtime, RhaiRuntime};
    use crate::traits::ScriptRuntime;
    use serial_test::serial;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[serial]
    fn load_script_returns_error_without_runtime() {
        clear_active_runtime();
        let result = load_script("script.rhai");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("Engine not initialized"));
    }

    #[test]
    #[serial]
    fn load_script_returns_error_for_missing_file() {
        clear_active_runtime();
        let mut runtime = RhaiRuntime::new().expect("runtime should initialize");
        set_active_runtime(&mut runtime);

        let result = load_script("/nonexistent/path/script.rhai");
        assert!(result.is_err());

        clear_active_runtime();
    }

    #[test]
    #[serial]
    fn load_script_loads_valid_script() {
        clear_active_runtime();
        let mut runtime = RhaiRuntime::new().expect("runtime should initialize");
        set_active_runtime(&mut runtime);

        // Create a temporary script file
        let mut temp_file = NamedTempFile::new().expect("temp file should create");
        writeln!(temp_file, r#"remap("A", "B");"#).expect("write should succeed");

        let result = load_script(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        // Verify the mapping was registered
        assert!(matches!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        ));

        clear_active_runtime();
    }

    #[test]
    fn check_script_valid_syntax() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "let x = 1 + 2;").unwrap();

        let result = check_script(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert_eq!(validation.valid, true);
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn check_script_invalid_syntax() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "let x = (1 + 2").unwrap(); // Missing closing paren

        let result = check_script(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert_eq!(validation.valid, false);
        assert!(!validation.errors.is_empty());
        assert!(!validation.errors[0].message.is_empty());
    }

    #[test]
    fn check_script_missing_file() {
        let result = check_script("/nonexistent/script.rhai");
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn eval_returns_error_when_runtime_missing() {
        clear_active_runtime();
        let result = eval(r#"remap("A","B");"#);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("Engine not initialized"));
    }

    #[test]
    #[serial]
    fn eval_executes_against_shared_runtime() {
        clear_active_runtime();
        let mut runtime = RhaiRuntime::new().expect("runtime should initialize");
        set_active_runtime(&mut runtime);

        let result = eval(r#"remap("A","B");"#);
        assert!(result.is_ok());

        assert!(matches!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        ));
        clear_active_runtime();
    }
}
