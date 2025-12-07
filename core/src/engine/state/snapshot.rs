//! State snapshot for serialization, FFI, and debugging.
//!
//! StateSnapshot provides a serializable view of EngineState suitable for:
//! - FFI exports to Python/JavaScript bindings
//! - Debug logging and telemetry
//! - State persistence and replay
//! - Test assertions and property testing

use serde::{Deserialize, Serialize};

use crate::engine::layout::{LayoutCompositor, LayoutId, ModifierCoordinator};
use crate::engine::state::{LayerId, StandardModifiers};
use crate::engine::KeyCode;

use super::EngineState;

/// Serializable snapshot of engine state.
///
/// A StateSnapshot captures the complete engine state at a point in time
/// in a format that can be:
/// - Serialized to JSON/MessagePack for FFI and logging
/// - Used for debugging and state inspection
/// - Compared for equality in tests
/// - Persisted to disk for state recovery
///
/// # Example
///
/// ```no_run
/// use keyrx_core::engine::state::{EngineState, snapshot::StateSnapshot};
/// use keyrx_core::engine::decision::timing::TimingConfig;
///
/// let state = EngineState::new(TimingConfig::default());
/// let snapshot: StateSnapshot = (&state).into();
///
/// // Serialize to JSON
/// let json = serde_json::to_string(&snapshot).unwrap();
/// println!("State: {}", json);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
#[allow(dead_code)] // Will be used in tasks 13-16 for history, persistence, and FFI
pub struct StateSnapshot {
    /// State version counter.
    pub version: u64,

    /// Pressed keys with their timestamps.
    pub pressed_keys: Vec<PressedKey>,

    /// Active layer stack (first = lowest priority, last = highest).
    pub active_layers: Vec<LayerId>,

    /// Base layer ID.
    pub base_layer: LayerId,

    /// Standard modifier state.
    pub standard_modifiers: StandardModifiers,

    /// Virtual modifier state (IDs of active modifiers).
    pub virtual_modifiers: Vec<u8>,

    /// Number of pending decisions.
    pub pending_count: usize,

    /// Multi-layout compositor snapshot (if available).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_compositor: Option<LayoutCompositorSnapshot>,
}

/// A pressed key with its press timestamp.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler,
)]
#[ffi(strategy = "json")]
#[allow(dead_code)] // Will be used in tasks 13-16 for history, persistence, and FFI
pub struct PressedKey {
    /// The key code.
    pub key: KeyCode,
    /// The timestamp when the key was pressed (microseconds).
    pub pressed_at: u64,
}

impl From<&EngineState> for StateSnapshot {
    /// Create a snapshot from a reference to EngineState.
    ///
    /// This efficiently captures the current state without cloning the
    /// entire EngineState. Only the relevant data is extracted and copied.
    fn from(state: &EngineState) -> Self {
        // Collect pressed keys with their timestamps
        let pressed_keys: Vec<PressedKey> = state
            .all_pressed_keys()
            .into_iter()
            .map(|(key, pressed_at)| PressedKey { key, pressed_at })
            .collect();

        // Get active layers (already in priority order)
        let active_layers = state.active_layers().to_vec();

        // Extract virtual modifiers that are active
        let virtual_mods = state.virtual_modifiers();
        let mut virtual_modifier_ids = Vec::new();
        for id in 0..=255u8 {
            if virtual_mods.is_active(id) {
                virtual_modifier_ids.push(id);
            }
        }

        Self {
            version: state.version(),
            pressed_keys,
            active_layers,
            base_layer: state.base_layer(),
            standard_modifiers: state.standard_modifiers(),
            virtual_modifiers: virtual_modifier_ids,
            pending_count: state.pending_count(),
            layout_compositor: None,
        }
    }
}

#[allow(dead_code)] // Will be used in tasks 13-16 for history, persistence, and FFI
impl StateSnapshot {
    /// Create an empty snapshot (useful for testing).
    pub fn empty() -> Self {
        Self {
            version: 0,
            pressed_keys: Vec::new(),
            active_layers: vec![0], // Base layer always active
            base_layer: 0,
            standard_modifiers: StandardModifiers::default(),
            virtual_modifiers: Vec::new(),
            pending_count: 0,
            layout_compositor: None,
        }
    }

    /// Create a snapshot with specific pressed keys.
    pub fn with_keys(keys: Vec<PressedKey>) -> Self {
        Self {
            pressed_keys: keys,
            ..Self::empty()
        }
    }

