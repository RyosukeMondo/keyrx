//! JSON error formatting for machine consumption.

use crate::error::types::ParseError;

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
        ParseError::SourceUnreadable { path, reason } => serde_json::json!({
            "error_code": "E010",
            "error_type": "SourceUnreadable",
            "message": format!("Cannot use {} as a script: {}", path.display(), reason),
            "suggestion": "Save the file as UTF-8, under the size limit, and make sure it is readable"
        })
        .to_string(),
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
