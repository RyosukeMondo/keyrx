//! Structured log entry types for KeyRx.
//!
//! This module provides structured log entry types with FFI-compatible
//! representations for bridging logs to the Flutter UI.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Log level enum compatible with tracing levels.
///
/// Serializes to uppercase strings (TRACE, DEBUG, INFO, WARN, ERROR) for
/// structured logging compliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[repr(C)]
pub enum LogLevel {
    /// Trace-level logging (most verbose).
    Trace = 0,
    /// Debug-level logging.
    Debug = 1,
    /// Info-level logging.
    Info = 2,
    /// Warning-level logging.
    Warn = 3,
    /// Error-level logging (least verbose).
    Error = 4,
}

impl Serialize for LogLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<&tracing::Level> for LogLevel {
    fn from(level: &tracing::Level) -> Self {
        match *level {
            tracing::Level::TRACE => LogLevel::Trace,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::ERROR => LogLevel::Error,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Structured log entry containing all required fields.
///
/// Fields are serialized with compliant names for structured logging:
/// - `timestamp`: ISO 8601 format (e.g., "2025-12-12T10:30:45.123Z")
/// - `level`: Uppercase (TRACE, DEBUG, INFO, WARN, ERROR)
/// - `service`: Service/module name (serialized from `target`)
/// - `event`: Event description (serialized from `message`)
///
/// # Example
/// ```rust
/// use keyrx_core::observability::entry::{LogEntry, LogLevel};
///
/// let entry = LogEntry::new(
///     LogLevel::Info,
///     "keyrx::engine",
///     "Engine started successfully",
/// );
///
/// assert_eq!(entry.level, LogLevel::Info);
/// assert_eq!(entry.target, "keyrx::engine");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// ISO 8601 timestamp when the log was created.
    #[serde(serialize_with = "serialize_timestamp")]
    pub timestamp: DateTime<Utc>,

    /// Log level (TRACE, DEBUG, INFO, WARN, ERROR).
    pub level: LogLevel,

    /// Service/module that generated the log (serializes as "service").
    #[serde(rename = "service")]
    pub target: String,

    /// Event description (serializes as "event").
    #[serde(rename = "event")]
    pub message: String,

    /// Optional structured context fields.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub fields: HashMap<String, String>,

    /// Optional span name if logged within a span.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<String>,
}

/// Serialize DateTime to ISO 8601 string format.
fn serialize_timestamp<S>(timestamp: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

impl LogEntry {
    /// Create a new log entry with the specified level, target, and message.
    ///
    /// # Arguments
    /// * `level` - Log level
    /// * `target` - Target module/component (serializes as "service")
    /// * `message` - Log message (serializes as "event")
    pub fn new(level: LogLevel, target: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            target: target.into(),
            message: message.into(),
            fields: HashMap::new(),
            span: None,
        }
    }

    /// Create a log entry with a specific timestamp.
    ///
    /// # Arguments
    /// * `timestamp` - The timestamp for this log entry
    /// * `level` - Log level
    /// * `target` - Target module/component
    /// * `message` - Log message
    pub fn with_timestamp(
        timestamp: DateTime<Utc>,
        level: LogLevel,
        target: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            timestamp,
            level,
            target: target.into(),
            message: message.into(),
            fields: HashMap::new(),
            span: None,
        }
    }

    /// Add a context field to the log entry.
    ///
    /// # Arguments
    /// * `key` - Field name
    /// * `value` - Field value
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Set the span name for this log entry.
    ///
    /// # Arguments
    /// * `span` - Span name
    pub fn with_span(mut self, span: impl Into<String>) -> Self {
        self.span = Some(span.into());
        self
    }

    /// Serialize this log entry to a JSON string.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convert this log entry to an FFI-compatible representation.
    ///
    /// The caller is responsible for freeing the returned `CLogEntry`
    /// using `c_log_entry_free`.
    pub fn to_ffi(&self) -> Result<CLogEntry, std::ffi::NulError> {
        let json = self.to_json().unwrap_or_else(|_| String::from("{}"));

        Ok(CLogEntry {
            timestamp: self.timestamp.timestamp_millis() as u64,
            level: self.level,
            target: CString::new(self.target.clone())?.into_raw(),
            message: CString::new(self.message.clone())?.into_raw(),
            json: CString::new(json)?.into_raw(),
        })
    }

    /// Get the timestamp as Unix milliseconds for FFI compatibility.
    pub fn timestamp_millis(&self) -> u64 {
        self.timestamp.timestamp_millis() as u64
    }
}

/// FFI-compatible log entry representation.
///
/// This structure can be safely passed across the FFI boundary.
/// All string fields are C-style null-terminated strings.
///
/// # Memory Management
/// The caller is responsible for freeing this structure using
/// `c_log_entry_free` to prevent memory leaks.
#[repr(C)]
pub struct CLogEntry {
    /// Unix timestamp in milliseconds.
    pub timestamp: u64,
    /// Log level.
    pub level: LogLevel,
    /// Target module (owned C string).
    pub target: *mut c_char,
    /// Log message (owned C string).
    pub message: *mut c_char,
    /// Full JSON representation (owned C string).
    pub json: *mut c_char,
}

/// Free a C log entry and its owned strings.
///
/// # Safety
/// - `entry` must be a valid pointer to a `CLogEntry` created by `to_ffi()`
/// - This function must be called exactly once per `CLogEntry`
/// - After calling this function, the pointer becomes invalid
#[no_mangle]
#[allow(unsafe_code)]
pub unsafe extern "C" fn c_log_entry_free(entry: *mut CLogEntry) {
    if entry.is_null() {
        return;
    }

    let entry = Box::from_raw(entry);

    // Free the C strings
    if !entry.target.is_null() {
        let _ = CString::from_raw(entry.target);
    }
    if !entry.message.is_null() {
        let _ = CString::from_raw(entry.message);
    }
    if !entry.json.is_null() {
        let _ = CString::from_raw(entry.json);
    }

    // Box will be dropped here
}

/// Create a log entry from FFI components.
///
/// # Arguments
/// * `timestamp` - Unix timestamp in milliseconds
/// * `level` - Log level
/// * `target` - Target module/component (null-terminated C string)
/// * `message` - Log message (null-terminated C string)
///
/// # Safety
/// - `target` and `message` must be valid null-terminated C strings
/// - The returned pointer must be freed with `c_log_entry_free`
#[no_mangle]
#[allow(unsafe_code)]
pub unsafe extern "C" fn c_log_entry_create(
    timestamp: u64,
    level: LogLevel,
    target: *const c_char,
    message: *const c_char,
) -> *mut CLogEntry {
    if target.is_null() || message.is_null() {
        return std::ptr::null_mut();
    }

    let target_str = match CStr::from_ptr(target).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let message_str = match CStr::from_ptr(message).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    // Convert timestamp from milliseconds to DateTime<Utc>
    let datetime = DateTime::from_timestamp_millis(timestamp as i64).unwrap_or_else(Utc::now);
    let entry = LogEntry::with_timestamp(datetime, level, target_str, message_str);

    match entry.to_ffi() {
        Ok(c_entry) => Box::into_raw(Box::new(c_entry)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_tracing() {
        assert_eq!(LogLevel::from(&tracing::Level::TRACE), LogLevel::Trace);
        assert_eq!(LogLevel::from(&tracing::Level::DEBUG), LogLevel::Debug);
        assert_eq!(LogLevel::from(&tracing::Level::INFO), LogLevel::Info);
        assert_eq!(LogLevel::from(&tracing::Level::WARN), LogLevel::Warn);
        assert_eq!(LogLevel::from(&tracing::Level::ERROR), LogLevel::Error);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Trace.to_string(), "TRACE");
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Warn.to_string(), "WARN");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Info, "test::module", "Test message");

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.target, "test::module");
        assert_eq!(entry.message, "Test message");
        assert!(entry.fields.is_empty());
        assert!(entry.span.is_none());
        // Timestamp should be recent (within last minute)
        assert!(entry.timestamp_millis() > 0);
        let now = Utc::now().timestamp_millis() as u64;
        assert!(entry.timestamp_millis() <= now);
        assert!(entry.timestamp_millis() >= now - 60000);
    }

    #[test]
    fn test_log_entry_with_fields() {
        let entry = LogEntry::new(LogLevel::Debug, "test", "Message")
            .with_field("key1", "value1")
            .with_field("key2", "value2");

        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields.get("key1"), Some(&"value1".to_string()));
        assert_eq!(entry.fields.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_log_entry_with_span() {
        let entry = LogEntry::new(LogLevel::Info, "test", "Message").with_span("test_span");

        assert_eq!(entry.span, Some("test_span".to_string()));
    }

    #[test]
    fn test_log_entry_to_json() {
        let entry =
            LogEntry::new(LogLevel::Info, "test", "Test message").with_field("user_id", "123");

        let json = entry.to_json().expect("Failed to serialize");

        // Verify compliant field names
        assert!(
            json.contains("\"level\":\"INFO\""),
            "Level should be uppercase"
        );
        assert!(
            json.contains("\"service\":\"test\""),
            "target should be renamed to service"
        );
        assert!(
            json.contains("\"event\":\"Test message\""),
            "message should be renamed to event"
        );
        assert!(json.contains("\"fields\""));
        assert!(json.contains("\"user_id\":\"123\""));
        // Verify ISO 8601 timestamp format
        assert!(
            json.contains("\"timestamp\":\"202"),
            "Timestamp should be ISO 8601 string"
        );
        assert!(json.contains("T"), "Timestamp should have T separator");
        assert!(json.contains("Z\""), "Timestamp should end with Z");
    }

    #[test]
    fn test_log_entry_json_compliance() {
        // Test full compliance with requirements 3.1-3.7
        let entry = LogEntry::new(LogLevel::Warn, "keyrx::engine", "device_connected")
            .with_field("device_id", "usb:1234:5678")
            .with_field("component", "device_registry");

        let json = entry.to_json().expect("Failed to serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Valid JSON");

        // Req 3.1: JSON format - verified by successful parse above
        // Req 3.2: ISO 8601 timestamp
        let ts = parsed["timestamp"].as_str().expect("timestamp is string");
        assert!(ts.ends_with('Z'), "Timestamp should be UTC");
        assert!(ts.contains('T'), "Timestamp should have T separator");
        // Req 3.3: Level field
        assert_eq!(parsed["level"], "WARN");
        // Req 3.4: Service field (was target)
        assert_eq!(parsed["service"], "keyrx::engine");
        // Req 3.5: Event field (was message)
        assert_eq!(parsed["event"], "device_connected");
        // Req 3.6: Context fields
        assert_eq!(parsed["fields"]["device_id"], "usb:1234:5678");
        assert_eq!(parsed["fields"]["component"], "device_registry");
    }

    #[test]
    fn test_log_entry_to_ffi() {
        let entry = LogEntry::new(LogLevel::Error, "test::ffi", "FFI test");

        let c_entry = entry.to_ffi().expect("Failed to convert to FFI");

        assert_eq!(c_entry.level, LogLevel::Error);
        assert_eq!(c_entry.timestamp, entry.timestamp_millis());

        // Verify strings are valid
        unsafe {
            assert!(!c_entry.target.is_null());
            assert!(!c_entry.message.is_null());
            assert!(!c_entry.json.is_null());

            let target_str = CStr::from_ptr(c_entry.target)
                .to_str()
                .expect("Invalid target");
            assert_eq!(target_str, "test::ffi");

            let message_str = CStr::from_ptr(c_entry.message)
                .to_str()
                .expect("Invalid message");
            assert_eq!(message_str, "FFI test");

            // Clean up
            let _ = CString::from_raw(c_entry.target);
            let _ = CString::from_raw(c_entry.message);
            let _ = CString::from_raw(c_entry.json);
        }
    }

    #[test]
    fn test_c_log_entry_create_and_free() {
        unsafe {
            let target = CString::new("test").unwrap();
            let message = CString::new("test message").unwrap();

            let entry = c_log_entry_create(
                1234567890,
                LogLevel::Warn,
                target.as_ptr(),
                message.as_ptr(),
            );

            assert!(!entry.is_null());

            let entry_ref = &*entry;
            assert_eq!(entry_ref.timestamp, 1234567890);
            assert_eq!(entry_ref.level, LogLevel::Warn);

            c_log_entry_free(entry);
        }
    }

    #[test]
    fn test_c_log_entry_create_with_null_returns_null() {
        unsafe {
            let target = CString::new("test").unwrap();

            let entry = c_log_entry_create(
                1234567890,
                LogLevel::Info,
                target.as_ptr(),
                std::ptr::null(),
            );

            assert!(entry.is_null());
        }
    }
}