    /// Create a snapshot with specific layers.
    pub fn with_layers(layers: Vec<LayerId>) -> Self {
        Self {
            active_layers: layers,
            ..Self::empty()
        }
    }

    /// Create a snapshot enriched with layout compositor data.
    pub fn with_layouts(
        state: &EngineState,
        layouts: &LayoutCompositor,
        coordinator: Option<&ModifierCoordinator>,
    ) -> Self {
        let mut snapshot: StateSnapshot = state.into();
        snapshot.layout_compositor = Some(LayoutCompositorSnapshot::from_components(
            layouts,
            coordinator,
        ));
        snapshot
    }

    /// Check if a key is pressed in this snapshot.
    #[inline]
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.iter().any(|pk| pk.key == key)
    }

    /// Check if a layer is active in this snapshot.
    #[inline]
    pub fn is_layer_active(&self, layer_id: LayerId) -> bool {
        self.active_layers.contains(&layer_id)
    }

    /// Check if a virtual modifier is active in this snapshot.
    #[inline]
    pub fn is_virtual_modifier_active(&self, modifier_id: u8) -> bool {
        self.virtual_modifiers.contains(&modifier_id)
    }

    /// Get the number of pressed keys.
    #[inline]
    pub fn pressed_key_count(&self) -> usize {
        self.pressed_keys.len()
    }

    /// Get the number of active layers.
    #[inline]
    pub fn active_layer_count(&self) -> usize {
        self.active_layers.len()
    }

    /// Get the number of active virtual modifiers.
    #[inline]
    pub fn virtual_modifier_count(&self) -> usize {
        self.virtual_modifiers.len()
    }
}

/// Summary of a layout's state for FFI/debugging.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutSnapshot {
    /// Stable layout identifier.
    pub id: LayoutId,
    /// Human-friendly layout name.
    pub name: String,
    /// Priority applied during resolution (higher wins).
    pub priority: i32,
    /// Whether the layout participates in composition.
    pub enabled: bool,
    /// Active layer identifiers for this layout.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_layers: Vec<LayerId>,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags associated with the layout.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Cross-layout modifiers visible to this layout.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<u8>,
}

/// Snapshot of the layout compositor including shared modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LayoutCompositorSnapshot {
    /// Layouts in resolution order (enabled and disabled).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layouts: Vec<LayoutSnapshot>,
    /// Modifiers shared across every layout.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_modifiers: Vec<u8>,
}

