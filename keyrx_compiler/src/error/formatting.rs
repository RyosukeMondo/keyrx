//! Error formatting utilities for user-friendly error messages.
//!
//! This module provides colored terminal output with code snippets,
//! location information, and helpful suggestions for fixing errors.

#![allow(dead_code)] // Functions will be used in CLI integration

use crate::error::types::{ImportStep, ParseError};
use colored::*;
use std::path::Path;

/// Helper function to encode bytes as hex string.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

/// Formats an import chain to show the path from main file to error location.
///
/// Displays as: main.rhai → a.rhai (line 5) → b.rhai (line 2)
///
/// If the import_chain is empty, returns an empty string.
fn format_import_chain(import_chain: &[ImportStep]) -> String {
    if import_chain.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    output.push_str(&format!("{}\n", "Import chain:".blue().bold()));

    for (i, step) in import_chain.iter().enumerate() {
        output.push_str(&format!(
            "  {} {} (line {})",
            step.file.display(),
            "→".blue(),
            step.line
        ));
        if i < import_chain.len() - 1 {
            output.push('\n');
        }
    }

    output.push_str("\n\n");
    output
}

/// Formats a ParseError with colored output, code snippets, and helpful suggestions.
///
/// This is the main error formatting function that shows:
/// - Import chain (if error occurred in imported file)
/// - File:line:column location in blue
/// - "Error:" label in red
/// - 3 lines of context around the error with line numbers
/// - Caret (^) pointing to error column in red
/// - "help:" section in green with specific suggestions
///
/// Respects NO_COLOR environment variable.
pub fn format_error(error: &ParseError, _file: &Path, source: &str) -> String {
    match error {
        ParseError::SyntaxError {
            file: error_file,
            line,
            column,
            message,
            import_chain,
        } => {
            let mut output = String::new();

            // Import chain (if present)
            output.push_str(&format_import_chain(import_chain));

            // Location header: file:line:column
            output.push_str(&format!(
                "{}\n",
                format!("{}:{}:{}", error_file.display(), line, column).blue()
            ));

            // Error label and message
            output.push_str(&format!("{} {}\n", "Error:".red().bold(), message));

            // Code snippet with context
            output.push_str(&format_code_snippet(source, *line, *column));

            // Help section
            output.push_str(&format!(
                "\n{} Check your Rhai script syntax at the indicated location.\n",
                "help:".green().bold()
            ));

            output
        }
        ParseError::InvalidPrefix {
            expected,
            got,
            context,
            import_chain,
        } => format_invalid_prefix_error(expected, got, context, import_chain),
        ParseError::ModifierIdOutOfRange {
            got,
            max,
            import_chain,
        } => format_range_error("Modifier", got, max, "MD", import_chain),
        ParseError::LockIdOutOfRange {
            got,
            max,
            import_chain,
        } => format_range_error("Lock", got, max, "LK", import_chain),
        ParseError::PhysicalModifierInMD { name, import_chain } => {
            format_physical_modifier_error(name, import_chain)
        }
        ParseError::MissingPrefix {
            key,
            context,
            import_chain,
        } => format_missing_prefix_error(key, context, import_chain),
        ParseError::ImportNotFound {
            path,
            searched_paths,
            import_chain,
        } => format_import_not_found_error(path, searched_paths, import_chain),
        ParseError::CircularImport { chain } => format_circular_import_error(chain),
        ParseError::ResourceLimitExceeded {
            limit_type,
            import_chain,
        } => format_resource_limit_error(limit_type, import_chain),
    }
}

