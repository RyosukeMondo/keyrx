//! Legacy plain-text error formatting, kept for backwards compatibility.
//!
//! `format_error_user_friendly()` predates the colored `format_error()` and is
//! still used by callers that want an uncolored, single-string message.

use crate::error::types::ParseError;

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
        ParseError::SourceUnreadable { path, reason } => {
            format!("Cannot use {} as a script: {}", path.display(), reason)
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