impl LayoutCompositorSnapshot {
    fn from_components(
        layouts: &LayoutCompositor,
        coordinator: Option<&ModifierCoordinator>,
    ) -> Self {
        let shared_modifiers = layouts.shared_modifiers().active_ids();
        let layout_snapshots = layouts
            .all_layouts()
            .map(|layout| {
                let metadata = layout.layout().metadata().clone();
                let modifiers = coordinator
                    .map(|coord| coord.modifiers_for_layout(layout.id()).active_ids())
                    .unwrap_or_default();
                LayoutSnapshot {
                    id: metadata.id,
                    name: metadata.name,
                    priority: layout.priority(),
                    enabled: layout.enabled(),
                    active_layers: layout.layer_ids(),
                    description: metadata.description,
                    tags: metadata.tags,
                    modifiers,
                }
            })
            .collect();

        Self {
            layouts: layout_snapshots,
            shared_modifiers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::decision::timing::TimingConfig;
    use crate::engine::layout::{
        CrossLayoutModifier, Layout, LayoutCompositor, LayoutMetadata, ModifierCoordinator,
        ModifierScope,
    };
    use crate::engine::state::{Modifier, Mutation, StandardModifier};

    #[test]
    fn empty_snapshot() {
        let snapshot = StateSnapshot::empty();
        assert_eq!(snapshot.version, 0);
        assert_eq!(snapshot.pressed_keys.len(), 0);
        assert_eq!(snapshot.active_layers, vec![0]);
        assert_eq!(snapshot.base_layer, 0);
        assert_eq!(snapshot.virtual_modifiers.len(), 0);
        assert_eq!(snapshot.pending_count, 0);
    }

    #[test]
    fn snapshot_with_layouts_includes_compositor_state() {
        let mut compositor = LayoutCompositor::new();
        compositor.add_layout(Layout::new(LayoutMetadata::new("coding", "Coding")), 5);
        let mut coordinator = ModifierCoordinator::new();
        coordinator.activate(CrossLayoutModifier::new(3, ModifierScope::Global));

        let state = EngineState::new(TimingConfig::default());
        let snapshot = StateSnapshot::with_layouts(&state, &compositor, Some(&coordinator));

        let layout_state = snapshot
            .layout_compositor
            .expect("layout compositor snapshot");
        assert_eq!(layout_state.layouts.len(), 1);
        assert_eq!(layout_state.layouts[0].id, "coding");
        assert_eq!(layout_state.layouts[0].priority, 5);
        assert_eq!(layout_state.layouts[0].modifiers, vec![3]);
    }

    #[test]
    fn snapshot_from_empty_state() {
        let state = EngineState::new(TimingConfig::default());
        let snapshot: StateSnapshot = (&state).into();

        assert_eq!(snapshot.version, 0);
        assert_eq!(snapshot.pressed_keys.len(), 0);
        assert_eq!(snapshot.active_layers, vec![0]);
        assert_eq!(snapshot.base_layer, 0);
        assert_eq!(snapshot.pending_count, 0);
    }

    #[test]
    fn snapshot_with_pressed_keys() {
        let mut state = EngineState::new(TimingConfig::default());

        state
            .apply(Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            })
            .unwrap();
        state
            .apply(Mutation::KeyDown {
                key: KeyCode::B,
                timestamp_us: 1100,
                is_repeat: false,
            })
            .unwrap();

        let snapshot: StateSnapshot = (&state).into();

        assert_eq!(snapshot.version, 2);
        assert_eq!(snapshot.pressed_key_count(), 2);
        assert!(snapshot.is_key_pressed(KeyCode::A));
        assert!(snapshot.is_key_pressed(KeyCode::B));

        // Verify keys are present with correct timestamps
        let key_a = snapshot.pressed_keys.iter().find(|pk| pk.key == KeyCode::A);
        let key_b = snapshot.pressed_keys.iter().find(|pk| pk.key == KeyCode::B);
        assert_eq!(key_a.unwrap().pressed_at, 1000);
        assert_eq!(key_b.unwrap().pressed_at, 1100);
    }

    #[test]
    fn snapshot_with_layers() {
        let mut state = EngineState::new(TimingConfig::default());

        state.apply(Mutation::PushLayer { layer_id: 1 }).unwrap();
        state.apply(Mutation::PushLayer { layer_id: 2 }).unwrap();

        let snapshot: StateSnapshot = (&state).into();

        assert_eq!(snapshot.version, 2);
        assert_eq!(snapshot.active_layer_count(), 3); // Base + 2 pushed
        assert_eq!(snapshot.active_layers, vec![0, 1, 2]);
        assert!(snapshot.is_layer_active(0));
        assert!(snapshot.is_layer_active(1));
        assert!(snapshot.is_layer_active(2));
    }

    #[test]
    fn snapshot_with_modifiers() {
        let mut state = EngineState::new(TimingConfig::default());

        // Activate virtual modifiers
        state
            .apply(Mutation::ActivateModifier { modifier_id: 5 })
            .unwrap();
        state
            .apply(Mutation::ActivateModifier { modifier_id: 10 })
            .unwrap();

        // Activate a standard modifier directly (normally done via key processing)
        state
            .modifiers_mut()
            .activate(Modifier::Standard(StandardModifier::Shift));

        let snapshot: StateSnapshot = (&state).into();

        assert_eq!(snapshot.version, 2);
        assert_eq!(snapshot.virtual_modifier_count(), 2);
        assert!(snapshot.is_virtual_modifier_active(5));
        assert!(snapshot.is_virtual_modifier_active(10));
        assert!(snapshot
            .standard_modifiers
            .is_active(StandardModifier::Shift));
    }

    #[test]
    fn snapshot_with_pending() {
        let mut state = EngineState::new(TimingConfig::default());

        state
            .apply(Mutation::AddTapHold {
                key: KeyCode::A,
                pressed_at: 1000,
                tap_action: KeyCode::B,
                hold_action: crate::engine::state::HoldAction::Key(KeyCode::C),
            })
            .unwrap();

        let snapshot: StateSnapshot = (&state).into();

        assert_eq!(snapshot.version, 1);
        assert_eq!(snapshot.pending_count, 1);
    }

    #[test]
    fn snapshot_serialization() {
        let mut state = EngineState::new(TimingConfig::default());

        state
            .apply(Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            })
            .unwrap();
        state.apply(Mutation::PushLayer { layer_id: 1 }).unwrap();

        let snapshot: StateSnapshot = (&state).into();

        // Serialize to JSON
        let json = serde_json::to_string(&snapshot).expect("serialize");
        assert!(json.contains("\"version\":2"));
        assert!(json.contains("\"pressed_keys\""));
        assert!(json.contains("\"active_layers\""));

        // Deserialize back
        let deserialized: StateSnapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(snapshot, deserialized);
    }

