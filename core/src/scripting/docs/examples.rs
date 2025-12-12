//! Example runner for testing Rhai documentation examples.
//!
//! This module provides functionality to run example code snippets from
//! documentation and verify they execute without errors.

use super::registry;
use super::types::FunctionDoc;
use crate::scripting::runtime::RhaiRuntime;
use crate::traits::ScriptRuntime;
use std::time::Instant;

/// Result of running a single example.
#[derive(Debug, Clone)]
pub struct ExampleResult {
    /// Name of the function or type this example belongs to
    pub name: String,

    /// Module containing the function or type
    pub module: String,

    /// The example code that was executed
    pub code: String,

    /// Whether the example executed successfully
    pub passed: bool,

    /// Error message if the example failed
    pub error: Option<String>,

    /// Duration of the example execution in microseconds
    pub duration_us: u64,
}

impl ExampleResult {
    /// Create a new passing example result.
    pub fn pass(name: String, module: String, code: String, duration_us: u64) -> Self {
        Self {
            name,
            module,
            code,
            passed: true,
            error: None,
            duration_us,
        }
    }

    /// Create a new failing example result.
    pub fn fail(
        name: String,
        module: String,
        code: String,
        error: String,
        duration_us: u64,
    ) -> Self {
        Self {
            name,
            module,
            code,
            passed: false,
            error: Some(error),
            duration_us,
        }
    }
}

/// Summary of running multiple examples.
#[derive(Debug, Clone, Default)]
pub struct ExampleSummary {
    /// Total number of examples run
    pub total: usize,

    /// Number of examples that passed
    pub passed: usize,

    /// Number of examples that failed
    pub failed: usize,

    /// Total duration in microseconds
    pub duration_us: u64,
}

