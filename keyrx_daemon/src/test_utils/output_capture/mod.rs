//! Output capture for reading daemon's virtual keyboard events.
//!
//! This module provides [`OutputCapture`] for finding and reading events
//! from the daemon's virtual output keyboard device.
//!
//! # Usage
//!
//! ```ignore
//! use keyrx_daemon::test_utils::OutputCapture;
//! use std::time::Duration;
//!
//! // Find the daemon's output device (polls until found or timeout)
//! let capture = OutputCapture::find_by_name(
//!     "keyrx Virtual Keyboard",
//!     Duration::from_secs(5)
//! )?;
//!
//! // Device path is available for debugging
//! println!("Found device at: {}", capture.device_path());
//! ```
//!
//! # Requirements
//!
//! - Linux with evdev support
//! - Read access to `/dev/input/event*` devices (typically requires `input` group)

use keyrx_core::runtime::event::KeyEvent;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::OutputCapture;
#[cfg(target_os = "windows")]
pub use windows::OutputCapture;

/// A captured keyboard event with its keycode.
///
/// This is a convenience type alias for the core `KeyEvent` type,
/// used for test assertions and comparisons.
#[allow(dead_code)] // Type alias for documentation/clarity
pub type CapturedEvent = KeyEvent;

/// Result of comparing captured and expected events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventComparison {
    /// Event matched at this position.
    Match(KeyEvent),
    /// Event at this position differs: (captured, expected).
    Mismatch {
        captured: KeyEvent,
        expected: KeyEvent,
    },
    /// Extra event captured that wasn't expected.
    Extra(KeyEvent),
    /// Expected event that wasn't captured.
    Missing(KeyEvent),
}

/// Detailed result of event assertion.
#[derive(Debug, Clone)]
pub struct EventAssertionResult {
    /// Whether all events matched.
    pub passed: bool,
    /// Detailed comparison for each position.
    pub comparisons: Vec<EventComparison>,
    /// Number of matching events.
    pub matches: usize,
    /// Number of mismatched events.
    pub mismatches: usize,
    /// Number of extra events (captured but not expected).
    pub extras: usize,
    /// Number of missing events (expected but not captured).
    pub missing: usize,
}

impl EventAssertionResult {
    /// Creates a new assertion result by comparing captured and expected events.
    fn new(captured: &[KeyEvent], expected: &[KeyEvent]) -> Self {
        let mut comparisons = Vec::new();
        let mut matches = 0;
        let mut mismatches = 0;
        let mut extras = 0;
        let mut missing = 0;

        let max_len = captured.len().max(expected.len());

        for i in 0..max_len {
            match (captured.get(i), expected.get(i)) {
                (Some(c), Some(e)) if c == e => {
                    comparisons.push(EventComparison::Match(c.clone()));
                    matches += 1;
                }
                (Some(c), Some(e)) => {
                    comparisons.push(EventComparison::Mismatch {
                        captured: c.clone(),
                        expected: e.clone(),
                    });
                    mismatches += 1;
                }
                (Some(c), None) => {
                    comparisons.push(EventComparison::Extra(c.clone()));
                    extras += 1;
                }
                (None, Some(e)) => {
                    comparisons.push(EventComparison::Missing(e.clone()));
                    missing += 1;
                }
                (None, None) => unreachable!(),
            }
        }

        let passed = mismatches == 0 && extras == 0 && missing == 0;

        Self {
            passed,
            comparisons,
            matches,
            mismatches,
            extras,
            missing,
        }
    }

