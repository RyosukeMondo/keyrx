//! Panic and recovery telemetry tracking.
//!
//! This module provides global tracking of panic events and recovery actions,
//! enabling the Flutter UI to monitor system health and show recovery notifications.
//!
//! # Design Philosophy
//!
//! - Thread-safe global counters for panics and recoveries
//! - No allocation in panic paths (pre-allocated event ring buffer)
//! - Efficient atomic operations for counter increments
//! - Detailed event log with backtraces for debugging
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::safety::panic_telemetry::{record_panic, get_telemetry};
//!
//! // Record a panic event
//! record_panic("keyboard_callback", "index out of bounds", None);
//!
//! // Get current telemetry stats
//! let stats = get_telemetry();
//! println!("Total panics: {}", stats.total_panics);
//! println!("Recoveries: {}", stats.total_recoveries);
//! ```

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of panic events to store in the ring buffer.
const MAX_PANIC_EVENTS: usize = 100;

/// Global panic counter.
static TOTAL_PANICS: AtomicU64 = AtomicU64::new(0);

/// Global recovery counter.
static TOTAL_RECOVERIES: AtomicU64 = AtomicU64::new(0);

/// Global circuit breaker open counter.
static CIRCUIT_BREAKER_OPENS: AtomicU64 = AtomicU64::new(0);

/// Global circuit breaker close counter.
static CIRCUIT_BREAKER_CLOSES: AtomicU64 = AtomicU64::new(0);

/// Ring buffer of recent panic events.
static PANIC_EVENTS: RwLock<Vec<PanicEvent>> = RwLock::new(Vec::new());

/// A panic event with context and backtrace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicEvent {
    /// Unix timestamp in milliseconds when the panic occurred.
    pub timestamp: u64,

    /// Context where the panic occurred (e.g., "keyboard_callback").
    pub context: String,

    /// Panic message.
    pub message: String,

    /// Optional backtrace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backtrace: Option<String>,

    /// Whether recovery was successful.
    pub recovered: bool,
}

/// Panic telemetry statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicTelemetry {
    /// Total number of panics caught.
    pub total_panics: u64,

    /// Total number of successful recoveries.
    pub total_recoveries: u64,

    /// Total number of times circuit breaker opened.
    pub circuit_breaker_opens: u64,

    /// Total number of times circuit breaker closed (recovered).
    pub circuit_breaker_closes: u64,

    /// Recent panic events (up to MAX_PANIC_EVENTS).
    pub recent_events: Vec<PanicEvent>,
}

/// Records a panic event in telemetry.
///
/// This function is called by `PanicGuard` when a panic is caught.
/// It increments counters and stores the event in the ring buffer.
///
/// # Arguments
///
/// * `context` - Where the panic occurred (e.g., "keyboard_callback")
/// * `message` - The panic message
/// * `backtrace` - Optional backtrace string
///
/// # Example
///
/// ```ignore
/// record_panic("input_processing", "division by zero", None);
/// ```
pub fn record_panic(
    context: impl Into<String>,
    message: impl Into<String>,
    backtrace: Option<String>,
) {
    // Increment panic counter
    TOTAL_PANICS.fetch_add(1, Ordering::Relaxed);

    let context_str = context.into();

    // Create event
    let event = PanicEvent {
        timestamp: current_timestamp_ms(),
        context: context_str.clone(),
        message: message.into(),
        backtrace,
        recovered: false,
    };

    // Store in ring buffer
    if let Ok(mut events) = PANIC_EVENTS.write() {
        // Keep only the most recent MAX_PANIC_EVENTS
        let events_len = events.len();
        if events_len >= MAX_PANIC_EVENTS {
            let drain_count = events_len - MAX_PANIC_EVENTS + 1;
            events.drain(0..drain_count);
        }

        events.push(event);
    }

    tracing::warn!(
        service = "keyrx",
        event = "panic_caught",
        context = %context_str,
        total_panics = TOTAL_PANICS.load(Ordering::Relaxed),
        "Panic caught and recorded in telemetry"
    );
}

