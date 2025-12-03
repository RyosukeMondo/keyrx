//! Golden session type definitions.
//!
//! Contains all data structures and enums for golden session recording,
//! verification, and error handling.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::drivers::keycodes::KeyCode;

/// Current schema version for golden session format.
pub const GOLDEN_SESSION_VERSION: &str = "1.0";

/// A golden session recording.
///
/// Captures a sequence of input events and their expected outputs for
/// regression testing. The format is designed to be human-readable JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenSession {
    /// Session name (unique identifier).
    pub name: String,

    /// Schema version for format compatibility.
    #[serde(default = "default_version")]
    pub version: String,

    /// Creation timestamp (ISO 8601 format).
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created: DateTime<Utc>,

    /// Session metadata (description, requirements, etc.).
    pub metadata: GoldenSessionMetadata,

    /// Recorded input events.
    pub events: Vec<GoldenEvent>,

    /// Expected outputs for verification.
    pub expected_outputs: Vec<ExpectedOutput>,
}

fn default_version() -> String {
    GOLDEN_SESSION_VERSION.to_string()
}

/// Metadata for a golden session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoldenSessionMetadata {
    /// Human-readable description of what this session tests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Linked requirement IDs (for traceability).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<String>,

    /// Additional custom tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// A recorded input event in a golden session.
///
/// Simplified representation of an input event for golden sessions,
/// focusing on the key data needed for replay and verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenEvent {
    /// Event type ("key_press" or "key_release").
    #[serde(rename = "type")]
    pub event_type: GoldenEventType,

    /// Key code (human-readable name).
    pub key: KeyCode,

    /// Timestamp in microseconds from session start.
    pub time_us: u64,
}

/// Type of golden event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoldenEventType {
    /// Key press (key down).
    KeyPress,
    /// Key release (key up).
    KeyRelease,
}

/// Expected output for verification.
///
/// Associates an output expectation with an event index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutput {
    /// Index of the event that triggers this output.
    pub event_index: usize,

    /// Expected output character or string.
    pub output: String,

    /// Acceptable timing range in microseconds [min, max].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timing_range_us: Option<[u64; 2]>,
}

/// Result of golden session verification.
#[derive(Debug, Clone)]
pub struct GoldenVerifyResult {
    /// Whether verification passed.
    pub passed: bool,

    /// List of differences found.
    pub differences: Vec<GoldenDifference>,

    /// Session name that was verified.
    pub session_name: String,

    /// Duration of verification in microseconds.
    pub duration_us: u64,
}

/// A difference found during golden verification.
#[derive(Debug, Clone)]
pub struct GoldenDifference {
    /// Event index where difference occurred.
    pub event_index: usize,

    /// Type of difference.
    pub diff_type: DifferenceType,

    /// Expected value.
    pub expected: String,

    /// Actual value.
    pub actual: String,
}

/// Type of difference found during verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifferenceType {
    /// Output content mismatch.
    OutputMismatch,
    /// Missing expected output.
    MissingOutput,
    /// Extra unexpected output.
    ExtraOutput,
    /// Timing outside acceptable range.
    TimingViolation,
}

impl std::fmt::Display for DifferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DifferenceType::OutputMismatch => write!(f, "output mismatch"),
            DifferenceType::MissingOutput => write!(f, "missing output"),
            DifferenceType::ExtraOutput => write!(f, "extra output"),
            DifferenceType::TimingViolation => write!(f, "timing violation"),
        }
    }
}

/// Error type for golden session operations.
#[derive(Debug, Error)]
pub enum GoldenSessionError {
    /// Invalid session name format.
    #[error("Invalid session name '{name}': {reason}")]
    InvalidName { name: String, reason: String },

    /// Script execution failed.
    #[error("Script execution failed: {0}")]
    ScriptError(String),

    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Session not found.
    #[error("Golden session not found: {0}")]
    NotFound(String),

    /// Update requires confirmation.
    #[error("Update requires confirmation: use --confirm flag to update '{0}'")]
    ConfirmationRequired(String),
}

impl GoldenSessionError {
    /// Create an invalid name error.
    pub fn invalid_name(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidName {
            name: name.into(),
            reason: reason.into(),
        }
    }
}

/// Result of recording a golden session.
#[derive(Debug)]
pub struct RecordResult {
    /// Name of the recorded session.
    pub session_name: String,
    /// Path where the session was saved.
    pub path: PathBuf,
    /// Number of events recorded.
    pub event_count: usize,
    /// Duration of recording in microseconds.
    pub duration_us: u64,
}

