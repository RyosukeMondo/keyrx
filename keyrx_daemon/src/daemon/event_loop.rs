//! Event loop processing for the keyrx daemon.
//!
//! This module contains the core event processing logic, including:
//!
//! - Event capture and dispatching
//! - Reload signal checking
//! - Statistics tracking
//! - Timeout handling for tap-hold

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use log::{info, trace, warn};

use crate::platform::Platform;
use crate::web::events::KeyEventData;

use super::event_broadcaster::EventBroadcaster;
use super::signals::SignalHandler;
use super::DaemonError;

/// Event loop statistics tracking.
struct EventLoopStats {
    /// Total number of events processed.
    event_count: u64,
    /// Last time statistics were logged.
    last_stats_time: std::time::Instant,
}

impl EventLoopStats {
    /// Creates new statistics tracker.
    fn new() -> Self {
        Self {
            event_count: 0,
            last_stats_time: std::time::Instant::now(),
        }
    }

    /// Records a processed event.
    fn record_event(&mut self) {
        self.event_count += 1;
    }

    /// Checks if it's time to log statistics and does so if needed.
    ///
    /// Returns `true` if statistics were logged.
    fn maybe_log_stats(&mut self) -> bool {
        const STATS_INTERVAL: Duration = Duration::from_secs(60);

        if self.last_stats_time.elapsed() >= STATS_INTERVAL {
            info!("Event loop stats: {} events processed", self.event_count);
            self.last_stats_time = std::time::Instant::now();
            true
        } else {
            false
        }
    }

    /// Returns the total number of events processed.
    fn total_events(&self) -> u64 {
        self.event_count
    }
}

/// Runs the main event processing loop.
///
/// This function captures keyboard events from the platform, processes them,
/// and injects output events. The loop continues until a shutdown signal
/// (SIGTERM or SIGINT) is received.
///
/// # Arguments
///
/// * `platform` - Platform abstraction for input/output operations
/// * `running` - Atomic flag controlling loop execution
/// * `signal_handler` - Signal handler for reload detection
/// * `reload_callback` - Callback to invoke when reload is requested
/// * `event_broadcaster` - Optional broadcaster for real-time WebSocket updates
///
/// # Event Processing Flow
///
/// For each input event:
/// 1. Check for reload signal (SIGHUP)
/// 2. Capture event from platform (blocking)
/// 3. Inject the event back through the platform
///
/// **Note**: The current implementation is simplified and does not perform
/// key remapping. Full remapping support would require the Platform trait
/// to expose device state and lookup tables, or for the Daemon to manage
/// remapping state independently.
///
/// # Signal Handling
///
/// - **SIGTERM/SIGINT**: Sets the running flag to false, causing graceful exit
/// - **SIGHUP**: Calls the reload callback (currently returns error)
///
/// # Errors
///
/// - `DaemonError::Platform`: Platform error during event capture or injection
/// - `DaemonError::RuntimeError`: Critical error during event processing
///
/// # Example
///
/// ```no_run
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use std::sync::Arc;
/// use keyrx_daemon::daemon::event_loop::run_event_loop;
/// use keyrx_daemon::daemon::DaemonError;
/// use keyrx_daemon::platform::Platform;
///
/// fn example(
///     platform: &mut Box<dyn Platform>,
///     running: Arc<AtomicBool>,
///     signal_handler: &keyrx_daemon::daemon::SignalHandler,
/// ) -> Result<(), DaemonError> {
///     run_event_loop(
///         platform,
///         running,
///         signal_handler,
///         || Err(DaemonError::RuntimeError("Reload not supported".to_string())),
///     )
/// }
/// ```
pub fn run_event_loop<F>(
    platform: &mut Box<dyn Platform>,
    running: Arc<AtomicBool>,
    signal_handler: &SignalHandler,
    mut reload_callback: F,
    event_broadcaster: Option<&EventBroadcaster>,
) -> Result<(), DaemonError>
where
    F: FnMut() -> Result<(), DaemonError>,
{
    info!("Starting event processing loop");

    let mut stats = EventLoopStats::new();

    // Main event loop
    while running.load(Ordering::SeqCst) {
        // Check for SIGHUP (reload request)
        if signal_handler.check_reload() {
            info!("Reload signal received (SIGHUP)");
            if let Err(e) = reload_callback() {
                // Log the error but continue running
                warn!("Configuration reload failed: {}", e);
            }
        }

        // Capture input event from platform (blocking with timeout to allow signal checking)
        // Note: capture_input() may return an error if no events are available
        // We treat this as non-fatal and continue the loop
        match platform.capture_input() {
            Ok(event) => {
                trace!("Input event: {:?}", event);

                // Broadcast key event to WebSocket clients if broadcaster is available
                if let Some(broadcaster) = event_broadcaster {
                    let event_data = KeyEventData {
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_micros() as u64,
                        key_code: format!("{:?}", event.keycode()),
                        event_type: match event.event_type() {
                            keyrx_core::runtime::KeyEventType::Press => "press".to_string(),
                            keyrx_core::runtime::KeyEventType::Release => "release".to_string(),
                        },
                        input: format!("{:?}", event.keycode()),
                        output: format!("{:?}", event.keycode()), // TODO: Show remapped output
                        latency: 0,                               // TODO: Calculate actual latency
                    };
                    broadcaster.broadcast_key_event(event_data);
                }

                // TODO: Process event through remapping engine
                // For now, just pass through the event
                let output_event = event;

                // Inject output event
                if let Err(e) = platform.inject_output(output_event) {
                    warn!("Failed to inject event: {}", e);
                } else {
                    stats.record_event();
                }
            }
            Err(e) => {
                // Check if we should exit
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                // Log non-fatal errors and continue
                trace!("Event capture returned error (may be timeout): {}", e);

                // Small sleep to prevent busy loop
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        // Periodic stats logging
        stats.maybe_log_stats();
    }

    info!(
        "Event loop stopped. Total events processed: {}",
        stats.total_events()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_loop_stats_new() {
        let stats = EventLoopStats::new();
        assert_eq!(stats.total_events(), 0);
    }

    #[test]
    fn test_event_loop_stats_record_event() {
        let mut stats = EventLoopStats::new();
        assert_eq!(stats.total_events(), 0);

        stats.record_event();
        assert_eq!(stats.total_events(), 1);

        stats.record_event();
        stats.record_event();
        assert_eq!(stats.total_events(), 3);
    }

    #[test]
    fn test_event_loop_stats_maybe_log_stats_not_yet() {
        let mut stats = EventLoopStats::new();
        // Immediately after creation, should not log
        assert!(!stats.maybe_log_stats());
    }
}
