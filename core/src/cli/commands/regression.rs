//! Regression testing command for golden session verification.
//!
//! Provides a command to replay all golden sessions and detect behavioral
//! regressions. Used in CI to block merges when regressions are detected.

use crate::cli::{OutputFormat, OutputWriter};
use crate::config::exit_codes::{ERROR, REGRESSION, SUCCESS};
use crate::uat::{GoldenSessionManager, GoldenVerifyResult};
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;

/// Exit codes for regression command (re-exported from config).
pub mod exit_codes {
    pub use crate::config::exit_codes::{ERROR, REGRESSION, SUCCESS};
}

/// Result of a single session verification.
#[derive(Debug, Clone, Serialize)]
pub struct SessionResult {
    /// Session name.
    pub name: String,
    /// Whether verification passed.
    pub passed: bool,
    /// Duration of verification in microseconds.
    pub duration_us: u64,
    /// Number of differences found (0 if passed).
    pub difference_count: usize,
    /// Error message if verification failed for non-regression reasons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Summary of all regression tests.
#[derive(Debug, Clone, Serialize)]
pub struct RegressionSummary {
    /// Total number of golden sessions.
    pub total: usize,
    /// Number of sessions that passed verification.
    pub passed: usize,
    /// Number of sessions with regressions.
    pub regressed: usize,
    /// Number of sessions that couldn't be verified (errors).
    pub errors: usize,
    /// Total duration of all verifications in microseconds.
    pub total_duration_us: u64,
    /// Average duration per session in microseconds.
    pub avg_duration_us: u64,
    /// Results for each session.
    pub sessions: Vec<SessionResult>,
}

impl RegressionSummary {
    fn new() -> Self {
        Self {
            total: 0,
            passed: 0,
            regressed: 0,
            errors: 0,
            total_duration_us: 0,
            avg_duration_us: 0,
            sessions: Vec::new(),
        }
    }

    fn add_result(&mut self, result: SessionResult) {
        self.total += 1;
        self.total_duration_us += result.duration_us;

        if result.error.is_some() {
            self.errors += 1;
        } else if result.passed {
            self.passed += 1;
        } else {
            self.regressed += 1;
        }

        self.sessions.push(result);
    }

    fn finalize(&mut self) {
        if self.total > 0 {
            self.avg_duration_us = self.total_duration_us / self.total as u64;
        }
    }
}

/// Regression testing command.
pub struct RegressionCommand {
    /// Update all golden sessions instead of just verifying.
    pub update_all: bool,
    /// Path to script for updating sessions.
    pub script: Option<PathBuf>,
    /// Custom golden directory.
    pub golden_dir: Option<PathBuf>,
    /// Output format (human or JSON).
    pub json: bool,
    /// Output writer.
    pub output: OutputWriter,
}

impl RegressionCommand {
    /// Create a new regression command.
    pub fn new(format: OutputFormat) -> Self {
        Self {
            update_all: false,
            script: None,
            golden_dir: None,
            json: matches!(format, OutputFormat::Json),
            output: OutputWriter::new(format),
        }
    }

    /// Set update-all mode.
    pub fn with_update_all(mut self, update_all: bool) -> Self {
        self.update_all = update_all;
        self
    }

    /// Set script path for updates.
    pub fn with_script(mut self, script: Option<PathBuf>) -> Self {
        self.script = script;
        self
    }

    /// Set custom golden directory.
    pub fn with_golden_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.golden_dir = dir;
        self
    }

    /// Enable JSON output.
    pub fn with_json(mut self, json: bool) -> Self {
        self.json = json;
        if json {
            self.output = OutputWriter::new(OutputFormat::Json);
        }
        self
    }

    /// Get the golden session manager.
    fn manager(&self) -> GoldenSessionManager {
        match &self.golden_dir {
            Some(dir) => GoldenSessionManager::with_dir(dir),
            None => GoldenSessionManager::new(),
        }
    }

