//! Unified event registry for FFI callbacks.
//!
//! Provides a single callback registration mechanism replacing per-domain
//! callback functions. All callbacks share the same C signature and receive
//! JSON-serialized payloads.

#![allow(unsafe_code)]

use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Standard FFI callback signature: receives pointer to JSON bytes and length.
pub type EventCallback = unsafe extern "C" fn(*const u8, usize);

/// Event types for all FFI callback categories.
///
/// This enum defines all possible event types across all domains,
/// replacing the previous per-domain callback functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    // Discovery domain events
    DiscoveryProgress,
    DiscoveryDuplicate,
    DiscoverySummary,

    // Engine domain events
    EngineState,

    // Validation domain events (future)
    ValidationProgress,
    ValidationResult,

    // Device domain events (future)
    DeviceConnected,
    DeviceDisconnected,
    DeviceUpdated,

    // Profile domain events
    ProfileUpdated,

    // Testing domain events (future)
    TestProgress,
    TestResult,

    // Analysis domain events (future)
    AnalysisProgress,
    AnalysisResult,

    // Diagnostics domain events (future)
    DiagnosticsLog,
    DiagnosticsMetric,

    // Recording domain events (future)
    RecordingStarted,
    RecordingStopped,

    // Monitor domain events
    RawInput,
    RawOutput,
}

impl Serialize for EventType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(self.code())
    }
}

impl EventType {
    /// Get the event type name for logging.
    pub fn name(&self) -> &'static str {
        match self {
            EventType::DiscoveryProgress => "discovery_progress",
            EventType::DiscoveryDuplicate => "discovery_duplicate",
            EventType::DiscoverySummary => "discovery_summary",
            EventType::EngineState => "engine_state",
            EventType::ValidationProgress => "validation_progress",
            EventType::ValidationResult => "validation_result",
            EventType::DeviceConnected => "device_connected",
            EventType::DeviceDisconnected => "device_disconnected",
            EventType::DeviceUpdated => "device_updated",
            EventType::ProfileUpdated => "profile_updated",
            EventType::TestProgress => "test_progress",
            EventType::TestResult => "test_result",
            EventType::AnalysisProgress => "analysis_progress",
            EventType::AnalysisResult => "analysis_result",
            EventType::DiagnosticsLog => "diagnostics_log",
            EventType::DiagnosticsMetric => "diagnostics_metric",
            EventType::RecordingStarted => "recording_started",
            EventType::RecordingStopped => "recording_stopped",
            EventType::RawInput => "raw_input",
            EventType::RawOutput => "raw_output",
        }
    }

    /// Convert raw integer code to EventType.
    pub fn from_i32(code: i32) -> Option<Self> {
        match code {
            0 => Some(EventType::DiscoveryProgress),
            1 => Some(EventType::DiscoveryDuplicate),
            2 => Some(EventType::DiscoverySummary),
            3 => Some(EventType::EngineState),
            4 => Some(EventType::ValidationProgress),
            5 => Some(EventType::ValidationResult),
            6 => Some(EventType::DeviceConnected),
            7 => Some(EventType::DeviceDisconnected),
            8 => Some(EventType::TestProgress),
            9 => Some(EventType::TestResult),
            10 => Some(EventType::AnalysisProgress),
            11 => Some(EventType::AnalysisResult),
            12 => Some(EventType::DiagnosticsLog),
            13 => Some(EventType::DiagnosticsMetric),
            14 => Some(EventType::RecordingStarted),
            15 => Some(EventType::RecordingStopped),
            16 => Some(EventType::RawInput),
            17 => Some(EventType::RawOutput),
            18 => Some(EventType::DeviceUpdated),
            19 => Some(EventType::ProfileUpdated),
            _ => None,
        }
    }

    /// Get the integer code for this event type.
    pub fn code(&self) -> i32 {
        match self {
            EventType::DiscoveryProgress => 0,
            EventType::DiscoveryDuplicate => 1,
            EventType::DiscoverySummary => 2,
            EventType::EngineState => 3,
            EventType::ValidationProgress => 4,
            EventType::ValidationResult => 5,
            EventType::DeviceConnected => 6,
            EventType::DeviceDisconnected => 7,
            EventType::TestProgress => 8,
            EventType::TestResult => 9,
            EventType::AnalysisProgress => 10,
            EventType::AnalysisResult => 11,
            EventType::DiagnosticsLog => 12,
            EventType::DiagnosticsMetric => 13,
            EventType::RecordingStarted => 14,
            EventType::RecordingStopped => 15,
            EventType::RawInput => 16,
            EventType::RawOutput => 17,
            EventType::DeviceUpdated => 18,
            EventType::ProfileUpdated => 19,
        }
    }
}

/// Unified event envelope that wraps all FFI events.
/// This ensures the event type is always available to the consumer.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UnifiedEvent<'a, T> {
    event_type: EventType,
    #[serde(flatten)]
    payload: &'a T,
}

