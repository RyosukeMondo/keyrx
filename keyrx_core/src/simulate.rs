//! Deterministic event-sequence simulation, driven with virtual time.
//!
//! This is not a second remapping implementation: it drives the SAME
//! [`crate::runtime::remapper::Remapper`] the daemon's live event loop uses
//! (`process` for input, `tick` for tap-hold timeouts), just fed a list of
//! timestamped inputs instead of live platform events. That is what makes it
//! safe to use for the daemon's `simulate`/`test` CLI, the REST/RPC
//! simulator endpoints, and the WASM simulator: a discrepancy between "what
//! the daemon would do" and "what simulate says" is a bug in the engine, not
//! two implementations that drifted apart.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::config::KeyCode;
use crate::runtime::remapper::Remapper;
use crate::runtime::KeyEvent;

/// One input event to feed the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimInput {
    /// Virtual timestamp, microseconds from the start of the simulation.
    pub at_us: u64,
    /// `None` routes through the wildcard/no-identity block, matching an
    /// event with no known device.
    pub device: Option<String>,
    pub press: bool,
    pub key: KeyCode,
}

/// One output event the engine produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimOutput {
    pub key: KeyCode,
    pub press: bool,
    pub at_us: u64,
}

/// One step of the simulation timeline.
#[derive(Debug, Clone)]
pub struct SimStep {
    /// `None` for a step produced by a tap-hold timeout firing with no new
    /// input (e.g. a hold threshold elapsing while nothing else happens).
    pub input: Option<SimInput>,
    pub outputs: Vec<SimOutput>,
    pub mapping_type: Option<&'static str>,
    pub triggered: bool,
    /// The active layer of the step's device (the input's device for an
    /// input step; the most recently seen device for a timeout step - see
    /// [`run`]).
    pub active_layer: Option<u8>,
    /// Modifier/lock ids active after this step - SHARED across every
    /// routed device (see `runtime::remapper` docs on cross-device state
    /// sharing), so this is the same regardless of which device is "current".
    pub active_modifiers: Vec<u8>,
    pub active_locks: Vec<u8>,
}

/// Runs `inputs` (any order; sorted here by `at_us`) through `remapper` with
/// virtual time: before each input at time `t`, ticks the engine to `t` so
/// any pending tap-hold timeout fires as its own step, then processes the
/// input. Ticks once more at `end_us` for trailing timeouts (e.g. a key
/// still held when the recorded sequence ends). `identities` resolves a
/// device id to the identities `when_device` patterns match against - for a
/// simulated device this is typically just the id itself
/// (`|id| alloc::vec![id.into()]`).
pub fn run(
    remapper: &mut Remapper,
    inputs: &[SimInput],
    end_us: u64,
    identities: impl Fn(&str) -> Vec<String>,
) -> Vec<SimStep> {
    let mut sorted: Vec<&SimInput> = inputs.iter().collect();
    sorted.sort_by_key(|e| e.at_us);

    let had_inputs = !sorted.is_empty();
    let mut steps = Vec::new();
    let mut last_device: Option<String> = None;
    for input in sorted {
        tick_step(remapper, input.at_us, &last_device, &mut steps, false);

        let event = if input.press {
            KeyEvent::press(input.key)
        } else {
            KeyEvent::release(input.key)
        }
        .with_timestamp(input.at_us);
        let event = match &input.device {
            Some(id) => event.with_device_id(id.clone()),
            None => event,
        };
        last_device = input.device.clone();

        let remapped = remapper.process(event, &identities, None);
        steps.push(SimStep {
            input: Some(input.clone()),
            outputs: to_outputs(&remapped.outputs),
            mapping_type: remapped.mapping_type,
            triggered: remapped.triggered,
            active_layer: remapped.active_layer,
            active_modifiers: remapper.active_modifiers(),
            active_locks: remapper.active_locks(),
        });
    }
    // Always record the final tick, even if it produced no KeyEvent output
    // (e.g. a hold-only modifier activating): it is the only place the
    // caller can observe state that changed right at the end of the run.
    tick_step(remapper, end_us, &last_device, &mut steps, had_inputs);
    steps
}

