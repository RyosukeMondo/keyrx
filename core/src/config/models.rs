use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Identifier for a virtual key within a layout.
pub type VirtualKeyId = String;

/// Identifier for a virtual layout definition.
pub type VirtualLayoutId = String;

/// Identifier for a hardware profile (wiring configuration).
pub type HardwareProfileId = String;

/// Identifier for a logical keymap.
pub type KeymapId = String;

/// Supported layout representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutType {
    Matrix,
    Semantic,
}

/// Visual metadata for positioning a virtual key in editors.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KeyPosition {
    pub x: f32,
    pub y: f32,
}

/// Visual size for a virtual key in editors.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KeySize {
    pub width: f32,
    pub height: f32,
}

/// Definition of a single virtual key within a layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualKeyDef {
    pub id: VirtualKeyId,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<KeyPosition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<KeySize>,
}

/// Layout-agnostic representation of keys used by hardware wiring and logical mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualLayout {
    pub id: VirtualLayoutId,
    pub name: String,
    pub layout_type: LayoutType,
    #[serde(default)]
    pub keys: Vec<VirtualKeyDef>,
}

/// Identifier for a concrete device instance (vendor/product + optional serial).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceInstanceId {
    pub vendor_id: u16,
    pub product_id: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
}

/// Wiring definition for a specific hardware device to a virtual layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub id: HardwareProfileId,
    pub vendor_id: u16,
    pub product_id: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub virtual_layout_id: VirtualLayoutId,
    /// Mapping of physical scancode -> virtual key identifier.
    #[serde(default)]
    pub wiring: HashMap<u16, VirtualKeyId>,
}

/// Action binding applied to a virtual key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ActionBinding {
    StandardKey(String),
    Macro(String),
    LayerToggle(String),
    Transparent,
}

/// A single logical layer of a keymap (virtual key -> action).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeymapLayer {
    pub name: String,
    #[serde(default)]
    pub bindings: HashMap<VirtualKeyId, ActionBinding>,
}

/// Logical mapping definition attached to a virtual layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keymap {
    pub id: KeymapId,
    pub name: String,
    pub virtual_layout_id: VirtualLayoutId,
    #[serde(default)]
    pub layers: Vec<KeymapLayer>,
}

/// Runtime assignment for a specific device: which wiring + keymap to apply.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileSlot {
    pub id: String,
    pub hardware_profile_id: HardwareProfileId,
    pub keymap_id: KeymapId,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub priority: u32,
}

/// Runtime slots associated to a single device instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceSlots {
    pub device: DeviceInstanceId,
    #[serde(default)]
    pub slots: Vec<ProfileSlot>,
}

/// Live runtime configuration for all connected devices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub devices: Vec<DeviceSlots>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn hardware_profile_round_trips() {
        let mut wiring = HashMap::new();
        wiring.insert(0x04, "KEY_A".to_string());

        let profile = HardwareProfile {
            id: "hw-stream-deck-left".into(),
            vendor_id: 0x0fd9,
            product_id: 0x0060,
            name: Some("Stream Deck Left".into()),
            virtual_layout_id: "layout-4x4".into(),
            wiring,
        };

        let encoded = serde_json::to_string(&profile).expect("serialize");
        let decoded: HardwareProfile = serde_json::from_str(&encoded).expect("deserialize");
        assert_eq!(decoded, profile);
    }

    #[test]
    fn runtime_config_defaults_are_populated_on_deserialize() {
        let encoded = json!({
            "devices": [{
                "device": {
                    "vendor_id": 1234,
                    "product_id": 5678
                },
                "slots": [{
                    "id": "slot-1",
                    "hardware_profile_id": "hw-1",
                    "keymap_id": "km-1"
                }]
            }]
        });

        let decoded: RuntimeConfig = serde_json::from_value(encoded).expect("deserialize");
        assert_eq!(decoded.devices.len(), 1);
        let slots = &decoded.devices[0].slots;
        assert_eq!(slots.len(), 1);
        assert!(!slots[0].active);
        assert_eq!(slots[0].priority, 0);
    }

    #[test]
    fn action_binding_uses_tagged_enum_shape() {
        let binding = ActionBinding::LayerToggle("fn".into());
        let value: Value = serde_json::to_value(&binding).expect("to value");
        assert_eq!(value["type"], "layer_toggle");
        assert_eq!(value["value"], "fn");
    }
}
