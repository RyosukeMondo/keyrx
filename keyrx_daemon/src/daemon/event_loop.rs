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
use keyrx_core::runtime::{check_tap_hold_timeouts, process_event_for_identities};
use log::{info, trace, warn};

use crate::platform::Platform;
use crate::web::events::{DaemonState, KeyEventData};

use super::event_broadcaster::EventBroadcaster;
use super::metrics::LatencyRecorder;
use super::remapping_state::RemappingState;
use super::signals::SignalHandler;
use super::telemetry::{DaemonTelemetry, TelemetryState};
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
        BaseKeyMapping::HoldOnly { .. } => "hold_only",
        BaseKeyMapping::ModifiedOutput { .. } => "modified_output",
        BaseKeyMapping::Sequence { .. } => "sequence",
    }
}

/// Returns current timestamp in microseconds since UNIX epoch.
pub(crate) fn current_timestamp_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

/// Formats output events description for logging and broadcasting.
fn format_output_description(output_events: &[keyrx_core::runtime::KeyEvent]) -> String {
    if output_events.is_empty() {
        "(suppressed)".to_string()
    } else {
        output_events
            .iter()
            .map(|e| format!("{:?}", e.keycode()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Builds a packed telemetry state snapshot from the live device state.
///
/// Mirrors the [`TelemetryState`] bit layout: modifiers 0..128, locks 0..64;
/// the active layer comes from the device block's layer modifiers.
fn build_telemetry_state(
    state: &keyrx_core::runtime::DeviceState,
    layers: &[u8],
) -> TelemetryState {
    let mut snapshot = TelemetryState::empty();
    for id in 0u8..128 {
        if state.is_modifier_active(id) {
            snapshot.set_modifier(id, true);
        }
    }
    for id in 0u8..64 {
        if state.is_lock_active(id) {
            snapshot.set_lock(id, true);
        }
    }
    snapshot.set_active_layer(super::remapping_state::active_layer(state, layers));
    snapshot
}

/// Builds the one wire record of a processed key event, shared by the
/// telemetry ring (REST/IPC/RPC) and the WebSocket feed.
#[allow(clippy::too_many_arguments)]
fn key_event_data(
    event: &keyrx_core::runtime::KeyEvent,
    input_keycode: keyrx_core::config::KeyCode,
    output_desc: &str,
    device_id: Option<String>,
    mapping_type: Option<&'static str>,
    mapping_triggered: bool,
    latency_us: u64,
) -> KeyEventData {
    let event_type = match event.event_type() {
        keyrx_core::runtime::KeyEventType::Press => "press",
        keyrx_core::runtime::KeyEventType::Release => "release",
    };
    KeyEventData {
        timestamp: current_timestamp_us(),
        key_code: format!("{input_keycode:?}"),
        event_type: event_type.to_string(),
        input: format!("{input_keycode:?}"),
        output: output_desc.to_string(),
        latency: latency_us,
        device_id: device_id.clone(),
        device_name: device_id,
        mapping_type: mapping_type.map(String::from),
        mapping_triggered,
    }
}

/// Records a processed event for pull consumers (telemetry) and pushes it to
/// WebSocket clients, with the state snapshot when a mapping fired.
fn publish_event(
    event_data: KeyEventData,
    state: Option<TelemetryState>,
    telemetry: Option<&DaemonTelemetry>,
    event_broadcaster: Option<&EventBroadcaster>,
) {
    if let Some(t) = telemetry {
        t.push_event(event_data.clone());
        if let Some(state) = &state {
            t.update_state(state.clone());
        }
    }
    if let Some(broadcaster) = event_broadcaster {
        broadcaster.broadcast_key_event(event_data);
        if let Some(state) = &state {
            broadcaster.broadcast_state(DaemonState::from_telemetry(state, None));
        }
    }
}

/// Injects timeout-generated events and records metrics.
fn inject_timeout_events(
    timeout_events: &[keyrx_core::runtime::KeyEvent],
    platform: &mut Box<dyn Platform>,
    stats: &mut EventLoopStats,
) {
    for output_event in timeout_events {
        if let Err(e) = platform.inject_output(output_event.clone()) {
            warn!("Failed to inject timeout event: {}", e);
        } else {
            stats.record_event();
            trace!("Tap-hold timeout event injected: {:?}", output_event);
        }
    }
}

/// Handles timeout checks when no input event is available.
fn handle_timeout_events(
    remapping_state: &mut Option<RemappingState>,
    platform: &mut Box<dyn Platform>,
    stats: &mut EventLoopStats,
) {
    if let Some(ref mut remap_state) = remapping_state {
        let current_time = current_timestamp_us();
        for state in remap_state.states_mut() {
            let timeout_events = check_tap_hold_timeouts(current_time, state);
            inject_timeout_events(&timeout_events, platform, stats);
        }
    }
}

/// Logs error when reload callback fails.
fn log_reload_error(e: &DaemonError) {
    warn!(
        "Configuration reload failed, keeping current mappings: {}",
        e
    );
}

/// Handles event capture errors by checking timeouts and sleeping.
fn handle_capture_error(
    platform_waited: bool,
    last_timeout_check: &mut Instant,
    remapping_state: &mut Option<RemappingState>,
    platform: &mut Box<dyn Platform>,
    stats: &mut EventLoopStats,
) {
    // Check tap-hold timeouts every 10ms when idle
    if last_timeout_check.elapsed() >= Duration::from_millis(10) {
        handle_timeout_events(remapping_state, platform, stats);
        *last_timeout_check = Instant::now();
    }

    // Avoid a busy loop, unless the platform already blocked waiting for
    // input (sleeping then would only add input latency).
    if !platform_waited {
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Processes a single input event through remapping and injection pipeline.
#[allow(clippy::too_many_arguments)]
fn process_input_event(
    event: keyrx_core::runtime::KeyEvent,
    remapping_state: &mut Option<RemappingState>,
    platform: &mut Box<dyn Platform>,
    stats: &mut EventLoopStats,
    latency_recorder: Option<&LatencyRecorder>,
    event_broadcaster: Option<&EventBroadcaster>,
    telemetry: Option<&DaemonTelemetry>,
) {
    let capture_time = Instant::now();
    trace!("Input event: {:?}", event);

    let device_id = event.device_id().map(String::from);
    let input_keycode = event.keycode();

    let remapped = remap_event(&event, remapping_state.as_mut(), platform.as_ref());
    let (output_events, mapping_type, mapping_triggered) =
        (remapped.outputs, remapped.mapping_type, remapped.triggered);

    let output_desc = format_output_description(&output_events);

    // Inject output events
    inject_output_events(&output_events, platform, stats);

    // Record latency metrics
    let latency_us = capture_time.elapsed().as_micros() as u64;
    if let Some(recorder) = latency_recorder {
        recorder.record(latency_us);
    }

    let state = remapped.state;
    let event_data = key_event_data(
        &event,
        input_keycode,
        &output_desc,
        device_id,
        mapping_type,
        mapping_triggered,
        latency_us,
    );
    publish_event(event_data, state, telemetry, event_broadcaster);
}

/// Processes event through remapping engine if available.
/// Ensures an event has a real timestamp for tap-hold timeout resolution.
///
/// Platform hooks (especially Windows) may deliver events with timestamp 0.
/// Tap-hold timeout checking compares press timestamps against real system time,
/// so a 0 timestamp causes instant hold resolution. This function stamps real
/// time on events that lack a timestamp, making both platforms behave correctly.
fn ensure_timestamp(event: keyrx_core::runtime::KeyEvent) -> keyrx_core::runtime::KeyEvent {
    if event.timestamp_us() == 0 {
        event.with_timestamp(current_timestamp_us())
    } else {
        event
    }
}

/// The result of running one event through the live config.
struct Remapped {
    outputs: Vec<keyrx_core::runtime::KeyEvent>,
    mapping_type: Option<&'static str>,
    triggered: bool,
    /// The device's state after the event, when a mapping fired.
    state: Option<TelemetryState>,
}

/// Routes `event` to its device's block and runs it through the remapping
/// engine (both platforms). Unmatched devices and pass-through mode return the
/// event unchanged.
fn remap_event(
    event: &keyrx_core::runtime::KeyEvent,
    remapping_state: Option<&mut RemappingState>,
    platform: &dyn Platform,
) -> Remapped {
    let pass_through = || Remapped {
        outputs: vec![event.clone()],
        mapping_type: None,
        triggered: false,
        state: None,
    };
    let Some(remap_state) = remapping_state else {
        return pass_through();
    };
    let Some(routed) = remap_state.route(event.device_id(), |id| device_identities(platform, id))
    else {
        return pass_through();
    };
    if let Some(ime) = platform.query_ime_state() {
        routed.state.set_ime_state(ime);
    }
    let identities: Vec<&str> = routed.identities.iter().map(String::as_str).collect();
    let mapping =
        routed
            .lookup
            .find_mapping_for_identities(event.keycode(), routed.state, &identities);
    let mapping_type = mapping.map(get_mapping_type);
    let triggered = mapping.is_some();
    // Ensure real timestamp for tap-hold timeout resolution
    let outputs = process_event_for_identities(
        ensure_timestamp(event.clone()),
        routed.lookup,
        routed.state,
        &identities,
    );
    let state = triggered.then(|| build_telemetry_state(routed.state, routed.layers));
    Remapped {
        outputs,
        mapping_type,
        triggered,
        state,
    }
}

/// The strings device patterns are matched against for `device_id`: its id,
/// name, path and serial, from the platform's device list (just the id when
/// the platform does not know it).
fn device_identities(platform: &dyn Platform, device_id: &str) -> Vec<String> {
    let mut ids = vec![device_id.to_string()];
    if let Some(serial) = device_id.strip_prefix("serial-") {
        ids.push(serial.to_string());
    }
    if let Ok(devices) = platform.list_devices() {
        if let Some(info) = devices.into_iter().find(|d| d.id == device_id) {
            ids.push(info.name);
            ids.push(info.path);
        }
    }
    ids
}

/// Injects output events through platform.
fn inject_output_events(
    output_events: &[keyrx_core::runtime::KeyEvent],
    platform: &mut Box<dyn Platform>,
    stats: &mut EventLoopStats,
) {
    // On Linux, grab() blocks original events so we MUST always inject.
    // On Windows, Raw Input doesn't block events so they flow naturally.
    for output_event in output_events {
        if let Err(e) = platform.inject_output(output_event.clone()) {
            warn!("Failed to inject event: {}", e);
        } else {
            stats.record_event();
        }
    }
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
/// * `reload_callback` - Called when a reload is requested; returns the remapping
///   state to switch to (`None` = pass-through). On `Err` the current state stays.
/// * `event_broadcaster` - Optional broadcaster for real-time WebSocket updates
/// * `remapping_state` - The live remapping state, replaced in place on reload
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
/// - **SIGHUP** / reload flag: swaps in the state returned by the reload callback
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
///         None,      // No event broadcaster
///         &mut None, // No remapping state (pass-through mode)
///         None, // No latency recording
///         None, // No telemetry
///     )
/// }
/// ```
#[allow(clippy::too_many_arguments)]
pub fn run_event_loop<F>(
    platform: &mut Box<dyn Platform>,
    running: Arc<AtomicBool>,
    signal_handler: &SignalHandler,
    mut reload_callback: F,
    event_broadcaster: Option<&EventBroadcaster>,
    remapping_state: &mut Option<RemappingState>,
    latency_recorder: Option<&LatencyRecorder>,
    telemetry: Option<&DaemonTelemetry>,
) -> Result<(), DaemonError>
where
    F: FnMut() -> Result<Option<RemappingState>, DaemonError>,
{
    info!("Starting event processing loop");

    let mut stats = EventLoopStats::new();
    let mut last_timeout_check = Instant::now();

    // Main event loop
    while running.load(Ordering::SeqCst) {
        // Check for SIGHUP (reload request)
        if signal_handler.check_reload() {
            info!("Reload requested (SIGHUP or profile activation)");
            match reload_callback() {
                Ok(new_state) => {
                    super::release_held_outputs(platform);
                    *remapping_state = new_state;
                    if let Some(t) = telemetry {
                        t.update_state(TelemetryState::empty());
                    }
                }
                Err(e) => log_reload_error(&e),
            }
        }

        // Capture and process input event from platform
        match platform.capture_input() {
            Ok(event) => {
                process_input_event(
                    event,
                    remapping_state,
                    platform,
                    &mut stats,
                    latency_recorder,
                    event_broadcaster,
                    telemetry,
                );
            }
            Err(e) => {
                // Exit if shutdown requested
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                trace!("Event capture returned error (may be timeout): {}", e);
                handle_capture_error(
                    matches!(e, crate::platform::PlatformError::NoInput),
                    &mut last_timeout_check,
                    remapping_state,
                    platform,
                    &mut stats,
                );
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
    telemetry: Option<&DaemonTelemetry>,
) -> Result<bool, DaemonError> {
    // Try to capture an input event (non-blocking on Windows)
    match platform.capture_input() {
        Ok(event) => {
            let capture_time = Instant::now();

            // Ensure real timestamp for tap-hold timeout resolution
            let event = ensure_timestamp(event);

            trace!("Input event: {:?}", event);

            // Get device info from event
            let device_id = event.device_id().map(String::from);
            let input_keycode = event.keycode();

            // Process event through remapping engine (routed per device)
            let remapped = remap_event(&event, remapping_state, platform.as_ref());
            let (output_events, mapping_type, mapping_triggered, state) = (
                remapped.outputs,
                remapped.mapping_type,
                remapped.triggered,
                remapped.state,
            );
            let output_desc = format_output_description(&output_events);

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

            let event_data = key_event_data(
                &event,
                input_keycode,
                &output_desc,
                device_id,
                mapping_type,
                mapping_triggered,
                latency_us,
            );
            publish_event(event_data, state, telemetry, event_broadcaster);

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
