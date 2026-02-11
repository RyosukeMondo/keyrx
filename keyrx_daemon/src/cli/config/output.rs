//! Layer 3: Output formatting and serialization.

use serde::Serialize;

/// JSON output for set-key operations.
#[derive(Serialize)]
pub struct SetKeyOutput {
    pub success: bool,
    pub key: String,
    pub layer: String,
    pub profile: String,
    pub compile_time_ms: Option<u64>,
}

/// JSON output for get-key operations.
#[derive(Serialize)]
pub struct GetKeyOutput {
    pub key: String,
    pub layer: String,
    pub mapping: Option<String>,
}

/// JSON output for validation.
#[derive(Serialize)]
pub struct ValidationOutput {
    pub success: bool,
    pub profile: String,
    pub errors: Vec<String>,
}

/// JSON output for show command.
#[derive(Serialize)]
pub struct ShowOutput {
    pub profile: String,
    pub device_id: String,
    pub layers: Vec<String>,
    pub mapping_count: usize,
}

/// JSON output for diff command.
#[derive(Serialize)]
pub struct DiffOutput {
    pub profile1: String,
    pub profile2: String,
    pub differences: Vec<String>,
}

/// Formats output as JSON or human-readable text.
pub fn format_set_key_result(
    key: String,
    target: String,
    layer: String,
    profile: String,
    compile_time: u64,
    json: bool,
) -> String {
    if json {
        let output = SetKeyOutput {
            success: true,
            key,
            layer,
            profile,
            compile_time_ms: Some(compile_time),
        };
        serde_json::to_string(&output).unwrap_or_default()
    } else {
        format!(
            "✓ Set {} -> {} in layer '{}' of profile '{}'\n  Compiled in {}ms",
            key, target, layer, profile, compile_time
        )
    }
}

/// Formats validation output.
pub fn format_validation_result(
    profile: String,
    success: bool,
    error: Option<String>,
    json: bool,
) -> String {
    if json {
        let output = ValidationOutput {
            success,
            profile,
            errors: error.into_iter().collect(),
        };
        serde_json::to_string(&output).unwrap_or_default()
    } else if success {
        format!("✓ Profile '{}' is valid", profile)
    } else {
        format!(
            "✗ Profile '{}' validation failed:\n  {}",
            profile,
            error.unwrap_or_default()
        )
    }
}
