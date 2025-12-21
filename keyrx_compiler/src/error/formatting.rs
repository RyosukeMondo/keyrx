use crate::error::types::ParseError;

/// Helper function to encode bytes as hex string.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

/// Formats a ParseError in a user-friendly format with code snippets and suggestions.
#[allow(dead_code)] // Will be used in error formatting tasks (task 19+)
pub fn format_error_user_friendly(error: &ParseError) -> String {
    match error {
        ParseError::SyntaxError {
            file,
            line,
            column,
            message,
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
        } => format_invalid_prefix_suggestion(expected, got, context),
        ParseError::ModifierIdOutOfRange { got, max } => {
            format!(
                "Modifier ID out of range: {} (valid range: MD_00 to MD_{:02X})\n\n\
                 Help: Custom modifier IDs must be in the range 00-{:02X} (0-{}).",
                got, max, max, max
            )
        }
        ParseError::LockIdOutOfRange { got, max } => {
            format!(
                "Lock ID out of range: {} (valid range: LK_00 to LK_{:02X})\n\n\
                 Help: Custom lock IDs must be in the range 00-{:02X} (0-{}).",
                got, max, max, max
            )
        }
        ParseError::PhysicalModifierInMD { name } => {
            format!(
                "Physical modifier name '{}' cannot be used with MD_ prefix.\n\n\
                 Physical modifiers (LShift, RShift, LCtrl, RCtrl, LAlt, RAlt, LMeta, RMeta)\n\
                 should be used directly without prefixes in input contexts, or with VK_ in output contexts.\n\n\
                 For custom modifiers, use MD_00 through MD_FE.\n\n\
                 Example: map(\"CapsLock\", \"MD_00\")  // CapsLock becomes custom modifier 00",
                name
            )
        }
        ParseError::MissingPrefix { key, context } => {
            format_missing_prefix_suggestion(key, context)
        }
        ParseError::ImportNotFound {
            path,
            searched_paths,
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
        ParseError::ResourceLimitExceeded { limit_type } => {
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
        ParseError::SyntaxError { file, line, column, message } => {
            serde_json::json!({
                "error_code": "E001",
                "error_type": "SyntaxError",
                "message": message,
                "file": file.to_string_lossy(),
                "line": line,
                "column": column,
                "suggestion": "Check your Rhai script syntax at the indicated location."
            }).to_string()
        }
        ParseError::InvalidPrefix { expected, got, context } => {
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
        ParseError::ModifierIdOutOfRange { got, max } => {
            serde_json::json!({
                "error_code": "E003",
                "error_type": "ModifierIdOutOfRange",
                "message": format!("Modifier ID {} is out of valid range", got),
                "got": got,
                "max": max,
                "valid_range": format!("MD_00 to MD_{:02X}", max),
                "suggestion": format!("Use a modifier ID between 00 and {:02X} ({} in decimal)", max, max)
            }).to_string()
        }
        ParseError::LockIdOutOfRange { got, max } => {
            serde_json::json!({
                "error_code": "E004",
                "error_type": "LockIdOutOfRange",
                "message": format!("Lock ID {} is out of valid range", got),
                "got": got,
                "max": max,
                "valid_range": format!("LK_00 to LK_{:02X}", max),
                "suggestion": format!("Use a lock ID between 00 and {:02X} ({} in decimal)", max, max)
            }).to_string()
        }
        ParseError::PhysicalModifierInMD { name } => {
            serde_json::json!({
                "error_code": "E005",
                "error_type": "PhysicalModifierInMD",
                "message": format!("Physical modifier name '{}' cannot be used with MD_ prefix", name),
                "physical_modifier": name,
                "suggestion": "Use MD_00 through MD_FE for custom modifiers. Physical modifiers (LShift, RShift, etc.) should not have MD_ prefix."
            }).to_string()
        }
        _ => format_remaining_error_json(error),
    }
}

#[allow(dead_code)] // Will be used in CLI tasks (task 16+)
fn format_remaining_error_json(error: &ParseError) -> String {
    match error {
        ParseError::MissingPrefix { key, context } => {
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
            }).to_string()
        }
        ParseError::ImportNotFound { path, searched_paths } => {
            serde_json::json!({
                "error_code": "E007",
                "error_type": "ImportNotFound",
                "message": format!("Import file not found: {}", path.display()),
                "path": path.to_string_lossy(),
                "searched_paths": searched_paths.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
                "suggestion": "Make sure the file exists and the path is correct"
            }).to_string()
        }
        ParseError::CircularImport { chain } => {
            serde_json::json!({
                "error_code": "E008",
                "error_type": "CircularImport",
                "message": "Circular import detected",
                "import_chain": chain.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
                "suggestion": "Remove the circular dependency by restructuring your imports"
            }).to_string()
        }
        ParseError::ResourceLimitExceeded { limit_type } => {
            serde_json::json!({
                "error_code": "E009",
                "error_type": "ResourceLimitExceeded",
                "message": format!("Resource limit exceeded: {}", limit_type),
                "limit_type": limit_type,
                "suggestion": "Simplify your script or break it into smaller parts"
            }).to_string()
        }
        _ => unreachable!(),
    }
}
