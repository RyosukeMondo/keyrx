//! Event types for WebSocket broadcasting.
//!
//! This module defines the event types that are broadcast from the daemon
//! to connected WebSocket clients for real-time monitoring.

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/// Events broadcast from the daemon to WebSocket clients.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonEvent {
    /// Current daemon state (modifiers, locks, layer).
    #[serde(rename = "state")]
    State {
        #[serde(flatten)]
        data: DaemonState,
        /// Sequence number for message ordering (WS-004)
        #[serde(rename = "seq")]
        sequence: u64,
    },

    /// Individual key event (press/release).
    #[serde(rename = "event")]
    KeyEvent {
        #[serde(flatten)]
        data: KeyEventData,
        /// Sequence number for message ordering (WS-004)
        #[serde(rename = "seq")]
        sequence: u64,
    },

    /// Latency statistics update.
    #[serde(rename = "latency")]
    Latency {
        #[serde(flatten)]
        data: LatencyStats,
        /// Sequence number for message ordering (WS-004)
        #[serde(rename = "seq")]
        sequence: u64,
    },

    /// Error notification (WS-005).
    #[serde(rename = "error")]
    Error {
        #[serde(flatten)]
        data: ErrorData,
        /// Sequence number for message ordering (WS-004)
        #[serde(rename = "seq")]
        sequence: u64,
    },
}

/// Current daemon state snapshot.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEventData {
    /// Timestamp in microseconds since UNIX epoch.
    #[typeshare(serialized_as = "number")]
    pub timestamp: u64,

    /// Key code (e.g., "KEY_A").
    #[serde(rename = "keyCode")]
    pub key_code: String,

    /// Event type ("press" or "release").
    #[serde(rename = "eventType")]
    pub event_type: String,

    /// Input key (before mapping).
    pub input: String,

    /// Output key (after mapping).
    pub output: String,

    /// Processing latency in microseconds.
    #[typeshare(serialized_as = "number")]
    pub latency: u64,

    /// Device ID (unique identifier for the source device).
    #[serde(rename = "deviceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,

    /// Device name (human-readable name).
    #[serde(rename = "deviceName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,

    /// Mapping type applied (e.g., "simple", "tap_hold", "layer_switch").
    #[serde(rename = "mappingType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_type: Option<String>,

    /// Whether a mapping was triggered for this event.
    #[serde(rename = "mappingTriggered")]
    pub mapping_triggered: bool,
}

/// Latency statistics.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Timestamp of this stats snapshot (microseconds since UNIX epoch).
    #[typeshare(serialized_as = "number")]
    pub timestamp: u64,
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
