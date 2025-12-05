//! Test command for running Rhai script tests.

use crate::cli::{
    Command, CommandContext, CommandError, CommandResult, ExitCode, OutputFormat, OutputWriter,
};
use crate::error::KeyRxError;

/// Exit codes for test command (re-exported from config).
pub mod exit_codes {
    pub use crate::config::exit_codes::{ASSERTION_FAIL, ERROR, PASS, TIMEOUT};
}
use crate::scripting::test_discovery::discover_tests;
use crate::scripting::test_runner::{TestResult, TestRunner, TestSummary};
use crate::scripting::RhaiRuntime;
use crate::traits::ScriptRuntime;
use anyhow::{Context, Result};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

/// Run tests in a Rhai script.
pub struct TestCommand {
    pub script_path: PathBuf,
    pub filter: Option<String>,
    pub watch: bool,
    pub output: OutputWriter,
}

/// JSON output structure for test results.
#[derive(Serialize)]
struct TestResultJson {
    name: String,
    passed: bool,
    message: String,
    duration_us: u64,
    line_number: Option<u32>,
}

impl From<&TestResult> for TestResultJson {
    fn from(r: &TestResult) -> Self {
        Self {
            name: r.name.clone(),
            passed: r.passed,
            message: r.message.clone(),
            duration_us: r.duration_us,
            line_number: r.line_number,
        }
    }
}

/// JSON output structure for test summary.
#[derive(Serialize)]
struct TestSummaryJson {
    total: usize,
    passed: usize,
    failed: usize,
    duration_us: u64,
    results: Vec<TestResultJson>,
}

impl TestCommand {
    pub fn new(script_path: PathBuf, format: OutputFormat) -> Self {
        Self {
            script_path,
            filter: None,
            watch: false,
            output: OutputWriter::new(format),
        }
    }

    pub fn with_filter(mut self, filter: Option<String>) -> Self {
        self.filter = filter;
        self
    }

    pub fn with_watch(mut self, watch: bool) -> Self {
        self.watch = watch;
        self
    }

    /// Run the test command.
    ///
    /// Returns the exit code: 0=pass, 1=error, 2=assertion fail, 3=timeout.
    pub fn run(&self) -> Result<i32> {
        if self.watch {
            self.run_watch_mode()
        } else {
            self.run_once()
        }
    }

    /// Execute the test command with CommandResult.
    ///
    /// Returns a CommandResult indicating test success or failure with pass/fail counts.
    pub fn execute_tests(&self) -> CommandResult<()> {
        match self.run_once_internal() {
            Ok((passed, failed)) => {
                if failed > 0 {
                    CommandResult::failure(
                        ExitCode::AssertionFailed,
                        CommandError::test_failure(
                            format!("{} test(s) failed", failed),
                            passed,
                            failed,
                        )
                        .to_string(),
                    )
                } else {
                    CommandResult::success_with_message(
                        (),
                        format!("All {} test(s) passed", passed),
                    )
                }
            }
            Err(e) => CommandResult::failure(ExitCode::GeneralError, e.to_string()),
        }
    }

    fn run_once(&self) -> Result<i32> {
        match self.run_once_internal() {
            Ok((_, failed)) => Ok(if failed > 0 { 2 } else { 0 }),
            Err(_) => Ok(1),
        }
    }

    fn run_once_internal(&self) -> Result<(usize, usize)> {
        // Validate script path exists
        if !self.script_path.exists() {
            return Err(KeyRxError::InvalidPath {
                path: self.script_path.display().to_string(),
                reason: "file not found".to_string(),
            }
            .into());
        }

        // Read and compile the script
        let script = std::fs::read_to_string(&self.script_path)
            .with_context(|| format!("Failed to read script: {}", self.script_path.display()))?;

        let engine = rhai::Engine::new();
        let ast = engine.compile(&script).map_err(|e| {
            let position = e.position();
            KeyRxError::ScriptCompileError {
                message: e.to_string(),
                line: position.line(),
                column: position.position(),
            }
        })?;

        // Discover tests
        let discovered = discover_tests(&ast);

        if discovered.is_empty() {
            self.output
                .warning("No test functions found (functions must start with 'test_')");
            return Ok((0, 0));
        }

        // Create runtime and load script
        let mut runtime = RhaiRuntime::new().context("Failed to create Rhai runtime")?;
        runtime
            .load_file(
                self.script_path
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid script path"))?,
            )
            .context("Failed to load script")?;

        // Run tests
        let runner = TestRunner::new();
        let results = match &self.filter {
            Some(pattern) => runner.run_filtered(&mut runtime, &discovered, pattern),
            None => runner.run_tests(&mut runtime, &discovered),
        };

        // Output results
        self.output_results(&results);

        // Return pass/fail counts
        let summary = TestSummary::from_results(&results);
        Ok((summary.passed, summary.failed))
    }

