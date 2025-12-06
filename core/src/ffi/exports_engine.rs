//! Engine state streaming exports for FFI.
//!
//! Provides a legacy-compatible `keyrx_on_state` callback registration
//! plus a delta-first state publisher with automatic full snapshot
//! fallback when versions drift.
#![allow(unsafe_code)]

use crate::engine::state::{snapshot::StateSnapshot, EngineState, StateDelta};
use crate::ffi::domains::engine::global_event_registry;
use crate::ffi::events::{EventCallback, EventType};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

// Sentinel indicating that no state has been sent yet.
const INITIAL_VERSION_SENTINEL: u64 = u64::MAX;

fn last_sent_version() -> &'static AtomicU64 {
    static VERSION: OnceLock<AtomicU64> = OnceLock::new();
    VERSION.get_or_init(|| AtomicU64::new(INITIAL_VERSION_SENTINEL))
}

fn reset_last_sent_version() {
    last_sent_version().store(INITIAL_VERSION_SENTINEL, Ordering::Relaxed);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StateUpdateEvent {
    delta: StateDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    full_snapshot: Option<StateSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    latency_us: Option<u64>,
}

fn should_send_full(delta: &StateDelta, last_version: u64, has_sent: bool) -> bool {
    let version_mismatch = has_sent && last_version != delta.from_version;
    let first_message = !has_sent;
    let delta_requests_full = delta.should_use_full_sync();

    first_message || version_mismatch || delta_requests_full
}

fn build_state_update_event(
    state: &EngineState,
    event: Option<String>,
    latency_us: Option<u64>,
) -> StateUpdateEvent {
    let delta = state.take_delta();
    let snapshot: StateSnapshot = state.into();

    let last_version = last_sent_version().load(Ordering::Relaxed);
    let has_sent = last_version != INITIAL_VERSION_SENTINEL;
    let send_full = should_send_full(&delta, last_version, has_sent);

    let full_snapshot = if send_full {
        Some(snapshot.clone())
    } else {
        None
    };

    let next_version = if send_full {
        snapshot.version
    } else {
        delta.to_version
    };
    last_sent_version().store(next_version, Ordering::Relaxed);

    StateUpdateEvent {
        delta,
        full_snapshot,
        event,
        latency_us,
    }
}

/// Publish an EngineState update to registered FFI listeners.
///
/// Sends a delta-first payload with an optional full snapshot fallback when:
/// - This is the first message after registration
/// - The previous sent version does not match the delta's `from_version`
/// - The delta heuristic indicates a full sync is more efficient
pub fn publish_state_update(state: &EngineState, event: Option<String>, latency_us: Option<u64>) {
    let payload = build_state_update_event(state, event, latency_us);
    global_event_registry().invoke(EventType::EngineState, &payload);
}

/// Register a legacy EngineState callback.
///
/// This provides compatibility for the older `keyrx_on_state` API expected by
/// the Flutter bindings. Call with `None` to unregister.
#[no_mangle]
pub extern "C" fn keyrx_on_state(callback: Option<EventCallback>) {
    global_event_registry().register(EventType::EngineState, callback);
    // Force the next publish to include a full snapshot for resync.
    reset_last_sent_version();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::state::{Mutation, StateResult};
    use crate::engine::{KeyCode, TimingConfig};

    fn apply_keydown(state: &mut EngineState) -> StateResult<()> {
        state
            .apply(Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 100,
                is_repeat: false,
            })
            .map(|_| ())
    }

    #[test]
    #[serial_test::serial]
    fn sends_full_snapshot_on_first_publish() {
        reset_last_sent_version();

        let state = EngineState::new(TimingConfig::default());
        let event = build_state_update_event(&state, Some("init".into()), Some(0));

        assert!(
            event.full_snapshot.is_some(),
            "first publish should include snapshot"
        );
        assert_eq!(event.delta.from_version, 0);
        assert_eq!(event.delta.to_version, 0);
    }

    #[test]
    #[serial_test::serial]
    fn sends_delta_after_initial_full_snapshot() {
        reset_last_sent_version();

        let mut state = EngineState::new(TimingConfig::default());
        let initial = build_state_update_event(&state, None, None); // bootstrap
        assert!(
            initial.full_snapshot.is_some(),
            "bootstrap should include snapshot"
        );

        apply_keydown(&mut state).expect("mutation applies");
        let event = build_state_update_event(&state, Some("key_down".into()), Some(500));
        assert!(
            event.full_snapshot.is_none(),
            "delta publish should not include snapshot"
        );
        assert_eq!(event.delta.from_version, 0);
        assert_eq!(event.delta.to_version, 1);
        assert_eq!(event.event.as_deref(), Some("key_down"));
        assert_eq!(event.latency_us, Some(500));
    }
}
