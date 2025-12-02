//! Test runner for Rhai scripts.
//!
//! This module provides test execution for Rhai scripts:
//! - Runs each test in an isolated context
//! - Collects structured TestResult with pass/fail status, messages, and timing

use super::runtime::RhaiRuntime;
use super::test_discovery::{filter_tests, DiscoveredTest};
use super::test_harness::{reset_test_context, TestHarness};
use crate::traits::ScriptRuntime;
use std::time::Instant;

/// Result of running a single test function.
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Name of the test function (e.g., "test_capslock_remap").
    pub name: String,
    /// Whether the test passed.
    pub passed: bool,
    /// Message describing the result or failure reason.
    pub message: String,
    /// Duration of the test execution in microseconds.
    pub duration_us: u64,
    /// Line number where the test function is defined (if available).
    pub line_number: Option<u32>,
}

impl TestResult {
    /// Create a new passing test result.
    pub fn pass(name: String, duration_us: u64, line_number: Option<u32>) -> Self {
        Self {
            name,
            passed: true,
            message: "Test passed".to_string(),
            duration_us,
            line_number,
        }
    }

    /// Create a new failing test result.
    pub fn fail(name: String, message: String, duration_us: u64, line_number: Option<u32>) -> Self {
        Self {
            name,
            passed: false,
            message,
            duration_us,
            line_number,
        }
    }
}

/// Summary of running multiple tests.
#[derive(Debug, Clone, Default)]
pub struct TestSummary {
    /// Total number of tests run.
    pub total: usize,
    /// Number of tests that passed.
    pub passed: usize,
    /// Number of tests that failed.
    pub failed: usize,
    /// Total duration in microseconds.
    pub duration_us: u64,
}

impl TestSummary {
    /// Create a new summary from test results.
    pub fn from_results(results: &[TestResult]) -> Self {
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;
        let duration_us = results.iter().map(|r| r.duration_us).sum();

        Self {
            total: results.len(),
            passed,
            failed,
            duration_us,
        }
    }

    /// Check if all tests passed.
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

/// Test runner for executing Rhai test functions.
pub struct TestRunner {
    /// Test harness for registering test primitives.
    harness: TestHarness,
}

impl TestRunner {
    /// Create a new test runner.
    pub fn new() -> Self {
        Self {
            harness: TestHarness::new(),
        }
    }

    /// Run a single test function.
    ///
    /// Resets the test context before running and captures any failures.
    fn run_single_test(&self, runtime: &mut RhaiRuntime, test: &DiscoveredTest) -> TestResult {
        // Reset context for clean test isolation
        reset_test_context();
        self.harness.reset_context();

        let start = Instant::now();

        // Call the test function
        let result = runtime.call_hook(&test.name);

        let duration_us = start.elapsed().as_micros() as u64;

        match result {
            Ok(()) => {
                // Check if any assertions failed via the test context
                let ctx = self.harness.context_snapshot();
                if ctx.all_passed() {
                    tracing::debug!(
                        service = "keyrx",
                        event = "test_passed",
                        component = "test_runner",
                        test = test.name,
                        duration_us = duration_us,
                        "Test passed"
                    );
                    TestResult::pass(test.name.clone(), duration_us, test.line_number)
                } else {
                    // Some assertions failed
                    let failed_assertions: Vec<_> = ctx
                        .assertions
                        .iter()
                        .filter(|a| !a.passed)
                        .map(|a| a.message.clone())
                        .collect();
                    let message = failed_assertions.join("; ");

                    tracing::debug!(
                        service = "keyrx",
                        event = "test_failed_assertion",
                        component = "test_runner",
                        test = test.name,
                        failures = ?failed_assertions,
                        "Test failed with assertion failures"
                    );
                    TestResult::fail(test.name.clone(), message, duration_us, test.line_number)
                }
            }
            Err(e) => {
                // Test execution failed (script error or panic caught)
                let message = format!("Execution error: {}", e);

                tracing::debug!(
                    service = "keyrx",
                    event = "test_failed_error",
                    component = "test_runner",
                    test = test.name,
                    error = %e,
                    "Test failed with execution error"
                );
                TestResult::fail(test.name.clone(), message, duration_us, test.line_number)
            }
        }
    }