/// Formats a code snippet with line numbers and a caret pointing to the error column.
fn format_code_snippet(source: &str, error_line: usize, error_column: usize) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut output = String::new();

    // Calculate context range (3 lines: 1 before, error line, 1 after)
    let start_line = error_line.saturating_sub(1).max(1);
    let end_line = (error_line + 1).min(lines.len());

    output.push('\n');

    for line_num in start_line..=end_line {
        let line_idx = line_num - 1;
        if line_idx >= lines.len() {
            break;
        }

        let line_content = lines[line_idx];

        // Line number and content
        if line_num == error_line {
            // Highlight error line
            output.push_str(&format!(
                "{:4} | {}\n",
                line_num.to_string().blue().bold(),
                line_content
            ));

            // Caret pointing to error column
            let spaces = " ".repeat(error_column.saturating_sub(1));
            output.push_str(&format!("     | {}{}\n", spaces, "^".red().bold()));
        } else {
            // Context lines
            output.push_str(&format!(
                "{:4} | {}\n",
                line_num.to_string().blue(),
                line_content
            ));
        }
    }

    output
}

/// Formats an InvalidPrefix error with specific suggestions.
fn format_invalid_prefix_error(
    expected: &str,
    got: &str,
    context: &str,
    import_chain: &[ImportStep],
) -> String {
    let mut output = String::new();

    // Import chain (if present)
    output.push_str(&format_import_chain(import_chain));

    output.push_str(&format!("{} ", "Error:".red().bold()));

    if got.starts_with("MD_") && !is_valid_hex_id(got, "MD_") {
        output.push_str(&format!("Invalid modifier ID format: {}\n", got.yellow()));
        output.push_str(&format!(
            "\n{} Use MD_00 through MD_FE for custom modifiers.\n",
            "help:".green().bold()
        ));
        output.push_str(
            "      Physical modifiers (LShift, RShift, etc.) should not have MD_ prefix.\n\n",
        );
        output.push_str(&format!(
            "      {}\n",
            "Example: map(\"VK_CapsLock\", \"MD_00\")  // CapsLock becomes custom modifier 00"
                .cyan()
        ));
    } else if got.starts_with("VK_") && context.contains("hold") {
        output.push_str(&format!(
            "tap_hold hold parameter must have MD_ prefix, got: {}\n",
            got.yellow()
        ));
        output.push_str(&format!(
            "\n{} The hold parameter expects a custom modifier (MD_XX), not a virtual key.\n",
            "help:".green().bold()
        ));
        output.push_str(&format!(
            "      {}\n",
            "Example: tap_hold(\"VK_Space\", \"VK_Space\", \"MD_00\", 200)".cyan()
        ));
    } else {
        output.push_str(&format!(
            "Invalid prefix: expected {}, got '{}'\n",
            expected.yellow(),
            got.yellow()
        ));
        output.push_str(&format!("Context: {}\n", context));
        output.push_str(&format!("\n{} Valid prefixes:\n", "help:".green().bold()));
        output.push_str("      - VK_ for virtual keys (e.g., VK_A, VK_Enter)\n");
        output.push_str("      - MD_ for custom modifiers (e.g., MD_00, MD_01)\n");
        output.push_str("      - LK_ for custom locks (e.g., LK_00, LK_01)\n");
    }

    output
}

/// Helper to check if a string has valid hex ID format (e.g., "MD_00").
fn is_valid_hex_id(s: &str, prefix: &str) -> bool {
    if !s.starts_with(prefix) {
        return false;
    }
    let hex_part = &s[prefix.len()..];
    hex_part.len() == 2 && hex_part.chars().all(|c| c.is_ascii_hexdigit())
}

/// Formats a range error (ModifierIdOutOfRange or LockIdOutOfRange).
fn format_range_error(
    type_name: &str,
    got: &u16,
    max: &u16,
    prefix: &str,
    import_chain: &[ImportStep],
) -> String {
    let mut output = String::new();

    // Import chain (if present)
    output.push_str(&format_import_chain(import_chain));

    output.push_str(&format!(
        "{} {} ID out of range: {}\n",
        "Error:".red().bold(),
        type_name,
        got.to_string().yellow()
    ));
    output.push_str(&format!(
        "\n{} {} IDs must be in the range {}_{:02X} to {}_{:02X} (0-{} in decimal).\n",
        "help:".green().bold(),
        type_name,
        prefix,
        0,
        prefix,
        max,
        max
    ));
    output.push_str(&format!(
        "      You provided: {}, which exceeds the maximum of {}.\n\n",
        got, max
    ));
    output.push_str(&format!(
        "      {}\n",
        format!("Example: map(\"VK_CapsLock\", \"{}_{:02X}\")", prefix, max).cyan()
    ));

    output
}