/// Result of updating a golden session.
#[derive(Debug)]
pub struct UpdateResult {
    /// Name of the updated session.
    pub session_name: String,
    /// Path where the session was saved.
    pub path: PathBuf,
    /// Number of events in the updated session.
    pub event_count: usize,
    /// Duration of the update operation in microseconds.
    pub duration_us: u64,
    /// Previous event count (for comparison).
    pub previous_event_count: usize,
}

// === Impl blocks ===

impl GoldenSession {
    /// Create a new golden session.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: GOLDEN_SESSION_VERSION.to_string(),
            created: Utc::now(),
            metadata: GoldenSessionMetadata::default(),
            events: Vec::new(),
            expected_outputs: Vec::new(),
        }
    }

    /// Create a golden session with metadata.
    pub fn with_metadata(name: impl Into<String>, metadata: GoldenSessionMetadata) -> Self {
        Self {
            name: name.into(),
            version: GOLDEN_SESSION_VERSION.to_string(),
            created: Utc::now(),
            metadata,
            events: Vec::new(),
            expected_outputs: Vec::new(),
        }
    }

    /// Add an event to the session.
    pub fn add_event(&mut self, event: GoldenEvent) {
        self.events.push(event);
    }

    /// Add an expected output.
    pub fn add_expected_output(&mut self, output: ExpectedOutput) {
        self.expected_outputs.push(output);
    }

    /// Serialize to human-readable JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl GoldenEvent {
    /// Create a key press event.
    pub fn key_press(key: KeyCode, time_us: u64) -> Self {
        Self {
            event_type: GoldenEventType::KeyPress,
            key,
            time_us,
        }
    }

    /// Create a key release event.
    pub fn key_release(key: KeyCode, time_us: u64) -> Self {
        Self {
            event_type: GoldenEventType::KeyRelease,
            key,
            time_us,
        }
    }
}

impl GoldenVerifyResult {
    /// Create a passing result.
    pub fn passed(session_name: impl Into<String>, duration_us: u64) -> Self {
        Self {
            passed: true,
            differences: Vec::new(),
            session_name: session_name.into(),
            duration_us,
        }
    }

