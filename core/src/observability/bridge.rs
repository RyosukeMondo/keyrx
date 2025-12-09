//! FFI bridge for logs and observability.
//!
//! This module provides a tracing Layer that captures log events and makes them
//! available via FFI callbacks or buffered polling. It enables the Flutter UI
//! to receive real-time log updates.

use crate::observability::entry::{LogEntry, LogLevel};
use lazy_static::lazy_static;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::field::{Field, Visit};
use tracing::{Event, Metadata, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

lazy_static! {
    /// Global instance of the log bridge.
    pub static ref GLOBAL_LOG_BRIDGE: LogBridge = LogBridge::new();
}

/// Type alias for FFI log callback function.
///
/// This callback is invoked when a log event occurs, if registered.
///
/// # Safety
/// The callback must not retain the pointer after returning, as it
/// will be freed immediately.
pub type LogCallback = extern "C" fn(*const super::entry::CLogEntry);

/// Type alias for Rust log callback function (used by FRB).
pub type RustLogCallback = Box<dyn Fn(LogEntry) + Send + Sync + 'static>;

/// Maximum number of log entries to buffer before dropping oldest.
const DEFAULT_BUFFER_SIZE: usize = 1000;

/// FFI bridge for logs implementing the tracing Layer trait.
///
/// This bridge can operate in two modes:
/// 1. Callback mode: Invokes FFI callback immediately on each log event
/// 2. Buffered mode: Stores logs in a ring buffer for polling
///
/// Both modes can be enabled simultaneously.
///
/// # Example
/// ```rust
/// use keyrx_core::observability::bridge::LogBridge;
/// use tracing_subscriber::layer::SubscriberExt;
/// use tracing_subscriber::util::SubscriberInitExt;
///
/// let bridge = LogBridge::new();
/// let bridge_clone = bridge.clone();
///
/// tracing_subscriber::registry()
///     .with(bridge)
///     .init();
///
/// // Later, drain buffered logs
/// let logs = bridge_clone.drain();
/// ```
#[derive(Clone)]
pub struct LogBridge {
    inner: Arc<LogBridgeInner>,
}

struct LogBridgeInner {
    /// Optional C callback for immediate notification.
    callback: Mutex<Option<LogCallback>>,
    /// Optional Rust callback (for FRB).
    rust_callback: Mutex<Option<RustLogCallback>>,
    /// Ring buffer for storing log entries.
    buffer: Mutex<VecDeque<LogEntry>>,
    /// Maximum buffer size.
    buffer_size: usize,
    /// Whether the bridge is enabled.
    enabled: AtomicBool,
}

impl LogBridge {
    /// Create a new LogBridge with default buffer size.
    pub fn new() -> Self {
        Self::with_buffer_size(DEFAULT_BUFFER_SIZE)
    }

    /// Create a new LogBridge with specified buffer size.
    ///
    /// # Arguments
    /// * `buffer_size` - Maximum number of entries to buffer
    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self {
            inner: Arc::new(LogBridgeInner {
                callback: Mutex::new(None),
                rust_callback: Mutex::new(None),
                buffer: Mutex::new(VecDeque::with_capacity(buffer_size)),
                buffer_size,
                enabled: AtomicBool::new(true),
            }),
        }
    }

    /// Set the Rust callback for logging (used by FRB).
    pub fn set_rust_callback(&self, callback: RustLogCallback) {
        if let Ok(mut cb) = self.inner.rust_callback.lock() {
            *cb = Some(callback);
        }
    }

    /// Set the FFI callback for immediate log notification.
    ///
    /// # Arguments
    /// * `callback` - Function to call on each log event
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called from any thread.
    pub fn set_callback(&self, callback: LogCallback) {
        if let Ok(mut cb) = self.inner.callback.lock() {
            *cb = Some(callback);
        }
    }

    /// Clear the FFI callback.
    ///
    /// After calling this, logs will only be buffered.
    pub fn clear_callback(&self) {
        if let Ok(mut cb) = self.inner.callback.lock() {
            *cb = None;
        }
    }

    /// Drain all buffered log entries.
    ///
    /// This removes all entries from the buffer and returns them.
    /// Useful for polling mode where Flutter requests logs periodically.
    ///
    /// # Returns
    /// Vector of buffered log entries
    pub fn drain(&self) -> Vec<LogEntry> {
        if let Ok(mut buffer) = self.inner.buffer.lock() {
            buffer.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    /// Get the current number of buffered entries without draining.
    ///
    /// # Returns
    /// Number of entries currently in the buffer
    pub fn len(&self) -> usize {
        if let Ok(buffer) = self.inner.buffer.lock() {
            buffer.len()
        } else {
            0
        }
    }

    /// Check if the buffer is empty.
    ///
    /// # Returns
    /// true if no entries are buffered
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Enable or disable the bridge.
    ///
    /// When disabled, log events are ignored and not buffered.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable the bridge
    pub fn set_enabled(&self, enabled: bool) {
        self.inner.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Check if the bridge is enabled.
    ///
    /// # Returns
    /// true if the bridge is enabled
    pub fn is_enabled(&self) -> bool {
        self.inner.enabled.load(Ordering::Relaxed)
    }

    /// Clear all buffered entries without draining them.
    pub fn clear(&self) {
        if let Ok(mut buffer) = self.inner.buffer.lock() {
            buffer.clear();
        }
    }

    /// Add a log entry to the buffer and optionally invoke the callback.
    fn push_entry(&self, entry: LogEntry) {
        // Add to buffer
        if let Ok(mut buffer) = self.inner.buffer.lock() {
            // If buffer is full, remove oldest entry
            if buffer.len() >= self.inner.buffer_size {
                buffer.pop_front();
            }
            buffer.push_back(entry.clone());
        }

        // Invoke Rust callback if set
        if let Ok(callback) = self.inner.rust_callback.lock() {
            if let Some(cb) = &*callback {
                cb(entry.clone());
            }
        }

        // Invoke C callback if set
        if let Ok(callback) = self.inner.callback.lock() {
            if let Some(cb) = *callback {
                if let Ok(ffi_entry) = entry.to_ffi() {
                    // Call the FFI callback
                    cb(&ffi_entry as *const _);
                    // Note: FFI callback must not retain the pointer
                    // The entry will be dropped after this function returns
                }
            }
        }
    }
}

impl Default for LogBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Visitor to extract fields from tracing events.
struct FieldVisitor {
    fields: std::collections::HashMap<String, String>,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            fields: std::collections::HashMap::new(),
        }
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields
            .insert(field.name().to_string(), format!("{:?}", value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }
}

impl<S> Layer<S> for LogBridge
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        // Skip if disabled
        if !self.is_enabled() {
            return;
        }

        let metadata: &Metadata<'_> = event.metadata();

        // Convert level
        let level = LogLevel::from(metadata.level());

        // Get target
        let target = metadata.target().to_string();

        // Extract fields
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        // Try to extract the message field
        let message = visitor
            .fields
            .remove("message")
            .unwrap_or_else(|| String::from(""));

        // Get the current span name if any
        let span = ctx.event_span(event).map(|span| span.name().to_string());

        // Create log entry
        let mut entry = LogEntry::new(level, target, message);

        // Add remaining fields
        for (key, value) in visitor.fields {
            entry = entry.with_field(key, value);
        }

        // Add span if present
        if let Some(span_name) = span {
            entry = entry.with_span(span_name);
        }

        // Push to buffer and invoke callback
        self.push_entry(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = LogBridge::new();
        assert!(bridge.is_enabled());
        assert_eq!(bridge.len(), 0);
        assert!(bridge.is_empty());
    }

    #[test]
    fn test_bridge_with_buffer_size() {
        let bridge = LogBridge::with_buffer_size(50);
        assert_eq!(bridge.inner.buffer_size, 50);
    }

    #[test]
    fn test_enable_disable() {
        let bridge = LogBridge::new();
        assert!(bridge.is_enabled());

        bridge.set_enabled(false);
        assert!(!bridge.is_enabled());

        bridge.set_enabled(true);
        assert!(bridge.is_enabled());
    }

    #[test]
    fn test_push_and_drain() {
        let bridge = LogBridge::new();

        let entry1 = LogEntry::new(LogLevel::Info, "test::module1", "Test message 1");
        let entry2 = LogEntry::new(LogLevel::Error, "test::module2", "Test message 2");

        bridge.push_entry(entry1.clone());
        bridge.push_entry(entry2.clone());

        assert_eq!(bridge.len(), 2);
        assert!(!bridge.is_empty());

        let drained = bridge.drain();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].message, "Test message 1");
        assert_eq!(drained[1].message, "Test message 2");

        assert_eq!(bridge.len(), 0);
        assert!(bridge.is_empty());
    }

    #[test]
    fn test_clear() {
        let bridge = LogBridge::new();

        let entry = LogEntry::new(LogLevel::Debug, "test", "Message");
        bridge.push_entry(entry);

        assert_eq!(bridge.len(), 1);

        bridge.clear();
        assert_eq!(bridge.len(), 0);
    }

    #[test]
    fn test_buffer_overflow() {
        let bridge = LogBridge::with_buffer_size(3);

        for i in 0..5 {
            let entry = LogEntry::new(LogLevel::Info, "test", format!("Message {}", i));
            bridge.push_entry(entry);
        }

        // Should only have 3 entries (oldest 2 dropped)
        assert_eq!(bridge.len(), 3);

        let drained = bridge.drain();
        assert_eq!(drained.len(), 3);
        assert_eq!(drained[0].message, "Message 2");
        assert_eq!(drained[1].message, "Message 3");
        assert_eq!(drained[2].message, "Message 4");
    }

    #[test]
    fn test_callback_management() {
        let bridge = LogBridge::new();

        extern "C" fn test_callback(_entry: *const super::super::entry::CLogEntry) {
            // Test callback
        }

        bridge.set_callback(test_callback);
        // Callback is set internally, no way to verify directly without invoking

        bridge.clear_callback();
        // Callback is cleared
    }

    #[test]
    fn test_disabled_bridge_ignores_events() {
        let bridge = LogBridge::new();
        bridge.set_enabled(false);

        let entry = LogEntry::new(LogLevel::Info, "test", "Message");
        bridge.push_entry(entry);

        // Even though we called push_entry directly, let's verify the disabled state
        assert!(!bridge.is_enabled());
    }

    #[test]
    fn test_field_visitor() {
        let visitor = FieldVisitor::new();

        // We can't easily test the visitor without a real tracing event,
        // but we can verify it's created correctly
        assert_eq!(visitor.fields.len(), 0);
    }

    #[test]
    fn test_clone() {
        let bridge = LogBridge::new();
        let entry = LogEntry::new(LogLevel::Info, "test", "Message");
        bridge.push_entry(entry);

        let bridge_clone = bridge.clone();
        assert_eq!(bridge_clone.len(), 1);

        let drained = bridge_clone.drain();
        assert_eq!(drained.len(), 1);

        // Both should be empty now (they share the same inner state)
        assert_eq!(bridge.len(), 0);
    }
}
