//! Golden session output comparison functions.
//!
//! Contains functions for comparing expected outputs from golden sessions
//! against actual outputs, with support for semantic comparison that ignores
//! non-deterministic values like timestamps.

use crate::engine::OutputAction;

use super::golden_types::{DifferenceType, ExpectedOutput, GoldenDifference};

/// Compare expected outputs from a golden session against actual outputs.
///
/// Performs semantic comparison, ignoring non-deterministic values like
/// exact timestamps. Returns a list of differences found.
pub fn compare_outputs(
    expected: &[ExpectedOutput],
    actual: &[OutputAction],
) -> Vec<GoldenDifference> {
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
pub fn outputs_match(expected: &str, actual: &str) -> bool {
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
pub fn remove_timestamps(s: &str) -> String {
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
pub fn find_number_end(s: &str, start: usize) -> usize {
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
}
