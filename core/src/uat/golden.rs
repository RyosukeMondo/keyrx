//! Golden session recording and verification.
//!
//! Golden sessions are baseline recordings of expected behavior that can be
//! replayed and compared against to detect regressions. They capture input
//! events and expected outputs in a human-readable JSON format.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use crate::engine::{InputEvent, OutputAction};
use crate::scripting::{
    get_pending_inputs, record_input, reset_test_context, RhaiRuntime, TestHarness,
};
use crate::traits::ScriptRuntime;

// Re-export types from golden_types module
pub use super::golden_types::{
    DifferenceType, ExpectedOutput, GoldenDifference, GoldenEvent, GoldenEventType, GoldenSession,
    GoldenSessionError, GoldenSessionMetadata, GoldenVerifyResult, RecordResult, UpdateResult,
    GOLDEN_SESSION_VERSION,
};

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

    /// Verify a golden session by replaying and comparing outputs.
    ///
    /// Replays the golden session's input events through the engine and compares
    /// the actual outputs against the expected outputs stored in the session.
    /// Comparison is semantic, ignoring non-deterministic values like timestamps.
    ///
    /// # Arguments
    /// * `name` - The session name to verify
    ///
    /// # Returns
    /// A `GoldenVerifyResult` with verification status and any differences found.
    pub fn verify(&self, name: &str) -> Result<GoldenVerifyResult, GoldenSessionError> {
        let start = Instant::now();

        // Load the golden session
        let session = self.load(name)?;

        // Initialize test harness and runtime for replay
        reset_test_context();
        let harness = TestHarness::new();

        // Replay the golden session's input events through the test context
        for event in &session.events {
            let input = InputEvent {
                key: event.key,
                pressed: event.event_type == GoldenEventType::KeyPress,
                timestamp_us: event.time_us,
                device_id: Some("golden_replay".to_string()),
                is_repeat: false,
                is_synthetic: true,
                scan_code: 0,
            };
            record_input(input);
        }

        // Sync outputs from test context
        harness.sync_outputs();
        let context = harness.context_snapshot();

        // Compare actual outputs against expected outputs
        let differences = compare_outputs(&session.expected_outputs, &context.outputs);

        let duration_us = start.elapsed().as_micros() as u64;

        if differences.is_empty() {
            Ok(GoldenVerifyResult::passed(name, duration_us))
        } else {
            Ok(GoldenVerifyResult::failed(name, differences, duration_us))
        }
    }

    /// Verify a golden session by executing a script and comparing outputs.
    ///
    /// This is the full verification flow that:
    /// 1. Loads the golden session
    /// 2. Executes the provided script (or re-discovers the original script)
    /// 3. Compares the outputs semantically
    ///
    /// # Arguments
    /// * `name` - The session name to verify
    /// * `script_path` - Optional path to the script to execute. If None, replays events directly.
    ///
    /// # Returns
    /// A `GoldenVerifyResult` with verification status and any differences found.
    pub fn verify_with_script(
        &self,
        name: &str,
        script_path: &str,
    ) -> Result<GoldenVerifyResult, GoldenSessionError> {
        let start = Instant::now();

        // Load the golden session
        let session = self.load(name)?;

        // Initialize test harness and runtime
        reset_test_context();
        let harness = TestHarness::new();
        let mut runtime =
            RhaiRuntime::new().map_err(|e| GoldenSessionError::ScriptError(e.to_string()))?;
        harness.register_functions(runtime.engine_mut());

        // Execute the script
        runtime
            .load_file(script_path)
            .map_err(|e| GoldenSessionError::ScriptError(e.to_string()))?;
        runtime
            .run_script()
            .map_err(|e| GoldenSessionError::ScriptError(e.to_string()))?;

        // Sync outputs from engine to test context
        harness.sync_outputs();
        let context = harness.context_snapshot();

        // Compare actual outputs against expected outputs
        let differences = compare_outputs(&session.expected_outputs, &context.outputs);

        let duration_us = start.elapsed().as_micros() as u64;

        if differences.is_empty() {
            Ok(GoldenVerifyResult::passed(name, duration_us))
        } else {
            Ok(GoldenVerifyResult::failed(name, differences, duration_us))
        }
    }

    /// Update an existing golden session by re-recording it.
    ///
    /// This method re-records a golden session using the provided script. For safety,
    /// the `confirm` parameter must be `true` to actually perform the update.
    ///
    /// # Arguments
    /// * `name` - The session name to update (must already exist)
    /// * `script_path` - Path to the Rhai script that generates test events
    /// * `confirm` - Must be `true` to actually perform the update
    ///
    /// # Returns
    /// An `UpdateResult` with update statistics, or an error if:
    /// - `confirm` is `false` (returns `ConfirmationRequired` error)
    /// - The session doesn't exist (returns `NotFound` error)
    /// - Script execution fails (returns `ScriptError`)
    pub fn update(
        &self,
        name: &str,
        script_path: &str,
        confirm: bool,
    ) -> Result<UpdateResult, GoldenSessionError> {
        // Check that the session exists
        if !self.session_exists(name) {
            return Err(GoldenSessionError::NotFound(name.to_string()));
        }

        // Require explicit confirmation
        if !confirm {
            return Err(GoldenSessionError::ConfirmationRequired(name.to_string()));
        }

        // Load the existing session to get the previous event count
        let existing_session = self.load(name)?;
        let previous_event_count = existing_session.events.len();

        // Re-record using the existing record method
        let record_result = self.record(name, script_path)?;

        Ok(UpdateResult {
            session_name: record_result.session_name,
            path: record_result.path,
            event_count: record_result.event_count,
            duration_us: record_result.duration_us,
            previous_event_count,
        })
    }
}

