//! The remapping/routing engine: every `device_start` block of a config, and
//! the runtime state of each input device it has seen.
//!
//! A config is a list of blocks (`device_start(pattern) ... device_end()`).
//! Each input device is routed to the FIRST block whose pattern matches one
//! of its identities (name, serial, path, id - see
//! [`crate::runtime::device_pattern`]) and gets its own [`DeviceState`]. A
//! device no block matches is passed through. Routing is decided once per
//! device and cached, so the hot path is a hash lookup plus the block's O(1)
//! key lookup.
//!
//! This is the SAME engine the daemon's live event loop and the
//! [`crate::simulate`] driver both use - see [`Remapper::process`] and
//! [`Remapper::tick`]. There is exactly one implementation of "what does
//! this config do with this event", so the CLI/REST/RPC "simulate" tooling
//! can never drift from what the daemon actually does.
//!
//! # Cross-device state sharing
//!
//! Every device a `Remapper` routes shares ONE [`SharedModifierState`] (see
//! its docs) for custom modifiers/locks: holding a layer key on one device
//! makes `when("MD_XX")` mappings apply on every other routed device. Only
//! tap-hold pending state and press→release output tracking are per-device.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use hashbrown::HashMap;

use crate::config::{BaseKeyMapping, DeviceConfig, ImeState};
use crate::runtime::device_pattern;
use crate::runtime::event::{check_tap_hold_timeouts, process_event_for_identities};
use crate::runtime::held_outputs::HeldOutputs;
use crate::runtime::state::{SharedModifierState, SharedState};
use crate::runtime::{DeviceState, KeyEvent, KeyLookup};

/// One `device_start` block.
struct Block {
    pattern: String,
    lookup: KeyLookup,
    /// Modifiers the block uses as `when` conditions (its layers), in
    /// declaration order - the order the lookup tries them in.
    layers: Vec<u8>,
}

/// A device seen by the engine.
struct DeviceSlot {
    /// Index into `blocks`; `None` = no block matches (pass-through).
    block: Option<usize>,
    identities: Vec<String>,
    state: DeviceState,
}

/// The lookup tables of a config, the state of every device it has routed,
/// and the modifier/lock state they share. See the module docs.
pub struct Remapper {
    blocks: Vec<Block>,
    /// Keyed by the event's device id (`""` for events without one).
    devices: HashMap<String, DeviceSlot>,
    /// Modifier/lock bits shared by every device in `devices`.
    shared: SharedState,
    /// Output keys held, across all devices (one output device).
    held: HeldOutputs,
}

/// What the caller needs to process one event of a routed device.
pub struct Routed<'a> {
    pub lookup: &'a KeyLookup,
    pub state: &'a mut DeviceState,
    pub identities: &'a [String],
    /// The block's layer modifiers (see [`active_layer`]).
    pub layers: &'a [u8],
}

/// The result of running one event through a [`Remapper`].
pub struct Remapped {
    pub outputs: Vec<KeyEvent>,
    /// A human-readable mapping kind ("simple", "tap_hold", ...), from the
    /// mapping that matched BEFORE processing (permissive hold may look the
    /// key up again once it activates a modifier; this reflects the first
    /// lookup, matching what triggered the event).
    pub mapping_type: Option<&'static str>,
    /// Whether a mapping matched at all (false = pass-through).
    pub triggered: bool,
    /// The device's active layer after processing (see [`active_layer`]).
    pub active_layer: Option<u8>,
}

/// The layer a device is on: the first of the block's layer modifiers that is
/// active, i.e. the layer whose mappings the lookup applies first.
pub fn active_layer(state: &DeviceState, layers: &[u8]) -> Option<u8> {
    layers
        .iter()
        .copied()
        .find(|&id| state.is_modifier_active(id))
}

