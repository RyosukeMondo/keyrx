//! Engine state streaming exports for FFI.
//!
//! Provides a legacy-compatible `keyrx_on_state` callback registration
//! plus a delta-first state publisher with automatic full snapshot
//! fallback when versions drift.
#![allow(unsafe_code)]

use crate::engine::layout::{LayoutCompositor, ModifierCoordinator};
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
    snapshot_override: Option<StateSnapshot>,
    event: Option<String>,
    latency_us: Option<u64>,
) -> StateUpdateEvent {
    let delta = state.take_delta();
    let snapshot: StateSnapshot = snapshot_override.unwrap_or_else(|| state.into());

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
    let payload = build_state_update_event(state, None, event, latency_us);
    global_event_registry().invoke(EventType::EngineState, &payload);
}

/// Publish an EngineState update with layout compositor context.
///
/// This variant enriches the snapshot with multi-layout metadata when provided,
/// enabling Flutter/FFI consumers to visualize active layouts and shared modifiers.
pub fn publish_state_update_with_layouts(
    state: &EngineState,
    layouts: Option<&LayoutCompositor>,
    coordinator: Option<&ModifierCoordinator>,
    event: Option<String>,
    latency_us: Option<u64>,
) {
    let snapshot = layouts.map(|layouts| StateSnapshot::with_layouts(state, layouts, coordinator));

    let payload = build_state_update_event(state, snapshot, event, latency_us);
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
        let event = build_state_update_event(&state, None, Some("init".into()), Some(0));

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
        let initial = build_state_update_event(&state, None, None, None); // bootstrap
        assert!(
            initial.full_snapshot.is_some(),
            "bootstrap should include snapshot"
        );

        apply_keydown(&mut state).expect("mutation applies");
        let event = build_state_update_event(&state, None, Some("key_down".into()), Some(500));
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

// ─── Engine Loop Lifecycle ────────────────────────────────────────────────

#[cfg(all(target_os = "windows", feature = "windows-driver"))]
use crate::drivers::WindowsInput;
#[cfg(all(target_os = "windows", feature = "windows-driver"))]
use crate::engine::{AdvancedEngine, TimingConfig};
#[cfg(all(target_os = "windows", feature = "windows-driver"))]
use crate::ffi::engine_instance::{clear_global_engine, set_global_engine};
#[cfg(all(target_os = "windows", feature = "windows-driver"))]
use crate::ffi::runtime::with_revolutionary_runtime;
#[cfg(all(target_os = "windows", feature = "windows-driver"))]
use crate::InputSource;
#[cfg(all(target_os = "windows", feature = "windows-driver"))]
use std::sync::{Arc, Mutex};
#[cfg(all(target_os = "windows", feature = "windows-driver"))]
use std::thread;

#[cfg(all(target_os = "windows", feature = "windows-driver"))]
static STOP_SIGNAL: OnceLock<Mutex<Option<tokio::sync::mpsc::Sender<()>>>> = OnceLock::new();

#[cfg(all(target_os = "windows", feature = "windows-driver"))]
fn get_stop_sender() -> &'static Mutex<Option<tokio::sync::mpsc::Sender<()>>> {
    STOP_SIGNAL.get_or_init(|| Mutex::new(None))
}

#[cfg(all(target_os = "windows", feature = "windows-driver"))]
/// # Safety
///
/// This function is unsafe because it interacts with global mutable state and is an FFI entry point.
#[no_mangle]
pub unsafe extern "C" fn keyrx_engine_start_loop() -> i32 {
    #[allow(clippy::unwrap_used)]
    let mut guard = get_stop_sender().lock().unwrap();
    if guard.is_some() {
        return -1; // Already running
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    *guard = Some(tx);

    thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                tracing::error!("Failed to create runtime: {}", e);
                return;
            }
        };

        rt.block_on(async move {
            if let Err(e) = run_engine_loop(&mut rx).await {
                tracing::error!("Engine loop failed: {}", e);
            }
        });

        // Cleanup
        #[allow(clippy::unwrap_used)]
        let mut guard = get_stop_sender().lock().unwrap();
        *guard = None;
    });

    0
}

