//! Linux-specific signal handling for daemon lifecycle management.
//!
//! This module implements signal handling using the `signal-hook` crate,
//! providing graceful shutdown via SIGTERM/SIGINT and configuration reload via SIGHUP.
//!
//! # Signal Handling Strategy
//!
//! - **SIGTERM/SIGINT**: Sets the `running` flag to `false` via `signal_hook::flag::register`.
//!   This is async-signal-safe and allows the main event loop to detect shutdown requests.
//!
//! - **SIGHUP**: Sets a reload flag that can be polled by the daemon. This enables
//!   hot-reloading of configuration without restarting the daemon.
//!
//! # Example
//!
//! ```no_run
//! use std::sync::atomic::{AtomicBool, Ordering};
//! use std::sync::Arc;
//! use keyrx_daemon::daemon::{install_signal_handlers, SignalHandler, ReloadState};
//!
//! // Create running flag
//! let running = Arc::new(AtomicBool::new(true));
//!
//! // Install signal handlers
//! let handler = install_signal_handlers(running.clone())
//!     .expect("Failed to install signal handlers");
//!
//! // Main event loop
//! while running.load(Ordering::SeqCst) {
//!     // Check for reload requests
//!     if handler.check_reload() {
//!         println!("Reloading configuration...");
//!     }
//!
//!     // Process events
//!     std::thread::sleep(std::time::Duration::from_millis(10));
//! }
//!
//! println!("Shutdown complete");
//! ```

use std::io;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::flag::register_conditional_default;

use crate::daemon::state::ReloadState;

/// Signal handler manager for the keyrx daemon.
///
/// This struct manages signal handler registration and provides methods
/// to check for reload requests. The shutdown signals (SIGTERM/SIGINT)
/// automatically set the `running` flag to `false`.
#[derive(Debug)]
pub struct SignalHandler {
    /// Reload state for SIGHUP handling.
    reload_state: ReloadState,
}

impl SignalHandler {
    /// Creates a new signal handler with the given reload state.
    fn new(reload_state: ReloadState) -> Self {
        Self { reload_state }
    }

    /// Checks if a configuration reload has been requested (via SIGHUP).
    ///
    /// This method clears the reload flag after checking, so subsequent calls
    /// will return `false` until another SIGHUP is received.
    ///
    /// # Returns
    ///
    /// `true` if a reload was requested since the last check, `false` otherwise.
    pub fn check_reload(&self) -> bool {
        self.reload_state.check_and_clear()
    }

    /// Returns the reload state for external access.
    pub fn reload_state(&self) -> &ReloadState {
        &self.reload_state
    }
}

