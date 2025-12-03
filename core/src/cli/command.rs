//! Command trait and context for standardized CLI command interface.
//!
//! This module provides a trait-based abstraction for CLI commands, enabling
//! consistent handling of command execution, output formatting, and exit codes
//! across all KeyRx subcommands.

use super::{CommandResult, OutputFormat, OutputWriter};
use std::path::PathBuf;

/// Standard interface for CLI commands.
///
/// All KeyRx commands implement this trait to provide consistent
/// execution semantics and exit code handling.
///
/// # Example
///
/// ```
/// use keyrx_core::cli::{Command, CommandContext, CommandResult};
///
/// struct MyCommand;
///
/// impl Command for MyCommand {
///     fn name(&self) -> &str {
///         "mycommand"
///     }
///
///     fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
///         CommandResult::success(())
///     }
/// }
/// ```
pub trait Command {
    /// Get the command name.
    ///
    /// Used for logging and diagnostics.
    fn name(&self) -> &str;

    /// Execute the command.
    ///
    /// Returns a `CommandResult` containing the exit code and any output.
    /// Commands should use the provided `CommandContext` for output formatting
    /// and configuration access.
    fn execute(&mut self, ctx: &CommandContext) -> CommandResult<()>;
}

/// Shared context passed to all commands during execution.
///
/// Contains output formatting, configuration, and other shared state
/// that commands need to access during execution.
///
/// # Example
///
/// ```
/// use keyrx_core::cli::{CommandContext, OutputFormat, Verbosity};
///
/// let ctx = CommandContext::new(OutputFormat::Human, Verbosity::Normal);
/// ctx.output().success("Operation completed");
/// ```
#[derive(Debug)]
pub struct CommandContext {
    /// Output writer for formatted output.
    output: OutputWriter,
    /// Verbosity level for command output.
    verbosity: Verbosity,
    /// Optional path to configuration file.
    config_path: Option<PathBuf>,
}

impl CommandContext {
    /// Create a new command context.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandContext, OutputFormat, Verbosity};
    ///
    /// let ctx = CommandContext::new(OutputFormat::Human, Verbosity::Normal);
    /// ```
    pub fn new(format: OutputFormat, verbosity: Verbosity) -> Self {
        Self {
            output: OutputWriter::new(format),
            verbosity,
            config_path: None,
        }
    }

    /// Create a new command context with a configuration path.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandContext, OutputFormat, Verbosity};
    /// use std::path::PathBuf;
    ///
    /// let ctx = CommandContext::with_config(
    ///     OutputFormat::Human,
    ///     Verbosity::Normal,
    ///     Some(PathBuf::from("config.toml"))
    /// );
    /// ```
    pub fn with_config(
        format: OutputFormat,
        verbosity: Verbosity,
        config_path: Option<PathBuf>,
    ) -> Self {
        Self {
            output: OutputWriter::new(format),
            verbosity,
            config_path,
        }
    }

    /// Get the output writer.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandContext, OutputFormat, Verbosity};
    ///
    /// let ctx = CommandContext::new(OutputFormat::Human, Verbosity::Normal);
    /// ctx.output().success("Done");
    /// ```
    pub fn output(&self) -> &OutputWriter {
        &self.output
    }

    /// Get the verbosity level.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandContext, OutputFormat, Verbosity};
    ///
    /// let ctx = CommandContext::new(OutputFormat::Human, Verbosity::Verbose);
    /// if ctx.verbosity().is_verbose() {
    ///     println!("Debug info");
    /// }
    /// ```
    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    /// Get the configuration file path if provided.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandContext, OutputFormat, Verbosity};
    /// use std::path::PathBuf;
    ///
    /// let ctx = CommandContext::with_config(
    ///     OutputFormat::Human,
    ///     Verbosity::Normal,
    ///     Some(PathBuf::from("config.toml"))
    /// );
    /// assert!(ctx.config_path().is_some());
    /// ```
    pub fn config_path(&self) -> Option<&PathBuf> {
        self.config_path.as_ref()
    }