/// Modifiers a config uses as positive `when` conditions, first use first.
pub fn layer_modifiers(config: &DeviceConfig) -> Vec<u8> {
    use crate::config::{Condition, ConditionItem, KeyMapping};
    let mut layers = Vec::new();
    let mut add = |id: u8| {
        if !layers.contains(&id) {
            layers.push(id);
        }
    };
    for mapping in &config.mappings {
        if let KeyMapping::Conditional { condition, .. } = mapping {
            match condition {
                Condition::ModifierActive(id) => add(*id),
                Condition::AllActive(items) => items.iter().for_each(|item| {
                    if let ConditionItem::ModifierActive(id) = item {
                        add(*id);
                    }
                }),
                _ => {}
            }
        }
    }
    layers
}

/// A human-readable mapping kind, for telemetry and the simulation tooling.
pub fn mapping_type_name(mapping: &BaseKeyMapping) -> &'static str {
    match mapping {
        BaseKeyMapping::Simple { .. } => "simple",
        BaseKeyMapping::Modifier { .. } => "modifier",
        BaseKeyMapping::Lock { .. } => "lock",
        BaseKeyMapping::TapHold { .. } => "tap_hold",
        BaseKeyMapping::HoldOnly { .. } => "hold_only",
        BaseKeyMapping::ModifiedOutput { .. } => "modified_output",
        BaseKeyMapping::Sequence { .. } => "sequence",
    }
}

impl Remapper {
    /// A config with a single block (e.g. one `device_start("*")`).
    pub fn new(config: &DeviceConfig) -> Self {
        Self::from_blocks(core::slice::from_ref(config))
    }

    /// All blocks of a config, in declaration order (first match wins).
    pub fn from_blocks(configs: &[DeviceConfig]) -> Self {
        Self {
            blocks: configs
                .iter()
                .map(|config| Block {
                    pattern: config.identifier.pattern.clone(),
                    lookup: KeyLookup::from_device_config(config),
                    layers: layer_modifiers(config),
                })
                .collect(),
            devices: HashMap::new(),
            shared: SharedModifierState::new_handle(),
            held: HeldOutputs::new(),
        }
    }

