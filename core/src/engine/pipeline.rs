//! Priority-aware multi-device mapping pipeline.
//!
//! Implements the 3-stage pipeline described in the design doc:
//! 1) Hardware Wiring: scancode -> virtual key
//! 2) Virtual Layout: virtual key context (reserved for future layout metadata)
//! 3) Logical Mapping: virtual key -> action binding
//!
//! The pipeline iterates active profile slots for the originating device in
//! priority order. The first slot that produces an action consumes the event.

use crate::config::models::{
    ActionBinding, DeviceInstanceId, HardwareProfile, HardwareProfileId, Keymap, KeymapId,
    RuntimeConfig, VirtualKeyId,
};
use crate::engine::{InputEvent, OutputAction};
use std::collections::HashMap;
use std::str::FromStr;

/// Result of processing a single input event through the mapping pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineResult {
    /// True when an active slot produced an action and consumed the event.
    pub consumed: bool,
    /// Virtual key resolved in stage 1.
    pub virtual_key: Option<VirtualKeyId>,
    /// Action binding resolved in stage 3.
    pub action: Option<ActionBinding>,
    /// Output action translated for simple standard-key bindings.
    pub output: Option<OutputAction>,
    /// Slot that handled the event.
    pub slot_id: Option<String>,
    /// Hardware profile used for the translation.
    pub hardware_profile_id: Option<HardwareProfileId>,
    /// Keymap that produced the action binding.
    pub keymap_id: Option<KeymapId>,
    /// Reason why no slot consumed the event (useful for diagnostics).
    pub skip_reason: Option<PipelineSkipReason>,
}

impl PipelineResult {
    fn consumed(
        slot_id: String,
        hardware_profile_id: HardwareProfileId,
        keymap_id: KeymapId,
        virtual_key: VirtualKeyId,
        action: ActionBinding,
        output: Option<OutputAction>,
    ) -> Self {
        Self {
            consumed: true,
            virtual_key: Some(virtual_key),
            action: Some(action),
            output,
            slot_id: Some(slot_id),
            hardware_profile_id: Some(hardware_profile_id),
            keymap_id: Some(keymap_id),
            skip_reason: None,
        }
    }

    fn skipped(reason: PipelineSkipReason) -> Self {
        Self {
            consumed: false,
            virtual_key: None,
            action: None,
            output: None,
            slot_id: None,
            hardware_profile_id: None,
            keymap_id: None,
            skip_reason: Some(reason),
        }
    }
}

/// Reasons an event was not consumed by the pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineSkipReason {
    /// Event did not contain enough metadata to identify the device.
    MissingDeviceIdentity,
    /// The device is not present in the runtime configuration.
    DeviceNotConfigured,
    /// All slots were inactive or failed resolution.
    NoActiveSlotMatched,
}

/// Stateless pipeline that resolves scancodes to actions using runtime config.
#[derive(Debug, Clone, Default)]
pub struct MappingPipeline {
    runtime_config: RuntimeConfig,
    hardware_profiles: HashMap<HardwareProfileId, HardwareProfile>,
    keymaps: HashMap<KeymapId, Keymap>,
}

impl MappingPipeline {
    /// Create a new pipeline with runtime configuration and resource maps.
    pub fn new(
        runtime_config: RuntimeConfig,
        hardware_profiles: HashMap<HardwareProfileId, HardwareProfile>,
        keymaps: HashMap<KeymapId, Keymap>,
    ) -> Self {
        Self {
            runtime_config,
            hardware_profiles,
            keymaps,
        }
    }

    /// Update the runtime configuration (e.g., after UI changes).
    pub fn set_runtime_config(&mut self, runtime_config: RuntimeConfig) {
        self.runtime_config = runtime_config;
    }

    /// Replace the hardware profile set.
    pub fn set_hardware_profiles(
        &mut self,
        hardware_profiles: HashMap<HardwareProfileId, HardwareProfile>,
    ) {
        self.hardware_profiles = hardware_profiles;
    }

    /// Replace the keymap set.
    pub fn set_keymaps(&mut self, keymaps: HashMap<KeymapId, Keymap>) {
        self.keymaps = keymaps;
    }

