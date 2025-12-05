//! Exit codes documentation command.
//!
//! Lists all exit codes with descriptions, supporting both human-readable
//! and JSON output formats.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode};
use serde::Serialize;

/// Display all exit codes with descriptions.
///
/// This command provides comprehensive documentation of all exit codes
/// used by KeyRx CLI commands, making it easy to understand and script
/// around command failures.
///
/// # Example
///
/// ```bash
/// keyrx exit-codes
/// keyrx exit-codes --json
/// ```
pub struct ExitCodesCommand;

impl ExitCodesCommand {
    /// Create a new exit codes command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExitCodesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ExitCodesCommand {
    fn name(&self) -> &str {
        "exit-codes"
    }

    fn execute(&mut self, ctx: &CommandContext) -> CommandResult<()> {
        let exit_codes = vec![
            ExitCodeInfo::from_code(ExitCode::Success),
            ExitCodeInfo::from_code(ExitCode::GeneralError),
            ExitCodeInfo::from_code(ExitCode::AssertionFailed),
            ExitCodeInfo::from_code(ExitCode::Timeout),
            ExitCodeInfo::from_code(ExitCode::ValidationFailed),
            ExitCodeInfo::from_code(ExitCode::PermissionDenied),
            ExitCodeInfo::from_code(ExitCode::DeviceNotFound),
            ExitCodeInfo::from_code(ExitCode::ScriptError),
            ExitCodeInfo::from_code(ExitCode::Panic),
        ];

        match ctx.output_format() {
            crate::cli::OutputFormat::Json | crate::cli::OutputFormat::Yaml => {
                let output = ExitCodesOutput { exit_codes };
                ctx.output().data(&output).ok();
            }
            _ => {
                println!("\nKeyRx Exit Codes");
                println!("================\n");
                println!("Exit codes used by all KeyRx CLI commands:\n");

                for info in &exit_codes {
                    let status_indicator = if info.code == 0 { "✓" } else { "✗" };
                    println!(
                        "  {} {:3} - {} ({})",
                        status_indicator, info.code, info.name, info.category
                    );
                    println!("       {}", info.description);

                    if !info.used_by.is_empty() {
                        println!("       Used by: {}", info.used_by.join(", "));
                    }
                    println!();
                }

                println!("Use --json flag for machine-readable output.");
            }
        }

        CommandResult::success(())
    }
}

/// Information about a single exit code.
#[derive(Debug, Clone, Serialize)]
struct ExitCodeInfo {
    /// The numeric exit code value.
    code: u8,
    /// The variant name (e.g., "Success", "GeneralError").
    name: &'static str,
    /// Human-readable description.
    description: &'static str,
    /// Category of the exit code (e.g., "success", "error", "validation").
    category: &'static str,
    /// Commands that commonly use this exit code.
    used_by: Vec<&'static str>,
}

impl ExitCodeInfo {
    /// Create exit code info from an ExitCode variant.
    fn from_code(code: ExitCode) -> Self {
        let (name, category, used_by) = match code {
            ExitCode::Success => ("Success", "success", vec!["all commands when successful"]),
            ExitCode::GeneralError => (
                "GeneralError",
                "error",
                vec![
                    "run",
                    "check",
                    "test",
                    "replay",
                    "discover",
                    "all commands on general failure",
                ],
            ),
            ExitCode::AssertionFailed => (
                "AssertionFailed",
                "test-failure",
                vec![
                    "test",
                    "uat",
                    "replay --verify",
                    "regression",
                    "golden verify",
                ],
            ),
            ExitCode::Timeout => ("Timeout", "timeout", vec!["test", "uat", "simulate", "run"]),
            ExitCode::ValidationFailed => (
                "ValidationFailed",
                "validation",
                vec!["check", "discover", "run"],
            ),
            ExitCode::PermissionDenied => (
                "PermissionDenied",
                "permissions",
                vec!["run", "devices", "discover"],
            ),
            ExitCode::DeviceNotFound => (
                "DeviceNotFound",
                "device",
                vec!["run", "devices", "discover"],
            ),
            ExitCode::ScriptError => (
                "ScriptError",
                "script",
                vec!["run", "test", "simulate", "replay"],
            ),
            ExitCode::Panic => ("Panic", "panic", vec!["any command on unrecoverable error"]),
        };

        Self {
            code: code.as_u8(),
            name,
            description: code.description(),
            category,
            used_by,
        }
    }
}

/// JSON output format for exit codes.
#[derive(Debug, Serialize)]
struct ExitCodesOutput {
    exit_codes: Vec<ExitCodeInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{OutputFormat, Verbosity};

    #[test]
    fn exit_codes_command_name() {
        let cmd = ExitCodesCommand::new();
        assert_eq!(cmd.name(), "exit-codes");
    }

    #[test]
    fn exit_codes_command_succeeds() {
        let mut cmd = ExitCodesCommand::new();
        let ctx = CommandContext::new(OutputFormat::Human, Verbosity::Normal);
        let result = cmd.execute(&ctx);
        assert!(result.is_success());
    }

    #[test]
    fn exit_codes_command_json_format() {
        let mut cmd = ExitCodesCommand::new();
        let ctx = CommandContext::new(OutputFormat::Json, Verbosity::Normal);
        let result = cmd.execute(&ctx);
        assert!(result.is_success());
    }

    #[test]
    fn exit_code_info_from_code() {
        let info = ExitCodeInfo::from_code(ExitCode::Success);
        assert_eq!(info.code, 0);
        assert_eq!(info.name, "Success");
        assert_eq!(info.category, "success");

        let info = ExitCodeInfo::from_code(ExitCode::ValidationFailed);
        assert_eq!(info.code, 4);
        assert_eq!(info.name, "ValidationFailed");
        assert_eq!(info.category, "validation");
        assert!(info.used_by.contains(&"check"));
    }

    #[test]
    fn all_exit_codes_covered() {
        // Ensure all exit codes have corresponding info
        let codes = vec![
            ExitCode::Success,
            ExitCode::GeneralError,
            ExitCode::AssertionFailed,
            ExitCode::Timeout,
            ExitCode::ValidationFailed,
            ExitCode::PermissionDenied,
            ExitCode::DeviceNotFound,
            ExitCode::ScriptError,
            ExitCode::Panic,
        ];

        for code in codes {
            let info = ExitCodeInfo::from_code(code);
            assert_eq!(info.code, code.as_u8());
            assert!(!info.name.is_empty());
            assert!(!info.description.is_empty());
            assert!(!info.category.is_empty());
        }
    }
}