/// Installs signal handlers for daemon lifecycle management.
///
/// This function sets up handlers for:
/// - **SIGTERM**: Sets `running` to `false` for graceful shutdown
/// - **SIGINT**: Same as SIGTERM (handles Ctrl+C)
/// - **SIGHUP**: Sets reload flag for configuration reload
///
/// # Arguments
///
/// * `running` - Atomic flag that controls the main event loop. Signal handlers
///   will set this to `false` when shutdown is requested.
///
/// # Returns
///
/// * `Ok(SignalHandler)` - Successfully installed handlers, returns manager for reload checking
/// * `Err(io::Error)` - Failed to register signal handlers
///
/// # Example
///
/// ```no_run
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use std::sync::Arc;
/// use keyrx_daemon::daemon::install_signal_handlers;
///
/// let running = Arc::new(AtomicBool::new(true));
/// let handler = install_signal_handlers(running.clone())
///     .expect("Failed to install signal handlers");
///
/// // The running flag will be set to false when SIGTERM or SIGINT is received
/// while running.load(Ordering::SeqCst) {
///     // Process events
///     # break; // Break for doctest
/// }
/// ```
///
/// # Platform Notes
///
/// This function is only available on Linux (requires the `linux` feature).
/// On other platforms, signal handling must be implemented differently.
///
/// # Errors
///
/// Returns `io::Error` if:
/// - Signal handler registration fails (rare, typically due to system limits)
/// - Invalid signal number (should not happen with constants)
pub fn install_signal_handlers(running: Arc<AtomicBool>) -> io::Result<SignalHandler> {
    // Register SIGTERM handler - sets running to FALSE on signal
    // Uses register_conditional_default which sets flag to false (the "default" value)
    register_conditional_default(SIGTERM, Arc::clone(&running))?;

    // Register SIGINT handler - sets running to FALSE on signal (Ctrl+C)
    register_conditional_default(SIGINT, Arc::clone(&running))?;

    // Create reload state for SIGHUP
    let reload_state = ReloadState::new();

    // Register SIGHUP handler - sets reload flag to TRUE on signal
    // Note: signal_hook::flag::register sets flag to TRUE, which is what we want for reload
    signal_hook::flag::register(SIGHUP, reload_state.flag())?;

    log::info!("Signal handlers installed (SIGTERM, SIGINT for shutdown; SIGHUP for reload)");

    Ok(SignalHandler::new(reload_state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_signal_handler_creation() {
        let reload_state = ReloadState::new();
        let handler = SignalHandler::new(reload_state);

        // Initially no reload requested
        assert!(!handler.check_reload());
    }

    #[test]
    fn test_signal_handler_check_reload() {
        let reload_state = ReloadState::new();

        // Simulate SIGHUP by setting the flag
        reload_state.flag().store(true, Ordering::SeqCst);

        let handler = SignalHandler::new(reload_state);

        // First check should return true
        assert!(handler.check_reload());

        // Second check should return false (flag cleared)
        assert!(!handler.check_reload());
    }

    #[test]
    fn test_signal_handler_reload_state_access() {
        let reload_state = ReloadState::new();
        let handler = SignalHandler::new(reload_state);

        // Set via reload_state
        handler.reload_state().flag().store(true, Ordering::SeqCst);

        // Should be detectable
        assert!(handler.check_reload());
    }

    #[test]
    fn test_install_signal_handlers() {
        let running = Arc::new(AtomicBool::new(true));

        // Install handlers
        let result = install_signal_handlers(Arc::clone(&running));
        assert!(result.is_ok());

        let handler = result.unwrap();

        // Running should still be true (no signal sent)
        assert!(running.load(Ordering::SeqCst));

        // No reload requested
        assert!(!handler.check_reload());
    }

    #[test]
    fn test_running_flag_thread_safety() {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        // Install handlers
        let handler = install_signal_handlers(Arc::clone(&running)).unwrap();

        // Spawn thread that simulates external flag modification
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            running_clone.store(false, Ordering::SeqCst);
        });

        // Wait for thread to modify flag
        handle.join().unwrap();

        // Flag should be false now
        assert!(!running.load(Ordering::SeqCst));

        // Reload check should still work independently
        assert!(!handler.check_reload());
    }

    #[test]
    fn test_multiple_handler_installations() {
        // Test that we can install handlers multiple times
        // (signal-hook allows this, newer registrations take precedence)
        let running1 = Arc::new(AtomicBool::new(true));
        let running2 = Arc::new(AtomicBool::new(true));

        let result1 = install_signal_handlers(Arc::clone(&running1));
        assert!(result1.is_ok());

        let result2 = install_signal_handlers(Arc::clone(&running2));
        assert!(result2.is_ok());
    }

    #[test]
    fn test_reload_state_multiple_checks() {
        let reload_state = ReloadState::new();
        let handler = SignalHandler::new(reload_state);

        // Multiple checks when no reload requested
        for _ in 0..5 {
            assert!(!handler.check_reload());
        }

        // Set reload flag
        handler.reload_state().flag().store(true, Ordering::SeqCst);

        // First check should return true
        assert!(handler.check_reload());

        // Subsequent checks should return false
        for _ in 0..5 {
            assert!(!handler.check_reload());
        }
    }

    // Note: We cannot easily test actual signal delivery in unit tests
    // because sending signals to the process would affect the test runner.
    // Integration tests can test this with subprocess isolation.
}