    /// Formats the assertion result as a detailed diff string.
    ///
    /// The output shows each position with markers:
    /// - `✓` for matches
    /// - `✗` for mismatches (shows both captured and expected)
    /// - `+` for extra captured events
    /// - `-` for missing expected events
    #[must_use]
    pub fn format_diff(&self) -> String {
        let mut output = String::new();

        // Summary line
        output.push_str(&format!(
            "Event assertion {}: {} matches, {} mismatches, {} extras, {} missing\n",
            if self.passed { "PASSED" } else { "FAILED" },
            self.matches,
            self.mismatches,
            self.extras,
            self.missing
        ));

        if self.comparisons.is_empty() {
            output.push_str("  (empty sequences)\n");
            return output;
        }

        output.push_str("\n  Idx  Status   Captured                         Expected\n");
        output.push_str("  ---  ------   --------                         --------\n");

        for (i, comparison) in self.comparisons.iter().enumerate() {
            match comparison {
                EventComparison::Match(event) => {
                    output.push_str(&format!(
                        "  {:3}  ✓ match  {:<32} {:<32}\n",
                        i,
                        format_event(event),
                        format_event(event)
                    ));
                }
                EventComparison::Mismatch { captured, expected } => {
                    output.push_str(&format!(
                        "  {:3}  ✗ diff   {:<32} {:<32}\n",
                        i,
                        format_event(captured),
                        format_event(expected)
                    ));
                }
                EventComparison::Extra(event) => {
                    output.push_str(&format!(
                        "  {:3}  + extra  {:<32} {:<32}\n",
                        i,
                        format_event(event),
                        "(none)"
                    ));
                }
                EventComparison::Missing(event) => {
                    output.push_str(&format!(
                        "  {:3}  - miss   {:<32} {:<32}\n",
                        i,
                        "(none)",
                        format_event(event)
                    ));
                }
            }
        }

        output
    }
}

/// Formats a KeyEvent for display in assertion output.
fn format_event(event: &KeyEvent) -> String {
    if event.is_press() {
        format!("Press({:?})", event.keycode())
    } else {
        format!("Release({:?})", event.keycode())
    }
}

/// Compares captured events against expected events.
///
/// This function performs a detailed comparison and returns a result
/// indicating whether the events match, along with a detailed diff.
///
/// # Arguments
///
/// * `captured` - Events that were actually captured
/// * `expected` - Events that were expected
///
/// # Returns
///
/// An [`EventAssertionResult`] containing the comparison details.
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::test_utils::{compare_events, KeyEvent};
/// use keyrx_core::config::KeyCode;
///
/// let captured = vec![
///     KeyEvent::Press(KeyCode::B),
///     KeyEvent::Release(KeyCode::B),
/// ];
/// let expected = vec![
///     KeyEvent::Press(KeyCode::B),
///     KeyEvent::Release(KeyCode::B),
/// ];
///
/// let result = compare_events(&captured, &expected);
/// assert!(result.passed);
/// ```
#[must_use]
pub fn compare_events(captured: &[KeyEvent], expected: &[KeyEvent]) -> EventAssertionResult {
    EventAssertionResult::new(captured, expected)
}

/// Asserts that captured events match expected events.
///
/// This function panics with a detailed diff if the events don't match,
/// making it suitable for use in tests.
///
/// # Arguments
///
/// * `captured` - Events that were actually captured
/// * `expected` - Events that were expected
///
/// # Panics
///
/// Panics with a detailed comparison diff if:
/// - Any event is in a different position than expected
/// - There are extra captured events
/// - There are missing expected events
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::test_utils::{assert_events, KeyEvent};
/// use keyrx_core::config::KeyCode;
///
/// let captured = vec![
///     KeyEvent::Press(KeyCode::B),
///     KeyEvent::Release(KeyCode::B),
/// ];
/// let expected = vec![
///     KeyEvent::Press(KeyCode::B),
///     KeyEvent::Release(KeyCode::B),
/// ];
///
/// // Passes - events match exactly
/// assert_events(&captured, &expected);
/// ```
///
/// # Failure Output
///
/// On failure, produces output like:
///
/// ```text
/// Event assertion FAILED: 1 matches, 1 mismatches, 0 extras, 0 missing
///
///   Idx  Status   Captured                         Expected
///   ---  ------   --------                         --------
///     0  ✓ match  Press(B)                         Press(B)
///     1  ✗ diff   Release(A)                       Release(B)
/// ```
pub fn assert_events(captured: &[KeyEvent], expected: &[KeyEvent]) {
    let result = compare_events(captured, expected);
    if !result.passed {
        panic!("\n{}", result.format_diff());
    }
}

