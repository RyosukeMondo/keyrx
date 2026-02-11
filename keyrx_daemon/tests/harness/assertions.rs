//! E2E Test Assertions and Test Data Builders.
//!
//! This module provides test helper structs and methods for creating
//! test events and tracking test teardown results.

use std::fmt;
use std::time::{Duration, Instant};

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;

use super::config::E2EConfig;
use super::error::{E2EError, TestTimeoutPhase};
use super::harness::E2EHarness;

pub struct TeardownResult {
    /// Whether SIGTERM was successfully sent to the daemon.
    pub sigterm_sent: bool,
    /// Whether SIGKILL was needed (daemon didn't respond to SIGTERM).
    pub sigkill_sent: bool,
    /// Whether the daemon shut down gracefully (responded to SIGTERM).
    pub graceful_shutdown: bool,
    /// The daemon's exit code, if available.
    pub exit_code: Option<i32>,
    /// Whether the config file was successfully removed.
    pub config_cleaned: bool,
    /// Any warnings that occurred during teardown.
    pub warnings: Vec<String>,
}

#[allow(dead_code)]
impl TeardownResult {
    /// Returns true if teardown completed without any warnings.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty() && self.config_cleaned
    }

    /// Returns true if the daemon was forcefully killed.
    #[must_use]
    pub fn was_force_killed(&self) -> bool {
        self.sigkill_sent
    }
}

impl fmt::Display for TeardownResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Teardown Result:")?;
        writeln!(f, "  SIGTERM sent: {}", self.sigterm_sent)?;
        writeln!(f, "  SIGKILL sent: {}", self.sigkill_sent)?;
        writeln!(f, "  Graceful shutdown: {}", self.graceful_shutdown)?;
        writeln!(f, "  Exit code: {:?}", self.exit_code)?;
        writeln!(f, "  Config cleaned: {}", self.config_cleaned)?;
        if !self.warnings.is_empty() {
            writeln!(f, "  Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "    - {}", warning)?;
            }
        }
        Ok(())
    }
}
// ============================================================================
// TestEvents - Helper for concise test event creation
// ============================================================================

/// Helper struct for creating test events concisely.
///
/// `TestEvents` provides associated functions for creating common event patterns
/// used in E2E tests. All methods return `Vec<KeyEvent>` for compatibility with
/// [`E2EHarness::inject`].
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::tests::e2e_harness::TestEvents;
/// use keyrx_core::config::KeyCode;
///
/// // Single key tap
/// let events = TestEvents::tap(KeyCode::A);
/// assert_eq!(events.len(), 2);
///
/// // Multiple taps
/// let events = TestEvents::taps(&[KeyCode::A, KeyCode::B, KeyCode::C]);
/// assert_eq!(events.len(), 6);
///
/// // Type a word
/// let events = TestEvents::type_keys(&[KeyCode::H, KeyCode::E, KeyCode::L, KeyCode::L, KeyCode::O]);
/// ```
#[allow(dead_code)]
pub struct TestEvents;

#[allow(dead_code)]
impl TestEvents {
    /// Creates a Press event for a single key.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::press(KeyCode::A);
    /// assert_eq!(events, vec![KeyEvent::Press(KeyCode::A)]);
    /// ```
    pub fn press(key: KeyCode) -> Vec<KeyEvent> {
        vec![KeyEvent::Press(key)]
    }

    /// Creates a Release event for a single key.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::release(KeyCode::A);
    /// assert_eq!(events, vec![KeyEvent::Release(KeyCode::A)]);
    /// ```
    pub fn release(key: KeyCode) -> Vec<KeyEvent> {
        vec![KeyEvent::Release(key)]
    }

    /// Creates a complete key tap (Press + Release).
    ///
    /// This is the most common pattern for testing key remapping.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::tap(KeyCode::A);
    /// assert_eq!(events, vec![
    ///     KeyEvent::Press(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::A),
    /// ]);
    /// ```
    pub fn tap(key: KeyCode) -> Vec<KeyEvent> {
        vec![KeyEvent::Press(key), KeyEvent::Release(key)]
    }

    /// Creates multiple Press events for a sequence of keys.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::presses(&[KeyCode::LShift, KeyCode::A]);
    /// assert_eq!(events, vec![
    ///     KeyEvent::Press(KeyCode::LShift),
    ///     KeyEvent::Press(KeyCode::A),
    /// ]);
    /// ```
    pub fn presses(keys: &[KeyCode]) -> Vec<KeyEvent> {
        keys.iter().map(|&k| KeyEvent::Press(k)).collect()
    }

