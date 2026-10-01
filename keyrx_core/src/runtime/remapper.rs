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
use crate::runtime::held_outputs::HeldOutputs;
use crate::runtime::permissive_hold::{self, DeviceCtx};
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
    /// Events held back while a tap-hold key is undecided (see
    /// [`crate::runtime::permissive_hold`]).
    buffer: Vec<KeyEvent>,
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
    /// See [`crate::runtime::permissive_hold`].
    pub buffer: &'a mut Vec<KeyEvent>,
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
        BaseKeyMapping::TapHoldKey { .. } => "tap_hold",
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
                    buffer: Vec::new(),
                },
            );
        }
        let slot = self.devices.get_mut(key)?;
        let block = &self.blocks[slot.block?];
        Some(Routed {
            lookup: &block.lookup,
            state: &mut slot.state,
            buffer: &mut slot.buffer,
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
        let now = event.timestamp_us();
        let mut ctx = DeviceCtx {
            lookup: routed.lookup,
            state: routed.state,
            identities: &identities,
            buffer: routed.buffer,
        };
        // Decide tap-holds whose threshold passed before this event, so the
        // live loop and the simulator agree however often the idle tick runs.
        let mut outputs = permissive_hold::tick(&mut ctx, now);
        outputs.extend(permissive_hold::feed(&mut ctx, event));
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
        for slot in self.devices.values_mut() {
            let Some(block) = slot.block.and_then(|b| self.blocks.get(b)) else {
                continue;
            };
            let identities: Vec<&str> = slot.identities.iter().map(String::as_str).collect();
            let mut ctx = DeviceCtx {
                lookup: &block.lookup,
                state: &mut slot.state,
                identities: &identities,
                buffer: &mut slot.buffer,
            };
            events.extend(permissive_hold::tick(&mut ctx, now_us));
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
#[path = "remapper_tests.rs"]
mod tests;