    /// Run all discovered tests.
    ///
    /// # Arguments
    /// * `runtime` - The Rhai runtime with the script loaded
    /// * `tests` - List of discovered tests to run
    ///
    /// # Returns
    /// A vector of test results.
    pub fn run_tests(
        &self,
        runtime: &mut RhaiRuntime,
        tests: &[DiscoveredTest],
    ) -> Vec<TestResult> {
        let mut results = Vec::with_capacity(tests.len());

        for test in tests {
            let result = self.run_single_test(runtime, test);
            results.push(result);
        }

        let summary = TestSummary::from_results(&results);
        tracing::info!(
            service = "keyrx",
            event = "test_run_complete",
            component = "test_runner",
            total = summary.total,
            passed = summary.passed,
            failed = summary.failed,
            duration_us = summary.duration_us,
            "Test run complete: {}/{} passed",
            summary.passed,
            summary.total
        );

        results
    }

    /// Run tests matching a filter pattern.
    ///
    /// The filter supports basic glob-style matching:
    /// - `*` matches any sequence of characters
    /// - `test_capslock*` matches all tests starting with `test_capslock`
    ///
    /// # Arguments
    /// * `runtime` - The Rhai runtime with the script loaded
    /// * `tests` - List of discovered tests
    /// * `filter` - Pattern to filter test names
    ///
    /// # Returns
    /// A vector of test results for matching tests.
    pub fn run_filtered(
        &self,
        runtime: &mut RhaiRuntime,
        tests: &[DiscoveredTest],
        filter: &str,
    ) -> Vec<TestResult> {
        let filtered = filter_tests(tests, filter);
        self.run_tests(runtime, &filtered)
    }

    /// Get a reference to the test harness.
    pub fn harness(&self) -> &TestHarness {
        &self.harness
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::test_discovery::discover_tests;
    use crate::scripting::RhaiRuntime;
    use crate::traits::ScriptRuntime;

    #[test]
    fn test_result_pass_creation() {
        let result = TestResult::pass("test_foo".to_string(), 1000, Some(5));
        assert!(result.passed);
        assert_eq!(result.name, "test_foo");
        assert_eq!(result.duration_us, 1000);
        assert_eq!(result.line_number, Some(5));
    }

    #[test]
    fn test_result_fail_creation() {
        let result = TestResult::fail(
            "test_bar".to_string(),
            "assertion failed".to_string(),
            500,
            Some(10),
        );
        assert!(!result.passed);
        assert_eq!(result.message, "assertion failed");
    }

    #[test]
    fn test_summary_from_results() {
        let results = vec![
            TestResult::pass("a".to_string(), 100, None),
            TestResult::pass("b".to_string(), 200, None),
            TestResult::fail("c".to_string(), "fail".to_string(), 150, None),
        ];
        let summary = TestSummary::from_results(&results);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.duration_us, 450);
        assert!(!summary.all_passed());
    }

