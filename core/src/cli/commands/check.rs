//! Script validation command.
//!
//! Full semantic validation with configurable options for checking scripts.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::validation::config::ValidationConfig;
use crate::validation::coverage::render_ascii_keyboard;
use crate::validation::engine::ValidationEngine;
use crate::validation::types::{ValidationOptions, ValidationResult, WarningCategory};
use anyhow::Result;
use std::path::PathBuf;

/// Validate and lint a Rhai script with full semantic validation.
pub struct CheckCommand {
    pub script_path: PathBuf,
    pub output: OutputWriter,
    /// Treat warnings as errors.
    pub strict: bool,
    /// Suppress warnings in output.
    pub no_warnings: bool,
    /// Include coverage report.
    pub coverage: bool,
    /// Include ASCII keyboard visualization.
    pub visual: bool,
    /// Custom config file path.
    pub config_path: Option<PathBuf>,
    /// Show current config and exit.
    pub show_config: bool,
}

impl CheckCommand {
    pub fn new(script_path: PathBuf, format: OutputFormat) -> Self {
        Self {
            script_path,
            output: OutputWriter::new(format),
            strict: false,
            no_warnings: false,
            coverage: false,
            visual: false,
            config_path: None,
            show_config: false,
        }
    }

    /// Enable strict mode (warnings as errors).
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Disable warnings in output.
    pub fn no_warnings(mut self) -> Self {
        self.no_warnings = true;
        self
    }

    /// Enable coverage report.
    pub fn with_coverage(mut self) -> Self {
        self.coverage = true;
        self
    }

    /// Enable ASCII keyboard visualization.
    pub fn with_visual(mut self) -> Self {
        self.visual = true;
        self
    }

    /// Set custom config file path.
    pub fn with_config(mut self, path: PathBuf) -> Self {
        self.config_path = Some(path);
        self
    }

    /// Enable show config mode.
    pub fn show_config(mut self) -> Self {
        self.show_config = true;
        self
    }

    /// Run the check command.
    fn run_internal(&self) -> Result<ValidationResult> {
        // Load config
        let config = match &self.config_path {
            Some(path) => ValidationConfig::load_from_path(path)
                .ok_or_else(|| anyhow::anyhow!("Failed to load config from: {}", path.display()))?,
            None => ValidationConfig::load(),
        };

        // Show config and exit if requested
        if self.show_config {
            self.print_config(&config)?;
            return Ok(ValidationResult::default());
        }

        // Read and validate script
        let script = std::fs::read_to_string(&self.script_path)?;
        let engine = ValidationEngine::with_config(config);

        let options = self.build_options();
        let result = engine.validate(&script, options);

        // Print results
        self.print_result(&result, &engine)?;

        Ok(result)
    }

    fn build_options(&self) -> ValidationOptions {
        let mut options = ValidationOptions::new();
        if self.strict {
            options = options.strict();
        }
        if self.no_warnings {
            options = options.no_warnings();
        }
        if self.coverage || self.visual {
            options = options.with_coverage();
        }
        if self.visual {
            options = options.with_visual();
        }
        options
    }

    fn print_config(&self, config: &ValidationConfig) -> Result<()> {
        match self.output.format() {
            OutputFormat::Json => {
                self.output.data(config)?;
            }
            OutputFormat::Human => {
                println!("ValidationConfig:");
                println!("  max_errors: {}", config.max_errors);
                println!("  max_suggestions: {}", config.max_suggestions);
                println!("  similarity_threshold: {}", config.similarity_threshold);
                println!(
                    "  blocked_keys_warning_threshold: {}",
                    config.blocked_keys_warning_threshold
                );
                println!("  max_cycle_depth: {}", config.max_cycle_depth);
                println!(
                    "  tap_timeout_warn_range: ({}, {})",
                    config.tap_timeout_warn_range.0, config.tap_timeout_warn_range.1
                );
                println!(
                    "  combo_timeout_warn_range: ({}, {})",
                    config.combo_timeout_warn_range.0, config.combo_timeout_warn_range.1
                );
                println!(
                    "  ui_validation_debounce_ms: {}",
                    config.ui_validation_debounce_ms
                );
            }
        }
        Ok(())
    }

    fn print_result(&self, result: &ValidationResult, engine: &ValidationEngine) -> Result<()> {
        match self.output.format() {
            OutputFormat::Json => {
                self.output.data(result)?;
            }
            OutputFormat::Human => {
                self.print_human_result(result, engine)?;
            }
        }
        Ok(())
    }

