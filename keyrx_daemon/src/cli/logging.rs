//! Structured JSON logging for CLI commands.
//!
//! Provides type-safe, structured logging in JSON format for CLI operations.
//! All logs follow the schema:
//! `{"timestamp":"...", "level":"...", "service":"keyrx_daemon", "event_type":"...", "context":{...}}`
//!
//! # Important
//!
//! This module is for **logging** diagnostic information to stderr or log files.
//! It is separate from **user output** which goes to stdout via the `common` module.
//! Do not mix logging with user-facing output.

use log::{debug, error, info, warn};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current Unix timestamp in ISO 8601 format.
///
/// Note: This uses a simplified calendar calculation and may be slightly
/// inaccurate for edge cases (leap years, month boundaries). For production
/// use, consider using the `chrono` crate for precise timestamps.
fn current_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| {
            let secs = d.as_secs();
            let nanos = d.subsec_nanos();
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                1970 + secs / 31_557_600,
                ((secs % 31_557_600) / 2_629_800) + 1,
                ((secs % 2_629_800) / 86400) + 1,
                (secs % 86400) / 3600,
                (secs % 3600) / 60,
                secs % 60,
                nanos / 1_000_000
            )
        })
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00.000Z"))
}

/// Logs when a CLI command execution starts.
pub fn log_command_start(command: &str, args: &str) {
    info!(
        r#"{{"timestamp":"{}","level":"INFO","service":"keyrx_daemon","event_type":"command_start","context":{{"command":"{}","args":"{}"}}}}"#,
        current_timestamp(),
        escape_json(command),
        escape_json(args)
    );
}

/// Logs when a CLI command execution completes successfully.
pub fn log_command_success(command: &str, duration_ms: u64) {
    info!(
        r#"{{"timestamp":"{}","level":"INFO","service":"keyrx_daemon","event_type":"command_success","context":{{"command":"{}","duration_ms":{}}}}}"#,
        current_timestamp(),
        escape_json(command),
        duration_ms
    );
}

/// Logs when a CLI command execution fails.
pub fn log_command_error(command: &str, error: &str) {
    error!(
        r#"{{"timestamp":"{}","level":"ERROR","service":"keyrx_daemon","event_type":"command_error","context":{{"command":"{}","error":"{}"}}}}"#,
        current_timestamp(),
        escape_json(command),
        escape_json(error)
    );
}

/// Logs profile activation events.
pub fn log_profile_activate(profile_name: &str, success: bool) {
    if success {
        info!(
            r#"{{"timestamp":"{}","level":"INFO","service":"keyrx_daemon","event_type":"profile_activate","context":{{"profile":"{}","success":true}}}}"#,
            current_timestamp(),
            escape_json(profile_name)
        );
    } else {
        warn!(
            r#"{{"timestamp":"{}","level":"WARN","service":"keyrx_daemon","event_type":"profile_activate","context":{{"profile":"{}","success":false}}}}"#,
            current_timestamp(),
            escape_json(profile_name)
        );
    }
}

/// Logs profile creation events.
pub fn log_profile_create(profile_name: &str, layer_count: usize) {
    info!(
        r#"{{"timestamp":"{}","level":"INFO","service":"keyrx_daemon","event_type":"profile_create","context":{{"profile":"{}","layer_count":{}}}}}"#,
        current_timestamp(),
        escape_json(profile_name),
        layer_count
    );
}

/// Logs profile deletion events.
pub fn log_profile_delete(profile_name: &str) {
    info!(
        r#"{{"timestamp":"{}","level":"INFO","service":"keyrx_daemon","event_type":"profile_delete","context":{{"profile":"{}"}}}}"#,
        current_timestamp(),
        escape_json(profile_name)
    );
}

/// Logs configuration validation events.
pub fn log_config_validate(profile_name: &str, success: bool, error: Option<&str>) {
    if success {
        info!(
            r#"{{"timestamp":"{}","level":"INFO","service":"keyrx_daemon","event_type":"config_validate","context":{{"profile":"{}","success":true}}}}"#,
            current_timestamp(),
            escape_json(profile_name)
        );
    } else {
        error!(
            r#"{{"timestamp":"{}","level":"ERROR","service":"keyrx_daemon","event_type":"config_validate","context":{{"profile":"{}","success":false,"error":"{}"}}}}"#,
            current_timestamp(),
            escape_json(profile_name),
            escape_json(error.unwrap_or("unknown error"))
        );
    }
}

/// Logs configuration changes (key mappings, etc.).
pub fn log_config_change(profile_name: &str, change_type: &str, key: &str, layer: &str) {
    debug!(
        r#"{{"timestamp":"{}","level":"DEBUG","service":"keyrx_daemon","event_type":"config_change","context":{{"profile":"{}","change_type":"{}","key":"{}","layer":"{}"}}}}"#,
        current_timestamp(),
        escape_json(profile_name),
        escape_json(change_type),
        escape_json(key),
        escape_json(layer)
    );
}

/// Logs layer operations.
pub fn log_layer_operation(profile_name: &str, operation: &str, layer_name: &str) {
    debug!(
        r#"{{"timestamp":"{}","level":"DEBUG","service":"keyrx_daemon","event_type":"layer_operation","context":{{"profile":"{}","operation":"{}","layer":"{}"}}}}"#,
        current_timestamp(),
        escape_json(profile_name),
        escape_json(operation),
        escape_json(layer_name)
    );
}

/// Logs device management operations.
pub fn log_device_operation(operation: &str, device_path: &str) {
    debug!(
        r#"{{"timestamp":"{}","level":"DEBUG","service":"keyrx_daemon","event_type":"device_operation","context":{{"operation":"{}","device":"{}"}}}}"#,
        current_timestamp(),
        escape_json(operation),
        escape_json(device_path)
    );
}

/// Escapes special characters in JSON strings.
fn escape_json(s: &str) -> String {
    s.replace('\\', r"\\")
        .replace('"', r#"\""#)
        .replace('\n', r"\n")
        .replace('\r', r"\r")
        .replace('\t', r"\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_format() {
        let ts = current_timestamp();
        // Should match ISO 8601 format: YYYY-MM-DDTHH:MM:SS.sssZ
        assert!(ts.contains('T'));
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 24);
    }

    #[test]
    fn test_escape_json_basic() {
        assert_eq!(escape_json("hello"), "hello");
    }

    #[test]
    fn test_escape_json_quotes() {
        assert_eq!(escape_json(r#"hello "world""#), r#"hello \"world\""#);
    }

    #[test]
    fn test_escape_json_backslash() {
        assert_eq!(escape_json(r"C:\path\to\file"), r"C:\\path\\to\\file");
    }

    #[test]
    fn test_escape_json_newline() {
        assert_eq!(escape_json("line1\nline2"), r"line1\nline2");
    }

    #[test]
    fn test_escape_json_tab() {
        assert_eq!(escape_json("col1\tcol2"), r"col1\tcol2");
    }
}
