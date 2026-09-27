//! Shared daemon telemetry — the single source of truth for live runtime state,
//! latency, and recent events queried over IPC and the web API.
//!
//! # Why this exists
//!
//! The keyboard event loop (`event_loop.rs`) owns the live `DeviceState` and a
//! [`LatencyRecorder`](super::metrics::LatencyRecorder), and broadcasts ephemeral
//! events to WebSocket clients. But pull-based consumers — the `keyrx metrics`
//! CLI and the REST endpoints (`/api/daemon/state`, `/api/metrics/*`) — need to
//! *query* current state, latency, and a tail of recent events at any time.
//!
//! [`DaemonTelemetry`] is the shared, thread-safe holder the event loop writes to
//! and IPC/web read from. It is cheap to clone via `Arc` and uses short, poison-
//! tolerant critical sections.
//!
//! # State bit layout (255-bit vector)
//!
//! The IPC `GetState` contract is a 255-element `Vec<bool>` with this packing,
//! mirrored by the REST `/api/daemon/state` handler:
//!
//! | Bits      | Meaning                  |
//! |-----------|--------------------------|
//! | `0..128`  | Modifiers `MD_00..MD_127`|
//! | `128..192`| Locks `LK_00..LK_63`     |
//! | `192..255`| Active layers            |
//!
//! [`TelemetryState`] owns the parse/format of this layout so callers don't
//! duplicate it.

use std::collections::VecDeque;
use std::sync::{Mutex, RwLock};

use super::metrics::LatencySnapshot;

/// Total number of bits in the packed state vector.
pub const STATE_BITS: usize = 255;

/// Modifier bits occupy indices `0..MODIFIER_END`.
const MODIFIER_END: usize = 128;
/// Lock bits occupy indices `MODIFIER_END..LOCK_END`.
const LOCK_END: usize = 192;

/// Default capacity of the recent-events ring buffer.
const DEFAULT_EVENTS_CAPACITY: usize = 1000;

/// A point-in-time snapshot of daemon modifier/lock/layer state.
///
/// Internally stores the packed 255-bit vector (see module docs) and exposes
/// typed accessors. Construct from a raw vector via [`from_raw`](Self::from_raw)
/// or build incrementally with [`set_modifier`](Self::set_modifier) etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryState {
    raw: Vec<bool>,
}

impl TelemetryState {
    /// Returns an all-inactive state (no modifiers, locks, or layers).
    pub fn empty() -> Self {
        Self {
            raw: vec![false; STATE_BITS],
        }
    }

    /// Builds a state from a raw bit vector, padding/truncating to [`STATE_BITS`].
    pub fn from_raw(mut raw: Vec<bool>) -> Self {
        raw.resize(STATE_BITS, false);
        Self { raw }
    }

    /// Marks modifier `id` (0-based, 0..128) active or inactive.
    pub fn set_modifier(&mut self, id: u8, active: bool) {
        let idx = id as usize;
        if idx < MODIFIER_END {
            self.raw[idx] = active;
        }
    }

    /// Marks lock `id` (0-based, 0..64) active or inactive.
    pub fn set_lock(&mut self, id: u8, active: bool) {
        let idx = MODIFIER_END + id as usize;
        if idx < LOCK_END {
            self.raw[idx] = active;
        }
    }

    /// Marks layer `id` (0-based, 0..63) active or inactive.
    pub fn set_layer(&mut self, id: u8, active: bool) {
        let idx = LOCK_END + id as usize;
        if idx < STATE_BITS {
            self.raw[idx] = active;
        }
    }

    /// Returns the packed 255-bit vector (the IPC `GetState` payload).
    pub fn raw(&self) -> &[bool] {
        &self.raw
    }

    /// Consumes the state, returning the owned packed vector.
    pub fn into_raw(self) -> Vec<bool> {
        self.raw
    }

    /// Returns active modifier labels (`MD_00`..`MD_127`).
    pub fn modifiers(&self) -> Vec<String> {
        self.raw[..MODIFIER_END]
            .iter()
            .enumerate()
            .filter(|(_, &on)| on)
            .map(|(i, _)| format!("MD_{i:02}"))
            .collect()
    }