    /// Process a single input event through the 3-stage pipeline.
    ///
    /// 1. Device resolution via VID/PID/Serial.
    /// 2. Iterate active slots in priority order (descending).
    /// 3. Translate scancode -> virtual key -> action.
    pub fn process_event(&self, event: &InputEvent) -> PipelineResult {
        let Some(device_id) = self.extract_device(event) else {
            return PipelineResult::skipped(PipelineSkipReason::MissingDeviceIdentity);
        };

        let Some(device_slots) = self
            .runtime_config
            .devices
            .iter()
            .find(|d| d.device == device_id)
        else {
            return PipelineResult::skipped(PipelineSkipReason::DeviceNotConfigured);
        };

        // Stable priority ordering: higher priority first, then declaration order.
        let mut active_slots: Vec<(usize, &crate::config::models::ProfileSlot)> = device_slots
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.active)
            .collect();
        active_slots.sort_by(|(a_idx, a), (b_idx, b)| {
            b.priority.cmp(&a.priority).then_with(|| a_idx.cmp(b_idx))
        });

        for (_, slot) in active_slots {
            let Some(profile) = self.hardware_profiles.get(&slot.hardware_profile_id) else {
                continue;
            };

            let Some(virtual_key) = profile.wiring.get(&event.scan_code) else {
                continue;
            };

            let Some(keymap) = self.keymaps.get(&slot.keymap_id) else {
                continue;
            };

            let Some(action) = resolve_action_binding(keymap, virtual_key) else {
                continue;
            };

            // Transparent bindings deliberately allow fallthrough to lower slots.
            if matches!(action, ActionBinding::Transparent) {
                continue;
            }

            let output = action_to_output(&action, event.pressed);

            return PipelineResult::consumed(
                slot.id.clone(),
                profile.id.clone(),
                keymap.id.clone(),
                virtual_key.clone(),
                action,
                output,
            );
        }

        PipelineResult::skipped(PipelineSkipReason::NoActiveSlotMatched)
    }

    fn extract_device(&self, event: &InputEvent) -> Option<DeviceInstanceId> {
        Some(DeviceInstanceId {
            vendor_id: event.vendor_id?,
            product_id: event.product_id?,
            serial: Some(event.serial_number.clone()?),
        })
    }
}

fn resolve_action_binding(keymap: &Keymap, virtual_key: &VirtualKeyId) -> Option<ActionBinding> {
    for layer in &keymap.layers {
        if let Some(binding) = layer.bindings.get(virtual_key) {
            return Some(binding.clone());
        }
    }
    None
}

