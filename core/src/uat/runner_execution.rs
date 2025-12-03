//! UAT test execution functionality.
//!
//! This module handles executing individual UAT tests by:
//! - Reading and compiling Rhai test scripts
//! - Executing test functions
//! - Capturing results and timing

use std::fs;

use rhai::Engine;

use super::runner_types::{UatResult, UatTest};

/// Execute a single UAT test and return the result.
pub(crate) fn execute_test(test: &UatTest) -> UatResult {
    let start_time = std::time::Instant::now();

    tracing::debug!(
        service = "keyrx",
        event = "uat_test_start",
        component = "uat_runner",
        test_name = %test.name,
        test_file = %test.file,
        category = %test.category,
        "Executing UAT test"
    );

    // Read the test file
    let content = match fs::read_to_string(&test.file) {
        Ok(c) => c,
        Err(e) => {
            let duration_us = start_time.elapsed().as_micros() as u64;
            return UatResult {
                test: test.clone(),
                passed: false,
                duration_us,
                error: Some(format!("Failed to read test file: {}", e)),
            };
        }
    };

    // Create Rhai engine
    let engine = Engine::new();

    // Compile and run the test
    let result = (|| -> Result<(), String> {
        let ast = engine
            .compile(&content)
            .map_err(|e| format!("Compilation error: {}", e))?;

        // Run the script to define functions
        engine
            .run_ast(&ast)
            .map_err(|e| format!("Script error: {}", e))?;

        // Call the test function
        engine
            .call_fn::<()>(&mut rhai::Scope::new(), &ast, &test.name, ())
            .map_err(|e| format!("Test execution error: {}", e))?;

        Ok(())
    })();

    let duration_us = start_time.elapsed().as_micros() as u64;

    match result {
        Ok(()) => {
            tracing::debug!(
                service = "keyrx",
                event = "uat_test_pass",
                component = "uat_runner",
                test_name = %test.name,
                duration_us = duration_us,
                "UAT test passed"
            );
            UatResult {
                test: test.clone(),
                passed: true,
                duration_us,
                error: None,
            }
        }
        Err(e) => {
            tracing::debug!(
                service = "keyrx",
                event = "uat_test_fail",
                component = "uat_runner",
                test_name = %test.name,
                duration_us = duration_us,
                error = %e,
                "UAT test failed"
            );
            UatResult {
                test: test.clone(),
                passed: false,
                duration_us,
                error: Some(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uat::runner_types::Priority;
    use std::fs;
    use tempfile::TempDir;

    fn create_test(name: &str, file: &str) -> UatTest {
        UatTest {
            name: name.to_string(),
            file: file.to_string(),
            category: "test".to_string(),
            priority: Priority::P2,
            requirements: vec![],
            latency_threshold: None,
        }
    }

    #[test]
    fn execute_test_passes_for_valid_script() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
fn uat_passing() {
    let x = 1 + 1;
}
"#;
        fs::write(&test_file, script).unwrap();

        let test = create_test("uat_passing", test_file.to_str().unwrap());
        let result = execute_test(&test);

        assert!(result.passed);
        assert!(result.error.is_none());
        assert!(result.duration_us > 0);
    }

    #[test]
    fn execute_test_fails_for_throwing_script() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
fn uat_failing() {
    throw "Test failed intentionally";
}
"#;
        fs::write(&test_file, script).unwrap();

        let test = create_test("uat_failing", test_file.to_str().unwrap());
        let result = execute_test(&test);

        assert!(!result.passed);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Test failed intentionally"));
    }

    #[test]
    fn execute_test_fails_for_compilation_error() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
fn uat_syntax_error( {
    let x = 1;
}
"#;
        fs::write(&test_file, script).unwrap();

        let test = create_test("uat_syntax_error", test_file.to_str().unwrap());
        let result = execute_test(&test);

        assert!(!result.passed);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Compilation error"));
    }

    #[test]
    fn execute_test_fails_for_missing_file() {
        let test = create_test("uat_missing", "/nonexistent/path/test.rhai");
        let result = execute_test(&test);

        assert!(!result.passed);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Failed to read test file"));
    }

    #[test]
    fn execute_test_fails_for_missing_function() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
fn other_function() {
    let x = 1;
}
"#;
        fs::write(&test_file, script).unwrap();

        let test = create_test("uat_nonexistent", test_file.to_str().unwrap());
        let result = execute_test(&test);

        assert!(!result.passed);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Test execution error"));
    }
}