/// Asserts that captured events match expected events, with a custom message.
///
/// Like [`assert_events`], but includes a custom message in the panic output.
///
/// # Arguments
///
/// * `captured` - Events that were actually captured
/// * `expected` - Events that were expected
/// * `msg` - Custom message to include in the panic output
///
/// # Panics
///
/// Panics with the custom message and a detailed comparison diff if events don't match.
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::test_utils::{assert_events_msg, KeyEvent};
/// use keyrx_core::config::KeyCode;
///
/// let captured = vec![KeyEvent::Press(KeyCode::A)];
/// let expected = vec![KeyEvent::Press(KeyCode::A)];
///
/// assert_events_msg(&captured, &expected, "Testing simple A key press");
/// ```
pub fn assert_events_msg(captured: &[KeyEvent], expected: &[KeyEvent], msg: &str) {
    let result = compare_events(captured, expected);
    if !result.passed {
        panic!("\n{}\n{}", msg, result.format_diff());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_core::config::KeyCode;

    #[test]
    fn test_compare_events_empty_sequences() {
        let result = compare_events(&[], &[]);
        assert!(result.passed);
        assert_eq!(result.matches, 0);
        assert_eq!(result.mismatches, 0);
        assert_eq!(result.extras, 0);
        assert_eq!(result.missing, 0);
        assert!(result.comparisons.is_empty());
    }

    #[test]
    fn test_compare_events_exact_match() {
        let captured = vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
            KeyEvent::Press(KeyCode::B),
            KeyEvent::Release(KeyCode::B),
        ];
        let expected = captured.clone();

        let result = compare_events(&captured, &expected);
        assert!(result.passed);
        assert_eq!(result.matches, 4);
        assert_eq!(result.mismatches, 0);
        assert_eq!(result.extras, 0);
        assert_eq!(result.missing, 0);

        // Verify all comparisons are matches
        for comparison in &result.comparisons {
            matches!(comparison, EventComparison::Match(_));
        }
    }

    #[test]
    fn test_compare_events_single_mismatch() {
        let captured = vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::B), // Mismatch here
        ];
        let expected = vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)];

        let result = compare_events(&captured, &expected);
        assert!(!result.passed);
        assert_eq!(result.matches, 1);
        assert_eq!(result.mismatches, 1);
        assert_eq!(result.extras, 0);
        assert_eq!(result.missing, 0);

        // Check the comparison details
        assert_eq!(
            result.comparisons[0],
            EventComparison::Match(KeyEvent::Press(KeyCode::A))
        );
        assert_eq!(
            result.comparisons[1],
            EventComparison::Mismatch {
                captured: KeyEvent::Release(KeyCode::B),
                expected: KeyEvent::Release(KeyCode::A),
            }
        );
    }

    #[test]
    fn test_compare_events_extra_captured() {
        let captured = vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
            KeyEvent::Press(KeyCode::B),   // Extra
            KeyEvent::Release(KeyCode::B), // Extra
        ];
        let expected = vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)];

        let result = compare_events(&captured, &expected);
        assert!(!result.passed);
        assert_eq!(result.matches, 2);
        assert_eq!(result.mismatches, 0);
        assert_eq!(result.extras, 2);
        assert_eq!(result.missing, 0);

        assert_eq!(
            result.comparisons[2],
            EventComparison::Extra(KeyEvent::Press(KeyCode::B))
        );
        assert_eq!(
            result.comparisons[3],
            EventComparison::Extra(KeyEvent::Release(KeyCode::B))
        );
    }

    #[test]
    fn test_compare_events_missing_expected() {
        let captured = vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)];
        let expected = vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
            KeyEvent::Press(KeyCode::B),   // Missing
            KeyEvent::Release(KeyCode::B), // Missing
        ];

        let result = compare_events(&captured, &expected);
        assert!(!result.passed);
        assert_eq!(result.matches, 2);
        assert_eq!(result.mismatches, 0);
        assert_eq!(result.extras, 0);
        assert_eq!(result.missing, 2);

        assert_eq!(
            result.comparisons[2],
            EventComparison::Missing(KeyEvent::Press(KeyCode::B))
        );
        assert_eq!(
            result.comparisons[3],
            EventComparison::Missing(KeyEvent::Release(KeyCode::B))
        );
    }

    #[test]
    fn test_compare_events_mixed_differences() {
        let captured = vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::B), // Mismatch
            KeyEvent::Press(KeyCode::C),   // Extra
        ];
        let expected = vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)];

        let result = compare_events(&captured, &expected);
        assert!(!result.passed);
        assert_eq!(result.matches, 1);
        assert_eq!(result.mismatches, 1);
        assert_eq!(result.extras, 1);
        assert_eq!(result.missing, 0);
    }

    #[test]
    fn test_assert_events_passes_on_match() {
        let captured = vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)];
        let expected = captured.clone();

        // Should not panic
        assert_events(&captured, &expected);
    }

    #[test]
    #[should_panic(expected = "Event assertion FAILED")]
    fn test_assert_events_panics_on_mismatch() {
        let captured = vec![KeyEvent::Press(KeyCode::A)];
        let expected = vec![KeyEvent::Press(KeyCode::B)];

        assert_events(&captured, &expected);
    }

    #[test]
    #[should_panic(expected = "Event assertion FAILED")]
    fn test_assert_events_panics_on_extra() {
        let captured = vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)];
        let expected = vec![KeyEvent::Press(KeyCode::A)];

        assert_events(&captured, &expected);
    }

    #[test]
    #[should_panic(expected = "Event assertion FAILED")]
    fn test_assert_events_panics_on_missing() {
        let captured = vec![KeyEvent::Press(KeyCode::A)];
        let expected = vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)];

        assert_events(&captured, &expected);
    }

    #[test]
    #[should_panic(expected = "Custom test message")]
    fn test_assert_events_msg_includes_message() {
        let captured = vec![KeyEvent::Press(KeyCode::A)];
        let expected = vec![KeyEvent::Press(KeyCode::B)];

        assert_events_msg(&captured, &expected, "Custom test message");
    }

    #[test]
    fn test_format_diff_empty_sequences() {
        let result = compare_events(&[], &[]);
        let diff = result.format_diff();

        assert!(diff.contains("PASSED"));
        assert!(diff.contains("0 matches"));
        assert!(diff.contains("(empty sequences)"));
    }

    #[test]
    fn test_format_diff_shows_all_markers() {
        let captured = vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::B), // Mismatch
            KeyEvent::Press(KeyCode::C),   // Extra
        ];
        let expected = vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
            KeyEvent::Press(KeyCode::D), // Will be missing from captured
            KeyEvent::Release(KeyCode::D), // Will be missing from captured
        ];

        let result = compare_events(&captured, &expected);
        let diff = result.format_diff();

        // Check that diff contains appropriate markers
        assert!(diff.contains("FAILED"));
        assert!(diff.contains("match")); // For the matching Press(A)
        assert!(diff.contains("diff")); // For the mismatch
        assert!(diff.contains("miss")); // For missing events
    }

    #[test]
    fn test_format_diff_column_alignment() {
        let captured = vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)];
        let expected = captured.clone();

        let result = compare_events(&captured, &expected);
        let diff = result.format_diff();

        // Check that the header and data columns are present
        assert!(diff.contains("Idx"));
        assert!(diff.contains("Status"));
        assert!(diff.contains("Captured"));
        assert!(diff.contains("Expected"));
    }

    #[test]
    fn test_event_comparison_equality() {
        let match1 = EventComparison::Match(KeyEvent::Press(KeyCode::A));
        let match2 = EventComparison::Match(KeyEvent::Press(KeyCode::A));
        let match3 = EventComparison::Match(KeyEvent::Press(KeyCode::B));

        assert_eq!(match1, match2);
        assert_ne!(match1, match3);

        let mismatch1 = EventComparison::Mismatch {
            captured: KeyEvent::Press(KeyCode::A),
            expected: KeyEvent::Press(KeyCode::B),
        };
        let mismatch2 = EventComparison::Mismatch {
            captured: KeyEvent::Press(KeyCode::A),
            expected: KeyEvent::Press(KeyCode::B),
        };

        assert_eq!(mismatch1, mismatch2);
    }

    #[test]
    fn test_format_event_helper() {
        assert_eq!(format_event(&KeyEvent::Press(KeyCode::A)), "Press(A)");
        assert_eq!(
            format_event(&KeyEvent::Release(KeyCode::Enter)),
            "Release(Enter)"
        );
    }

    #[test]
    fn test_compare_events_captures_key_types_correctly() {
        // Test with various key codes to ensure proper handling
        let captured = vec![
            KeyEvent::Press(KeyCode::LShift),
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
            KeyEvent::Release(KeyCode::LShift),
        ];
        let expected = captured.clone();

        let result = compare_events(&captured, &expected);
        assert!(result.passed);
        assert_eq!(result.matches, 4);
    }

    #[test]
    fn test_captured_event_type_alias() {
        // This just verifies the type alias compiles and works
        let event: CapturedEvent = KeyEvent::Press(KeyCode::D);
        assert_eq!(event, KeyEvent::Press(KeyCode::D));
    }
}