/// Unified registry for all FFI event callbacks.
///
/// Replaces the previous per-domain callback registration functions with a
/// single unified system. All callbacks receive JSON-serialized payloads.
///
/// # Thread Safety
/// EventRegistry is thread-safe and can be shared across threads using Arc.
/// Registration and invocation can happen concurrently.
///
/// # Example
/// ```no_run
/// use keyrx_core::ffi::events::{EventRegistry, EventType};
///
/// let registry = EventRegistry::new();
///
/// // Register a callback
/// unsafe extern "C" fn my_callback(ptr: *const u8, len: usize) {
///     // Handle JSON payload
/// }
/// registry.register(EventType::DiscoveryProgress, Some(my_callback));
///
/// // Invoke callback with data
/// let data = serde_json::json!({"stage": "scanning", "progress": 0.5});
/// registry.invoke(EventType::DiscoveryProgress, &data);
///
/// // Unregister callback
/// registry.register(EventType::DiscoveryProgress, None);
/// ```
#[derive(Clone)]
pub struct EventRegistry {
    callbacks: Arc<RwLock<HashMap<EventType, EventCallback>>>,
}

impl EventRegistry {
    /// Create a new empty event registry.
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a callback for an event type.
    ///
    /// If `callback` is `Some`, registers the callback. If `None`, unregisters
    /// the callback for that event type. Latest registration replaces previous.
    ///
    /// # Parameters
    /// - `event_type`: The type of event to register for
    /// - `callback`: Optional callback function pointer
    pub fn register(&self, event_type: EventType, callback: Option<EventCallback>) {
        let mut guard = match self.callbacks.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Some(cb) = callback {
            guard.insert(event_type, cb);
        } else {
            guard.remove(&event_type);
        }
    }

    /// Invoke the registered callback for an event type with a JSON payload.
    ///
    /// If no callback is registered for the event type, silently discards the
    /// event (per requirement 2.4). Serializes the payload to JSON and passes
    /// it to the callback.
    ///
    /// # Parameters
    /// - `event_type`: The type of event to invoke
    /// - `payload`: Data to serialize as JSON and pass to callback
    ///
    /// # Errors
    /// Logs warnings if:
    /// - JSON serialization fails
    /// - Lock acquisition fails
    pub fn invoke<T: Serialize>(&self, event_type: EventType, payload: &T) {
        // Get callback with read lock (released immediately)
        let callback = {
            match self.callbacks.read() {
                Ok(guard) => guard.get(&event_type).copied(),
                Err(_) => {
                    tracing::warn!(
                        service = "keyrx",
                        component = "ffi_events",
                        event = "invoke_failed",
                        event_type = event_type.name(),
                        "Failed to acquire read lock for callback invocation"
                    );
                    return;
                }
            }
        };

        // If no callback registered, silently discard (requirement 2.4)
        let Some(cb) = callback else { return };

        // Wrap payload in UnifiedEvent to ensure eventType is present
        let event = UnifiedEvent {
            event_type,
            payload,
        };

        // Serialize payload to JSON
        match serde_json::to_vec(&event) {
            Ok(bytes) => {
                // SAFETY: We leak the memory here to ensure it survives the async FFI call.
                // The Dart side MUST call keyrx_free_event_payload to reclaim this memory.
                let len = bytes.len();
                let ptr = Box::into_raw(bytes.into_boxed_slice()) as *const u8;
                unsafe {
                    cb(ptr, len);
                }
            }
            Err(err) => {
                tracing::warn!(
                    service = "keyrx",
                    component = "ffi_events",
                    event = "serialization_failed",
                    event_type = event_type.name(),
                    error = %err,
                    "Failed to serialize payload for FFI callback"
                );
            }
        }
    }

    /// Check if a callback is registered for an event type.
    pub fn is_registered(&self, event_type: EventType) -> bool {
        self.callbacks
            .read()
            .map(|guard| guard.contains_key(&event_type))
            .unwrap_or_else(|poisoned| poisoned.into_inner().contains_key(&event_type))
    }

    /// Get the number of registered callbacks.
    pub fn callback_count(&self) -> usize {
        self.callbacks
            .read()
            .map(|guard| guard.len())
            .unwrap_or_else(|poisoned| poisoned.into_inner().len())
    }

    /// Clear all registered callbacks.
    pub fn clear(&self) {
        let mut guard = self
            .callbacks
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.clear();
    }
}