#[cfg(all(target_os = "windows", feature = "windows-driver"))]
/// # Safety
///
/// This function is unsafe because it interacts with global mutable state and is an FFI entry point.
#[no_mangle]
pub unsafe extern "C" fn keyrx_engine_stop_loop() -> i32 {
    #[allow(clippy::unwrap_used)]
    let guard = get_stop_sender().lock().unwrap();
    if let Some(tx) = guard.as_ref() {
        let tx = tx.clone();
        // Spawn a thread to send the signal to avoid blocking FFI if channel is full (unlikely)
        thread::spawn(move || {
            let _ = tx.blocking_send(());
        });
        0
    } else {
        -1 // Not running
    }
}

#[cfg(all(target_os = "windows", feature = "windows-driver"))]
async fn run_engine_loop(stop_rx: &mut tokio::sync::mpsc::Receiver<()>) -> anyhow::Result<()> {
    tracing::info!("Starting engine loop from FFI");

    // 1. Get Runtime
    let runtime_arc = with_revolutionary_runtime(|rt| Ok(rt.rhai_runtime().clone()))
        .map_err(|e| anyhow::anyhow!("Runtime not initialized: {}", e))?;

    // 2. Setup Input
    let mut input = WindowsInput::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize WindowsInput: {}", e))?;
    input.start().await?;

    // 3. Build Engine
    // Note: We're not fully initializing the engine with device registry/profiles here yet
    // because that requires more intricate setup that is done in `RunCommand`.
    // For now, this is sufficient to capture keys and run the script.
    let engine = AdvancedEngine::new(runtime_arc, TimingConfig::default());

    // We can try to attach device registry if available in RevolutionaryRuntime
    let device_registry = with_revolutionary_runtime(|rt| Ok(rt.device_registry().clone())).ok();
    let engine = if let Some(reg) = device_registry {
        engine.with_device_registry(reg)
    } else {
        engine
    };

    // Wrap in Arc<Mutex> and share globally
    let engine = Arc::new(Mutex::new(engine));
    set_global_engine(engine.clone());

    tracing::info!("Engine loop running");

    // 4. Loop
    let mut last_timestamp = 0u64;

    loop {
        tokio::select! {
            _ = stop_rx.recv() => {
                tracing::info!("Stop signal received");
                break;
            }
            res = input.poll_events() => {
                let events = match res {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!("Input polling error: {}", e);
                         break;
                    }
                };

                for event in events {
                    // Update engine and collect actions
                    let mut actions = Vec::new();
                    {
                        #[allow(clippy::unwrap_used)]
                        let mut engine_guard = engine.lock().unwrap();

                        // Update engine time
                        if event.timestamp_us > last_timestamp {
                            actions.extend(engine_guard.tick(event.timestamp_us));
                            last_timestamp = event.timestamp_us;
                        }

                        // Process
                        actions.extend(engine_guard.process_event(event.clone()));
                    }

                    // Emit RawInput for Monitor
                    use crate::ffi::domains::engine::global_event_registry;
                    use crate::ffi::events::EventType;
                    global_event_registry().invoke(EventType::RawInput, &event);

                    // Pass to discovery session (if active)
                    crate::ffi::domains::discovery::process_discovery_event(&event);

                    for action in actions {
                        // Emit RawOutput for Monitor
                        global_event_registry().invoke(EventType::RawOutput, &action);
                        if let Err(e) = input.send_output(action).await {
                            tracing::error!("Failed to send output: {}", e);
                        }
                    }
                }
            }
        }
    }

    clear_global_engine();
    input.stop().await?;
    tracing::info!("Engine loop stopped");
    Ok(())
}