fn action_to_output(action: &ActionBinding, pressed: bool) -> Option<OutputAction> {
    match action {
        ActionBinding::StandardKey(name) => crate::drivers::keycodes::KeyCode::from_str(name)
            .ok()
            .map(|key| {
                if pressed {
                    OutputAction::KeyDown(key)
                } else {
                    OutputAction::KeyUp(key)
                }
            }),
        ActionBinding::Transparent => Some(OutputAction::PassThrough),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::{DeviceSlots, KeymapLayer, ProfileSlot};
    use crate::engine::KeyCode;

    fn device_id() -> DeviceInstanceId {
        DeviceInstanceId {
            vendor_id: 0x1,
            product_id: 0x2,
            serial: Some("abc".into()),
        }
    }

    fn make_event(scan_code: u16, pressed: bool) -> InputEvent {
        InputEvent::with_full_identity(
            KeyCode::A,
            pressed,
            0,
            Some("/dev/input/mock".to_string()),
            false,
            false,
            scan_code,
            Some("abc".to_string()),
            Some(0x1),
            Some(0x2),
        )
    }

    fn wiring_profile(id: &str, wiring: &[(u16, &str)]) -> HardwareProfile {
        let mut wiring_map = HashMap::new();
        for (scan, key) in wiring {
            wiring_map.insert(*scan, (*key).to_string());
        }
        HardwareProfile {
            id: id.to_string(),
            vendor_id: 0x1,
            product_id: 0x2,
            name: None,
            virtual_layout_id: "layout-1".into(),
            wiring: wiring_map,
        }
    }

    fn keymap(id: &str, bindings: &[(&str, ActionBinding)]) -> Keymap {
        let mut map = HashMap::new();
        for (vk, action) in bindings {
            map.insert((*vk).to_string(), action.clone());
        }
        Keymap {
            id: id.to_string(),
            name: "test".into(),
            virtual_layout_id: "layout-1".into(),
            layers: vec![KeymapLayer {
                name: "base".into(),
                bindings: map,
            }],
        }
    }

    fn runtime_config_with_slots(slots: Vec<ProfileSlot>) -> RuntimeConfig {
        RuntimeConfig {
            devices: vec![DeviceSlots {
                device: device_id(),
                slots,
            }],
        }
    }

    #[test]
    fn skips_when_device_metadata_missing() {
        let pipeline = MappingPipeline::default();
        let event = InputEvent::key_down(KeyCode::A, 0);

        let result = pipeline.process_event(&event);

        assert!(!result.consumed);
        assert_eq!(
            result.skip_reason,
            Some(PipelineSkipReason::MissingDeviceIdentity)
        );
    }

    #[test]
    fn consumes_first_active_slot_by_priority() {
        let hw1 = wiring_profile("hw-1", &[(4, "VK_A")]);
        let hw2 = wiring_profile("hw-2", &[(4, "VK_A")]);
        let km1 = keymap(
            "km-1",
            &[("VK_A", ActionBinding::StandardKey("Escape".into()))],
        );
        let km2 = keymap("km-2", &[("VK_A", ActionBinding::StandardKey("B".into()))]);

        let slots = vec![
            ProfileSlot {
                id: "slot-low".into(),
                hardware_profile_id: hw2.id.clone(),
                keymap_id: km2.id.clone(),
                active: true,
                priority: 1,
            },
            ProfileSlot {
                id: "slot-high".into(),
                hardware_profile_id: hw1.id.clone(),
                keymap_id: km1.id.clone(),
                active: true,
                priority: 10,
            },
        ];

        let pipeline = MappingPipeline::new(
            runtime_config_with_slots(slots),
            HashMap::from([(hw1.id.clone(), hw1), (hw2.id.clone(), hw2)]),
            HashMap::from([(km1.id.clone(), km1), (km2.id.clone(), km2)]),
        );

        let event = make_event(4, true);
        let result = pipeline.process_event(&event);

        assert!(result.consumed);
        assert_eq!(result.slot_id.as_deref(), Some("slot-high"));
        assert_eq!(result.output, Some(OutputAction::KeyDown(KeyCode::Escape)));
    }

    #[test]
    fn transparent_binding_falls_through_to_next_slot() {
        let hw = wiring_profile("hw", &[(5, "VK_B")]);
        let km_transparent = keymap("km-t", &[("VK_B", ActionBinding::Transparent)]);
        let km_action = keymap("km-a", &[("VK_B", ActionBinding::StandardKey("C".into()))]);

        let slots = vec![
            ProfileSlot {
                id: "slot-transparent".into(),
                hardware_profile_id: hw.id.clone(),
                keymap_id: km_transparent.id.clone(),
                active: true,
                priority: 5,
            },
            ProfileSlot {
                id: "slot-handler".into(),
                hardware_profile_id: hw.id.clone(),
                keymap_id: km_action.id.clone(),
                active: true,
                priority: 1,
            },
        ];

        let pipeline = MappingPipeline::new(
            runtime_config_with_slots(slots),
            HashMap::from([(hw.id.clone(), hw)]),
            HashMap::from([
                (km_transparent.id.clone(), km_transparent),
                (km_action.id.clone(), km_action),
            ]),
        );

        let event = make_event(5, false);
        let result = pipeline.process_event(&event);

        assert!(result.consumed);
        assert_eq!(result.slot_id.as_deref(), Some("slot-handler"));
        assert_eq!(result.output, Some(OutputAction::KeyUp(KeyCode::C)));
    }

    #[test]
    fn skips_when_no_active_slot_matches() {
        let hw = wiring_profile("hw", &[(7, "VK_C")]);
        let km = keymap("km", &[("VK_C", ActionBinding::StandardKey("D".into()))]);
        let slots = vec![ProfileSlot {
            id: "inactive".into(),
            hardware_profile_id: hw.id.clone(),
            keymap_id: km.id.clone(),
            active: false,
            priority: 1,
        }];

        let pipeline = MappingPipeline::new(
            runtime_config_with_slots(slots),
            HashMap::from([(hw.id.clone(), hw)]),
            HashMap::from([(km.id.clone(), km)]),
        );

        let event = make_event(7, true);
        let result = pipeline.process_event(&event);

        assert!(!result.consumed);
        assert_eq!(
            result.skip_reason,
            Some(PipelineSkipReason::NoActiveSlotMatched)
        );
    }
}