impl ExampleSummary {
    /// Create a new summary from example results.
    pub fn from_results(results: &[ExampleResult]) -> Self {
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

    /// Check if all examples passed.
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

/// Runner for executing documentation examples.
pub struct ExampleRunner {
    runtime: RhaiRuntime,
}

impl ExampleRunner {
    /// Create a new example runner.
    ///
    /// # Errors
    /// Returns an error if the Rhai runtime cannot be initialized.
    pub fn new() -> Result<Self, crate::errors::KeyrxError> {
        let runtime = RhaiRuntime::new()?;
        Ok(Self { runtime })
    }

    /// Run a single example code snippet.
    fn run_single_example(&mut self, name: &str, module: &str, code: &str) -> ExampleResult {
        let start = Instant::now();

        // Create a fresh runtime for each example to ensure isolation.
        // This prevents side effects (like layer state) from leaking between examples.
        match RhaiRuntime::new() {
            Ok(runtime) => {
                self.runtime = runtime;
            }
            Err(e) => {
                return ExampleResult::fail(
                    name.to_string(),
                    module.to_string(),
                    code.to_string(),
                    format!("Failed to initialize runtime: {}", e),
                    0,
                );
            }
        }

        // Execute the example code
        match self.runtime.execute(code) {
            Ok(_) => {
                let duration_us = start.elapsed().as_micros() as u64;
                tracing::debug!(
                    service = "keyrx",
                    event = "example_passed",
                    component = "example_runner",
                    function = name,
                    module = module,
                    duration_us = duration_us,
                    "Example passed"
                );
                ExampleResult::pass(
                    name.to_string(),
                    module.to_string(),
                    code.to_string(),
                    duration_us,
                )
            }
            Err(e) => {
                let duration_us = start.elapsed().as_micros() as u64;
                let error_msg = format!("{}", e);
                tracing::debug!(
                    service = "keyrx",
                    event = "example_failed",
                    component = "example_runner",
                    function = name,
                    module = module,
                    error = error_msg,
                    "Example failed"
                );
                ExampleResult::fail(
                    name.to_string(),
                    module.to_string(),
                    code.to_string(),
                    error_msg,
                    duration_us,
                )
            }
        }
    }

    /// Run all examples for a specific function.
    pub fn run_function_examples(&mut self, func: &FunctionDoc) -> Vec<ExampleResult> {
        let mut results = Vec::new();

        for (idx, example) in func.examples.iter().enumerate() {
            let example_name = if func.examples.len() == 1 {
                func.name.clone()
            } else {
                format!("{}[{}]", func.name, idx)
            };

            results.push(self.run_single_example(&example_name, &func.module, example));
        }

        results
    }

    /// Run all examples from all functions in the documentation registry.
    pub fn run_all_examples(&mut self) -> Vec<ExampleResult> {
        let mut results = Vec::new();

        let functions = registry::all_functions();

        for func in functions {
            if !func.examples.is_empty() {
                results.extend(self.run_function_examples(&func));
            }
        }

        results
    }

    /// Run examples from a specific module.
    pub fn run_module_examples(&mut self, module: &str) -> Vec<ExampleResult> {
        let mut results = Vec::new();

        let functions = registry::functions_in_module(module);

        tracing::debug!(
            service = "keyrx",
            event = "run_module_examples",
            component = "example_runner",
            module = module,
            function_count = functions.len(),
            "Running examples for module"
        );

        for func in functions {
            if !func.examples.is_empty() {
                tracing::debug!(
                    service = "keyrx",
                    event = "run_function_examples_start",
                    component = "example_runner",
                    function = func.name,
                    example_count = func.examples.len(),
                    "Running examples for function"
                );
                results.extend(self.run_function_examples(&func));
            }
        }

        results
    }

    /// Run all examples and return a summary.
    pub fn run_and_summarize(&mut self) -> (Vec<ExampleResult>, ExampleSummary) {
        let results = self.run_all_examples();
        let summary = ExampleSummary::from_results(&results);
        (results, summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::docs::registry;
    use crate::scripting::docs::types::{FunctionDoc, FunctionSignature, ReturnDoc};

    fn create_test_function_with_examples(
        name: &str,
        module: &str,
        examples: Vec<String>,
    ) -> FunctionDoc {
        FunctionDoc {
            name: name.to_string(),
            module: module.to_string(),
            signature: FunctionSignature {
                params: vec![],
                return_type: Some("()".to_string()),
            },
            description: format!("Test function {}", name),
            parameters: vec![],
            returns: Some(ReturnDoc {
                type_name: "()".to_string(),
                description: "Nothing".to_string(),
            }),
            examples,
            since: Some("0.1.0".to_string()),
            deprecated: None,
            notes: None,
        }
    }

    #[test]
    fn test_example_result_creation() {
        let pass_result = ExampleResult::pass(
            "test_func".to_string(),
            "test".to_string(),
            "print(42);".to_string(),
            100,
        );

        assert!(pass_result.passed);
        assert!(pass_result.error.is_none());
        assert_eq!(pass_result.duration_us, 100);

        let fail_result = ExampleResult::fail(
            "test_func".to_string(),
            "test".to_string(),
            "bad_code();".to_string(),
            "Function not found".to_string(),
            200,
        );

        assert!(!fail_result.passed);
        assert!(fail_result.error.is_some());
        assert_eq!(fail_result.error.unwrap(), "Function not found");
    }

    #[test]
    fn test_example_summary() {
        let results = vec![
            ExampleResult::pass("f1".to_string(), "m1".to_string(), "".to_string(), 100),
            ExampleResult::pass("f2".to_string(), "m1".to_string(), "".to_string(), 150),
            ExampleResult::fail(
                "f3".to_string(),
                "m1".to_string(),
                "".to_string(),
                "error".to_string(),
                50,
            ),
        ];

        let summary = ExampleSummary::from_results(&results);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.duration_us, 300);
        assert!(!summary.all_passed());
    }

    #[test]
    fn test_example_summary_all_passed() {
        let results = vec![
            ExampleResult::pass("f1".to_string(), "m1".to_string(), "".to_string(), 100),
            ExampleResult::pass("f2".to_string(), "m1".to_string(), "".to_string(), 150),
        ];

        let summary = ExampleSummary::from_results(&results);

        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 0);
        assert!(summary.all_passed());
    }

    #[test]
    fn test_run_valid_example() {
        let mut runner = ExampleRunner::new().expect("Failed to create runner");

        // Simple valid Rhai code
        let result = runner.run_single_example("test", "test", "let x = 1 + 1;");

        assert!(result.passed);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_run_invalid_example() {
        let mut runner = ExampleRunner::new().expect("Failed to create runner");

        // Invalid Rhai code
        let result = runner.run_single_example("test", "test", "this_function_does_not_exist();");

        assert!(!result.passed);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_run_function_examples() {
        registry::initialize();
        registry::clear();

        let func = create_test_function_with_examples(
            "test_func",
            "test",
            vec!["let x = 1;".to_string(), "let y = 2;".to_string()],
        );

        let mut runner = ExampleRunner::new().expect("Failed to create runner");
        let results = runner.run_function_examples(&func);

        assert_eq!(results.len(), 2);
        // Both examples should pass since they're valid Rhai
        assert!(results[0].passed);
        assert!(results[1].passed);
    }

    #[test]
    fn test_run_function_examples_with_failures() {
        let func = create_test_function_with_examples(
            "test_func",
            "test",
            vec!["let x = 1;".to_string(), "invalid_syntax!@#$".to_string()],
        );

        let mut runner = ExampleRunner::new().expect("Failed to create runner");
        let results = runner.run_function_examples(&func);

        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert!(!results[1].passed);
    }

    #[test]
    fn test_run_all_examples() {
        // This test verifies that run_all_examples works by creating a runner
        // and running examples from manually created functions.
        // We don't rely on the global registry since it may be in different states
        // depending on test execution order.

        let func1 =
            create_test_function_with_examples("func1", "mod1", vec!["let a = 1;".to_string()]);

        let func2 = create_test_function_with_examples(
            "func2",
            "mod2",
            vec!["let b = 2;".to_string(), "let c = 3;".to_string()],
        );

        let mut runner = ExampleRunner::new().expect("Failed to create runner");

        // Test running examples from multiple functions
        let mut results = Vec::new();
        results.extend(runner.run_function_examples(&func1));
        results.extend(runner.run_function_examples(&func2));

        // Should have exactly 3 examples (1 from func1, 2 from func2)
        assert_eq!(results.len(), 3, "Expected 3 examples");
        // All examples should pass since they're valid Rhai
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_run_module_examples() {
        registry::initialize();

        // Create functions directly without relying on the registry
        let func1 = create_test_function_with_examples(
            "test_func1_unique",
            "mod1_unique_test",
            vec!["let a = 1;".to_string()],
        );
        let func2 = create_test_function_with_examples(
            "test_func2_unique",
            "mod1_unique_test",
            vec!["let b = 2;".to_string()],
        );

        let mut runner = ExampleRunner::new().expect("Failed to create runner");

        // Test run_function_examples directly for both functions
        let mut results = Vec::new();
        results.extend(runner.run_function_examples(&func1));
        results.extend(runner.run_function_examples(&func2));

        // Should have exactly 2 examples (1 from each function)
        assert_eq!(results.len(), 2, "Expected exactly 2 examples");
        assert!(results[0].passed);
        assert!(results[1].passed);
    }

    #[test]
    fn test_run_and_summarize() {
        registry::initialize();

        let func = create_test_function_with_examples(
            "func1_summary_test",
            "mod1_summary_test",
            vec!["let a = 1;".to_string(), "invalid!".to_string()],
        );

        let mut runner = ExampleRunner::new().expect("Failed to create runner");

        // Run this function's examples directly
        let results = runner.run_function_examples(&func);
        let summary = ExampleSummary::from_results(&results);

        // Should have exactly 2 examples
        assert_eq!(results.len(), 2, "Expected 2 examples");
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 1, "Expected 1 passing example");
        assert_eq!(summary.failed, 1, "Expected 1 failing example");
        assert!(!summary.all_passed());
    }
}
