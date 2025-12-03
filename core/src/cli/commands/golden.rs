//! Golden session CLI commands.
//!
//! Provides commands for recording, verifying, and updating golden sessions
//! for regression testing.

use crate::cli::{OutputFormat, OutputWriter};
use crate::uat::{GoldenSessionError, GoldenSessionManager};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Exit codes for golden session commands.
pub mod exit_codes {
    /// Operation completed successfully.
    pub const SUCCESS: i32 = 0;
    /// General error (file not found, parse error, etc.).
    pub const ERROR: i32 = 1;
    /// Verification failed (outputs didn't match).
    pub const VERIFICATION_FAILED: i32 = 2;
    /// Confirmation required but not provided.
    pub const CONFIRMATION_REQUIRED: i32 = 3;
}

/// Golden session subcommand type.
#[derive(Debug, Clone)]
pub enum GoldenSubcommand {
    /// Record a new golden session.
    Record {
        /// Session name.
        name: String,
        /// Path to the script that generates events.
        script: PathBuf,
    },
    /// Verify an existing golden session.
    Verify {
        /// Session name to verify.
        name: String,
        /// Optional script to run for verification.
        script: Option<PathBuf>,
    },
    /// Update an existing golden session.
    Update {
        /// Session name to update.
        name: String,
        /// Path to the script that generates events.
        script: PathBuf,
        /// Confirmation flag (required for update).
        confirm: bool,
    },
    /// List all golden sessions.
    List,
}

/// Command for golden session operations.
pub struct GoldenCommand {
    /// The subcommand to execute.
    subcommand: GoldenSubcommand,
    /// Custom golden directory (optional).
    golden_dir: Option<PathBuf>,
    /// Output writer.
    output: OutputWriter,
}

impl GoldenCommand {
    /// Create a new golden command.
    pub fn new(subcommand: GoldenSubcommand, format: OutputFormat) -> Self {
        Self {
            subcommand,
            golden_dir: None,
            output: OutputWriter::new(format),
        }
    }