    /// Routes an event from `device_id` to its block. `identities` resolves a
    /// device seen for the first time (name, serial, path...); it is not
    /// called again for that device. `None` means: pass the event through.
    pub fn route(
        &mut self,
        device_id: Option<&str>,
        identities: impl FnOnce(&str) -> Vec<String>,
    ) -> Option<Routed<'_>> {
        let key = device_id.unwrap_or_default();
        if !self.devices.contains_key(key) {
            let ids = if key.is_empty() {
                Vec::new()
            } else {
                identities(key)
            };
            let block = self.block_for(&ids);
            self.devices.insert(
                key.into(),
                DeviceSlot {
                    block,
                    identities: ids,
                    state: DeviceState::new_sharing(&self.shared),
                },
            );
        }
        let slot = self.devices.get_mut(key)?;
        let block = &self.blocks[slot.block?];
        Some(Routed {
            lookup: &block.lookup,
            state: &mut slot.state,
            identities: &slot.identities,
            layers: &block.layers,
        })
    }

    /// Routes and fully processes one event: looks up its mapping, runs it
    /// through the tap-hold/permissive-hold pipeline, and reports the
    /// mapping kind and the device's resulting active layer. An event from a
    /// device no block matches (or when there are no blocks at all) passes
    /// through unchanged. `ime`, when given, is applied to the device's
    /// state before the mapping lookup (conditions may test `IME`/`LANG_*`).
    ///
    /// The outputs are the transitions the OS must see: see [`HeldOutputs`].
    pub fn process(
        &mut self,
        event: KeyEvent,
        identities: impl FnOnce(&str) -> Vec<String>,
        ime: Option<ImeState>,
    ) -> Remapped {
        let mut remapped = self.process_raw(event, identities, ime);
        remapped.outputs = self.held.normalize(remapped.outputs);
        remapped
    }

    fn process_raw(
        &mut self,
        event: KeyEvent,
        identities: impl FnOnce(&str) -> Vec<String>,
        ime: Option<ImeState>,
    ) -> Remapped {
        let Some(routed) = self.route(event.device_id(), identities) else {
            return Remapped {
                outputs: alloc::vec![event],
                mapping_type: None,
                triggered: false,
                active_layer: None,
            };
        };
        if let Some(ime) = ime {
            routed.state.set_ime_state(ime);
        }
        let identities: Vec<&str> = routed.identities.iter().map(String::as_str).collect();
        let mapping =
            routed
                .lookup
                .find_mapping_for_identities(event.keycode(), routed.state, &identities);
        let mapping_type = mapping.map(mapping_type_name);
        let triggered = mapping.is_some();
        let outputs = process_event_for_identities(event, routed.lookup, routed.state, &identities);
        let active_layer = active_layer(routed.state, routed.layers);
        Remapped {
            outputs,
            mapping_type,
            triggered,
            active_layer,
        }
    }

    /// Checks tap-hold timeouts for every routed device at `now_us` and
    /// returns the events they generate (a key transitioning from pending to
    /// held). Call this periodically (daemon: every ~10ms when idle) or, for
    /// virtual time, before each simulated input (see [`crate::simulate`]).
    pub fn tick(&mut self, now_us: u64) -> Vec<KeyEvent> {
        let mut events = Vec::new();
        for state in self.states_mut() {
            events.extend(check_tap_hold_timeouts(now_us, state));
        }
        self.held.normalize(events)
    }

    /// First block whose pattern matches the device. `"*"` also matches a
    /// device with no known identity (e.g. an event without a device id).
    fn block_for(&self, identities: &[String]) -> Option<usize> {
        let ids: Vec<&str> = identities.iter().map(String::as_str).collect();
        self.blocks
            .iter()
            .position(|b| b.pattern == "*" || device_pattern::matches_any(&ids, &b.pattern))
    }

    /// Runtime state of `device_id`, if it has been routed to a block.
    pub fn state_of(&self, device_id: Option<&str>) -> Option<&DeviceState> {
        let slot = self.devices.get(device_id.unwrap_or_default())?;
        slot.block.map(|_| &slot.state)
    }

    /// The active layer of `device_id` (see [`active_layer`]), if routed.
    pub fn active_layer_of(&self, device_id: Option<&str>) -> Option<u8> {
        let slot = self.devices.get(device_id.unwrap_or_default())?;
        let layers = &self.blocks[slot.block?].layers;
        active_layer(&slot.state, layers)
    }

    /// Runtime states of every routed device (tap-hold timeouts, IME).
    pub fn states_mut(&mut self) -> impl Iterator<Item = &mut DeviceState> {
        self.devices
            .values_mut()
            .filter(|slot| slot.block.is_some())
            .map(|slot| &mut slot.state)
    }

    /// Custom modifier ids currently active - SHARED across every routed
    /// device (see the module docs on cross-device state sharing).
    pub fn active_modifiers(&self) -> Vec<u8> {
        let shared = self.shared.lock();
        (0..=SharedModifierState::max_id())
            .filter(|&id| shared.is_modifier_active(id))
            .collect()
    }

    /// Custom lock ids currently active - SHARED across every routed device.
    pub fn active_locks(&self) -> Vec<u8> {
        let shared = self.shared.lock();
        (0..=SharedModifierState::max_id())
            .filter(|&id| shared.is_lock_active(id))
            .collect()
    }

    /// Number of `device_start` blocks.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BaseKeyMapping, Condition, DeviceIdentifier, KeyCode, KeyMapping};
    use alloc::string::ToString;

    fn block(pattern: &str, from: KeyCode, to: KeyCode) -> DeviceConfig {
        DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: pattern.to_string(),
            },
            mappings: alloc::vec![KeyMapping::simple(from, to)],
        }
    }

    fn names(name: &'static str) -> impl FnOnce(&str) -> Vec<String> {
        move |id| alloc::vec![id.to_string(), name.to_string()]
    }

    fn output(state: &mut Remapper, id: &str, name: &'static str) -> Option<KeyCode> {
        let routed = state.route(Some(id), names(name))?;
        match routed.lookup.find_mapping(KeyCode::A, routed.state)? {
            BaseKeyMapping::Simple { to, .. } => Some(*to),
            _ => None,
        }
    }

    #[test]
    fn devices_use_the_first_matching_block() {
        let mut state = Remapper::from_blocks(&[
            block("*numpad*", KeyCode::A, KeyCode::B),
            block("*", KeyCode::A, KeyCode::C),
        ]);
        assert_eq!(output(&mut state, "path-7", "USB NumPad"), Some(KeyCode::B));
        assert_eq!(
            output(&mut state, "path-3", "USB Keyboard"),
            Some(KeyCode::C)
        );
    }

    #[test]
    fn unmatched_devices_pass_through() {
        let mut state = Remapper::from_blocks(&[block("*numpad*", KeyCode::A, KeyCode::B)]);
        assert!(state.route(Some("path-3"), names("USB Keyboard")).is_none());
        assert!(state.state_of(Some("path-3")).is_none());
        assert_eq!(state.states_mut().count(), 0);
    }

    /// Custom modifiers/locks are ONE state shared by every routed device
    /// (DSL manual §"Cross-Device State Sharing"); only tap-hold pending
    /// state and press/release output tracking stay per-device.
    #[test]
    fn modifier_state_is_shared_across_devices_but_pressed_keys_are_not() {
        let mut state = Remapper::new(&block("*", KeyCode::A, KeyCode::B));
        state
            .route(Some("kbd-1"), names("One"))
            .unwrap()
            .state
            .set_modifier(0);
        let other = state.route(Some("kbd-2"), names("Two")).unwrap();
        assert!(
            other.state.is_modifier_active(0),
            "kbd-2 must see kbd-1's modifier - modifier/lock state is shared"
        );

        // Press/release tracking, in contrast, stays private to each device:
        // releasing A on kbd-2 (which never pressed it) must not disturb
        // kbd-1's tracked press.
        state
            .route(Some("kbd-1"), names("One"))
            .unwrap()
            .state
            .record_press(KeyCode::A, &[KeyCode::LShift, KeyCode::B]);
        state
            .route(Some("kbd-2"), names("Two"))
            .unwrap()
            .state
            .clear_press(KeyCode::A);
        let kbd1 = state.state_of(Some("kbd-1")).unwrap();
        assert_eq!(
            kbd1.get_release_key(KeyCode::A).as_slice(),
            &[KeyCode::LShift, KeyCode::B],
            "kbd-1's press tracking must be untouched by a release on kbd-2"
        );
    }

    /// A foot pedal (device A) used as a layer key for a separate keyboard
    /// (device B): holding the pedal's F24 sets MD_00, which a `when`
    /// mapping on B's block reacts to - the scenario the DSL manual's
    /// cross-device section documents.
    #[test]
    fn a_layer_key_on_one_device_affects_mappings_on_another() {
        let pedal = DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*pedal*".to_string(),
            },
            mappings: alloc::vec![KeyMapping::modifier(KeyCode::F24, 0)],
        };
        let keyboard = DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".to_string(),
            },
            mappings: alloc::vec![KeyMapping::conditional(
                Condition::ModifierActive(0),
                alloc::vec![BaseKeyMapping::Simple {
                    from: KeyCode::A,
                    to: KeyCode::B,
                }],
            )],
        };
        let mut remap = Remapper::from_blocks(&[pedal, keyboard]);

        // Before the pedal is held, A on the keyboard passes through
        // unmapped (no matching mapping for A on the base layer).
        let before = remap.process(
            KeyEvent::press(KeyCode::A),
            |id| alloc::vec![id.to_string()],
            None,
        );
        assert!(!before.triggered);

        // Hold the pedal: sets the shared MD_00.
        let hold = remap.process(
            KeyEvent::press(KeyCode::F24).with_device_id("pedal-1".to_string()),
            |id| alloc::vec![id.to_string()],
            None,
        );
        assert!(hold.triggered);

        // Now A on the keyboard (a different device) is remapped to B.
        let after = remap.process(
            KeyEvent::press(KeyCode::A).with_device_id("keyboard-1".to_string()),
            |id| alloc::vec![id.to_string()],
            None,
        );
        assert!(after.triggered);
        assert_eq!(after.outputs.len(), 1);
        assert_eq!(after.outputs[0].keycode(), KeyCode::B);
    }

    #[test]
    fn identities_are_resolved_once_per_device() {
        let mut state = Remapper::new(&block("*", KeyCode::A, KeyCode::B));
        let mut calls = 0;
        for _ in 0..3 {
            state.route(Some("kbd"), |id| {
                calls += 1;
                alloc::vec![id.to_string()]
            });
        }
        assert_eq!(calls, 1);
    }

    #[test]
    fn events_without_device_use_a_wildcard_block() {
        let mut wildcard = Remapper::new(&block("*", KeyCode::A, KeyCode::B));
        assert!(wildcard.route(None, |_| unreachable!()).is_some());
        let mut specific = Remapper::new(&block("*numpad*", KeyCode::A, KeyCode::B));
        assert!(specific.route(None, |_| unreachable!()).is_none());
    }

    #[test]
    fn active_layer_is_the_first_active_layer_modifier() {
        let config = DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".to_string(),
            },
            mappings: alloc::vec![
                KeyMapping::Conditional {
                    condition: Condition::ModifierActive(0x0A),
                    mappings: alloc::vec![BaseKeyMapping::Simple {
                        from: KeyCode::H,
                        to: KeyCode::Left,
                    }],
                },
                KeyMapping::Conditional {
                    condition: Condition::ModifierActive(0x02),
                    mappings: alloc::vec![BaseKeyMapping::Simple {
                        from: KeyCode::J,
                        to: KeyCode::Down,
                    }],
                },
            ],
        };
        let mut remap = Remapper::new(&config);
        let routed = remap
            .route(Some("kbd"), |id| alloc::vec![id.to_string()])
            .unwrap();
        assert_eq!(routed.layers, &[0x0A, 0x02]);
        assert_eq!(active_layer(routed.state, routed.layers), None);
        routed.state.set_modifier(0x02);
        assert_eq!(active_layer(routed.state, routed.layers), Some(0x02));
        routed.state.set_modifier(0x0A);
        assert_eq!(active_layer(routed.state, routed.layers), Some(0x0A));
    }

    #[test]
    fn tick_fires_tap_hold_timeouts_for_every_routed_device() {
        let config = DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".to_string(),
            },
            mappings: alloc::vec![KeyMapping::tap_hold(
                KeyCode::CapsLock,
                KeyCode::Escape,
                0,
                200
            )],
        };
        let mut remap = Remapper::new(&config);
        let remapped = remap.process(
            KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
            |id| alloc::vec![id.to_string()],
            None,
        );
        assert!(
            remapped.outputs.is_empty(),
            "press alone must not resolve tap/hold yet"
        );

        // Past the 200ms threshold: ticking must fire the hold (activate MD_00).
        let events = remap.tick(250_000);
        assert!(
            events.is_empty(),
            "hold-only activation has no KeyEvent output"
        );
        assert!(remap.active_modifiers().contains(&0));
    }

    #[test]
    fn process_reports_pass_through_for_unmatched_devices() {
        let mut remap = Remapper::from_blocks(&[block("*numpad*", KeyCode::A, KeyCode::B)]);
        let remapped = remap.process(
            KeyEvent::press(KeyCode::A).with_device_id("kbd".to_string()),
            |id| alloc::vec![id.to_string()],
            None,
        );
        assert!(!remapped.triggered);
        assert_eq!(remapped.outputs.len(), 1);
        assert_eq!(remapped.outputs[0].keycode(), KeyCode::A);
    }
}
