//! E2E Test Error Types.
//!
//! This module defines error types for E2E test operations.

use std::fmt;
use std::time::Duration;

use keyrx_daemon::test_utils::VirtualDeviceError;
use keyrx_core::runtime::KeyEvent;

// ============================================================================
// TestTimeoutPhase - Phase tracking for timeout diagnostics
// ============================================================================

/// Represents the phase of an E2E test for timeout diagnostics.
///
/// When a test times out, this enum identifies exactly which phase was
/// executing, enabling precise diagnosis of where the test hung.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestTimeoutPhase {
    /// Environment setup (creation of virtual devices, etc)
    Setup,
    /// Event injection (sending events to virtual keyboard)
    Injection,
    /// Event capture (collecting events from daemon's output)
    Capture,
    /// Verification (comparing captured vs expected events)
    Verification,
    /// Teardown (cleaning up resources)
    Teardown,
    /// User-defined test logic
    TestLogic,
}

impl fmt::Display for TestTimeoutPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestTimeoutPhase::Setup => write!(f, "setup"),
            TestTimeoutPhase::Injection => write!(f, "event injection"),
            TestTimeoutPhase::Capture => write!(f, "event capture"),
            TestTimeoutPhase::Verification => write!(f, "verification"),
            TestTimeoutPhase::Teardown => write!(f, "teardown"),
            TestTimeoutPhase::TestLogic => write!(f, "test logic"),
        }
    }
}

// ============================================================================
// E2EError - Error types for E2E test operations
// ============================================================================

/// Errors that can occur during E2E test operations.
///
/// This error type wraps [`VirtualDeviceError`] and adds E2E-specific error
/// variants for test setup, execution, and verification.
#[allow(dead_code)]
#[derive(Debug)]
pub enum E2EError {
    /// Error from virtual device operations (VirtualKeyboard, OutputCapture).
    VirtualDevice(VirtualDeviceError),

    /// Failed to create or serialize test configuration.
    ConfigError {
        /// Description of what went wrong
        message: String,
    },

    /// Failed to start daemon subprocess.
    DaemonStartError {
        /// Description of what went wrong
        message: String,
        /// Standard error output from daemon, if available
        stderr: Option<String>,
    },

    /// Daemon exited unexpectedly during test.
    DaemonCrashed {
        /// Exit code if available
        exit_code: Option<i32>,
        /// Standard error output from daemon, if available
        stderr: Option<String>,
    },

    /// Test verification failed - captured events don't match expected.
    VerificationFailed {
        /// Events that were captured during the test
        captured: Vec<KeyEvent>,
        /// Events that were expected
        expected: Vec<KeyEvent>,
        /// Detailed diff message
        diff: String,
    },

    /// Test timed out waiting for expected condition.
    Timeout {
        /// What operation timed out
        operation: String,
        /// How long we waited
        timeout_ms: u64,
    },

    /// Test exceeded its overall time limit.
    TestTimeout {
        /// Which phase the test was in when it timed out.
        phase: TestTimeoutPhase,
        /// The timeout limit that was exceeded.
        timeout: Duration,
        /// Total time elapsed since test start.
        elapsed: Duration,
        /// Additional diagnostic context.
        context: String,
    },

    /// I/O error during test operations.
    Io(std::io::Error),
}

impl std::error::Error for E2EError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            E2EError::VirtualDevice(e) => Some(e),
            E2EError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for E2EError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            E2EError::VirtualDevice(e) => write!(f, "virtual device error: {}", e),
            E2EError::ConfigError { message } => write!(f, "config error: {}", message),
            E2EError::DaemonStartError { message, stderr } => {
                write!(f, "daemon start error: {}", message)?;
                if let Some(stderr) = stderr {
                    write!(f, "\nstderr: {}", stderr)?;
                }
                Ok(())
            }
            E2EError::DaemonCrashed { exit_code, stderr } => {
                write!(f, "daemon crashed")?;
                if let Some(code) = exit_code {
                    write!(f, " with exit code {}", code)?;
                }
                if let Some(stderr) = stderr {
                    write!(f, "\nstderr: {}", stderr)?;
                }
                Ok(())
            }
            E2EError::VerificationFailed {
                captured,
                expected,
                diff,
            } => {
                writeln!(f, "verification failed:")?;
                writeln!(f, "  expected {} event(s): {:?}", expected.len(), expected)?;
                writeln!(f, "  captured {} event(s): {:?}", captured.len(), captured)?;
                write!(f, "\n{}", diff)
            }
            E2EError::Timeout {
                operation,
                timeout_ms,
            } => {
                write!(
                    f,
                    "timeout after {}ms waiting for {}",
                    timeout_ms, operation
                )
            }
            E2EError::TestTimeout {
                phase,
                timeout,
                elapsed,
                context,
            } => {
                let context_msg = if context.is_empty() {
                    String::new()
                } else {
                    format!("\nContext: {}", context)
                };
                write!(
                    f,
                    "E2E TEST TIMEOUT: Phase {} took {:.2}s ({}s limit).{}\nCheck for hung daemon processes and verify timeout configuration.",
                    phase,
                    elapsed.as_secs_f64(),
                    timeout.as_secs(),
                    context_msg
                )
            }
            E2EError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<VirtualDeviceError> for E2EError {
    fn from(err: VirtualDeviceError) -> Self {
        E2EError::VirtualDevice(err)
    }
}

impl From<std::io::Error> for E2EError {
    fn from(err: std::io::Error) -> Self {
        E2EError::Io(err)
    }
}