    /// Creates multiple Release events for a sequence of keys.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::releases(&[KeyCode::A, KeyCode::LShift]);
    /// assert_eq!(events, vec![
    ///     KeyEvent::Release(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::LShift),
    /// ]);
    /// ```
    pub fn releases(keys: &[KeyCode]) -> Vec<KeyEvent> {
        keys.iter().map(|&k| KeyEvent::Release(k)).collect()
    }

    /// Creates multiple key taps in sequence.
    ///
    /// Each key is pressed and released before the next key.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::taps(&[KeyCode::A, KeyCode::B]);
    /// assert_eq!(events, vec![
    ///     KeyEvent::Press(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::A),
    ///     KeyEvent::Press(KeyCode::B),
    ///     KeyEvent::Release(KeyCode::B),
    /// ]);
    /// ```
    pub fn taps(keys: &[KeyCode]) -> Vec<KeyEvent> {
        keys.iter()
            .flat_map(|&k| vec![KeyEvent::Press(k), KeyEvent::Release(k)])
            .collect()
    }

    /// Creates events for typing a sequence of keys.
    ///
    /// This is an alias for [`taps`](Self::taps) with a more intuitive name
    /// for simulating keyboard typing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Type "hello"
    /// let events = TestEvents::type_keys(&[
    ///     KeyCode::H, KeyCode::E, KeyCode::L, KeyCode::L, KeyCode::O
    /// ]);
    /// ```
    pub fn type_keys(keys: &[KeyCode]) -> Vec<KeyEvent> {
        Self::taps(keys)
    }

    /// Creates events for a modified key press (e.g., Shift+A).
    ///
    /// The modifier is pressed first, then the key is tapped, then the
    /// modifier is released. This produces the correct event sequence for
    /// modified key combinations.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Shift+A (for typing uppercase 'A')
    /// let events = TestEvents::modified(KeyCode::LShift, KeyCode::A);
    /// assert_eq!(events, vec![
    ///     KeyEvent::Press(KeyCode::LShift),
    ///     KeyEvent::Press(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::LShift),
    /// ]);
    /// ```
    pub fn modified(modifier: KeyCode, key: KeyCode) -> Vec<KeyEvent> {
        vec![
            KeyEvent::Press(modifier),
            KeyEvent::Press(key),
            KeyEvent::Release(key),
            KeyEvent::Release(modifier),
        ]
    }

    /// Creates events for a key press with multiple modifiers.
    ///
    /// Modifiers are pressed in order, then the key is tapped, then modifiers
    /// are released in reverse order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Ctrl+Shift+C
    /// let events = TestEvents::with_modifiers(
    ///     &[KeyCode::LCtrl, KeyCode::LShift],
    ///     KeyCode::C,
    /// );
    /// assert_eq!(events, vec![
    ///     KeyEvent::Press(KeyCode::LCtrl),
    ///     KeyEvent::Press(KeyCode::LShift),
    ///     KeyEvent::Press(KeyCode::C),
    ///     KeyEvent::Release(KeyCode::C),
    ///     KeyEvent::Release(KeyCode::LShift),
    ///     KeyEvent::Release(KeyCode::LCtrl),
    /// ]);
    /// ```
    pub fn with_modifiers(modifiers: &[KeyCode], key: KeyCode) -> Vec<KeyEvent> {
        let mut events = Vec::with_capacity(modifiers.len() * 2 + 2);

        // Press modifiers in order
        for &modifier in modifiers {
            events.push(KeyEvent::Press(modifier));
        }

        // Tap the key
        events.push(KeyEvent::Press(key));
        events.push(KeyEvent::Release(key));

        // Release modifiers in reverse order
        for &modifier in modifiers.iter().rev() {
            events.push(KeyEvent::Release(modifier));
        }

        events
    }

    /// Creates events for holding a modifier while typing multiple keys.
    ///
    /// The modifier is held down while all keys are tapped in sequence,
    /// then the modifier is released.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Shift held while typing "ABC"
    /// let events = TestEvents::hold_while_typing(
    ///     KeyCode::LShift,
    ///     &[KeyCode::A, KeyCode::B, KeyCode::C],
    /// );
    /// // Produces: Press(LShift), Press(A), Release(A), Press(B), Release(B), ...
    /// ```
    pub fn hold_while_typing(modifier: KeyCode, keys: &[KeyCode]) -> Vec<KeyEvent> {
        let mut events = Vec::with_capacity(keys.len() * 2 + 2);

        // Press modifier
        events.push(KeyEvent::Press(modifier));

        // Tap each key
        for &key in keys {
            events.push(KeyEvent::Press(key));
            events.push(KeyEvent::Release(key));
        }

        // Release modifier
        events.push(KeyEvent::Release(modifier));

        events
    }

