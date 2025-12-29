//! Event types for WebSocket broadcasting.
//!
//! This module defines the event types that are broadcast from the daemon
//! to connected WebSocket clients for real-time monitoring.

use serde::{Deserialize, Serialize};

/// Events broadcast from the daemon to WebSocket clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum DaemonEvent {
    /// Current daemon state (modifiers, locks, layer).
    #[serde(rename = "state")]
    State(DaemonState),

    /// Individual key event (press/release).
    #[serde(rename = "event")]
    KeyEvent(KeyEventData),

    /// Latency statistics update.
    #[serde(rename = "latency")]
    Latency(LatencyStats),
}

/// Current daemon state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonState {
    /// Active modifier IDs (e.g., ["MD_00", "MD_01"]).
    pub modifiers: Vec<String>,

    /// Active lock IDs (e.g., ["LK_00"]).
    pub locks: Vec<String>,

    /// Current active layer name.
    pub layer: String,
}

/// Individual key event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEventData {
    /// Timestamp in microseconds since UNIX epoch.
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
    pub latency: u64,
}

/// Latency statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    /// Minimum latency in microseconds.
    pub min: u64,

    /// Average latency in microseconds.
    pub avg: u64,

    /// Maximum latency in microseconds.
    pub max: u64,

    /// 95th percentile latency in microseconds.
    pub p95: u64,

    /// 99th percentile latency in microseconds.
    pub p99: u64,

    /// Timestamp of this stats snapshot (microseconds since UNIX epoch).
    pub timestamp: u64,
}
