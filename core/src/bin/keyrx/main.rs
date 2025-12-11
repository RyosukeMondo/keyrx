#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
//! KeyRx CLI binary
//!
//! This binary uses println! and eprintln! for user-facing output,
//! which is intentional and distinct from internal logging.
//!
//! # Module Organization
//! - `args`: CLI argument definitions (Cli struct, Commands enum, subcommands)
//! - `dispatch`: Command routing and execution
//! - `commands_core`: Core engine commands (run, simulate, check, discover)
//! - `commands_config`: Configuration commands (devices, hardware, layout, keymap, runtime)
//! - `commands_test`: Testing commands (test, replay, analyze, uat, regression, doctor, repl)

mod args;
mod commands_config;
mod commands_core;
mod commands_test;
mod dispatch;

use clap::Parser;
use keyrx_core::cli::{OutputFormat, Verbosity};
use keyrx_core::config::load_config;
use keyrx_core::observability::StructuredLogger;
use std::process::ExitCode;
use tracing::{debug, error, info};

use args::Cli;
use keyrx_core::cli::CommandContext;

fn parse_format(s: &str, json_flag: bool) -> OutputFormat {
    if json_flag {
        return OutputFormat::Json;
    }

    match s.to_lowercase().as_str() {
        "json" => OutputFormat::Json,
        "yaml" | "yml" => OutputFormat::Yaml,
        _ => OutputFormat::Human,
    }
}

pub fn parse_hex_or_decimal_u16(value: &str) -> Result<u16, String> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16).map_err(|err| format!("Invalid hex value '{value}': {err}"))
    } else {
        trimmed.parse::<u16>().or_else(|_| {
            u16::from_str_radix(trimmed, 16)
                .map_err(|err| format!("Invalid number or hex value '{value}': {err}"))
        })
    }
}

fn install_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        // Extract panic location
        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        // Extract panic message
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic message".to_string()
        };

        // Log panic at error level (if logger is initialized)
        error!(
            location = %location,
            message = %message,
            "Panic occurred"
        );

        // Print to stderr for visibility (works even if tracing isn't initialized yet)
        eprintln!("Error: Panic at {}: {}", location, message);
        eprintln!("This is a bug. Please report it at: https://github.com/keyrx/keyrx/issues");

        // Exit with code 101 (Rust panic convention)
        std::process::exit(101);
    }));
}

#[tokio::main]
async fn main() -> ExitCode {
    // Install panic handler to catch panics and return exit code 101
    install_panic_handler();

    let cli = Cli::parse();
    let format = parse_format(&cli.output_format, cli.json);

    // Initialize structured logger
    // Use human-readable format for CLI to make debugging easier
    // Logger level is controlled by RUST_LOG environment variable
    let log_format = match std::env::var("KEYRX_LOG_FORMAT")
        .unwrap_or_else(|_| "pretty".to_string())
        .as_str()
    {
        "json" => keyrx_core::observability::OutputFormat::Json,
        _ => keyrx_core::observability::OutputFormat::Pretty,
    };

    if let Err(e) = StructuredLogger::new().with_format(log_format).init() {
        // If logger init fails, print to stderr but continue
        eprintln!("Warning: Failed to initialize logger: {}", e);
    }

    debug!(
        output_format = ?format,
        config_path = ?cli.config,
        "CLI initialized"
    );

    // Load configuration from file (or use defaults)
    let config = load_config(cli.config.as_deref());

    info!(
        tap_timeout_ms = config.timing.tap_timeout_ms,
        combo_timeout_ms = config.timing.combo_timeout_ms,
        "Configuration loaded"
    );

    // Create command context
    let ctx = CommandContext::with_config(format, Verbosity::Normal, cli.config);

    // Execute command and get result
    let result = dispatch::run_command(cli.command, &ctx, config).await;

    // Extract exit code and handle errors
    if result.is_success() {
        debug!("Command completed successfully");
        ExitCode::SUCCESS
    } else {
        let exit_code = result.exit_code();
        error!(
            exit_code = exit_code as u8,
            message_count = result.messages().len(),
            "Command failed"
        );

        // Print error messages
        for msg in result.messages() {
            eprintln!("Error: {msg}");
        }
        exit_code.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_output_format_flag_and_alias() {
        let cli = Cli::try_parse_from(["keyrx", "check", "--output-format", "yaml", "script.rhai"])
            .expect("output-format flag should parse globally");
        assert_eq!(cli.output_format, "yaml");

        let cli = Cli::try_parse_from(["keyrx", "check", "--format", "json", "script.rhai"])
            .expect("format alias should still work");
        assert_eq!(cli.output_format, "json");
    }

    #[test]
    fn parses_json_shortcut_after_subcommand() {
        let cli = Cli::try_parse_from(["keyrx", "check", "script.rhai", "--json"])
            .expect("--json should be accepted globally");
        assert!(cli.json);
    }

    #[test]
    fn parse_format_defaults_to_human_on_unknown_values() {
        assert_eq!(parse_format("human", false), OutputFormat::Human);
        assert_eq!(parse_format("unknown", false), OutputFormat::Human);
    }

    #[test]
    fn parse_format_respects_json_flag_priority() {
        assert_eq!(parse_format("yaml", true), OutputFormat::Json);
        assert_eq!(parse_format("json", true), OutputFormat::Json);
    }

    #[test]
    fn parses_hex_or_decimal() {
        assert_eq!(parse_hex_or_decimal_u16("0x1b1c").unwrap(), 0x1b1c);
        assert_eq!(parse_hex_or_decimal_u16("1b1c").unwrap(), 0x1b1c);
        assert_eq!(parse_hex_or_decimal_u16("7000").unwrap(), 7000);
    }
}