/// Records a successful recovery from a panic.
///
/// This should be called after a panic was caught and handled successfully.
///
/// # Example
///
/// ```ignore
/// if panic_result.is_ok() {
///     record_recovery();
/// }
/// ```
pub fn record_recovery() {
    // Only record recovery if we have outstanding panics
    let current_panics = TOTAL_PANICS.load(Ordering::Relaxed);
    let current_recoveries = TOTAL_RECOVERIES.load(Ordering::Relaxed);

    if current_recoveries < current_panics {
        TOTAL_RECOVERIES.fetch_add(1, Ordering::Relaxed);

        // Mark the most recent panic event as recovered
        if let Ok(mut events) = PANIC_EVENTS.write() {
            if let Some(last_event) = events.last_mut() {
                last_event.recovered = true;
            }
        }

        tracing::info!(
            service = "keyrx",
            event = "panic_recovery",
            total_recoveries = current_recoveries + 1,
            "Recovery from panic successful"
        );
    }
}

/// Records a circuit breaker opening.
///
/// This should be called when a circuit breaker transitions to the open state.
///
/// # Example
///
/// ```ignore
/// record_circuit_breaker_open("driver", 5);
/// ```
pub fn record_circuit_breaker_open(context: impl Into<String>, failure_count: usize) {
    CIRCUIT_BREAKER_OPENS.fetch_add(1, Ordering::Relaxed);

    tracing::error!(
        service = "keyrx",
        event = "circuit_breaker_open",
        context = context.into(),
        failure_count = failure_count,
        total_opens = CIRCUIT_BREAKER_OPENS.load(Ordering::Relaxed),
        "Circuit breaker opened"
    );
}

/// Records a circuit breaker closing (recovery).
///
/// This should be called when a circuit breaker transitions back to closed state.
///
/// # Example
///
/// ```ignore
/// record_circuit_breaker_close("driver");
/// ```
pub fn record_circuit_breaker_close(context: impl Into<String>) {
    CIRCUIT_BREAKER_CLOSES.fetch_add(1, Ordering::Relaxed);

    tracing::info!(
        service = "keyrx",
        event = "circuit_breaker_close",
        context = context.into(),
        total_closes = CIRCUIT_BREAKER_CLOSES.load(Ordering::Relaxed),
        "Circuit breaker closed after recovery"
    );
}

/// Gets current panic telemetry statistics.
///
/// Returns a snapshot of all counters and recent events.
///
/// # Example
///
/// ```ignore
/// let stats = get_telemetry();
/// if stats.total_panics > 0 {
///     show_warning_notification();
/// }
/// ```
pub fn get_telemetry() -> PanicTelemetry {
    let recent_events = PANIC_EVENTS
        .read()
        .map(|events| events.clone())
        .unwrap_or_default();

    PanicTelemetry {
        total_panics: TOTAL_PANICS.load(Ordering::Relaxed),
        total_recoveries: TOTAL_RECOVERIES.load(Ordering::Relaxed),
        circuit_breaker_opens: CIRCUIT_BREAKER_OPENS.load(Ordering::Relaxed),
        circuit_breaker_closes: CIRCUIT_BREAKER_CLOSES.load(Ordering::Relaxed),
        recent_events,
    }
}

/// Resets all telemetry counters and clears events.
///
/// This is primarily for testing. In production, counters should accumulate
/// for the lifetime of the process.
///
/// # Example
///
/// ```ignore
/// reset_telemetry(); // Start fresh for new test
/// ```
pub fn reset_telemetry() {
    TOTAL_PANICS.store(0, Ordering::Relaxed);
    TOTAL_RECOVERIES.store(0, Ordering::Relaxed);
    CIRCUIT_BREAKER_OPENS.store(0, Ordering::Relaxed);
    CIRCUIT_BREAKER_CLOSES.store(0, Ordering::Relaxed);

    if let Ok(mut events) = PANIC_EVENTS.write() {
        events.clear();
    }

    tracing::debug!(
        service = "keyrx",
        event = "telemetry_reset",
        "Panic telemetry reset"
    );
}

