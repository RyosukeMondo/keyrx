//! Structured logging configuration for KeyRx daemon
//!
//! This module provides JSON-formatted structured logging using the tracing crate.
//! Log format: {timestamp, level, service, event, context}
//!
//! IMPORTANT: Never log secrets, PII, or sensitive data
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize structured JSON logging
///
/// Reads log level from RUST_LOG environment variable.
/// Defaults to "info" if not set.
///
/// Format example:
/// ```json
/// {
///   "timestamp": "2026-02-01T12:00:00.000Z",
///   "level": "INFO",
///   "target": "keyrx_daemon",
///   "message": "Server started",
///   "fields": {
///     "port": 9867,
///     "version": "0.1.0"
///   }
/// }
/// ```
pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let fmt_layer = fmt::layer()
        .json()
        .with_target(true)
        .with_current_span(true)
        .with_span_list(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .flatten_event(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    tracing::info!(
        service = "keyrx-daemon",
        event = "logging_initialized",
        "Structured logging initialized"
    );
}

/// Initialize CLI-friendly logging (human-readable, not JSON)
///
/// Used for CLI commands where JSON output would interfere with user experience.
pub fn init_cli() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warn"))
        .unwrap();

    let fmt_layer = fmt::layer()
        .compact()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

/// Initialize logging for tests (compact, errors only)
#[cfg(test)]
pub fn init_test() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("error")
        .with_test_writer()
        .compact()
        .try_init();
}

/// Helper macro for performance measurement
#[macro_export]
macro_rules! measure {
    ($name:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        tracing::info!(
            event = concat!($name, "_completed"),
            duration_ms = duration.as_millis() as u64,
            duration_us = duration.as_micros() as u64,
        );
        result
    }};
}

/// Helper macro for error logging with context
#[macro_export]
macro_rules! log_error {
    ($error:expr, $($field:tt)*) => {
        tracing::error!(
            error = %$error,
            error_type = std::any::type_name_of_val(&$error),
            $($field)*
        )
    };
}

/// Sanitize sensitive data before logging
///
/// Redacts fields that might contain sensitive information
pub fn sanitize_context<T: serde::Serialize>(value: T) -> serde_json::Value {
    let json = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
    sanitize_json(json)
}

fn sanitize_json(mut value: serde_json::Value) -> serde_json::Value {
    match &mut value {
        serde_json::Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                let key_lower = key.to_lowercase();
                if is_sensitive_key(&key_lower) {
                    *val = serde_json::Value::String("[REDACTED]".to_string());
                } else {
                    *val = sanitize_json(val.clone());
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                *item = sanitize_json(item.clone());
            }
        }
        _ => {}
    }
    value
}

fn is_sensitive_key(key: &str) -> bool {
    const SENSITIVE_KEYS: &[&str] = &[
        "password",
        "secret",
        "token",
        "apikey",
        "api_key",
        "authorization",
        "auth",
        "privatekey",
        "private_key",
        "sessionid",
        "session_id",
        "cookie",
        "credentials",
    ];

    SENSITIVE_KEYS.contains(&key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sanitize_password() {
        let data = json!({
            "username": "user",
            "password": "secret123"
        });

        let sanitized = sanitize_json(data);
        assert_eq!(sanitized["username"], "user");
        assert_eq!(sanitized["password"], "[REDACTED]");
    }

    #[test]
    fn test_sanitize_nested() {
        let data = json!({
            "user": {
                "id": "123",
                "apiKey": "sk-123456"
            }
        });

        let sanitized = sanitize_json(data);
        assert_eq!(sanitized["user"]["id"], "123");
        assert_eq!(sanitized["user"]["apiKey"], "[REDACTED]");
    }

    #[test]
    fn test_sanitize_array() {
        let data = json!([
            {"username": "user1", "token": "abc"},
            {"username": "user2", "secret": "xyz"}
        ]);

        let sanitized = sanitize_json(data);
        assert_eq!(sanitized[0]["username"], "user1");
        assert_eq!(sanitized[0]["token"], "[REDACTED]");
        assert_eq!(sanitized[1]["username"], "user2");
        assert_eq!(sanitized[1]["secret"], "[REDACTED]");
    }

    #[test]
    fn test_safe_data_not_redacted() {
        let data = json!({
            "userId": "123",
            "action": "create",
            "timestamp": 1234567890
        });

        let sanitized = sanitize_json(data.clone());
        assert_eq!(sanitized, data);
    }
}
