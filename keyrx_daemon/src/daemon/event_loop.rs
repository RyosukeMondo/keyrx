//! Event loop processing for the keyrx daemon.
//!
//! This module contains the core event processing logic, including:
//!
//! - Event capture and dispatching
//! - Reload signal checking
//! - Statistics tracking
//! - Timeout handling for tap-hold
//! - Key remapping via keyrx_core runtime

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Re-export Instant from std::time for internal use
use std::time::Instant;

use keyrx_core::config::BaseKeyMapping;
use keyrx_core::runtime::{check_tap_hold_timeouts, process_event};
use log::{info, trace, warn};

use crate::platform::Platform;
use crate::web::events::KeyEventData;

use super::event_broadcaster::EventBroadcaster;
use super::metrics::LatencyRecorder;
use super::remapping_state::RemappingState;
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

/// Determines mapping type string from a BaseKeyMapping.
fn get_mapping_type(mapping: &BaseKeyMapping) -> &'static str {
    match mapping {
        BaseKeyMapping::Simple { .. } => "simple",
        BaseKeyMapping::Modifier { .. } => "modifier",
        BaseKeyMapping::Lock { .. } => "lock",
        BaseKeyMapping::TapHold { .. } => "tap_hold",
        BaseKeyMapping::ModifiedOutput { .. } => "modified_output",
    }
}

/// Returns current timestamp in microseconds since UNIX epoch.
fn current_timestamp_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