/// Formats a PhysicalModifierInMD error.
fn format_physical_modifier_error(name: &str, import_chain: &[ImportStep]) -> String {
    let mut output = String::new();

    // Import chain (if present)
    output.push_str(&format_import_chain(import_chain));

    output.push_str(&format!(
        "{} Physical modifier name '{}' cannot be used with MD_ prefix.\n",
        "Error:".red().bold(),
        name.yellow()
    ));
    output.push_str(&format!(
        "\n{} Physical modifiers (LShift, RShift, LCtrl, RCtrl, LAlt, RAlt, LMeta, RMeta)\n",
        "help:".green().bold()
    ));
    output.push_str("      should be used directly without prefixes in input contexts,\n");
    output.push_str("      or with VK_ in output contexts.\n\n");
    output.push_str("      For custom modifiers, use MD_00 through MD_FE.\n\n");
    output.push_str(&format!(
        "      {}\n",
        "Example: map(\"VK_CapsLock\", \"MD_00\")  // CapsLock becomes custom modifier 00".cyan()
    ));
    output.push_str(&format!(
        "               {}\n",
        "when(\"MD_00\", || { ... })           // When custom modifier 00 is active".cyan()
    ));

    output
}

/// Formats a MissingPrefix error.
fn format_missing_prefix_error(key: &str, context: &str, import_chain: &[ImportStep]) -> String {
    let mut output = String::new();

    // Import chain (if present)
    output.push_str(&format_import_chain(import_chain));

    output.push_str(&format!(
        "{} Missing prefix for key '{}'\n",
        "Error:".red().bold(),
        key.yellow()
    ));
    output.push_str(&format!("Context: {}\n", context));

    if context.contains("output") || context.contains("to") {
        output.push_str(&format!(
            "\n{} Output keys must have VK_, MD_, or LK_ prefix.\n",
            "help:".green().bold()
        ));
        output.push_str(&format!(
            "      Try: {} for virtual key output\n\n",
            format!("VK_{}", key).cyan()
        ));
        output.push_str("      Examples:\n");
        output.push_str(&format!(
            "      - {}  // Remap A to B (virtual key)\n",
            "map(\"VK_A\", \"VK_B\")".cyan()
        ));
        output.push_str(&format!(
            "      - {}  // CapsLock acts as custom modifier 00\n",
            "map(\"VK_CapsLock\", \"MD_00\")".cyan()
        ));
        output.push_str(&format!(
            "      - {}  // ScrollLock toggles custom lock 00\n",
            "map(\"VK_ScrollLock\", \"LK_00\")".cyan()
        ));
    } else {
        output.push_str(&format!(
            "\n{} Keys must have VK_, MD_, or LK_ prefix in this context.\n",
            "help:".green().bold()
        ));
    }

    output
}

/// Formats an ImportNotFound error.
fn format_import_not_found_error(
    path: &Path,
    searched_paths: &[std::path::PathBuf],
    import_chain: &[ImportStep],
) -> String {
    let mut output = String::new();

    // Import chain (if present)
    output.push_str(&format_import_chain(import_chain));

    output.push_str(&format!(
        "{} Import file not found: {}\n",
        "Error:".red().bold(),
        path.display().to_string().yellow()
    ));

    if !searched_paths.is_empty() {
        output.push_str("\nSearched paths:\n");
        for p in searched_paths {
            output.push_str(&format!("  - {}\n", p.display()));
        }
    }

    output.push_str(&format!(
        "\n{} Make sure the file exists and the path is correct.\n",
        "help:".green().bold()
    ));
    output.push_str("      Import paths are resolved relative to the importing file.\n");

    output
}