    /// Returns active lock labels (`LK_00`..`LK_63`).
    pub fn locks(&self) -> Vec<String> {
        self.raw[MODIFIER_END..LOCK_END]
            .iter()
            .enumerate()
            .filter(|(_, &on)| on)
            .map(|(i, _)| format!("LK_{i:02}"))
            .collect()
    }

    /// Returns the active layer label, if any layer bit is set.
    ///
    /// Returns the lowest-indexed active layer (`layer_<n>`), or `None` for the
    /// implicit base layer (no layer bits set).
    pub fn active_layer(&self) -> Option<String> {
        self.raw[LOCK_END..STATE_BITS]
            .iter()
            .position(|&on| on)
            .map(|i| format!("layer_{i}"))
    }

    /// Number of active modifiers.
    pub fn active_modifier_count(&self) -> usize {
        self.raw[..MODIFIER_END].iter().filter(|&&on| on).count()
    }

    /// Number of active locks.
    pub fn active_lock_count(&self) -> usize {
        self.raw[MODIFIER_END..LOCK_END]
            .iter()
            .filter(|&&on| on)
            .count()
    }
}

impl Default for TelemetryState {
    fn default() -> Self {
        Self::empty()
    }
}

/// Thread-safe holder for live daemon telemetry, shared via `Arc`.
///
/// The event loop calls [`update_state`](Self::update_state),
/// [`update_latency`](Self::update_latency), and [`push_event`](Self::push_event);
/// IPC/web read via [`state`](Self::state), [`latency`](Self::latency), and
/// [`recent_events`](Self::recent_events). All locks are held only briefly and
/// recover from poisoning rather than panicking.
#[derive(Debug)]
pub struct DaemonTelemetry {
    state: RwLock<TelemetryState>,
    latency: RwLock<LatencySnapshot>,
    events: Mutex<VecDeque<String>>,
    events_capacity: usize,
}

impl DaemonTelemetry {
    /// Creates telemetry with the default event-history capacity.
    pub fn new() -> Self {
        Self::with_events_capacity(DEFAULT_EVENTS_CAPACITY)
    }

    /// Creates telemetry with a custom event-history capacity.
    pub fn with_events_capacity(events_capacity: usize) -> Self {
        Self {
            state: RwLock::new(TelemetryState::empty()),
            latency: RwLock::new(LatencySnapshot::empty()),
            events: Mutex::new(VecDeque::with_capacity(events_capacity.min(1024))),
            events_capacity,
        }
    }

    /// Replaces the current state snapshot (called when modifier/lock/layer changes).
    pub fn update_state(&self, state: TelemetryState) {
        *self.state.write().unwrap_or_else(|e| e.into_inner()) = state;
    }

