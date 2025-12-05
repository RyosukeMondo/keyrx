//! Script Testing Integration Tests
//!
//! Tests verifying the script testing workflow:
//! - Test script writing and discovery
//! - Test runner execution
//! - Test result reporting

use keyrx_core::cli::commands::TestCommand;
use keyrx_core::cli::OutputFormat;
use keyrx_core::scripting::test_discovery::discover_tests;
use keyrx_core::scripting::test_runner::{TestRunner, TestSummary};
use keyrx_core::scripting::RhaiRuntime;
use keyrx_core::traits::ScriptRuntime;
use std::fs;
use tempfile::TempDir;

/// Test that a Rhai test script can be written, discovered, and executed.
#[test]
fn write_test_script_discover_and_run() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let script_path = temp_dir.path().join("test_remapping.rhai");

    // Write a test script with multiple test functions
    let script_content = r#"
        // Helper function (not a test)
        fn helper_add(a, b) {
            a + b
        }

        // Test: Simple arithmetic
        fn test_arithmetic() {
            let result = helper_add(2, 3);
            if result != 5 {
                throw "Expected 5, got " + result;
            }
        }

        // Test: String operations
        fn test_string_concat() {
            let s = "hello" + " world";
            if s != "hello world" {
                throw "String concat failed";
            }
        }

        // Test: Array operations
        fn test_array_push() {
            let arr = [];
            arr.push(1);
            arr.push(2);
            if arr.len() != 2 {
                throw "Array length should be 2";
            }
        }
    "#;
    fs::write(&script_path, script_content).expect("write script");

    // Compile for discovery
    let engine = rhai::Engine::new();
    let ast = engine.compile(script_content).expect("compile script");

    // Discover tests
    let tests = discover_tests(&ast);
    assert_eq!(tests.len(), 3, "Should discover 3 test functions");

    // Verify test names
    let test_names: Vec<&str> = tests.iter().map(|t| t.name.as_str()).collect();
    assert!(test_names.contains(&"test_arithmetic"));
    assert!(test_names.contains(&"test_string_concat"));
    assert!(test_names.contains(&"test_array_push"));
    assert!(
        !test_names.contains(&"helper_add"),
        "Helper should not be discovered"
    );

    // Create runtime and run tests
    let mut runtime = RhaiRuntime::new().expect("create runtime");
    runtime
        .load_file(script_path.to_str().unwrap())
        .expect("load script");

    let runner = TestRunner::new();
    let results = runner.run_tests(&mut runtime, &tests);

    // Verify all tests pass
    let summary = TestSummary::from_results(&results);
    assert_eq!(summary.total, 3);
    assert_eq!(summary.passed, 3);
    assert_eq!(summary.failed, 0);
    assert!(summary.all_passed());
}

/// Test that failing tests are properly detected and reported.
#[test]
fn test_script_with_failures_detected() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let script_path = temp_dir.path().join("failing_tests.rhai");

    let script_content = r#"
        fn test_passing() {
            let x = 1 + 1;
        }

        fn test_failing() {
            throw "This test intentionally fails";
        }

        fn test_another_pass() {
            let arr = [1, 2, 3];
        }
    "#;
    fs::write(&script_path, script_content).expect("write script");

    let engine = rhai::Engine::new();
    let ast = engine.compile(script_content).expect("compile");
    let tests = discover_tests(&ast);

    let mut runtime = RhaiRuntime::new().expect("create runtime");
    runtime
        .load_file(script_path.to_str().unwrap())
        .expect("load script");

    let runner = TestRunner::new();
    let results = runner.run_tests(&mut runtime, &tests);

    let summary = TestSummary::from_results(&results);
    assert_eq!(summary.total, 3);
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 1);
    assert!(!summary.all_passed());

    // Verify the failing test has proper error message
    let failed = results
        .iter()
        .find(|r| !r.passed)
        .expect("find failed test");
    assert_eq!(failed.name, "test_failing");
    assert!(failed.message.contains("intentionally fails"));
}

/// Test the TestCommand CLI interface end-to-end.
#[test]
fn test_command_end_to_end() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let script_path = temp_dir.path().join("cli_test.rhai");

    let script_content = r#"
        fn test_simple() {
            let x = 42;
        }
    "#;
    fs::write(&script_path, script_content).expect("write script");

    let cmd = TestCommand::new(script_path, OutputFormat::Human);
    let exit_code = cmd.run().expect("run test command");

    assert_eq!(exit_code, 0, "Exit code should be 0 for passing tests");
}

/// Test error handling for missing test files.
#[test]
fn error_handling_missing_file() {
    let cmd = TestCommand::new("/nonexistent/script.rhai".into(), OutputFormat::Human);
    let result = cmd.run();
    // The test command returns a Result<i32, _> so we check if it's an error
    // or if it returns a non-zero exit code
    match result {
        Err(_) => {} // Expected error
        Ok(code) => assert_ne!(code, 0, "Should return non-zero exit code for missing file"),
    }
}

/// Test test filtering functionality.
#[test]
fn test_filtering_integration() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let script_path = temp_dir.path().join("filtered.rhai");

    let script_content = r#"
        fn test_capslock_basic() { }
        fn test_capslock_advanced() { }
        fn test_layer_push() { }
        fn test_layer_pop() { }
        fn test_modifier_shift() { }
    "#;
    fs::write(&script_path, script_content).expect("write");

    // Test with capslock filter
    let cmd = TestCommand::new(script_path.clone(), OutputFormat::Human)
        .with_filter(Some("test_capslock*".to_string()));
    let result = cmd.run().expect("run filtered");
    assert_eq!(result, 0);

    // Test with layer filter
    let cmd2 = TestCommand::new(script_path, OutputFormat::Human)
        .with_filter(Some("test_layer*".to_string()));
    let result2 = cmd2.run().expect("run filtered 2");
    assert_eq!(result2, 0);
}
