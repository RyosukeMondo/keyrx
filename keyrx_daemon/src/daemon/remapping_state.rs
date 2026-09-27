//! Remapping state for the event loop: every `device_start` block of the live
//! config, and the runtime state of each input device.
//!
//! A config is a list of blocks (`device_start(pattern) ... device_end()`).
//! Each input device is routed to the FIRST block whose pattern matches one of
//! its identities (name, serial, path, id - see
//! `keyrx_core::runtime::device_pattern`) and gets its own `DeviceState`
//! (modifiers, locks, tap-hold). A device no block matches is passed through.
//! Routing is decided once per device and cached, so the hot path is a hash
//! lookup plus the block's O(1) key lookup.

use std::collections::HashMap;

use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::device_pattern;
use keyrx_core::runtime::{DeviceState, KeyLookup};

/// One `device_start` block.
struct Block {
    pattern: String,
    lookup: KeyLookup,
}

/// A device seen by the event loop.
struct DeviceSlot {
    /// Index into `blocks`; `None` = no block matches (pass-through).
    block: Option<usize>,
    identities: Vec<String>,
    state: DeviceState,
}

/// The lookup tables of the live config and the state of each device.
pub struct RemappingState {
    blocks: Vec<Block>,
    /// Keyed by the event's device id (`""` for events without one).
    devices: HashMap<String, DeviceSlot>,
}

/// What the event loop needs to process one event of a routed device.
pub struct Routed<'a> {
    pub lookup: &'a KeyLookup,
    pub state: &'a mut DeviceState,
    pub identities: &'a [String],
}

impl RemappingState {
    /// A config with a single block (e.g. one `device_start("*")`).
    pub fn new(config: &DeviceConfig) -> Self {
        Self::from_blocks(std::slice::from_ref(config))
    }

    /// All blocks of a config, in declaration order (first match wins).
    pub fn from_blocks(configs: &[DeviceConfig]) -> Self {
        Self {
            blocks: configs
                .iter()
                .map(|config| Block {
                    pattern: config.identifier.pattern.clone(),
                    lookup: KeyLookup::from_device_config(config),
                })
                .collect(),
            devices: HashMap::new(),
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
                key.to_string(),
                DeviceSlot {
                    block,
                    identities: ids,
                    state: DeviceState::new(),
                },
            );
        }
        let slot = self.devices.get_mut(key)?;
        let block = &self.blocks[slot.block?];
        Some(Routed {
            lookup: &block.lookup,
            state: &mut slot.state,
            identities: &slot.identities,
        })
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

    /// Runtime states of every routed device (tap-hold timeouts, IME).
    pub fn states_mut(&mut self) -> impl Iterator<Item = &mut DeviceState> {
        self.devices
            .values_mut()
            .filter(|slot| slot.block.is_some())
            .map(|slot| &mut slot.state)
    }

    /// Number of `device_start` blocks.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_core::config::{DeviceIdentifier, KeyCode, KeyMapping};

    fn block(pattern: &str, from: KeyCode, to: KeyCode) -> DeviceConfig {
        DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: pattern.to_string(),
            },
            mappings: vec![KeyMapping::simple(from, to)],
        }
    }

    fn names(name: &'static str) -> impl FnOnce(&str) -> Vec<String> {
        move |id| vec![id.to_string(), name.to_string()]
    }

    fn output(state: &mut RemappingState, id: &str, name: &'static str) -> Option<KeyCode> {
        let routed = state.route(Some(id), names(name))?;
        match routed.lookup.find_mapping(KeyCode::A, routed.state)? {
            keyrx_core::config::BaseKeyMapping::Simple { to, .. } => Some(*to),
            _ => None,
        }
    }

    #[test]
    fn devices_use_the_first_matching_block() {
        let mut state = RemappingState::from_blocks(&[
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
        let mut state = RemappingState::from_blocks(&[block("*numpad*", KeyCode::A, KeyCode::B)]);
        assert!(state.route(Some("path-3"), names("USB Keyboard")).is_none());
        assert!(state.state_of(Some("path-3")).is_none());
        assert_eq!(state.states_mut().count(), 0);
    }

    #[test]
    fn each_device_has_its_own_state() {
        let mut state = RemappingState::new(&block("*", KeyCode::A, KeyCode::B));
        state
            .route(Some("kbd-1"), names("One"))
            .unwrap()
            .state
            .set_modifier(0);
        let other = state.route(Some("kbd-2"), names("Two")).unwrap();
        assert!(!other.state.is_modifier_active(0));
        assert!(state.state_of(Some("kbd-1")).unwrap().is_modifier_active(0));
    }

    #[test]
    fn identities_are_resolved_once_per_device() {
        let mut state = RemappingState::new(&block("*", KeyCode::A, KeyCode::B));
        let mut calls = 0;
        for _ in 0..3 {
            state.route(Some("kbd"), |id| {
                calls += 1;
                vec![id.to_string()]
            });
        }
        assert_eq!(calls, 1);
    }

    #[test]
    fn events_without_device_use_a_wildcard_block() {
        let mut wildcard = RemappingState::new(&block("*", KeyCode::A, KeyCode::B));
        assert!(wildcard.route(None, |_| unreachable!()).is_some());
        let mut specific = RemappingState::new(&block("*numpad*", KeyCode::A, KeyCode::B));
        assert!(specific.route(None, |_| unreachable!()).is_none());
    }
}