    /// Run the regression command.
    ///
    /// Returns the exit code:
    /// - 0: All sessions passed verification
    /// - 1: Error (file not found, parse error, etc.)
    /// - 2: Regressions detected
    pub fn run(&self) -> Result<i32> {
        let manager = self.manager();

        // List all golden sessions
        let sessions = manager
            .list_sessions()
            .context("Failed to list golden sessions")?;

        if sessions.is_empty() {
            if self.json {
                let summary = RegressionSummary::new();
                self.output.data(&summary)?;
            } else {
                self.output.warning("No golden sessions found.");
                self.output.warning(&format!(
                    "  Golden directory: {}",
                    manager.golden_dir().display()
                ));
                self.output
                    .warning("  Use 'keyrx golden record' to create golden sessions.");
            }
            return Ok(SUCCESS);
        }

        if !self.json {
            self.output.success(&format!(
                "Running regression tests for {} golden session(s)...",
                sessions.len()
            ));
        }

        let mut summary = RegressionSummary::new();
        let mut failed_sessions: Vec<(String, GoldenVerifyResult)> = Vec::new();

        // Verify each session
        for session_name in &sessions {
            let start = Instant::now();
            let result = manager.verify(session_name);
            let duration_us = start.elapsed().as_micros() as u64;

            match result {
                Ok(verify_result) => {
                    let session_result = SessionResult {
                        name: session_name.clone(),
                        passed: verify_result.passed,
                        duration_us,
                        difference_count: verify_result.differences.len(),
                        error: None,
                    };

                    if !verify_result.passed {
                        failed_sessions.push((session_name.clone(), verify_result));
                    }

                    summary.add_result(session_result);
                }
                Err(e) => {
                    let session_result = SessionResult {
                        name: session_name.clone(),
                        passed: false,
                        duration_us,
                        difference_count: 0,
                        error: Some(e.to_string()),
                    };
                    summary.add_result(session_result);
                }
            }
        }

        summary.finalize();

        // Output results
        if self.json {
            self.output.data(&summary)?;
        } else {
            self.output_human_results(&summary, &failed_sessions);
        }

        // Determine exit code
        if summary.errors > 0 {
            Ok(ERROR)
        } else if summary.regressed > 0 {
            Ok(REGRESSION)
        } else {
            Ok(SUCCESS)
        }
    }

    /// Output human-readable results.
    fn output_human_results(
        &self,
        summary: &RegressionSummary,
        failed: &[(String, GoldenVerifyResult)],
    ) {
        println!();
        println!("{}", "═".repeat(60));
        println!("  Regression Test Results");
        println!("{}", "═".repeat(60));

        // Summary
        println!();
        println!(
            "  Sessions: {} total, {} passed, {} regressed, {} errors",
            summary.total, summary.passed, summary.regressed, summary.errors
        );
        println!(
            "  Duration: {} µs (avg {} µs/session)",
            summary.total_duration_us, summary.avg_duration_us
        );

        // Show passed sessions
        if summary.passed > 0 && summary.total <= 20 {
            println!();
            println!("  Passed:");
            for session in &summary.sessions {
                if session.passed && session.error.is_none() {
                    println!("    ✓ {} ({} µs)", session.name, session.duration_us);
                }
            }
        }

        // Show regressions with details
        if !failed.is_empty() {
            println!();
            println!("  Regressions Detected:");
            for (name, result) in failed {
                println!("    ✗ {} ({} differences)", name, result.differences.len());

                // Show first few differences
                for (i, diff) in result.differences.iter().take(3).enumerate() {
                    println!(
                        "      [{}] Event {}: {} - expected: '{}', actual: '{}'",
                        i + 1,
                        diff.event_index,
                        diff.diff_type,
                        diff.expected,
                        diff.actual
                    );
                }

                if result.differences.len() > 3 {
                    println!(
                        "      ... and {} more differences",
                        result.differences.len() - 3
                    );
                }
            }
        }

        // Show errors
        let errors: Vec<_> = summary
            .sessions
            .iter()
            .filter(|s| s.error.is_some())
            .collect();
        if !errors.is_empty() {
            println!();
            println!("  Errors:");
            for session in errors {
                if let Some(ref error) = session.error {
                    println!("    ✗ {}: {}", session.name, error);
                }
            }
        }

        // Final status
        println!();
        if summary.regressed > 0 || summary.errors > 0 {
            println!("  Status: FAILED");
            if summary.regressed > 0 {
                println!(
                    "  Hint: Use 'keyrx golden verify <name>' to see full diff for each session."
                );
                println!("  Hint: Use 'keyrx golden update <name> --confirm' to update if change is intentional.");
            }
        } else {
            println!("  Status: PASSED ✓");
        }

        println!("{}", "═".repeat(60));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_correct() {
        assert_eq!(exit_codes::SUCCESS, 0);
        assert_eq!(exit_codes::ERROR, 1);
        assert_eq!(exit_codes::REGRESSION, 2);
    }

    #[test]
    fn regression_command_new() {
        let cmd = RegressionCommand::new(OutputFormat::Human);
        assert!(!cmd.update_all);
        assert!(cmd.script.is_none());
        assert!(cmd.golden_dir.is_none());
        assert!(!cmd.json);
    }

    #[test]
    fn regression_command_builder_methods() {
        let cmd = RegressionCommand::new(OutputFormat::Human)
            .with_update_all(true)
            .with_script(Some(PathBuf::from("script.rhai")))
            .with_golden_dir(Some(PathBuf::from("/custom/golden")))
            .with_json(true);

        assert!(cmd.update_all);
        assert_eq!(cmd.script, Some(PathBuf::from("script.rhai")));
        assert_eq!(cmd.golden_dir, Some(PathBuf::from("/custom/golden")));
        assert!(cmd.json);
    }

    #[test]
    fn regression_summary_new() {
        let summary = RegressionSummary::new();
        assert_eq!(summary.total, 0);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.regressed, 0);
        assert_eq!(summary.errors, 0);
        assert_eq!(summary.total_duration_us, 0);
        assert!(summary.sessions.is_empty());
    }