    fn print_human_result(
        &self,
        result: &ValidationResult,
        engine: &ValidationEngine,
    ) -> Result<()> {
        let path_str = self.script_path.display();

        // Print errors
        for error in &result.errors {
            let location = error
                .location
                .as_ref()
                .map(|l| {
                    if let Some(col) = l.column {
                        format!("{}:{}:{}", path_str, l.line, col)
                    } else {
                        format!("{}:{}", path_str, l.line)
                    }
                })
                .unwrap_or_else(|| path_str.to_string());

            println!(
                "\x1b[31merror[{}]\x1b[0m: {} ({})",
                error.code, error.message, location
            );

            if let Some(ref loc) = error.location {
                if let Some(ref context) = loc.context {
                    println!("  | {}", context);
                }
            }

            if !error.suggestions.is_empty() {
                let max_suggestions = engine.config().max_suggestions;
                let suggestions: Vec<_> = error.suggestions.iter().take(max_suggestions).collect();
                println!(
                    "  \x1b[36mhelp\x1b[0m: Did you mean: {}?",
                    suggestions
                        .iter()
                        .map(|s| format!("'{}'", s))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        // Print warnings (unless suppressed)
        if !self.no_warnings {
            for warning in &result.warnings {
                let category = match warning.category {
                    WarningCategory::Conflict => "conflict",
                    WarningCategory::Safety => "safety",
                    WarningCategory::Performance => "performance",
                };

                let location = warning
                    .location
                    .as_ref()
                    .map(|l| {
                        if let Some(col) = l.column {
                            format!("{}:{}:{}", path_str, l.line, col)
                        } else {
                            format!("{}:{}", path_str, l.line)
                        }
                    })
                    .unwrap_or_else(|| path_str.to_string());

                println!(
                    "\x1b[33mwarning[{}]\x1b[0m: [{}] {} ({})",
                    warning.code, category, warning.message, location
                );

                if let Some(ref loc) = warning.location {
                    if let Some(ref context) = loc.context {
                        println!("  | {}", context);
                    }
                }
            }
        }

        // Print coverage if requested
        if self.coverage {
            if let Some(ref coverage) = result.coverage {
                println!("\n\x1b[1mCoverage Report:\x1b[0m");
                println!("  Remapped: {} keys", coverage.remapped.len());
                println!("  Blocked: {} keys", coverage.blocked.len());
                println!("  Tap-Hold: {} keys", coverage.tap_hold.len());
                println!("  Combo triggers: {} keys", coverage.combo_triggers.len());
                println!("  Total affected: {} keys", coverage.affected_count());

                if !coverage.layers.is_empty() {
                    println!("\n  Per-layer coverage:");
                    for (layer, layer_cov) in &coverage.layers {
                        println!(
                            "    {}: {} remapped, {} blocked",
                            layer,
                            layer_cov.remapped.len(),
                            layer_cov.blocked.len()
                        );
                    }
                }
            }
        }

        // Print visual keyboard if requested
        if self.visual {
            if let Some(ref coverage) = result.coverage {
                println!("\n\x1b[1mKeyboard Visualization:\x1b[0m");
                println!("{}", render_ascii_keyboard(coverage));
            }
        }

        // Print summary
        let error_count = result.errors.len();
        let warning_count = result.warnings.len();

        if result.is_valid && warning_count == 0 {
            self.output
                .success(&format!("Script '{}' is valid", path_str));
        } else if result.is_valid {
            println!(
                "\n\x1b[32m[OK]\x1b[0m Script '{}' is valid with {} warning(s)",
                path_str, warning_count
            );
        } else {
            println!(
                "\n\x1b[31m[FAIL]\x1b[0m Script '{}' has {} error(s) and {} warning(s)",
                path_str, error_count, warning_count
            );
        }

        Ok(())
    }

    fn result_to_command_result(&self, result: ValidationResult) -> CommandResult<()> {
        if result.has_errors() {
            // Build error message with location info
            let mut error_messages = Vec::new();
            for error in &result.errors {
                let location = error
                    .location
                    .as_ref()
                    .map(|l| {
                        if let Some(col) = l.column {
                            format!("{}:{}:{}", self.script_path.display(), l.line, col)
                        } else {
                            format!("{}:{}", self.script_path.display(), l.line)
                        }
                    })
                    .unwrap_or_else(|| self.script_path.display().to_string());
                error_messages.push(format!("{} ({})", error.message, location));
            }

            CommandResult::failure(
                ExitCode::ValidationFailed,
                format!("Script has {} error(s)", result.errors.len()),
            )
        } else if self.strict && result.has_warnings() {
            CommandResult::failure(
                ExitCode::AssertionFailed,
                format!(
                    "Script has {} warning(s) in strict mode",
                    result.warnings.len()
                ),
            )
        } else {
            CommandResult::success(())
        }
    }
}

impl Command for CheckCommand {
    fn name(&self) -> &str {
        "check"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        match self.run_internal() {
            Ok(result) => self.result_to_command_result(result),
            Err(err) => CommandResult::failure(
                ExitCode::GeneralError,
                format!("Check command failed: {err}"),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Verbosity;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_script(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    fn create_ctx(format: OutputFormat) -> CommandContext {
        CommandContext::new(format, Verbosity::Normal)
    }

    #[test]
    fn validates_valid_script() {
        let file = create_script(r#"remap("CapsLock", "Escape");"#);
        let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
        let result = cmd.execute(&create_ctx(OutputFormat::Human));
        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }

    #[test]
    fn detects_invalid_key() {
        let file = create_script(r#"remap("InvalidKey", "Escape");"#);
        let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
        let result = cmd.execute(&create_ctx(OutputFormat::Human));
        assert!(result.is_failure());
        assert_eq!(result.exit_code(), ExitCode::ValidationFailed);
    }

    #[test]
    fn strict_mode_fails_on_warnings() {
        let file = create_script(
            r#"
            remap("A", "B");
            remap("A", "C");
        "#,
        );
        let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).strict();
        let result = cmd.execute(&create_ctx(OutputFormat::Human));
        assert!(result.is_failure());
        assert_eq!(result.exit_code(), ExitCode::AssertionFailed);
    }

    #[test]
    fn no_warnings_suppresses_warnings() {
        let file = create_script(
            r#"
            remap("A", "B");
            remap("A", "C");
        "#,
        );
        let mut cmd =
            CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).no_warnings();
        let result = cmd.execute(&create_ctx(OutputFormat::Human));
        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }

    #[test]
    fn coverage_flag_works() {
        let file = create_script(r#"remap("A", "B");"#);
        let mut cmd =
            CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).with_coverage();
        let result = cmd.execute(&create_ctx(OutputFormat::Human));
        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }

    #[test]
    fn visual_flag_works() {
        let file = create_script(r#"remap("A", "B");"#);
        let mut cmd =
            CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).with_visual();
        let result = cmd.execute(&create_ctx(OutputFormat::Human));
        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }

    #[test]
    fn show_config_works() {
        let file = create_script("");
        let mut cmd =
            CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).show_config();
        let result = cmd.execute(&create_ctx(OutputFormat::Human));
        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }

    #[test]
    fn json_output_works() {
        let file = create_script(r#"remap("CapsLock", "Escape");"#);
        let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Json);
        let result = cmd.execute(&create_ctx(OutputFormat::Json));
        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }

    #[test]
    fn custom_config_works() {
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, "max_errors = 5").unwrap();

        let script_file = create_script(r#"remap("CapsLock", "Escape");"#);
        let mut cmd = CheckCommand::new(script_file.path().to_path_buf(), OutputFormat::Human)
            .with_config(config_file.path().to_path_buf());
        let result = cmd.execute(&create_ctx(OutputFormat::Human));
        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }

    #[test]
    fn invalid_config_fails() {
        let script_file = create_script(r#"remap("CapsLock", "Escape");"#);
        let mut cmd = CheckCommand::new(script_file.path().to_path_buf(), OutputFormat::Human)
            .with_config(PathBuf::from("/nonexistent/config.toml"));
        let result = cmd.execute(&create_ctx(OutputFormat::Human));
        assert!(result.is_failure());
        assert_eq!(result.exit_code(), ExitCode::GeneralError);
    }

    #[test]
    fn detects_undefined_layer() {
        let file = create_script(r#"layer_push("undefined_layer");"#);
        let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
        let result = cmd.execute(&create_ctx(OutputFormat::Human));
        assert!(result.is_failure());
        assert_eq!(result.exit_code(), ExitCode::ValidationFailed);
    }

    #[test]
    fn accepts_defined_layer() {
        let file = create_script(
            r#"
            define_layer("nav");
            layer_push("nav");
        "#,
        );
        let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
        let result = cmd.execute(&create_ctx(OutputFormat::Human));
        assert!(result.is_success());
        assert_eq!(result.exit_code(), ExitCode::Success);
    }
}