/// Formats a CircularImport error.
fn format_circular_import_error(chain: &[std::path::PathBuf]) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "{} Circular import detected:\n\n",
        "Error:".red().bold()
    ));

    for (i, path) in chain.iter().enumerate() {
        output.push_str(&format!("  {}. {}", i + 1, path.display()));
        if i < chain.len() - 1 {
            output.push_str(&format!(" {}\n", "→".blue()));
        } else {
            output.push('\n');
        }
    }

    output.push_str(&format!(
        "\n{} Remove the circular dependency by restructuring your imports.\n",
        "help:".green().bold()
    ));
    output.push_str("      Consider extracting common code to a separate module\n");
    output.push_str("      that both files can import from.\n");

    output
}

/// Formats a ResourceLimitExceeded error.
fn format_resource_limit_error(limit_type: &str, import_chain: &[ImportStep]) -> String {
    let mut output = String::new();

    // Import chain (if present)
    output.push_str(&format_import_chain(import_chain));

    output.push_str(&format!(
        "{} Resource limit exceeded: {}\n",
        "Error:".red().bold(),
        limit_type.yellow()
    ));
    output.push_str(&format!(
        "\n{} Your script is too complex. Consider simplifying or breaking it into smaller parts.\n",
        "help:".green().bold()
    ));
    output.push_str("      - Reduce nesting depth\n");
    output.push_str("      - Split large configurations into multiple files using imports\n");
    output.push_str("      - Reduce the total number of mappings\n");

    output
}

/// Formats a ParseError in a user-friendly format with code snippets and suggestions.
///
/// This is a legacy function kept for backwards compatibility.
/// Use `format_error()` for the new colored output with code snippets.
#[allow(dead_code)]
pub fn format_error_user_friendly(error: &ParseError) -> String {
    match error {
        ParseError::SyntaxError {
            file,
            line,
            column,
            message,
            import_chain: _,
        } => {
            format!(
                "{}:{}:{}: Syntax error: {}\n\n\
                 Help: Check your Rhai script syntax at the indicated location.",
                file.display(),
                line,
                column,
                message
            )
        }
        ParseError::InvalidPrefix {
            expected,
            got,
            context,
            import_chain: _,
        } => format_invalid_prefix_suggestion(expected, got, context),
        ParseError::ModifierIdOutOfRange {
            got,
            max,
            import_chain: _,
        } => {
            format!(
                "Modifier ID out of range: {} (valid range: MD_00 to MD_{:02X})\n\n\
                 Help: Custom modifier IDs must be in the range 00-{:02X} (0-{}).",
                got, max, max, max
            )
        }
        ParseError::LockIdOutOfRange {
            got,
            max,
            import_chain: _,
        } => {
            format!(
                "Lock ID out of range: {} (valid range: LK_00 to LK_{:02X})\n\n\
                 Help: Custom lock IDs must be in the range 00-{:02X} (0-{}).",
                got, max, max, max
            )
        }
        ParseError::PhysicalModifierInMD {
            name,
            import_chain: _,
        } => {
            format!(
                "Physical modifier name '{}' cannot be used with MD_ prefix.\n\n\
                 Physical modifiers (LShift, RShift, LCtrl, RCtrl, LAlt, RAlt, LMeta, RMeta)\n\
                 should be used directly without prefixes in input contexts, or with VK_ in output contexts.\n\n\
                 For custom modifiers, use MD_00 through MD_FE.\n\n\
                 Example: map(\"CapsLock\", \"MD_00\")  // CapsLock becomes custom modifier 00",
                name
            )
        }
        ParseError::MissingPrefix {
            key,
            context,
            import_chain: _,
        } => format_missing_prefix_suggestion(key, context),
        ParseError::ImportNotFound {
            path,
            searched_paths,
            import_chain: _,
        } => {
            let mut msg = format!("Import file not found: {}\n", path.display());
            if !searched_paths.is_empty() {
                msg.push_str("\nSearched paths:\n");
                for p in searched_paths {
                    msg.push_str(&format!("  - {}\n", p.display()));
                }
            }
            msg.push_str("\nHelp: Make sure the file exists and the path is correct.");
            msg
        }
        ParseError::CircularImport { chain } => {
            let mut msg = String::from("Circular import detected:\n");
            for (i, path) in chain.iter().enumerate() {
                msg.push_str(&format!("  {}. {}", i + 1, path.display()));
                if i < chain.len() - 1 {
                    msg.push_str(" →\n");
                }
            }
            msg.push_str("\n\nHelp: Remove the circular dependency by restructuring your imports.");
            msg
        }
        ParseError::ResourceLimitExceeded {
            limit_type,
            import_chain: _,
        } => {
            format!(
                "Resource limit exceeded: {}\n\n\
                 Help: Your script is too complex. Consider simplifying or breaking it into smaller parts.",
                limit_type
            )
        }
    }
}