    /// Create a failing result with differences.
    pub fn failed(
        session_name: impl Into<String>,
        differences: Vec<GoldenDifference>,
        duration_us: u64,
    ) -> Self {
        Self {
            passed: false,
            differences,
            session_name: session_name.into(),
            duration_us,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_session_new_sets_defaults() {
        let session = GoldenSession::new("test_session");

        assert_eq!(session.name, "test_session");
        assert_eq!(session.version, GOLDEN_SESSION_VERSION);
        assert!(session.events.is_empty());
        assert!(session.expected_outputs.is_empty());
    }

    #[test]
    fn golden_session_serializes_to_readable_json() {
        let mut session = GoldenSession::new("basic_typing");
        session.metadata.description = Some("Basic key mapping test".to_string());
        session.metadata.requirements = vec!["1.1".to_string()];

        session.add_event(GoldenEvent::key_press(KeyCode::A, 0));
        session.add_event(GoldenEvent::key_release(KeyCode::A, 50000));

        session.add_expected_output(ExpectedOutput {
            event_index: 0,
            output: "a".to_string(),
            timing_range_us: Some([0, 1000]),
        });

        let json = session.to_json().unwrap();

        // Verify it's human-readable (contains newlines and indentation)
        assert!(json.contains('\n'));
        assert!(json.contains("  "));

        // Verify key fields are present
        assert!(json.contains("\"name\": \"basic_typing\""));
        assert!(json.contains("\"version\": \"1.0\""));
        assert!(json.contains("\"description\": \"Basic key mapping test\""));
        assert!(json.contains("\"key_press\""));
    }

    #[test]
    fn golden_session_roundtrip() {
        let mut session = GoldenSession::new("roundtrip_test");
        session.add_event(GoldenEvent::key_press(KeyCode::B, 100));
        session.add_expected_output(ExpectedOutput {
            event_index: 0,
            output: "b".to_string(),
            timing_range_us: None,
        });

        let json = session.to_json().unwrap();
        let restored = GoldenSession::from_json(&json).unwrap();

        assert_eq!(restored.name, session.name);
        assert_eq!(restored.version, session.version);
        assert_eq!(restored.events.len(), 1);
        assert_eq!(restored.expected_outputs.len(), 1);
    }

    #[test]
    fn golden_event_key_press() {
        let event = GoldenEvent::key_press(KeyCode::A, 1000);
        assert_eq!(event.event_type, GoldenEventType::KeyPress);
        assert_eq!(event.key, KeyCode::A);
        assert_eq!(event.time_us, 1000);
    }

    #[test]
    fn golden_event_key_release() {
        let event = GoldenEvent::key_release(KeyCode::A, 2000);
        assert_eq!(event.event_type, GoldenEventType::KeyRelease);
        assert_eq!(event.key, KeyCode::A);
        assert_eq!(event.time_us, 2000);
    }

    #[test]
    fn golden_verify_result_passed() {
        let result = GoldenVerifyResult::passed("test", 1000);
        assert!(result.passed);
        assert!(result.differences.is_empty());
        assert_eq!(result.session_name, "test");
        assert_eq!(result.duration_us, 1000);
    }

    #[test]
    fn golden_verify_result_failed() {
        let diffs = vec![GoldenDifference {
            event_index: 0,
            diff_type: DifferenceType::OutputMismatch,
            expected: "a".to_string(),
            actual: "b".to_string(),
        }];

        let result = GoldenVerifyResult::failed("test", diffs, 2000);
        assert!(!result.passed);
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.session_name, "test");
        assert_eq!(result.duration_us, 2000);
    }

    #[test]
    fn difference_type_display() {
        assert_eq!(
            DifferenceType::OutputMismatch.to_string(),
            "output mismatch"
        );
        assert_eq!(DifferenceType::MissingOutput.to_string(), "missing output");
        assert_eq!(DifferenceType::ExtraOutput.to_string(), "extra output");
        assert_eq!(
            DifferenceType::TimingViolation.to_string(),
            "timing violation"
        );
    }

    #[test]
    fn golden_session_metadata_skips_empty_fields() {
        let session = GoldenSession::new("minimal");
        let json = session.to_json().unwrap();

        // Empty optional fields should not appear in JSON
        assert!(!json.contains("\"description\""));
        assert!(!json.contains("\"requirements\""));
        assert!(!json.contains("\"tags\""));
    }

    #[test]
    fn expected_output_skips_none_timing() {
        let output = ExpectedOutput {
            event_index: 0,
            output: "a".to_string(),
            timing_range_us: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(!json.contains("timing_range_us"));
    }

    #[test]
    fn golden_session_error_display() {
        let err = GoldenSessionError::invalid_name("bad name", "contains spaces");
        assert!(err.to_string().contains("bad name"));
        assert!(err.to_string().contains("contains spaces"));

        let err = GoldenSessionError::ScriptError("syntax error".to_string());
        assert!(err.to_string().contains("syntax error"));

        let err = GoldenSessionError::NotFound("missing".to_string());
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn record_result_fields() {
        let result = RecordResult {
            session_name: "test".to_string(),
            path: PathBuf::from("tests/golden/test.krx"),
            event_count: 5,
            duration_us: 1000,
        };

        assert_eq!(result.session_name, "test");
        assert_eq!(result.path, PathBuf::from("tests/golden/test.krx"));
        assert_eq!(result.event_count, 5);
        assert_eq!(result.duration_us, 1000);
    }

    #[test]
    fn confirmation_required_error_display() {
        let err = GoldenSessionError::ConfirmationRequired("test_session".to_string());
        let msg = err.to_string();
        assert!(msg.contains("confirmation"));
        assert!(msg.contains("--confirm"));
        assert!(msg.contains("test_session"));
    }

    #[test]
    fn update_result_fields() {
        let result = UpdateResult {
            session_name: "updated".to_string(),
            path: PathBuf::from("tests/golden/updated.krx"),
            event_count: 10,
            duration_us: 5000,
            previous_event_count: 8,
        };

        assert_eq!(result.session_name, "updated");
        assert_eq!(result.path, PathBuf::from("tests/golden/updated.krx"));
        assert_eq!(result.event_count, 10);
        assert_eq!(result.duration_us, 5000);
        assert_eq!(result.previous_event_count, 8);
    }

    #[test]
    fn golden_difference_fields() {
        let diff = GoldenDifference {
            event_index: 5,
            diff_type: DifferenceType::OutputMismatch,
            expected: "expected_value".to_string(),
            actual: "actual_value".to_string(),
        };

        assert_eq!(diff.event_index, 5);
        assert_eq!(diff.diff_type, DifferenceType::OutputMismatch);
        assert_eq!(diff.expected, "expected_value");
        assert_eq!(diff.actual, "actual_value");
    }
}