/// Gets the current Unix timestamp in milliseconds.
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn test_record_panic() {
        reset_telemetry();

        record_panic("test_context", "test panic", None);

        let stats = get_telemetry();
        assert_eq!(stats.total_panics, 1);
        assert_eq!(stats.recent_events.len(), 1);
        assert_eq!(stats.recent_events[0].context, "test_context");
        assert_eq!(stats.recent_events[0].message, "test panic");
        assert!(!stats.recent_events[0].recovered);
    }

    #[test]
    #[serial_test::serial]
    fn test_record_recovery() {
        reset_telemetry();

        record_panic("test_context", "test panic", None);
        record_recovery();

        let stats = get_telemetry();
        assert_eq!(stats.total_panics, 1);
        assert_eq!(stats.total_recoveries, 1);
        assert_eq!(stats.recent_events.len(), 1);
        assert!(stats.recent_events[0].recovered);
    }

    #[test]
    #[serial_test::serial]
    fn test_circuit_breaker_telemetry() {
        reset_telemetry();

        record_circuit_breaker_open("driver", 5);
        record_circuit_breaker_close("driver");

        let stats = get_telemetry();
        assert_eq!(stats.circuit_breaker_opens, 1);
        assert_eq!(stats.circuit_breaker_closes, 1);
    }

    #[test]
    #[serial_test::serial] // Run sequentially to avoid shared global state issues
    fn test_ring_buffer_limit() {
        reset_telemetry();

        // Record more than MAX_PANIC_EVENTS
        for i in 0..(MAX_PANIC_EVENTS + 50) {
            record_panic(format!("context_{}", i), format!("panic_{}", i), None);
        }

        let stats = get_telemetry();
        assert_eq!(stats.total_panics, (MAX_PANIC_EVENTS + 50) as u64);
        assert_eq!(stats.recent_events.len(), MAX_PANIC_EVENTS);

        // Verify we kept the most recent events
        let last_event = &stats.recent_events[MAX_PANIC_EVENTS - 1];
        assert_eq!(
            last_event.context,
            format!("context_{}", MAX_PANIC_EVENTS + 49)
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_multiple_panics() {
        reset_telemetry();

        record_panic("context1", "panic1", Some("backtrace1".to_string()));
        record_panic("context2", "panic2", None);
        record_panic("context3", "panic3", Some("backtrace3".to_string()));

        let stats = get_telemetry();
        assert_eq!(stats.total_panics, 3);
        assert_eq!(stats.recent_events.len(), 3);
        assert_eq!(stats.recent_events[0].context, "context1");
        assert_eq!(stats.recent_events[1].context, "context2");
        assert_eq!(stats.recent_events[2].context, "context3");
    }

    #[test]
    #[serial_test::serial]
    fn test_reset_telemetry() {
        record_panic("test", "panic", None);
        record_recovery();
        record_circuit_breaker_open("test", 1);

        reset_telemetry();

        let stats = get_telemetry();
        assert_eq!(stats.total_panics, 0);
        assert_eq!(stats.total_recoveries, 0);
        assert_eq!(stats.circuit_breaker_opens, 0);
        assert_eq!(stats.circuit_breaker_closes, 0);
        assert_eq!(stats.recent_events.len(), 0);
    }

    #[test]
    #[serial_test::serial] // Run sequentially to avoid shared global state issues
    fn test_telemetry_serialization() {
        reset_telemetry();

        record_panic("test", "message", Some("backtrace".to_string()));

        let stats = get_telemetry();
        let json = serde_json::to_string(&stats).unwrap();

        assert!(json.contains("total_panics"));
        assert!(json.contains("recent_events"));

        let deserialized: PanicTelemetry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_panics, 1);
    }
}
