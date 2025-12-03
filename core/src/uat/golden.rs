//! Golden session recording and verification.
//!
//! Golden sessions are baseline recordings of expected behavior that can be
//! replayed and compared against to detect regressions. They capture input
//! events and expected outputs in a human-readable JSON format.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::drivers::keycodes::KeyCode;
use crate::scripting::{get_pending_inputs, reset_test_context, RhaiRuntime, TestHarness};
use crate::traits::ScriptRuntime;

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

/// Manager for golden session operations.
#[derive(Debug)]
pub struct GoldenSessionManager {
    /// Directory containing golden session files.
    golden_dir: PathBuf,
}

impl GoldenSessionManager {
    /// Create a new golden session manager with the default directory (`tests/golden/`).
    pub fn new() -> Self {
        Self {
            golden_dir: PathBuf::from("tests/golden"),
        }
    }

    /// Create a new golden session manager with a custom directory.
    pub fn with_dir(golden_dir: impl Into<PathBuf>) -> Self {
        Self {
            golden_dir: golden_dir.into(),
        }
    }

    /// Get the path to a golden session file.
    pub fn session_path(&self, name: &str) -> PathBuf {
        self.golden_dir.join(format!("{}.krx", name))
    }

    /// Get the golden directory path.
    pub fn golden_dir(&self) -> &PathBuf {
        &self.golden_dir
    }

    /// Validate a session name format.
    ///
    /// Valid names must:
    /// - Be non-empty
    /// - Contain only alphanumeric characters, underscores, and hyphens
    /// - Start with a letter or underscore
    /// - Be at most 64 characters
    pub fn validate_name(name: &str) -> Result<(), GoldenSessionError> {
        if name.is_empty() {
            return Err(GoldenSessionError::invalid_name(
                name,
                "name cannot be empty",
            ));
        }
        if name.len() > 64 {
            return Err(GoldenSessionError::invalid_name(
                name,
                "name cannot exceed 64 characters",
            ));
        }
        // Check first character (we already verified name is non-empty)
        let first_char = match name.chars().next() {
            Some(c) => c,
            None => {
                return Err(GoldenSessionError::invalid_name(
                    name,
                    "name cannot be empty",
                ))
            }
        };
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return Err(GoldenSessionError::invalid_name(
                name,
                "name must start with a letter or underscore",
            ));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(GoldenSessionError::invalid_name(
                name,
                "name can only contain letters, numbers, underscores, and hyphens",
            ));
        }
        Ok(())
    }

    /// Record a golden session by executing a script.
    ///
    /// Executes the script at `script_path`, captures input events and outputs,
    /// and saves the session to `tests/golden/<name>.krx` as JSON.
    ///
    /// # Arguments
    /// * `name` - The session name (must be valid per `validate_name`)
    /// * `script_path` - Path to the Rhai script that generates test events
    ///
    /// # Returns
    /// A `RecordResult` with recording statistics, or an error if recording fails.
    pub fn record(
        &self,
        name: &str,
        script_path: &str,
    ) -> Result<RecordResult, GoldenSessionError> {
        // Validate the session name
        Self::validate_name(name)?;

        let start = Instant::now();

        // Initialize test harness and runtime
        reset_test_context();
        let harness = TestHarness::new();
        let mut runtime =
            RhaiRuntime::new().map_err(|e| GoldenSessionError::ScriptError(e.to_string()))?;
        harness.register_functions(runtime.engine_mut());

        // Load and execute the script
        runtime
            .load_file(script_path)
            .map_err(|e| GoldenSessionError::ScriptError(e.to_string()))?;
        runtime
            .run_script()
            .map_err(|e| GoldenSessionError::ScriptError(e.to_string()))?;

        // Sync outputs from engine to test context
        harness.sync_outputs();

        // Capture events from test context
        let inputs = get_pending_inputs();
        let context = harness.context_snapshot();

        // Build the golden session
        let mut session = GoldenSession::new(name);

        // Convert input events to golden events
        for input in &inputs {
            let event = GoldenEvent {
                event_type: if input.pressed {
                    GoldenEventType::KeyPress
                } else {
                    GoldenEventType::KeyRelease
                },
                key: input.key,
                time_us: input.timestamp_us,
            };
            session.add_event(event);
        }

        // Convert outputs to expected outputs
        for (index, output) in context.outputs.iter().enumerate() {
            let output_str = format!("{:?}", output);
            session.add_expected_output(ExpectedOutput {
                event_index: index,
                output: output_str,
                timing_range_us: None,
            });
        }

        // Ensure the golden directory exists
        fs::create_dir_all(&self.golden_dir)?;

        // Serialize and save
        let path = self.session_path(name);
        let json = session.to_json()?;
        fs::write(&path, json)?;

        let duration_us = start.elapsed().as_micros() as u64;

        Ok(RecordResult {
            session_name: name.to_string(),
            path,
            event_count: session.events.len(),
            duration_us,
        })
    }

    /// Load a golden session from disk.
    pub fn load(&self, name: &str) -> Result<GoldenSession, GoldenSessionError> {
        let path = self.session_path(name);
        if !path.exists() {
            return Err(GoldenSessionError::NotFound(name.to_string()));
        }
        let json = fs::read_to_string(&path)?;
        let session = GoldenSession::from_json(&json)?;
        Ok(session)
    }

    /// List all golden sessions in the directory.
    pub fn list_sessions(&self) -> Result<Vec<String>, GoldenSessionError> {
        if !self.golden_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        for entry in fs::read_dir(&self.golden_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "krx") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    sessions.push(name.to_string());
                }
            }
        }
        sessions.sort();
        Ok(sessions)
    }

    /// Check if a golden session exists.
    pub fn session_exists(&self, name: &str) -> bool {
        self.session_path(name).exists()
    }
}

