//! JSON implementation of the CLI output formatter.

use super::formatter::OutputFormatter;
use serde::Serialize;

fn to_pretty_json(value: serde_json::Value) -> String {
    match serde_json::to_string_pretty(&value) {
        Ok(text) => text,
        Err(err) => serde_json::json!({
            "status": "error",
            "message": "failed to serialize formatter output",
            "detail": err.to_string()
        })
        .to_string(),
    }
}

/// Formatter that renders CLI output as pretty-printed JSON.
pub struct JsonFormatter;

impl JsonFormatter {
    /// Create a new JSON formatter.
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_success(&self, message: &str) -> String {
        to_pretty_json(serde_json::json!({
            "status": "success",
            "message": message
        }))
    }

    fn format_error(&self, message: &str) -> String {
        to_pretty_json(serde_json::json!({
            "status": "error",
            "message": message
        }))
    }

    fn format_warning(&self, message: &str) -> String {
        to_pretty_json(serde_json::json!({
            "status": "warning",
            "message": message
        }))
    }

    fn format_data<T: Serialize + ?Sized>(&self, data: &T) -> serde_json::Result<String> {
        serde_json::to_string_pretty(data)
    }
}