    /// Returns a clone of the current state snapshot.
    pub fn state(&self) -> TelemetryState {
        self.state.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Returns the packed 255-bit state vector (IPC `GetState` payload).
    pub fn raw_state(&self) -> Vec<bool> {
        self.state
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .raw()
            .to_vec()
    }

    /// Replaces the latest latency snapshot (called by the broadcast task ~1Hz).
    pub fn update_latency(&self, snapshot: LatencySnapshot) {
        *self.latency.write().unwrap_or_else(|e| e.into_inner()) = snapshot;
    }

    /// Returns the latest latency snapshot.
    pub fn latency(&self) -> LatencySnapshot {
        self.latency
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Appends a recent-event description, evicting the oldest past capacity.
    pub fn push_event(&self, description: String) {
        let mut events = self.events.lock().unwrap_or_else(|e| e.into_inner());
        if events.len() >= self.events_capacity {
            events.pop_front();
        }
        events.push_back(description);
    }

    /// Returns the most recent `count` events, oldest first.
    pub fn recent_events(&self, count: usize) -> Vec<String> {
        let events = self.events.lock().unwrap_or_else(|e| e.into_inner());
        let start = events.len().saturating_sub(count);
        events.iter().skip(start).cloned().collect()
    }

    /// Clears the recent-event history. Returns the number of events removed.
    pub fn clear_events(&self) -> usize {
        let mut events = self.events.lock().unwrap_or_else(|e| e.into_inner());
        let removed = events.len();
        events.clear();
        removed
    }
}

impl Default for DaemonTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_state_all_inactive() {
        let s = TelemetryState::empty();
        assert_eq!(s.raw().len(), STATE_BITS);
        assert!(s.modifiers().is_empty());
        assert!(s.locks().is_empty());
        assert_eq!(s.active_layer(), None);
        assert_eq!(s.active_modifier_count(), 0);
        assert_eq!(s.active_lock_count(), 0);
    }

    #[test]
    fn test_from_raw_pads_and_truncates() {
        assert_eq!(TelemetryState::from_raw(vec![]).raw().len(), STATE_BITS);
        assert_eq!(
            TelemetryState::from_raw(vec![true; 1000]).raw().len(),
            STATE_BITS
        );
    }

    #[test]
    fn test_set_and_read_modifiers_locks_layers() {
        let mut s = TelemetryState::empty();
        s.set_modifier(0, true);
        s.set_modifier(5, true);
        s.set_lock(1, true);
        s.set_layer(2, true);

        assert_eq!(s.modifiers(), vec!["MD_00", "MD_05"]);
        assert_eq!(s.locks(), vec!["LK_01"]);
        assert_eq!(s.active_layer(), Some("layer_2".to_string()));
        assert_eq!(s.active_modifier_count(), 2);
        assert_eq!(s.active_lock_count(), 1);
    }

    #[test]
    fn test_set_out_of_range_is_ignored() {
        let mut s = TelemetryState::empty();
        s.set_modifier(200, true); // beyond 128 modifier slots
        s.set_lock(200, true); // beyond 64 lock slots
        assert!(s.modifiers().is_empty());
        assert!(s.locks().is_empty());
    }

    #[test]
    fn test_raw_layout_matches_packing() {
        let mut s = TelemetryState::empty();
        s.set_lock(0, true); // -> raw index 128
        s.set_layer(0, true); // -> raw index 192
        assert!(s.raw()[128]);
        assert!(s.raw()[192]);
        assert!(!s.raw()[0]);
    }

    #[test]
    fn test_telemetry_state_roundtrip() {
        let t = DaemonTelemetry::new();
        let mut s = TelemetryState::empty();
        s.set_modifier(3, true);
        t.update_state(s.clone());
        assert_eq!(t.state(), s);
        assert!(t.raw_state()[3]);
    }

    #[test]
    fn test_latency_roundtrip() {
        let t = DaemonTelemetry::new();
        let mut snap = LatencySnapshot::empty();
        snap.avg_us = 1234;
        snap.p99_us = 5678;
        t.update_latency(snap.clone());
        let got = t.latency();
        assert_eq!(got.avg_us, 1234);
        assert_eq!(got.p99_us, 5678);
    }

    #[test]
    fn test_events_ring_buffer_bounded() {
        let t = DaemonTelemetry::with_events_capacity(3);
        for i in 0..5 {
            t.push_event(format!("e{i}"));
        }
        // Oldest two evicted; newest three remain, oldest first.
        assert_eq!(t.recent_events(10), vec!["e2", "e3", "e4"]);
    }

    #[test]
    fn test_recent_events_count_limit() {
        let t = DaemonTelemetry::new();
        for i in 0..10 {
            t.push_event(format!("e{i}"));
        }
        assert_eq!(t.recent_events(2), vec!["e8", "e9"]);
        assert_eq!(t.recent_events(0), Vec::<String>::new());
    }

    #[test]
    fn test_clear_events() {
        let t = DaemonTelemetry::new();
        t.push_event("a".to_string());
        t.push_event("b".to_string());
        assert_eq!(t.clear_events(), 2);
        assert!(t.recent_events(10).is_empty());
        assert_eq!(t.clear_events(), 0);
    }
}