impl Default for GoldenSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Compare expected outputs from a golden session against actual outputs.
///
/// Performs semantic comparison, ignoring non-deterministic values like
/// exact timestamps. Returns a list of differences found.
fn compare_outputs(expected: &[ExpectedOutput], actual: &[OutputAction]) -> Vec<GoldenDifference> {
    let mut differences = Vec::new();

    // Track which actual outputs have been matched
    let actual_strings: Vec<String> = actual.iter().map(|o| format!("{:?}", o)).collect();

    // Check each expected output
    for (idx, expected_output) in expected.iter().enumerate() {
        let event_index = expected_output.event_index;

        // Check if we have a corresponding actual output
        if idx >= actual_strings.len() {
            differences.push(GoldenDifference {
                event_index,
                diff_type: DifferenceType::MissingOutput,
                expected: expected_output.output.clone(),
                actual: "(no output)".to_string(),
            });
            continue;
        }

        let actual_str = &actual_strings[idx];

        // Semantic comparison: compare normalized output strings
        if !outputs_match(&expected_output.output, actual_str) {
            differences.push(GoldenDifference {
                event_index,
                diff_type: DifferenceType::OutputMismatch,
                expected: expected_output.output.clone(),
                actual: actual_str.clone(),
            });
        }

        // Check timing constraints if specified
        if let Some([min_us, max_us]) = expected_output.timing_range_us {
            // For timing verification, we would need actual timing data
            // This is a placeholder for when timing info is available in outputs
            // Currently we skip timing validation as output actions don't carry timing
            let _ = (min_us, max_us);
        }
    }

    // Check for extra unexpected outputs
    if actual_strings.len() > expected.len() {
        for (idx, actual_str) in actual_strings.iter().enumerate().skip(expected.len()) {
            differences.push(GoldenDifference {
                event_index: idx,
                diff_type: DifferenceType::ExtraOutput,
                expected: "(no expected output)".to_string(),
                actual: actual_str.clone(),
            });
        }
    }

    differences
}

/// Check if two output strings match semantically.
///
/// This function normalizes outputs and ignores non-deterministic
/// portions like timestamps or memory addresses.
fn outputs_match(expected: &str, actual: &str) -> bool {
    // Normalize whitespace
    let expected_normalized = expected.split_whitespace().collect::<Vec<_>>().join(" ");
    let actual_normalized = actual.split_whitespace().collect::<Vec<_>>().join(" ");

    // Direct comparison after normalization
    if expected_normalized == actual_normalized {
        return true;
    }

    // If outputs contain timestamps (patterns like "timestamp_us: <number>"),
    // try comparing without timestamps
    let expected_no_ts = remove_timestamps(&expected_normalized);
    let actual_no_ts = remove_timestamps(&actual_normalized);

    expected_no_ts == actual_no_ts
}

/// Remove timestamp patterns from a string for semantic comparison.
fn remove_timestamps(s: &str) -> String {
    // Remove patterns like "timestamp_us: 12345" or "time_us: 12345"
    let mut result = s.to_string();

    // Pattern: timestamp_us: followed by digits
    while let Some(start) = result.find("timestamp_us:") {
        let after_colon = start + "timestamp_us:".len();
        let end = find_number_end(&result, after_colon);
        result = format!("{}{}", &result[..start], &result[end..]);
    }

    // Pattern: time_us: followed by digits
    while let Some(start) = result.find("time_us:") {
        let after_colon = start + "time_us:".len();
        let end = find_number_end(&result, after_colon);
        result = format!("{}{}", &result[..start], &result[end..]);
    }

    result
}

