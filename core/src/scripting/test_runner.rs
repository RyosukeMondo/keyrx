//! Test discovery and runner for Rhai scripts.
//!
//! This module provides test discovery and execution for Rhai scripts:
//! - Discovers test functions by `test_` prefix (Rhai doesn't support attributes)
//! - Runs each test in an isolated context
//! - Collects structured TestResult with pass/fail status, messages, and timing

use super::runtime::RhaiRuntime;
use super::test_harness::{reset_test_context, TestHarness};
use crate::traits::ScriptRuntime;
use rhai::AST;
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

/// Discovered test function with metadata.
#[derive(Debug, Clone)]
pub struct DiscoveredTest {
    /// Function name.
    pub name: String,
    /// Line number where the function is defined.
    pub line_number: Option<u32>,
}

/// Discover test functions in a Rhai AST.
///
/// Finds all functions with names starting with `test_` prefix.
/// Rhai doesn't support attributes like Rust's `#[test]`, so we use
/// naming conventions instead.
///
/// # Arguments
/// * `ast` - The compiled Rhai AST to search
///
/// # Returns
/// A vector of discovered test names.
pub fn discover_tests(ast: &AST) -> Vec<DiscoveredTest> {
    let mut tests = Vec::new();

    for fn_def in ast.iter_functions() {
        if fn_def.name.starts_with("test_") {
            tests.push(DiscoveredTest {
                name: fn_def.name.to_string(),
                // ScriptFnMetadata doesn't expose line numbers, so we can't get them here.
                // Line numbers would require access to the internal ScriptFnDef structure.
                line_number: None,
            });
        }
    }

    // Sort by name for consistent ordering (line numbers not available from metadata)
    tests.sort_by(|a, b| a.name.cmp(&b.name));

    tracing::debug!(
        service = "keyrx",
        event = "tests_discovered",
        component = "test_runner",
        count = tests.len(),
        "Discovered {} test functions",
        tests.len()
    );

    tests
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
        let filtered: Vec<_> = tests
            .iter()
            .filter(|t| matches_filter(&t.name, filter))
            .cloned()
            .collect();

        tracing::debug!(
            service = "keyrx",
            event = "tests_filtered",
            component = "test_runner",
            filter = filter,
            matched = filtered.len(),
            total = tests.len(),
            "Filtered tests: {}/{} match pattern",
            filtered.len(),
            tests.len()
        );

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

/// Check if a test name matches a filter pattern.
///
/// Supports basic glob-style matching with `*` as wildcard.
fn matches_filter(name: &str, pattern: &str) -> bool {
    if pattern.is_empty() || pattern == "*" {
        return true;
    }

    // Handle patterns with wildcards
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();

        if parts.len() == 2 {
            // Simple prefix/suffix matching
            let prefix = parts[0];
            let suffix = parts[1];

            if prefix.is_empty() {
                // *suffix - match at end
                return name.ends_with(suffix);
            } else if suffix.is_empty() {
                // prefix* - match at start
                return name.starts_with(prefix);
            } else {
                // prefix*suffix - match both
                return name.starts_with(prefix) && name.ends_with(suffix);
            }
        }
        // For complex patterns, fall back to contains check on parts
        return parts.iter().all(|p| p.is_empty() || name.contains(p));
    }

    // Exact match
    name == pattern
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_matches_filter_exact() {
        assert!(matches_filter("test_foo", "test_foo"));
        assert!(!matches_filter("test_foo", "test_bar"));
    }

    #[test]
    fn test_matches_filter_wildcard_suffix() {
        assert!(matches_filter("test_capslock_remap", "test_capslock*"));
        assert!(matches_filter("test_capslock", "test_capslock*"));
        assert!(!matches_filter("test_layer", "test_capslock*"));
    }

    #[test]
    fn test_matches_filter_wildcard_prefix() {
        assert!(matches_filter("test_something_remap", "*remap"));
        assert!(matches_filter("remap", "*remap"));
        assert!(!matches_filter("test_layer", "*remap"));
    }

    #[test]
    fn test_matches_filter_wildcard_both() {
        assert!(matches_filter("test_capslock_remap", "*capslock*"));
        assert!(matches_filter("capslock", "*capslock*"));
        assert!(!matches_filter("test_layer", "*capslock*"));
    }

    #[test]
    fn test_matches_filter_empty_or_star() {
        assert!(matches_filter("anything", ""));
        assert!(matches_filter("anything", "*"));
    }

    #[test]
    fn discover_tests_finds_test_prefix() {
        let mut runtime = RhaiRuntime::new().expect("runtime creation");

        // Load a script with test functions
        let script = r#"
            fn test_alpha() { }
            fn test_beta() { }
            fn helper_function() { }
            fn test_gamma() { }
        "#;
        runtime.execute(script).expect("script execution");

        // Get AST for discovery - need to compile it
        let ast = rhai::Engine::new().compile(script).expect("compilation");
        let tests = discover_tests(&ast);

        assert_eq!(tests.len(), 3);
        assert!(tests.iter().any(|t| t.name == "test_alpha"));
        assert!(tests.iter().any(|t| t.name == "test_beta"));
        assert!(tests.iter().any(|t| t.name == "test_gamma"));
        assert!(!tests.iter().any(|t| t.name == "helper_function"));
    }

    #[test]
    fn discover_tests_empty_script() {
        let script = r#"
            fn helper() { }
            let x = 42;
        "#;
        let ast = rhai::Engine::new().compile(script).expect("compilation");
        let tests = discover_tests(&ast);

        assert!(tests.is_empty());
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
    fn run_filtered_matches_pattern() {
        let tests = vec![
            DiscoveredTest {
                name: "test_capslock_remap".to_string(),
                line_number: Some(1),
            },
            DiscoveredTest {
                name: "test_capslock_block".to_string(),
                line_number: Some(2),
            },
            DiscoveredTest {
                name: "test_layer_push".to_string(),
                line_number: Some(3),
            },
        ];

        let filtered: Vec<_> = tests
            .iter()
            .filter(|t| matches_filter(&t.name, "test_capslock*"))
            .collect();

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].name, "test_capslock_remap");
        assert_eq!(filtered[1].name, "test_capslock_block");
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
    fn discovered_test_struct_clone() {
        let test = DiscoveredTest {
            name: "test_clone".to_string(),
            line_number: Some(42),
        };
        let cloned = test.clone();
        assert_eq!(cloned.name, "test_clone");
        assert_eq!(cloned.line_number, Some(42));
    }

    #[test]
    fn discover_tests_sorted_by_name() {
        let script = r#"
            fn test_zebra() { }
            fn test_alpha() { }
            fn test_middle() { }
        "#;
        let ast = rhai::Engine::new().compile(script).expect("compilation");
        let tests = discover_tests(&ast);

        assert_eq!(tests.len(), 3);
        // Should be sorted alphabetically
        assert_eq!(tests[0].name, "test_alpha");
        assert_eq!(tests[1].name, "test_middle");
        assert_eq!(tests[2].name, "test_zebra");
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
    fn run_filtered_with_no_matches() {
        let tests = vec![
            DiscoveredTest {
                name: "test_foo".to_string(),
                line_number: None,
            },
            DiscoveredTest {
                name: "test_bar".to_string(),
                line_number: None,
            },
        ];

        let filtered: Vec<_> = tests
            .iter()
            .filter(|t| matches_filter(&t.name, "test_nonexistent*"))
            .collect();

        assert!(filtered.is_empty());
    }

    #[test]
    fn matches_filter_complex_pattern() {
        // Multiple wildcards (simplified handling)
        assert!(matches_filter("test_foo_bar_baz", "*foo*baz*"));
        assert!(matches_filter("test_foo_bar_baz", "*bar*"));
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