/// Runs the main event processing loop.
///
/// This function captures keyboard events from the platform, processes them
/// through the remapping engine (if provided), and injects output events.
/// The loop continues until a shutdown signal (SIGTERM or SIGINT) is received.
///
/// # Arguments
///
/// * `platform` - Platform abstraction for input/output operations
/// * `running` - Atomic flag controlling loop execution
/// * `signal_handler` - Signal handler for reload detection
/// * `reload_callback` - Callback to invoke when reload is requested
/// * `event_broadcaster` - Optional broadcaster for real-time WebSocket updates
/// * `remapping_state` - Optional remapping state for key remapping (KeyLookup + DeviceState)
/// * `latency_recorder` - Optional lock-free latency recorder for metrics
///
/// # Event Processing Flow
///
/// For each input event:
/// 1. Check for reload signal (SIGHUP)
/// 2. Capture event from platform (blocking)
/// 3. Process event through remapping engine (if remapping_state provided)
/// 4. Inject output events through platform
/// 5. Record latency (if latency_recorder provided)
///
/// Periodically (every 10ms when no events):
/// - Check tap-hold timeouts and inject any pending hold events
///
/// # Signal Handling
///
/// - **SIGTERM/SIGINT**: Sets the running flag to false, causing graceful exit
/// - **SIGHUP**: Calls the reload callback to reload configuration
///
/// # Performance
///
/// - Key lookup: O(1), ~5ns (HashMap with robin hood hashing)
/// - Latency recording: O(1), ~10-50ns (lock-free atomic operations)
/// - Target: <100μs for 95th percentile total processing
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
///         None, // No event broadcaster
///         None, // No remapping state (pass-through mode)
///         None, // No latency recording
///     )
/// }
/// ```
pub fn run_event_loop<F>(
    platform: &mut Box<dyn Platform>,
    running: Arc<AtomicBool>,
    signal_handler: &SignalHandler,
    mut reload_callback: F,
    event_broadcaster: Option<&EventBroadcaster>,
    mut remapping_state: Option<&mut RemappingState>,
    latency_recorder: Option<&LatencyRecorder>,
) -> Result<(), DaemonError>
where
    F: FnMut() -> Result<(), DaemonError>,
{
    info!("Starting event processing loop");

    let mut stats = EventLoopStats::new();
    let mut last_timeout_check = Instant::now();

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
                let capture_time = Instant::now();
                trace!("Input event: {:?}", event);

                // Get device info from event
                let device_id = event.device_id().map(String::from);
                let input_keycode = event.keycode();

                // Process event through remapping engine if available
                let (output_events, mapping_type, mapping_triggered) =
                    if let Some(ref mut remap_state) = remapping_state {
                        // Get lookup and state references together to avoid borrow conflicts
                        let (lookup, state) = remap_state.lookup_and_state_mut();

                        // Look up mapping to determine type before processing
                        let mapping = lookup.find_mapping(input_keycode, state);
                        let mapping_type_str = mapping.map(get_mapping_type);
                        let triggered = mapping.is_some();

                        // Process the event through the remapping engine
                        let outputs = process_event(event.clone(), lookup, state);

                        (outputs, mapping_type_str, triggered)
                    } else {
                        // Pass-through mode - no remapping
                        (vec![event.clone()], None, false)
                    };

                // Compute output description for broadcast
                let output_desc = if output_events.is_empty() {
                    "(suppressed)".to_string()
                } else {
                    output_events
                        .iter()
                        .map(|e| format!("{:?}", e.keycode()))
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                // Inject output events
                // On Linux, grab() blocks original events so we MUST always inject.
                // On Windows, Raw Input doesn't block events so they flow naturally.
                for output_event in &output_events {
                    if let Err(e) = platform.inject_output(output_event.clone()) {
                        warn!("Failed to inject event: {}", e);
                    } else {
                        stats.record_event();
                    }
                }

                // Record latency after injection
                let latency_us = capture_time.elapsed().as_micros() as u64;
                if let Some(recorder) = latency_recorder {
                    recorder.record(latency_us);
                }

                // Broadcast key event to WebSocket clients if broadcaster is available
                if let Some(broadcaster) = event_broadcaster {
                    let timestamp = current_timestamp_us();

                    let event_data = KeyEventData {
                        timestamp,
                        key_code: format!("{:?}", input_keycode),
                        event_type: match event.event_type() {
                            keyrx_core::runtime::KeyEventType::Press => "press".to_string(),
                            keyrx_core::runtime::KeyEventType::Release => "release".to_string(),
                        },
                        input: format!("{:?}", input_keycode),
                        output: output_desc.clone(),
                        latency: latency_us,
                        device_id: device_id.clone(),
                        device_name: device_id.clone(),
                        mapping_type: mapping_type.map(String::from),
                        mapping_triggered,
                    };

                    log::debug!(
                        "Event loop: About to broadcast {} event for key {:?} (mapping_triggered: {}, output: {})",
                        event_data.event_type, input_keycode, mapping_triggered, output_desc
                    );
                    broadcaster.broadcast_key_event(event_data);
                } else {
                    log::warn!("Event loop: EventBroadcaster is None! Events will not be sent to WebSocket clients");
                }
            }
            Err(e) => {
                // Check if we should exit
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                // Log non-fatal errors and continue
                trace!("Event capture returned error (may be timeout): {}", e);

                // Check tap-hold timeouts every 10ms when idle
                if last_timeout_check.elapsed() >= Duration::from_millis(10) {
                    if let Some(ref mut remap_state) = remapping_state {
                        let current_time = current_timestamp_us();
                        let timeout_events =
                            check_tap_hold_timeouts(current_time, remap_state.state_mut());

                        // Inject any timeout-generated events (e.g., hold action triggered)
                        for output_event in &timeout_events {
                            if let Err(e) = platform.inject_output(output_event.clone()) {
                                warn!("Failed to inject timeout event: {}", e);
                            } else {
                                stats.record_event();
                                trace!("Tap-hold timeout event injected: {:?}", output_event);
                            }
                        }
                    }
                    last_timeout_check = Instant::now();
                }

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

/// Process a single event from the platform (non-blocking).
///
/// This function is designed for platforms like Windows where the event loop
/// must be integrated with a system message pump. It attempts to capture one
/// event and process it, returning immediately if no event is available.
///
/// # Arguments
///
/// * `platform` - Platform abstraction for input/output operations
/// * `event_broadcaster` - Optional broadcaster for real-time WebSocket updates
/// * `remapping_state` - Optional remapping state for key remapping
/// * `latency_recorder` - Optional latency recorder for metrics
///
/// # Returns
///
/// * `Ok(true)` - An event was processed
/// * `Ok(false)` - No event was available (non-blocking return)
/// * `Err(...)` - A fatal error occurred
pub fn process_one_event(
    platform: &mut Box<dyn Platform>,
    event_broadcaster: Option<&EventBroadcaster>,
    remapping_state: Option<&mut RemappingState>,
    latency_recorder: Option<&LatencyRecorder>,
) -> Result<bool, DaemonError> {
    // Try to capture an input event (non-blocking on Windows)
    match platform.capture_input() {
        Ok(event) => {
            let capture_time = Instant::now();
            trace!("Input event: {:?}", event);

            // Get device info from event
            let device_id = event.device_id().map(String::from);
            let input_keycode = event.keycode();

            // Process event through remapping engine if available
            let (output_events, mapping_type, mapping_triggered) =
                if let Some(remap_state) = remapping_state {
                    let (lookup, state) = remap_state.lookup_and_state_mut();
                    let mapping = lookup.find_mapping(input_keycode, state);
                    let mapping_type_str = mapping.map(get_mapping_type);
                    let triggered = mapping.is_some();
                    let outputs = process_event(event.clone(), lookup, state);
                    (outputs, mapping_type_str, triggered)
                } else {
                    // Pass-through mode - no remapping
                    (vec![event.clone()], None, false)
                };

            // Compute output description for broadcast
            let output_desc = if output_events.is_empty() {
                "(suppressed)".to_string()
            } else {
                output_events
                    .iter()
                    .map(|e| format!("{:?}", e.keycode()))
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            // Only inject output events if remapping was triggered
            // In pass-through mode (no remapping), we must NOT inject because:
            // 1. The original key event will reach applications naturally
            // 2. Injecting would cause a feedback loop (captured again by Raw Input)
            if mapping_triggered {
                for output_event in &output_events {
                    if let Err(e) = platform.inject_output(output_event.clone()) {
                        warn!("Failed to inject event: {}", e);
                    }
                }
            }

            // Record latency after injection
            let latency_us = capture_time.elapsed().as_micros() as u64;
            if let Some(recorder) = latency_recorder {
                recorder.record(latency_us);
            }

            // Broadcast key event to WebSocket clients if broadcaster is available
            if let Some(broadcaster) = event_broadcaster {
                let timestamp = current_timestamp_us();

                let event_data = KeyEventData {
                    timestamp,
                    key_code: format!("{:?}", input_keycode),
                    event_type: match event.event_type() {
                        keyrx_core::runtime::KeyEventType::Press => "press".to_string(),
                        keyrx_core::runtime::KeyEventType::Release => "release".to_string(),
                    },
                    input: format!("{:?}", input_keycode),
                    output: output_desc,
                    latency: latency_us,
                    device_id: device_id.clone(),
                    device_name: device_id,
                    mapping_type: mapping_type.map(String::from),
                    mapping_triggered,
                };
                broadcaster.broadcast_key_event(event_data);
            }

            Ok(true)
        }
        Err(_) => {
            // No event available (non-blocking return)
            Ok(false)
        }
    }
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