    fn run_watch_mode(&self) -> Result<i32> {
        // Initial run
        let mut last_exit_code = self.run_once()?;

        println!("\n[Watching for changes... Press Ctrl+C to exit]\n");

        // Set up file watcher
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() {
                        let _ = tx.send(());
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )
        .context("Failed to create file watcher")?;

        // Watch the script file's parent directory
        let watch_path = self
            .script_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
            .context("Failed to watch directory")?;

        // Watch loop
        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(()) => {
                    // Debounce: drain any additional events
                    while rx.try_recv().is_ok() {}

                    println!("\n[File changed, re-running tests...]\n");

                    match self.run_once_internal() {
                        Ok((_, failed)) => last_exit_code = if failed > 0 { 2 } else { 0 },
                        Err(e) => {
                            self.output.error(&format!("Test run failed: {e}"));
                            last_exit_code = 1;
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Continue watching
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        Ok(last_exit_code)
    }

    fn output_results(&self, results: &[TestResult]) {
        let summary = TestSummary::from_results(results);

        match self.output.format() {
            OutputFormat::Json => {
                let json_output = TestSummaryJson {
                    total: summary.total,
                    passed: summary.passed,
                    failed: summary.failed,
                    duration_us: summary.duration_us,
                    results: results.iter().map(TestResultJson::from).collect(),
                };
                let _ = self.output.data(&json_output);
            }
            _ => {
                self.output_human_results(results, &summary);
            }
        }
    }

    fn output_human_results(&self, results: &[TestResult], summary: &TestSummary) {
        // Header
        println!(
            "\nRunning {} tests from '{}'",
            results.len(),
            self.script_path.display()
        );
        println!("{}", "─".repeat(60));

        // Individual test results
        for result in results {
            let marker = if result.passed { "✓" } else { "✗" };

            println!("{} {} ({} µs)", marker, result.name, result.duration_us);

            if !result.passed {
                println!("    └─ {}", result.message);
            }
        }

        // Summary
        println!("{}", "─".repeat(60));
        if summary.all_passed() {
            println!(
                "All {} tests passed in {} µs",
                summary.total, summary.duration_us
            );
        } else {
            println!(
                "{}/{} tests passed, {} failed ({} µs)",
                summary.passed, summary.total, summary.failed, summary.duration_us
            );
        }
    }
}

impl Command for TestCommand {
    fn name(&self) -> &str {
        "test"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        self.execute_tests()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_command_no_tests_found() {
        let mut file = NamedTempFile::new().expect("create temp file");
        writeln!(file, "fn helper() {{ 42 }}").expect("write file");

        let cmd = TestCommand::new(file.path().to_path_buf(), OutputFormat::Human);
        let result = cmd.execute_tests();

        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }

    #[test]
    fn test_command_passing_test() {
        let mut file = NamedTempFile::with_suffix(".rhai").expect("create temp file");
        writeln!(file, "fn test_simple() {{ let x = 1 + 1; }}").expect("write file");

        let cmd = TestCommand::new(file.path().to_path_buf(), OutputFormat::Human);
        let result = cmd.execute_tests();

        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }

    #[test]
    fn test_command_failing_test() {
        let mut file = NamedTempFile::with_suffix(".rhai").expect("create temp file");
        writeln!(file, "fn test_fail() {{ throw \"expected failure\"; }}").expect("write file");

        let cmd = TestCommand::new(file.path().to_path_buf(), OutputFormat::Human);
        let result = cmd.execute_tests();

        assert!(result.is_failure());
        assert_eq!(result.exit_code(), ExitCode::AssertionFailed);
    }

    #[test]
    fn test_command_file_not_found() {
        let cmd = TestCommand::new(PathBuf::from("/nonexistent/path.rhai"), OutputFormat::Human);
        let result = cmd.execute_tests();

        assert!(result.is_failure());
        assert_eq!(result.exit_code(), ExitCode::GeneralError);
    }

    #[test]
    fn test_command_with_filter() {
        let mut file = NamedTempFile::with_suffix(".rhai").expect("create temp file");
        writeln!(
            file,
            r#"
            fn test_alpha() {{ let x = 1; }}
            fn test_beta() {{ let y = 2; }}
            fn test_gamma() {{ let z = 3; }}
        "#
        )
        .expect("write file");

        let cmd = TestCommand::new(file.path().to_path_buf(), OutputFormat::Human)
            .with_filter(Some("test_alpha*".to_string()));
        let result = cmd.execute_tests();

        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }

    #[test]
    fn test_command_json_output() {
        let mut file = NamedTempFile::with_suffix(".rhai").expect("create temp file");
        writeln!(file, "fn test_json() {{ let x = 42; }}").expect("write file");

        let cmd = TestCommand::new(file.path().to_path_buf(), OutputFormat::Json);
        let result = cmd.execute_tests();

        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }

    #[test]
    fn test_command_syntax_error() {
        let mut file = NamedTempFile::with_suffix(".rhai").expect("create temp file");
        writeln!(file, "fn test_bad {{ }}").expect("write file"); // Missing parens

        let cmd = TestCommand::new(file.path().to_path_buf(), OutputFormat::Human);
        let result = cmd.execute_tests();

        assert!(result.is_failure());
        assert_eq!(result.exit_code(), ExitCode::GeneralError);
    }
}