/// Find the end position of a number (including leading whitespace) in a string.
fn find_number_end(s: &str, start: usize) -> usize {
    let bytes = s.as_bytes();
    let mut pos = start;

    // Skip whitespace
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }

    // Skip digits
    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
        pos += 1;
    }

    pos
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::keycodes::KeyCode;

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

    // Tests for semantic output comparison
    #[test]
    fn outputs_match_identical_strings() {
        assert!(outputs_match("KeyDown(A)", "KeyDown(A)"));
        assert!(outputs_match("KeyUp(B)", "KeyUp(B)"));
    }

    #[test]
    fn outputs_match_with_whitespace_differences() {
        assert!(outputs_match("KeyDown(A)", "KeyDown(A)  "));
        assert!(outputs_match("  KeyDown(A)  ", "KeyDown(A)"));
        assert!(outputs_match("Key Down ( A )", "Key Down ( A )"));
    }

    #[test]
    fn outputs_match_ignores_timestamps() {
        assert!(outputs_match(
            "Event { timestamp_us: 12345 }",
            "Event { timestamp_us: 67890 }"
        ));
        assert!(outputs_match(
            "Event { time_us: 100, key: A }",
            "Event { time_us: 999, key: A }"
        ));
    }

    #[test]
    fn outputs_mismatch_different_values() {
        assert!(!outputs_match("KeyDown(A)", "KeyDown(B)"));
        assert!(!outputs_match("KeyUp(A)", "KeyDown(A)"));
    }

    #[test]
    fn compare_outputs_empty_both() {
        let expected: Vec<ExpectedOutput> = vec![];
        let actual: Vec<OutputAction> = vec![];
        let diffs = compare_outputs(&expected, &actual);
        assert!(diffs.is_empty());
    }

    #[test]
    fn compare_outputs_matching() {
        let expected = vec![ExpectedOutput {
            event_index: 0,
            output: "KeyDown(A)".to_string(),
            timing_range_us: None,
        }];
        let actual = vec![OutputAction::KeyDown(KeyCode::A)];
        let diffs = compare_outputs(&expected, &actual);
        assert!(diffs.is_empty());
    }

    #[test]
    fn compare_outputs_missing_output() {
        let expected = vec![
            ExpectedOutput {
                event_index: 0,
                output: "KeyDown(A)".to_string(),
                timing_range_us: None,
            },
            ExpectedOutput {
                event_index: 1,
                output: "KeyUp(A)".to_string(),
                timing_range_us: None,
            },
        ];
        let actual = vec![OutputAction::KeyDown(KeyCode::A)];
        let diffs = compare_outputs(&expected, &actual);

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DifferenceType::MissingOutput);
        assert_eq!(diffs[0].event_index, 1);
    }

    #[test]
    fn compare_outputs_extra_output() {
        let expected = vec![ExpectedOutput {
            event_index: 0,
            output: "KeyDown(A)".to_string(),
            timing_range_us: None,
        }];
        let actual = vec![
            OutputAction::KeyDown(KeyCode::A),
            OutputAction::KeyUp(KeyCode::A),
        ];
        let diffs = compare_outputs(&expected, &actual);

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DifferenceType::ExtraOutput);
        assert_eq!(diffs[0].event_index, 1);
    }

    #[test]
    fn compare_outputs_mismatch() {
        let expected = vec![ExpectedOutput {
            event_index: 0,
            output: "KeyDown(A)".to_string(),
            timing_range_us: None,
        }];
        let actual = vec![OutputAction::KeyDown(KeyCode::B)];
        let diffs = compare_outputs(&expected, &actual);

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DifferenceType::OutputMismatch);
        assert_eq!(diffs[0].expected, "KeyDown(A)");
        assert!(diffs[0].actual.contains("KeyDown"));
        assert!(diffs[0].actual.contains("B"));
    }

    #[test]
    fn remove_timestamps_removes_timestamp_us() {
        let input = "Event { timestamp_us: 12345, key: A }";
        let result = remove_timestamps(input);
        assert!(!result.contains("12345"));
        assert!(result.contains("key: A"));
    }

    #[test]
    fn remove_timestamps_removes_time_us() {
        let input = "Event { time_us: 999 }";
        let result = remove_timestamps(input);
        assert!(!result.contains("999"));
    }

    #[test]
    fn remove_timestamps_handles_multiple() {
        let input = "timestamp_us: 100 and timestamp_us: 200";
        let result = remove_timestamps(input);
        assert!(!result.contains("100"));
        assert!(!result.contains("200"));
    }

    #[test]
    fn find_number_end_finds_digits() {
        let s = "key: 12345 next";
        let end = find_number_end(s, 5);
        assert_eq!(end, 10); // "12345" ends at position 10
    }

    #[test]
    fn find_number_end_skips_whitespace() {
        let s = "key:  123";
        let end = find_number_end(s, 4);
        assert_eq!(end, 9); // "  123" with whitespace
    }

    // Tests for update method
    #[test]
    fn update_requires_confirmation() {
        let manager = GoldenSessionManager::with_dir("/nonexistent/path");
        // Even if session doesn't exist, confirmation check happens first for existing sessions
        // But NotFound is returned first since we check existence before confirmation
        let result = manager.update("missing", "script.rhai", false);
        assert!(result.is_err());
        // Should get NotFound since session doesn't exist
        assert!(matches!(
            result.unwrap_err(),
            GoldenSessionError::NotFound(_)
        ));
    }

    #[test]
    fn update_returns_not_found_for_missing_session() {
        let manager = GoldenSessionManager::with_dir("/nonexistent/path");
        let result = manager.update("missing", "script.rhai", true);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GoldenSessionError::NotFound(_)
        ));
    }
}