    #[test]
    fn regression_summary_add_passed_result() {
        let mut summary = RegressionSummary::new();
        summary.add_result(SessionResult {
            name: "test".to_string(),
            passed: true,
            duration_us: 1000,
            difference_count: 0,
            error: None,
        });

        assert_eq!(summary.total, 1);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.regressed, 0);
        assert_eq!(summary.errors, 0);
        assert_eq!(summary.total_duration_us, 1000);
    }

    #[test]
    fn regression_summary_add_regressed_result() {
        let mut summary = RegressionSummary::new();
        summary.add_result(SessionResult {
            name: "test".to_string(),
            passed: false,
            duration_us: 1000,
            difference_count: 2,
            error: None,
        });

        assert_eq!(summary.total, 1);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.regressed, 1);
        assert_eq!(summary.errors, 0);
    }

    #[test]
    fn regression_summary_add_error_result() {
        let mut summary = RegressionSummary::new();
        summary.add_result(SessionResult {
            name: "test".to_string(),
            passed: false,
            duration_us: 100,
            difference_count: 0,
            error: Some("Parse error".to_string()),
        });

        assert_eq!(summary.total, 1);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.regressed, 0);
        assert_eq!(summary.errors, 1);
    }

    #[test]
    fn regression_summary_finalize_calculates_average() {
        let mut summary = RegressionSummary::new();
        summary.add_result(SessionResult {
            name: "test1".to_string(),
            passed: true,
            duration_us: 1000,
            difference_count: 0,
            error: None,
        });
        summary.add_result(SessionResult {
            name: "test2".to_string(),
            passed: true,
            duration_us: 2000,
            difference_count: 0,
            error: None,
        });
        summary.finalize();

        assert_eq!(summary.total, 2);
        assert_eq!(summary.total_duration_us, 3000);
        assert_eq!(summary.avg_duration_us, 1500);
    }

    #[test]
    fn regression_summary_finalize_handles_empty() {
        let mut summary = RegressionSummary::new();
        summary.finalize();

        assert_eq!(summary.avg_duration_us, 0);
    }

    #[test]
    fn manager_uses_default_dir() {
        let cmd = RegressionCommand::new(OutputFormat::Human);
        let manager = cmd.manager();
        assert_eq!(manager.golden_dir(), &PathBuf::from("tests/golden"));
    }

    #[test]
    fn manager_uses_custom_dir() {
        let cmd = RegressionCommand::new(OutputFormat::Human)
            .with_golden_dir(Some(PathBuf::from("/custom/golden")));
        let manager = cmd.manager();
        assert_eq!(manager.golden_dir(), &PathBuf::from("/custom/golden"));
    }

    #[test]
    fn run_with_empty_directory() {
        let cmd = RegressionCommand::new(OutputFormat::Human)
            .with_golden_dir(Some(PathBuf::from("/nonexistent/path")));
        let result = cmd.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), exit_codes::SUCCESS);
    }

    #[test]
    fn session_result_serializes() {
        let result = SessionResult {
            name: "test".to_string(),
            passed: true,
            duration_us: 1000,
            difference_count: 0,
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"passed\":true"));
        assert!(!json.contains("error")); // Should be skipped when None
    }

    #[test]
    fn session_result_with_error_serializes() {
        let result = SessionResult {
            name: "test".to_string(),
            passed: false,
            duration_us: 100,
            difference_count: 0,
            error: Some("Failed".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"error\":\"Failed\""));
    }

    #[test]
    fn regression_summary_serializes() {
        let mut summary = RegressionSummary::new();
        summary.add_result(SessionResult {
            name: "test".to_string(),
            passed: true,
            duration_us: 1000,
            difference_count: 0,
            error: None,
        });
        summary.finalize();

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"total\":1"));
        assert!(json.contains("\"passed\":1"));
        assert!(json.contains("\"sessions\":"));
    }
}
