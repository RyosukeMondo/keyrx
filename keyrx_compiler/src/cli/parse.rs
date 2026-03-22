//! Parse subcommand handler.
//!
//! Handles the `parse` subcommand which parses Rhai scripts and displays
//! the parsed configuration structure.

use std::fmt;
use std::io;
use std::path::Path;

use crate::error::ParseError as ParserParseError;
use crate::parser::Parser;

/// Errors that can occur during the parse subcommand.
///
/// Note: This is distinct from `crate::error::ParseError` which is used by the parser.
/// This error type is specific to the parse subcommand CLI operation.
#[derive(Debug)]
#[allow(dead_code)] // Will be used in task 16
#[allow(clippy::enum_variant_names)]
pub enum ParseCommandError {
    /// Failed to parse Rhai script.
    ParseError(ParserParseError),

    /// Failed to serialize to JSON.
    JsonError(serde_json::Error),

    /// I/O error during file operations.
    IoError(io::Error),
}

impl fmt::Display for ParseCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(err) => write!(f, "Parse error: {:?}", err),
            Self::JsonError(err) => write!(f, "JSON serialization error: {}", err),
            Self::IoError(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for ParseCommandError {}

impl From<io::Error> for ParseCommandError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<ParserParseError> for ParseCommandError {
    fn from(err: ParserParseError) -> Self {
        Self::ParseError(err)
    }
}

impl From<serde_json::Error> for ParseCommandError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonError(err)
    }
}

/// Handles the parse subcommand.
///
/// # Arguments
///
/// * `input` - Path to the input .rhai script file.
/// * `json` - If true, output JSON format; otherwise, output human-readable summary.
///
/// # Returns
///
/// `Ok(())` on success, or `ParseCommandError` on failure.
pub fn handle_parse(input: &Path, json: bool) -> Result<(), ParseCommandError> {
    let mut parser = Parser::new();
    let config = parser.parse_script(input)?;

    if json {
        // Output JSON format
        let json_output = serde_json::to_string_pretty(&config)?;
        println!("{}", json_output);
    } else {
        // Output human-readable summary
        print_summary(&config);
    }

    Ok(())
}

/// Prints a human-readable summary of the parsed configuration.
fn print_summary(config: &keyrx_core::config::ConfigRoot) {
    println!("Configuration parsed successfully:");
    println!("  Version: {}", config.version);
    println!("  Devices: {}", config.devices.len());

    for (idx, device) in config.devices.iter().enumerate() {
        println!(
            "    Device {}: {} ({} mappings)",
            idx + 1,
            device.identifier.pattern,
            device.mappings.len()
        );

        // Show detailed breakdown of mapping types
        let mut simple = 0;
        let mut modifier = 0;
        let mut lock = 0;
        let mut tap_hold = 0;
        let mut modified_output = 0;
        let mut conditional = 0;

        for mapping in &device.mappings {
            match mapping {
                keyrx_core::config::KeyMapping::Base(base) => match base {
                    keyrx_core::config::BaseKeyMapping::Simple { .. } => simple += 1,
                    keyrx_core::config::BaseKeyMapping::Modifier { .. } => modifier += 1,
                    keyrx_core::config::BaseKeyMapping::Lock { .. } => lock += 1,
                    keyrx_core::config::BaseKeyMapping::TapHold { .. } => tap_hold += 1,
                    keyrx_core::config::BaseKeyMapping::HoldOnly { .. } => tap_hold += 1,
                    keyrx_core::config::BaseKeyMapping::ModifiedOutput { .. } => {
                        modified_output += 1
                    }
                },
                keyrx_core::config::KeyMapping::Conditional { .. } => conditional += 1,
            }
        }

        let mut details = Vec::new();
        if simple > 0 {
            details.push(format!("Simple: {}", simple));
        }
        if modifier > 0 {
            details.push(format!("Modifier: {}", modifier));
        }
        if lock > 0 {
            details.push(format!("Lock: {}", lock));
        }
        if tap_hold > 0 {
            details.push(format!("TapHold: {}", tap_hold));
        }
        if modified_output > 0 {
            details.push(format!("ModifiedOutput: {}", modified_output));
        }
        if conditional > 0 {
            details.push(format!("Conditional: {}", conditional));
        }

        if !details.is_empty() {
            println!("      {}", details.join(", "));
        }
    }

    println!("  Metadata:");
    println!("    Compiler version: {}", config.metadata.compiler_version);
    println!(
        "    Compilation timestamp: {}",
        config.metadata.compilation_timestamp
    );
    println!("    Source hash: {}", config.metadata.source_hash);
}