    /// Get the output format.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandContext, OutputFormat, Verbosity};
    ///
    /// let ctx = CommandContext::new(OutputFormat::Json, Verbosity::Normal);
    /// match ctx.output_format() {
    ///     OutputFormat::Json => println!("JSON mode"),
    ///     OutputFormat::Human => println!("Human mode"),
    /// }
    /// ```
    pub fn output_format(&self) -> OutputFormat {
        self.output.format()
    }
}

/// Verbosity level for command output.
///
/// Controls how much diagnostic information is displayed during
/// command execution.
///
/// # Example
///
/// ```
/// use keyrx_core::cli::Verbosity;
///
/// let v = Verbosity::Verbose;
/// assert!(v.is_verbose());
/// assert!(!v.is_quiet());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Suppress non-essential output.
    Quiet,
    /// Standard output level.
    #[default]
    Normal,
    /// Detailed diagnostic output.
    Verbose,
}

impl Verbosity {
    /// Check if verbosity is at verbose level.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::Verbosity;
    ///
    /// assert!(Verbosity::Verbose.is_verbose());
    /// assert!(!Verbosity::Normal.is_verbose());
    /// ```
    pub const fn is_verbose(self) -> bool {
        matches!(self, Verbosity::Verbose)
    }

    /// Check if verbosity is at quiet level.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::Verbosity;
    ///
    /// assert!(Verbosity::Quiet.is_quiet());
    /// assert!(!Verbosity::Normal.is_quiet());
    /// ```
    pub const fn is_quiet(self) -> bool {
        matches!(self, Verbosity::Quiet)
    }

    /// Check if verbosity is at normal or higher level.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::Verbosity;
    ///
    /// assert!(Verbosity::Normal.is_normal_or_verbose());
    /// assert!(Verbosity::Verbose.is_normal_or_verbose());
    /// assert!(!Verbosity::Quiet.is_normal_or_verbose());
    /// ```
    pub const fn is_normal_or_verbose(self) -> bool {
        matches!(self, Verbosity::Normal | Verbosity::Verbose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_context_new() {
        let ctx = CommandContext::new(OutputFormat::Human, Verbosity::Normal);
        assert_eq!(ctx.output_format(), OutputFormat::Human);
        assert_eq!(ctx.verbosity(), Verbosity::Normal);
        assert!(ctx.config_path().is_none());
    }

    #[test]
    fn command_context_with_config() {
        let config_path = PathBuf::from("test.toml");
        let ctx = CommandContext::with_config(
            OutputFormat::Json,
            Verbosity::Verbose,
            Some(config_path.clone()),
        );
        assert_eq!(ctx.output_format(), OutputFormat::Json);
        assert_eq!(ctx.verbosity(), Verbosity::Verbose);
        assert_eq!(ctx.config_path(), Some(&config_path));
    }

    #[test]
    fn verbosity_is_verbose() {
        assert!(Verbosity::Verbose.is_verbose());
        assert!(!Verbosity::Normal.is_verbose());
        assert!(!Verbosity::Quiet.is_verbose());
    }

    #[test]
    fn verbosity_is_quiet() {
        assert!(Verbosity::Quiet.is_quiet());
        assert!(!Verbosity::Normal.is_quiet());
        assert!(!Verbosity::Verbose.is_quiet());
    }

    #[test]
    fn verbosity_is_normal_or_verbose() {
        assert!(!Verbosity::Quiet.is_normal_or_verbose());
        assert!(Verbosity::Normal.is_normal_or_verbose());
        assert!(Verbosity::Verbose.is_normal_or_verbose());
    }

    #[test]
    fn verbosity_default() {
        let v = Verbosity::default();
        assert_eq!(v, Verbosity::Normal);
    }

    struct TestCommand {
        name: String,
    }

    impl Command for TestCommand {
        fn name(&self) -> &str {
            &self.name
        }

        fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
            CommandResult::success(())
        }
    }

    #[test]
    fn command_trait_implementation() {
        let mut cmd = TestCommand {
            name: "test".to_string(),
        };
        assert_eq!(cmd.name(), "test");

        let ctx = CommandContext::new(OutputFormat::Human, Verbosity::Normal);
        let result = cmd.execute(&ctx);
        assert!(result.is_success());
    }
}