#[allow(dead_code)] // Will be used in error formatting tasks (task 19+)
fn format_invalid_prefix_suggestion(expected: &str, got: &str, context: &str) -> String {
    if got.starts_with("MD_") {
        format!(
            "Unknown key prefix: {} (use MD_00 through MD_FE for custom modifiers)\n\n\
             Example: Instead of 'MD_LShift', use 'MD_00' for a custom modifier.",
            got
        )
    } else if got.starts_with("VK_") && context.contains("hold") {
        format!(
            "tap_hold hold parameter must have MD_ prefix, got: {}\n\n\
             Example: tap_hold(\"Space\", \"VK_Space\", \"MD_00\", 200)",
            got
        )
    } else {
        format!(
            "Invalid prefix: expected {}, got '{}' (context: {})\n\n\
             Valid prefixes:\n\
             - VK_ for virtual keys (e.g., VK_A, VK_Enter)\n\
             - MD_ for custom modifiers (e.g., MD_00, MD_01)\n\
             - LK_ for custom locks (e.g., LK_00, LK_01)",
            expected, got, context
        )
    }
}

#[allow(dead_code)] // Will be used in error formatting tasks (task 19+)
fn format_missing_prefix_suggestion(key: &str, context: &str) -> String {
    if context.contains("output") || context.contains("to") {
        format!(
            "Output must have VK_, MD_, or LK_ prefix: {} → use VK_{} for virtual key\n\n\
             Examples:\n\
             - map(\"A\", \"VK_B\")        // Remap A to B (virtual key)\n\
             - map(\"CapsLock\", \"MD_00\") // CapsLock acts as custom modifier 00\n\
             - map(\"ScrollLock\", \"LK_00\") // ScrollLock toggles custom lock 00",
            key, key
        )
    } else {
        format!(
            "Missing prefix for key '{}' (context: {})\n\n\
             Use VK_ for virtual keys, MD_ for modifiers, LK_ for locks.",
            key, context
        )
    }
}