impl Default for GoldenSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

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
    fn golden_session_manager_default_dir() {
        let manager = GoldenSessionManager::new();
        assert_eq!(manager.golden_dir, PathBuf::from("tests/golden"));
    }

    #[test]
    fn golden_session_manager_custom_dir() {
        let manager = GoldenSessionManager::with_dir("/custom/path");
        assert_eq!(manager.golden_dir, PathBuf::from("/custom/path"));
    }

    #[test]
    fn golden_session_manager_session_path() {
        let manager = GoldenSessionManager::new();
        let path = manager.session_path("my_session");
        assert_eq!(path, PathBuf::from("tests/golden/my_session.krx"));
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

    // Tests for session name validation
    #[test]
    fn validate_name_accepts_valid_names() {
        assert!(GoldenSessionManager::validate_name("basic_typing").is_ok());
        assert!(GoldenSessionManager::validate_name("test123").is_ok());
        assert!(GoldenSessionManager::validate_name("_private").is_ok());
        assert!(GoldenSessionManager::validate_name("layer-switch").is_ok());
        assert!(GoldenSessionManager::validate_name("Test_Name-123").is_ok());
    }

    #[test]
    fn validate_name_rejects_empty() {
        let result = GoldenSessionManager::validate_name("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, GoldenSessionError::InvalidName { .. }));
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn validate_name_rejects_long_names() {
        let long_name = "a".repeat(65);
        let result = GoldenSessionManager::validate_name(&long_name);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("64 characters"));
    }

    #[test]
    fn validate_name_rejects_invalid_start() {
        let result = GoldenSessionManager::validate_name("123test");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("start with a letter"));

        let result = GoldenSessionManager::validate_name("-test");
        assert!(result.is_err());
    }

    #[test]
    fn validate_name_rejects_invalid_chars() {
        let result = GoldenSessionManager::validate_name("test name");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("only contain"));

        let result = GoldenSessionManager::validate_name("test.name");
        assert!(result.is_err());

        let result = GoldenSessionManager::validate_name("test/name");
        assert!(result.is_err());
    }

    // Tests for session error types
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

    // Tests for session existence and listing
    #[test]
    fn session_exists_returns_false_for_missing() {
        let manager = GoldenSessionManager::with_dir("/nonexistent/path");
        assert!(!manager.session_exists("test"));
    }

    #[test]
    fn list_sessions_returns_empty_for_missing_dir() {
        let manager = GoldenSessionManager::with_dir("/nonexistent/path");
        let sessions = manager.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn load_returns_not_found_for_missing() {
        let manager = GoldenSessionManager::with_dir("/nonexistent/path");
        let result = manager.load("missing");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GoldenSessionError::NotFound(_)
        ));
    }

    // Test RecordResult
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
}