    #[test]
    fn snapshot_query_methods() {
        let snapshot = StateSnapshot {
            version: 5,
            pressed_keys: vec![
                PressedKey {
                    key: KeyCode::A,
                    pressed_at: 1000,
                },
                PressedKey {
                    key: KeyCode::B,
                    pressed_at: 1100,
                },
            ],
            active_layers: vec![0, 1, 2],
            base_layer: 0,
            standard_modifiers: StandardModifiers::default(),
            virtual_modifiers: vec![5, 10],
            pending_count: 3,
            layout_compositor: None,
        };

        assert!(snapshot.is_key_pressed(KeyCode::A));
        assert!(snapshot.is_key_pressed(KeyCode::B));
        assert!(!snapshot.is_key_pressed(KeyCode::C));

        assert!(snapshot.is_layer_active(0));
        assert!(snapshot.is_layer_active(1));
        assert!(snapshot.is_layer_active(2));
        assert!(!snapshot.is_layer_active(3));

        assert!(snapshot.is_virtual_modifier_active(5));
        assert!(snapshot.is_virtual_modifier_active(10));
        assert!(!snapshot.is_virtual_modifier_active(15));

        assert_eq!(snapshot.pressed_key_count(), 2);
        assert_eq!(snapshot.active_layer_count(), 3);
        assert_eq!(snapshot.virtual_modifier_count(), 2);
    }

    #[test]
    fn snapshot_with_keys_builder() {
        let keys = vec![
            PressedKey {
                key: KeyCode::A,
                pressed_at: 1000,
            },
            PressedKey {
                key: KeyCode::B,
                pressed_at: 1100,
            },
        ];

        let snapshot = StateSnapshot::with_keys(keys.clone());

        assert_eq!(snapshot.pressed_keys, keys);
        assert_eq!(snapshot.active_layers, vec![0]);
        assert_eq!(snapshot.version, 0);
    }

    #[test]
    fn snapshot_with_layers_builder() {
        let layers = vec![0, 1, 2, 3];
        let snapshot = StateSnapshot::with_layers(layers.clone());

        assert_eq!(snapshot.active_layers, layers);
        assert_eq!(snapshot.pressed_keys.len(), 0);
        assert_eq!(snapshot.version, 0);
    }

    #[test]
    fn snapshot_multiple_pressed_keys() {
        let mut state = EngineState::new(TimingConfig::default());

        // Add multiple keys
        state
            .apply(Mutation::KeyDown {
                key: KeyCode::Z,
                timestamp_us: 1000,
                is_repeat: false,
            })
            .unwrap();
        state
            .apply(Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1100,
                is_repeat: false,
            })
            .unwrap();
        state
            .apply(Mutation::KeyDown {
                key: KeyCode::M,
                timestamp_us: 1200,
                is_repeat: false,
            })
            .unwrap();

        let snapshot: StateSnapshot = (&state).into();

        // All keys should be present
        assert_eq!(snapshot.pressed_key_count(), 3);
        assert!(snapshot.is_key_pressed(KeyCode::A));
        assert!(snapshot.is_key_pressed(KeyCode::M));
        assert!(snapshot.is_key_pressed(KeyCode::Z));
    }

    #[test]
    fn snapshot_virtual_modifiers_extracted() {
        let mut state = EngineState::new(TimingConfig::default());

        // Activate several virtual modifiers
        state
            .apply(Mutation::ActivateModifier { modifier_id: 0 })
            .unwrap();
        state
            .apply(Mutation::ActivateModifier { modifier_id: 100 })
            .unwrap();
        state
            .apply(Mutation::ActivateModifier { modifier_id: 254 })
            .unwrap();

        let snapshot: StateSnapshot = (&state).into();

        assert_eq!(snapshot.virtual_modifiers.len(), 3);
        assert!(snapshot.virtual_modifiers.contains(&0));
        assert!(snapshot.virtual_modifiers.contains(&100));
        assert!(snapshot.virtual_modifiers.contains(&254));
    }

    #[test]
    fn snapshot_base_layer_preserved() {
        let state = EngineState::with_base_layer(5, TimingConfig::default());
        let snapshot: StateSnapshot = (&state).into();

        assert_eq!(snapshot.base_layer, 5);
        assert!(snapshot.active_layers.contains(&5));
    }
}