    #[test]
    fn run_tests_passing() {
        let mut runtime = RhaiRuntime::new().expect("runtime creation");
        let runner = TestRunner::new();

        // Load a simple passing test script
        let script = r#"
            fn test_pass() {
                let x = 1 + 1;
            }
        "#;
        runtime.execute(script).expect("script execution");

        // Compile for discovery
        let ast = rhai::Engine::new().compile(script).expect("compilation");
        let tests = discover_tests(&ast);

        // Need to load the script properly for call_hook to work
        let temp_path = std::env::temp_dir().join("test_passing.rhai");
        std::fs::write(&temp_path, script).expect("write temp file");
        runtime
            .load_file(temp_path.to_str().unwrap())
            .expect("load file");

        let results = runner.run_tests(&mut runtime, &tests);

        assert_eq!(results.len(), 1);
        assert!(results[0].passed);

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn run_tests_failing() {
        let mut runtime = RhaiRuntime::new().expect("runtime creation");
        let runner = TestRunner::new();

        // Load a failing test script (throws an error)
        let script = r#"
            fn test_fail() {
                throw "intentional failure";
            }
        "#;

        let temp_path = std::env::temp_dir().join("test_failing.rhai");
        std::fs::write(&temp_path, script).expect("write temp file");
        runtime
            .load_file(temp_path.to_str().unwrap())
            .expect("load file");

        let ast = rhai::Engine::new().compile(script).expect("compilation");
        let tests = discover_tests(&ast);

        let results = runner.run_tests(&mut runtime, &tests);

        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert!(results[0].message.contains("Execution error"));

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_runner_default() {
        let runner = TestRunner::default();
        assert!(runner.harness().context_snapshot().inputs.is_empty());
    }

    #[test]
    fn test_summary_all_passed_true() {
        let results = vec![
            TestResult::pass("a".to_string(), 100, None),
            TestResult::pass("b".to_string(), 200, None),
            TestResult::pass("c".to_string(), 150, None),
        ];
        let summary = TestSummary::from_results(&results);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.passed, 3);
        assert_eq!(summary.failed, 0);
        assert!(summary.all_passed());
    }

    #[test]
    fn test_summary_empty_results() {
        let results: Vec<TestResult> = vec![];
        let summary = TestSummary::from_results(&results);

        assert_eq!(summary.total, 0);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 0);
        assert!(summary.all_passed()); // Empty means no failures
    }

    #[test]
    fn test_summary_default() {
        let summary = TestSummary::default();
        assert_eq!(summary.total, 0);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.duration_us, 0);
    }

    #[test]
    fn run_tests_multiple_tests_all_pass() {
        let mut runtime = RhaiRuntime::new().expect("runtime creation");
        let runner = TestRunner::new();

        let script = r#"
            fn test_one() {
                let x = 1;
            }
            fn test_two() {
                let y = 2;
            }
        "#;

        let temp_path = std::env::temp_dir().join("test_multiple_pass.rhai");
        std::fs::write(&temp_path, script).expect("write temp file");
        runtime
            .load_file(temp_path.to_str().unwrap())
            .expect("load file");

        let ast = rhai::Engine::new().compile(script).expect("compilation");
        let tests = discover_tests(&ast);

        let results = runner.run_tests(&mut runtime, &tests);

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.passed));

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn run_tests_mixed_pass_fail() {
        let mut runtime = RhaiRuntime::new().expect("runtime creation");
        let runner = TestRunner::new();

        let script = r#"
            fn test_pass() {
                let x = 1;
            }
            fn test_fail() {
                throw "failure";
            }
        "#;

        let temp_path = std::env::temp_dir().join("test_mixed.rhai");
        std::fs::write(&temp_path, script).expect("write temp file");
        runtime
            .load_file(temp_path.to_str().unwrap())
            .expect("load file");

        let ast = rhai::Engine::new().compile(script).expect("compilation");
        let tests = discover_tests(&ast);

        let results = runner.run_tests(&mut runtime, &tests);

        assert_eq!(results.len(), 2);
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.iter().filter(|r| !r.passed).count();
        assert_eq!(passed, 1);
        assert_eq!(failed, 1);

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_result_debug_output() {
        let result = TestResult::pass("test_debug".to_string(), 500, Some(10));
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("test_debug"));
        assert!(debug_str.contains("500"));
    }

    #[test]
    fn test_result_clone() {
        let result = TestResult::fail(
            "test_clone".to_string(),
            "cloned failure".to_string(),
            100,
            None,
        );
        let cloned = result.clone();
        assert_eq!(cloned.name, "test_clone");
        assert_eq!(cloned.message, "cloned failure");
        assert!(!cloned.passed);
    }

    #[test]
    fn test_harness_access_via_runner() {
        let runner = TestRunner::new();
        let harness = runner.harness();

        // Verify we can access the harness and its methods
        harness.reset_context();
        let ctx = harness.context_snapshot();
        assert!(ctx.inputs.is_empty());
    }
}