    /// Creates events from raw event data for custom patterns.
    ///
    /// This is useful when you need a specific event sequence that doesn't
    /// fit the other helper methods.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Custom pattern: press A, press B, release A, release B
    /// let events = TestEvents::from_events(&[
    ///     KeyEvent::Press(KeyCode::A),
    ///     KeyEvent::Press(KeyCode::B),
    ///     KeyEvent::Release(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::B),
    /// ]);
    /// ```
    pub fn from_events(events: &[KeyEvent]) -> Vec<KeyEvent> {
        events.to_vec()
    }

    /// Creates an empty event sequence.
    ///
    /// Useful for testing scenarios where no events are expected.
    pub fn empty() -> Vec<KeyEvent> {
        Vec::new()
    }
}

// ============================================================================
// Test Timeout Handling
// ============================================================================

/// Default timeout for E2E tests (30 seconds).
///
/// This is generous to allow for slow CI environments while still catching
/// genuinely hung tests. Tests can override this with custom timeouts.
/// Default timeout for E2E tests (30 seconds).
#[allow(dead_code)]
pub const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Result type for tests with timeout.
#[allow(dead_code)]
pub type TimeoutResult<T> = Result<T, E2EError>;

/// Runs a test with a specific timeout and phased diagnostics.
///
/// This function wraps the execution of a test and provides automatic
/// timeout monitoring. If the test (including setup and verification)
/// exceeds the timeout, an `E2EError::TestTimeout` is returned with
/// diagnostic information about which phase was active.
///
/// # Arguments
///
/// * `timeout` - Maximum duration allowed for the entire test
/// * `config` - Test configuration to use for setup
/// * `test_fn` - Closure containing the actual test logic
#[allow(dead_code)]
pub fn with_timeout<F, T>(timeout: Duration, config: E2EConfig, test_fn: F) -> TimeoutResult<T>
where
    F: FnOnce(&mut E2EHarness, &dyn Fn(TestTimeoutPhase)) -> Result<T, E2EError> + Send,
    T: Send,
{
    let start = Instant::now();
    let (tx, rx) = std::sync::mpsc::channel();

    // Use a RefCell logic inside and a shared atomic phase if needed,
    // but for simplicity we'll use a thread and a channel.
    let phase = std::sync::Arc::new(std::sync::Mutex::new(TestTimeoutPhase::Setup));
    let phase_clone = phase.clone();

    let set_phase = move |p: TestTimeoutPhase| {
        if let Ok(mut guard) = phase_clone.lock() {
            *guard = p;
        }
    };

    std::thread::scope(|s| {
        s.spawn(move || {
            let result = (|| {
                set_phase(TestTimeoutPhase::Setup);
                let mut harness = E2EHarness::setup(config)?;

                set_phase(TestTimeoutPhase::TestLogic);
                test_fn(&mut harness, &set_phase)
            })();
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let last_phase = *phase.lock().unwrap();
                let elapsed = start.elapsed();
                Err(E2EError::TestTimeout {
                    phase: last_phase,
                    timeout,
                    elapsed,
                    context: format!(
                        "Test hung during {} phase. The daemon process will be cleaned up.",
                        last_phase
                    ),
                })
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // Thread panicked
                panic!("Test thread panicked during execution");
            }
        }
    })
}

/// Runs a test with the default E2E timeout (30s).
#[allow(dead_code)]
pub fn with_default_timeout<F, T>(config: E2EConfig, test_fn: F) -> TimeoutResult<T>
where
    F: FnOnce(&mut E2EHarness, &dyn Fn(TestTimeoutPhase)) -> Result<T, E2EError> + Send,
    T: Send,
{
    with_timeout(DEFAULT_TEST_TIMEOUT, config, test_fn)
}

/// Convenience wrapper for tests that don't need phase reporting.
#[allow(dead_code)]
pub fn run_test_with_timeout<F, T>(
    timeout: Duration,
    config: E2EConfig,
    test_fn: F,
) -> TimeoutResult<T>
where
    F: FnOnce(&mut E2EHarness) -> Result<T, E2EError> + Send,
    T: Send,
{
    with_timeout(timeout, config, |harness, _set_phase| test_fn(harness))
}
// ============================================================================
// Unit Tests
// ============================================================================

// E2EHarness infrastructure tests require Linux-specific features to run the daemon
// These tests are separate from the actual E2E tests in virtual_e2e_test.rs
// To run these tests: cargo test --test e2e_harness --features linux
