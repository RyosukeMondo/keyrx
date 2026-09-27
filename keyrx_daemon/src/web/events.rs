//! Monitoring types shared by every transport.
//!
//! `DaemonState`, `KeyEventData` and `LatencyStats` are THE wire format for
//! live state, key events and latency — on the WebSocket push feed, the REST
//! endpoints (`/api/daemon/state`, `/api/metrics/*`), WS-RPC and MCP. They are
//! exported to TypeScript with typeshare (`keyrx_ui/src/types/generated.ts`),
//! and `tests/api_contract_test.rs` pins real responses as fixtures the UI
//! tests consume. Never hand-build these shapes with `json!`.

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::daemon::{LatencySnapshot, TelemetryState};

/// Events broadcast from the daemon to WebSocket clients.
/// Note: This enum uses #[serde(flatten)] which is not supported by typeshare,
/// so we skip it and maintain a manual TypeScript definition in index.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonEvent {
    /// Current daemon state (modifiers, locks, layer).
    #[serde(rename = "state")]
    State {
        payload: DaemonState,
        /// Sequence number for message ordering (WS-004)
        #[serde(rename = "seq")]
        sequence: u64,
    },

    /// Individual key event (press/release).
    #[serde(rename = "event")]
    KeyEvent {
        payload: KeyEventData,
        /// Sequence number for message ordering (WS-004)
        #[serde(rename = "seq")]
        sequence: u64,
    },

    /// Latency statistics update.
    #[serde(rename = "latency")]
    Latency {
        payload: LatencyStats,
        /// Sequence number for message ordering (WS-004)
        #[serde(rename = "seq")]
        sequence: u64,
    },

    /// Error notification (WS-005).
    #[serde(rename = "error")]
    Error {
        payload: ErrorData,
        /// Sequence number for message ordering (WS-004)
        #[serde(rename = "seq")]
        sequence: u64,
    },
}

/// Current daemon state snapshot.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonState {
    /// Active modifier IDs (e.g., ["MD_00", "MD_01"]).
    pub modifiers: Vec<String>,

    /// Active lock IDs (e.g., ["LK_00"]).
    pub locks: Vec<String>,

    /// Current active layer name.
    pub layer: String,

    /// Currently active profile name (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_profile: Option<String>,
}

/// Individual key event data.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyEventData {
    /// Timestamp in microseconds since UNIX epoch.
    #[typeshare(serialized_as = "number")]
    pub timestamp: u64,

    /// Key code (e.g., "KEY_A").
    pub key_code: String,

    /// Event type ("press" or "release").
    pub event_type: String,

    /// Input key (before mapping).
    pub input: String,

    /// Output key (after mapping).
    pub output: String,

    /// Processing latency in microseconds.
    #[typeshare(serialized_as = "number")]
    pub latency: u64,

    /// Device ID (unique identifier for the source device).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,

    /// Device name (human-readable name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,

    /// Mapping type applied (e.g., "simple", "tap_hold", "layer_switch").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_type: Option<String>,

    /// Whether a mapping was triggered for this event.
    pub mapping_triggered: bool,
}

/// Latency statistics.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatencyStats {
    /// Minimum latency in microseconds.
    #[typeshare(serialized_as = "number")]
    pub min: u64,

    /// Average latency in microseconds.
    #[typeshare(serialized_as = "number")]
    pub avg: u64,

    /// Maximum latency in microseconds.
    #[typeshare(serialized_as = "number")]
    pub max: u64,

    /// 95th percentile latency in microseconds.
    #[typeshare(serialized_as = "number")]
    pub p95: u64,

    /// 99th percentile latency in microseconds.
    #[typeshare(serialized_as = "number")]
    pub p99: u64,

    /// Median latency in microseconds.
    #[typeshare(serialized_as = "number")]
    pub p50: u64,

    /// Number of samples the statistics were computed from.
    #[typeshare(serialized_as = "number")]
    pub samples: u64,

    /// Timestamp of this stats snapshot (microseconds since UNIX epoch).
    #[typeshare(serialized_as = "number")]
    pub timestamp: u64,
}

impl LatencyStats {
    /// Wire form of a latency snapshot.
    pub fn from_snapshot(snapshot: &LatencySnapshot) -> Self {
        Self {
            min: snapshot.min_us,
            avg: snapshot.avg_us,
            max: snapshot.max_us,
            p95: snapshot.p95_us,
            p99: snapshot.p99_us,
            p50: snapshot.p50_us,
            samples: snapshot.sample_count,
            timestamp: snapshot.timestamp_us,
        }
    }
}

impl DaemonState {
    /// Wire form of the packed telemetry state. The base layer is `"Base"`.
    pub fn from_telemetry(state: &TelemetryState, active_profile: Option<String>) -> Self {
        Self {
            modifiers: state.modifiers(),
            locks: state.locks(),
            layer: state.active_layer().unwrap_or_else(|| "Base".to_string()),
            active_profile,
        }
    }
}

impl KeyEventData {
    /// One-line text form for the CLI / IPC events tail: `press A -> B`.
    pub fn summary(&self) -> String {
        format!("{} {} -> {}", self.event_type, self.input, self.output)
    }

    /// An unmapped key press of `input`, for tests.
    #[cfg(test)]
    pub(crate) fn test_press(input: &str) -> Self {
        Self {
            timestamp: 0,
            key_code: input.to_string(),
            event_type: "press".to_string(),
            input: input.to_string(),
            output: input.to_string(),
            latency: 0,
            device_id: None,
            device_name: None,
            mapping_type: None,
            mapping_triggered: false,
        }
    }
}

/// Error notification data (WS-005).
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    /// Error code (e.g., "CONFIG_LOAD_FAILED", "PROFILE_NOT_FOUND").
    pub code: String,

    /// Human-readable error message.
    pub message: String,

    /// Additional context (e.g., file path, profile name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Timestamp in microseconds since UNIX epoch.
    #[typeshare(serialized_as = "number")]
    pub timestamp: u64,
}