impl Default for EventRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;
    use serial_test::serial;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
    static LAST_PAYLOAD: Mutex<Option<Vec<u8>>> = Mutex::new(None);

    unsafe extern "C" fn test_callback(ptr: *const u8, len: usize) {
        CALL_COUNT.fetch_add(1, Ordering::SeqCst);

        // Capture payload for verification
        let slice = std::slice::from_raw_parts(ptr, len);
        if let Ok(mut guard) = LAST_PAYLOAD.lock() {
            *guard = Some(slice.to_vec());
        }
    }

    fn reset_test_state() {
        CALL_COUNT.store(0, Ordering::SeqCst);
        if let Ok(mut guard) = LAST_PAYLOAD.lock() {
            *guard = None;
        }
    }

    #[test]
    #[serial]
    fn registry_starts_empty() {
        let registry = EventRegistry::new();
        assert_eq!(registry.callback_count(), 0);
        assert!(!registry.is_registered(EventType::DiscoveryProgress));
    }

    #[test]
    #[serial]
    fn can_register_and_unregister_callbacks() {
        let registry = EventRegistry::new();

        registry.register(EventType::DiscoveryProgress, Some(test_callback));
        assert!(registry.is_registered(EventType::DiscoveryProgress));
        assert_eq!(registry.callback_count(), 1);

        registry.register(EventType::DiscoveryProgress, None);
        assert!(!registry.is_registered(EventType::DiscoveryProgress));
        assert_eq!(registry.callback_count(), 0);
    }

    #[test]
    #[serial]
    fn latest_registration_replaces_previous() {
        reset_test_state();
        let registry = EventRegistry::new();

        unsafe extern "C" fn callback1(_ptr: *const u8, _len: usize) {}
        unsafe extern "C" fn callback2(_ptr: *const u8, _len: usize) {}

        registry.register(EventType::DiscoveryProgress, Some(callback1));
        registry.register(EventType::DiscoveryProgress, Some(callback2));

        // Should only have one callback registered
        assert_eq!(registry.callback_count(), 1);
    }

    #[test]
    #[serial]
    fn invoke_calls_registered_callback_with_unified_payload() {
        reset_test_state();
        let registry = EventRegistry::new();

        registry.register(EventType::DiscoveryProgress, Some(test_callback));

        let payload = json!({"stage": "scanning", "progress": 0.5});
        registry.invoke(EventType::DiscoveryProgress, &payload);

        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);

        // Verify payload was passed correctly AND has eventType
        if let Ok(guard) = LAST_PAYLOAD.lock() {
            assert!(guard.is_some());
            let json_str = String::from_utf8(guard.clone().unwrap()).unwrap();
            let data: serde_json::Value = serde_json::from_str(&json_str).unwrap();

            // Check eventType presence
            assert_eq!(
                data["eventType"].as_i64(),
                Some(EventType::DiscoveryProgress.code() as i64)
            );

            // Check flattened payload
            assert_eq!(data["stage"].as_str(), Some("scanning"));
            assert_eq!(data["progress"].as_f64(), Some(0.5));
        }
    }

    #[test]
    #[serial]
    fn invoke_without_callback_is_silent() {
        reset_test_state();
        let registry = EventRegistry::new();

        // No callback registered
        let payload = json!({"test": "data"});
        registry.invoke(EventType::DiscoveryProgress, &payload);

        // Should not pass to callback
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 0);
    }

    #[test]
    #[serial]
    fn multiple_event_types_independent() {
        reset_test_state();
        let registry = EventRegistry::new();

        registry.register(EventType::DiscoveryProgress, Some(test_callback));
        registry.register(EventType::EngineState, Some(test_callback));

        assert!(registry.is_registered(EventType::DiscoveryProgress));
        assert!(registry.is_registered(EventType::EngineState));
        assert_eq!(registry.callback_count(), 2);

        let payload = json!({"test": "data"});
        registry.invoke(EventType::DiscoveryProgress, &payload);
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);

        registry.invoke(EventType::EngineState, &payload);
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 2);
    }

    #[test]
    #[serial]
    fn clear_removes_all_callbacks() {
        let registry = EventRegistry::new();

        registry.register(EventType::DiscoveryProgress, Some(test_callback));
        registry.register(EventType::EngineState, Some(test_callback));
        assert_eq!(registry.callback_count(), 2);

        registry.clear();
        assert_eq!(registry.callback_count(), 0);
        assert!(!registry.is_registered(EventType::DiscoveryProgress));
        assert!(!registry.is_registered(EventType::EngineState));
    }

    #[test]
    #[serial]
    fn registry_is_clonable() {
        let registry1 = EventRegistry::new();
        registry1.register(EventType::DiscoveryProgress, Some(test_callback));

        let registry2 = registry1.clone();
        assert!(registry2.is_registered(EventType::DiscoveryProgress));

        // Both share the same underlying storage
        registry2.register(EventType::EngineState, Some(test_callback));
        assert!(registry1.is_registered(EventType::EngineState));
    }

    #[test]
    #[serial]
    fn event_type_codes_are_unique_and_serializable() {
        use std::collections::HashSet;

        let types = [
            EventType::DiscoveryProgress,
            EventType::DiscoveryDuplicate,
            EventType::DiscoverySummary,
            EventType::EngineState,
            EventType::ValidationProgress,
            EventType::ValidationResult,
            EventType::DeviceConnected,
            EventType::DeviceDisconnected,
            EventType::DeviceUpdated,
            EventType::TestProgress,
            EventType::TestResult,
            EventType::AnalysisProgress,
            EventType::AnalysisResult,
            EventType::DiagnosticsLog,
            EventType::DiagnosticsMetric,
            EventType::RecordingStarted,
            EventType::RecordingStopped,
            EventType::RawInput,
            EventType::RawOutput,
        ];

        let mut codes = HashSet::new();
        for t in types.iter() {
            assert!(codes.insert(t.code()), "Duplicate code for {:?}", t);

            // formatting verification
            let json = serde_json::to_string(t).unwrap();
            assert_eq!(json, t.code().to_string());
        }
    }
}