/// Formats a ParseError as a JSON object for machine consumption.
#[allow(dead_code)] // Will be used in CLI tasks (task 16+)
pub fn format_error_json(error: &ParseError) -> String {
    match error {
        ParseError::SyntaxError {
            file,
            line,
            column,
            message,
            import_chain: _,
        } => {
            serde_json::json!({
                "error_code": "E001",
                "error_type": "SyntaxError",
                "message": message,
                "file": file.to_string_lossy(),
                "line": line,
                "column": column,
                "suggestion": "Check your Rhai script syntax at the indicated location."
            })
            .to_string()
        }
        ParseError::InvalidPrefix {
            expected,
            got,
            context,
            import_chain: _,
        } => {
            let suggestion = if got.starts_with("MD_") && !got.chars().nth(3).is_some_and(|c| c.is_ascii_hexdigit()) {
                format!("Use MD_00 through MD_FE for custom modifiers, not physical modifier names like '{}'", got)
            } else if got.starts_with("VK_") && context.contains("hold") {
                "tap_hold hold parameter must have MD_ prefix for custom modifiers".to_string()
            } else {
                "Use VK_ for virtual keys, MD_ for custom modifiers (00-FE), LK_ for custom locks (00-FE)".to_string()
            };

            serde_json::json!({
                "error_code": "E002",
                "error_type": "InvalidPrefix",
                "message": format!("Invalid prefix: expected {}, got '{}'", expected, got),
                "expected": expected,
                "got": got,
                "context": context,
                "suggestion": suggestion
            }).to_string()
        }
        ParseError::ModifierIdOutOfRange {
            got,
            max,
            import_chain: _,
        } => {
            serde_json::json!({
                "error_code": "E003",
                "error_type": "ModifierIdOutOfRange",
                "message": format!("Modifier ID {} is out of valid range", got),
                "got": got,
                "max": max,
                "valid_range": format!("MD_00 to MD_{:02X}", max),
                "suggestion": format!("Use a modifier ID between 00 and {:02X} ({} in decimal)", max, max)
            })
            .to_string()
        }
        ParseError::LockIdOutOfRange {
            got,
            max,
            import_chain: _,
        } => {
            serde_json::json!({
                "error_code": "E004",
                "error_type": "LockIdOutOfRange",
                "message": format!("Lock ID {} is out of valid range", got),
                "got": got,
                "max": max,
                "valid_range": format!("LK_00 to LK_{:02X}", max),
                "suggestion": format!("Use a lock ID between 00 and {:02X} ({} in decimal)", max, max)
            })
            .to_string()
        }
        ParseError::PhysicalModifierInMD {
            name,
            import_chain: _,
        } => {
            serde_json::json!({
                "error_code": "E005",
                "error_type": "PhysicalModifierInMD",
                "message": format!("Physical modifier name '{}' cannot be used with MD_ prefix", name),
                "physical_modifier": name,
                "suggestion": "Use MD_00 through MD_FE for custom modifiers. Physical modifiers (LShift, RShift, etc.) should not have MD_ prefix."
            })
            .to_string()
        }
        _ => format_remaining_error_json(error),
    }
}

#[allow(dead_code)] // Will be used in CLI tasks (task 16+)
fn format_remaining_error_json(error: &ParseError) -> String {
    match error {
        ParseError::MissingPrefix {
            key,
            context,
            import_chain: _,
        } => {
            let suggestion = if context.contains("output") || context.contains("to") {
                format!("Add prefix to '{}': use VK_{} for virtual key, MD_XX for custom modifier, or LK_XX for custom lock", key, key)
            } else {
                "Keys must have VK_, MD_, or LK_ prefix in this context".to_string()
            };

            serde_json::json!({
                "error_code": "E006",
                "error_type": "MissingPrefix",
                "message": format!("Missing prefix for key '{}'", key),
                "key": key,
                "context": context,
                "suggestion": suggestion
            })
            .to_string()
        }
        ParseError::ImportNotFound {
            path,
            searched_paths,
            import_chain: _,
        } => {
            serde_json::json!({
                "error_code": "E007",
                "error_type": "ImportNotFound",
                "message": format!("Import file not found: {}", path.display()),
                "path": path.to_string_lossy(),
                "searched_paths": searched_paths.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
                "suggestion": "Make sure the file exists and the path is correct"
            })
            .to_string()
        }
        ParseError::CircularImport { chain } => {
            serde_json::json!({
                "error_code": "E008",
                "error_type": "CircularImport",
                "message": "Circular import detected",
                "import_chain": chain.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
                "suggestion": "Remove the circular dependency by restructuring your imports"
            })
            .to_string()
        }
        ParseError::ResourceLimitExceeded {
            limit_type,
            import_chain: _,
        } => {
            serde_json::json!({
                "error_code": "E009",
                "error_type": "ResourceLimitExceeded",
                "message": format!("Resource limit exceeded: {}", limit_type),
                "limit_type": limit_type,
                "suggestion": "Simplify your script or break it into smaller parts"
            })
            .to_string()
        }
        _ => unreachable!(),
    }
}