/// Ticks `remapper` to `at_us`; if any tap-hold timeout fired, records it as
/// its own timeline step (attributed to `device`, the most recently
/// processed input's device - timeout events themselves are not tagged with
/// a device). `force` records a step even with no output events (used for
/// the trailing end-of-run tick, so its state snapshot is never lost).
fn tick_step(
    remapper: &mut Remapper,
    at_us: u64,
    device: &Option<String>,
    steps: &mut Vec<SimStep>,
    force: bool,
) {
    let events = remapper.tick(at_us);
    if events.is_empty() && !force {
        return;
    }
    steps.push(SimStep {
        input: None,
        outputs: to_outputs(&events),
        mapping_type: None,
        triggered: true,
        active_layer: device
            .as_deref()
            .and_then(|id| remapper.active_layer_of(Some(id))),
        active_modifiers: remapper.active_modifiers(),
        active_locks: remapper.active_locks(),
    });
}

fn to_outputs(events: &[KeyEvent]) -> Vec<SimOutput> {
    events
        .iter()
        .map(|e| SimOutput {
            key: e.keycode(),
            press: e.is_press(),
            at_us: e.timestamp_us(),
        })
        .collect()
}

/// An invariant that holds for ANY config, so it can be checked without
/// knowing what the config is supposed to do: no key is left "stuck" -
/// every output key pressed during the run was released by the end of it.
/// Returns the keys that are still held, empty if none (i.e. the invariant
/// holds).
pub fn stuck_keys(steps: &[SimStep]) -> Vec<KeyCode> {
    let mut held: Vec<KeyCode> = Vec::new();
    for step in steps {
        for output in &step.outputs {
            if output.press {
                if !held.contains(&output.key) {
                    held.push(output.key);
                }
            } else {
                held.retain(|&k| k != output.key);
            }
        }
    }
    held
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DeviceConfig, DeviceIdentifier, KeyMapping};
    use alloc::string::ToString;
    use alloc::vec;

    fn wildcard_identities(id: &str) -> Vec<String> {
        vec![id.to_string()]
    }

    fn tap_hold_config() -> DeviceConfig {
        DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".to_string(),
            },
            mappings: vec![KeyMapping::tap_hold(
                KeyCode::CapsLock,
                KeyCode::Escape,
                0,
                200,
            )],
        }
    }

    #[test]
    fn a_quick_tap_produces_the_tap_output() {
        let mut remapper = Remapper::new(&tap_hold_config());
        let inputs = [
            SimInput {
                at_us: 0,
                device: None,
                press: true,
                key: KeyCode::CapsLock,
            },
            SimInput {
                at_us: 50_000,
                device: None,
                press: false,
                key: KeyCode::CapsLock,
            },
        ];
        let steps = run(&mut remapper, &inputs, 60_000, wildcard_identities);
        let outputs: Vec<SimOutput> = steps.iter().flat_map(|s| s.outputs.clone()).collect();
        assert_eq!(
            outputs,
            vec![
                SimOutput {
                    key: KeyCode::Escape,
                    press: true,
                    at_us: 50_000
                },
                SimOutput {
                    key: KeyCode::Escape,
                    press: false,
                    at_us: 50_000
                },
            ]
        );
        assert!(stuck_keys(&steps).is_empty());
    }

    #[test]
    fn a_hold_past_the_threshold_fires_on_tick_not_on_release() {
        let mut remapper = Remapper::new(&tap_hold_config());
        let inputs = [SimInput {
            at_us: 0,
            device: None,
            press: true,
            key: KeyCode::CapsLock,
        }];
        // No release event at all: the 300us end-of-run tick must still
        // observe the hold past its 200ms threshold and activate MD_00.
        let steps = run(&mut remapper, &inputs, 300_000, wildcard_identities);
        assert!(steps.last().unwrap().active_modifiers.contains(&0));
    }

    #[test]
    fn no_events_produces_no_steps() {
        let mut remapper = Remapper::new(&tap_hold_config());
        let steps = run(&mut remapper, &[], 0, wildcard_identities);
        assert!(steps.is_empty());
    }
}