    /// Set a custom golden session directory.
    pub fn with_golden_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.golden_dir = dir;
        self
    }

    /// Get the golden session manager.
    fn manager(&self) -> GoldenSessionManager {
        match &self.golden_dir {
            Some(dir) => GoldenSessionManager::with_dir(dir),
            None => GoldenSessionManager::new(),
        }
    }

    /// Run the golden command.
    ///
    /// Returns the exit code:
    /// - 0: Success
    /// - 1: Error (file not found, parse error, etc.)
    /// - 2: Verification failed
    /// - 3: Confirmation required but not provided
    pub fn run(&self) -> Result<i32> {
        match &self.subcommand {
            GoldenSubcommand::Record { name, script } => self.run_record(name, script),
            GoldenSubcommand::Verify { name, script } => self.run_verify(name, script.as_ref()),
            GoldenSubcommand::Update {
                name,
                script,
                confirm,
            } => self.run_update(name, script, *confirm),
            GoldenSubcommand::List => self.run_list(),
        }
    }

    /// Record a new golden session.
    fn run_record(&self, name: &str, script: &Path) -> Result<i32> {
        let manager = self.manager();
        let script_str = script.to_string_lossy();

        self.output
            .success(&format!("Recording golden session '{}'...", name));

        match manager.record(name, &script_str) {
            Ok(result) => {
                self.output.success(&format!(
                    "Golden session '{}' recorded successfully",
                    result.session_name
                ));
                self.output
                    .success(&format!("  Path: {}", result.path.display()));
                self.output
                    .success(&format!("  Events: {}", result.event_count));
                self.output
                    .success(&format!("  Duration: {} µs", result.duration_us));
                Ok(exit_codes::SUCCESS)
            }
            Err(GoldenSessionError::InvalidName { name, reason }) => {
                self.output
                    .error(&format!("Invalid session name '{}': {}", name, reason));
                Ok(exit_codes::ERROR)
            }
            Err(GoldenSessionError::ScriptError(msg)) => {
                self.output.error(&format!("Script error: {}", msg));
                Ok(exit_codes::ERROR)
            }
            Err(e) => Err(e).context("Failed to record golden session"),
        }
    }

    /// Verify an existing golden session.
    fn run_verify(&self, name: &str, script: Option<&PathBuf>) -> Result<i32> {
        let manager = self.manager();

        self.output
            .success(&format!("Verifying golden session '{}'...", name));

        let result = match script {
            Some(script_path) => {
                let script_str = script_path.to_string_lossy();
                manager
                    .verify_with_script(name, &script_str)
                    .context("Failed to verify golden session")?
            }
            None => manager
                .verify(name)
                .context("Failed to verify golden session")?,
        };

        if result.passed {
            self.output.success(&format!(
                "Golden session '{}' verified successfully",
                result.session_name
            ));
            self.output
                .success(&format!("  Duration: {} µs", result.duration_us));
            Ok(exit_codes::SUCCESS)
        } else {
            self.output.error(&format!(
                "Golden session '{}' verification failed",
                result.session_name
            ));
            self.output.error(&format!(
                "  Differences found: {}",
                result.differences.len()
            ));

            // Show first few differences
            for (i, diff) in result.differences.iter().take(5).enumerate() {
                self.output.error(&format!(
                    "  [{}] Event {}: {} - expected: '{}', actual: '{}'",
                    i + 1,
                    diff.event_index,
                    diff.diff_type,
                    diff.expected,
                    diff.actual
                ));
            }

            if result.differences.len() > 5 {
                self.output.error(&format!(
                    "  ... and {} more differences",
                    result.differences.len() - 5
                ));
            }

            Ok(exit_codes::VERIFICATION_FAILED)
        }
    }

    /// Update an existing golden session.
    fn run_update(&self, name: &str, script: &Path, confirm: bool) -> Result<i32> {
        let manager = self.manager();
        let script_str = script.to_string_lossy();

        self.output
            .success(&format!("Updating golden session '{}'...", name));

        match manager.update(name, &script_str, confirm) {
            Ok(result) => {
                self.output.success(&format!(
                    "Golden session '{}' updated successfully",
                    result.session_name
                ));
                self.output
                    .success(&format!("  Path: {}", result.path.display()));
                self.output.success(&format!(
                    "  Events: {} (was {})",
                    result.event_count, result.previous_event_count
                ));
                self.output
                    .success(&format!("  Duration: {} µs", result.duration_us));
                Ok(exit_codes::SUCCESS)
            }
            Err(GoldenSessionError::ConfirmationRequired(session)) => {
                self.output.warning(&format!(
                    "Update requires confirmation. Session '{}' will be overwritten.",
                    session
                ));
                self.output
                    .warning("  Use --confirm flag to proceed with the update.");
                Ok(exit_codes::CONFIRMATION_REQUIRED)
            }
            Err(GoldenSessionError::NotFound(session)) => {
                self.output
                    .error(&format!("Golden session '{}' not found", session));
                self.output
                    .warning("  Use 'keyrx golden record' to create a new session first.");
                Ok(exit_codes::ERROR)
            }
            Err(GoldenSessionError::ScriptError(msg)) => {
                self.output.error(&format!("Script error: {}", msg));
                Ok(exit_codes::ERROR)
            }
            Err(e) => Err(e).context("Failed to update golden session"),
        }
    }

    /// List all golden sessions.
    fn run_list(&self) -> Result<i32> {
        let manager = self.manager();

        let sessions = manager
            .list_sessions()
            .context("Failed to list golden sessions")?;

        if sessions.is_empty() {
            self.output.warning("No golden sessions found.");
            self.output.warning(&format!(
                "  Golden directory: {}",
                manager.golden_dir().display()
            ));
        } else {
            self.output
                .success(&format!("Found {} golden session(s):", sessions.len()));
            for session in &sessions {
                self.output.success(&format!("  - {}", session));
            }
        }

        Ok(exit_codes::SUCCESS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_correct() {
        assert_eq!(exit_codes::SUCCESS, 0);
        assert_eq!(exit_codes::ERROR, 1);
        assert_eq!(exit_codes::VERIFICATION_FAILED, 2);
        assert_eq!(exit_codes::CONFIRMATION_REQUIRED, 3);
    }

    #[test]
    fn golden_command_new() {
        let cmd = GoldenCommand::new(GoldenSubcommand::List, OutputFormat::Human);
        assert!(cmd.golden_dir.is_none());
    }

    #[test]
    fn golden_command_with_golden_dir() {
        let cmd = GoldenCommand::new(GoldenSubcommand::List, OutputFormat::Human)
            .with_golden_dir(Some(PathBuf::from("/custom/path")));
        assert_eq!(cmd.golden_dir, Some(PathBuf::from("/custom/path")));
    }

    #[test]
    fn golden_subcommand_record() {
        let sub = GoldenSubcommand::Record {
            name: "test".to_string(),
            script: PathBuf::from("script.rhai"),
        };
        match sub {
            GoldenSubcommand::Record { name, script } => {
                assert_eq!(name, "test");
                assert_eq!(script, PathBuf::from("script.rhai"));
            }
            _ => panic!("Expected Record subcommand"),
        }
    }

    #[test]
    fn golden_subcommand_verify() {
        let sub = GoldenSubcommand::Verify {
            name: "test".to_string(),
            script: None,
        };
        match sub {
            GoldenSubcommand::Verify { name, script } => {
                assert_eq!(name, "test");
                assert!(script.is_none());
            }
            _ => panic!("Expected Verify subcommand"),
        }
    }

    #[test]
    fn golden_subcommand_verify_with_script() {
        let sub = GoldenSubcommand::Verify {
            name: "test".to_string(),
            script: Some(PathBuf::from("verify.rhai")),
        };
        match sub {
            GoldenSubcommand::Verify { name, script } => {
                assert_eq!(name, "test");
                assert_eq!(script, Some(PathBuf::from("verify.rhai")));
            }
            _ => panic!("Expected Verify subcommand"),
        }
    }

    #[test]
    fn golden_subcommand_update() {
        let sub = GoldenSubcommand::Update {
            name: "test".to_string(),
            script: PathBuf::from("update.rhai"),
            confirm: true,
        };
        match sub {
            GoldenSubcommand::Update {
                name,
                script,
                confirm,
            } => {
                assert_eq!(name, "test");
                assert_eq!(script, PathBuf::from("update.rhai"));
                assert!(confirm);
            }
            _ => panic!("Expected Update subcommand"),
        }
    }

    #[test]
    fn golden_subcommand_list() {
        let sub = GoldenSubcommand::List;
        assert!(matches!(sub, GoldenSubcommand::List));
    }

    #[test]
    fn manager_uses_default_dir() {
        let cmd = GoldenCommand::new(GoldenSubcommand::List, OutputFormat::Human);
        let manager = cmd.manager();
        assert_eq!(manager.golden_dir(), &PathBuf::from("tests/golden"));
    }

    #[test]
    fn manager_uses_custom_dir() {
        let cmd = GoldenCommand::new(GoldenSubcommand::List, OutputFormat::Human)
            .with_golden_dir(Some(PathBuf::from("/custom/golden")));
        let manager = cmd.manager();
        assert_eq!(manager.golden_dir(), &PathBuf::from("/custom/golden"));
    }

    #[test]
    fn run_list_empty_dir() {
        let cmd = GoldenCommand::new(GoldenSubcommand::List, OutputFormat::Human)
            .with_golden_dir(Some(PathBuf::from("/nonexistent/path")));
        let result = cmd.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), exit_codes::SUCCESS);
    }
}
