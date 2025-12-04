//! State delta types for incremental updates.
//!
//! Instead of sending full state snapshots, we can send deltas that capture
//! only what changed. This reduces serialization overhead and enables more
//! efficient FFI communication with the Flutter UI.

use crate::engine::state::LayerId;
use crate::engine::KeyCode;
use serde::{Deserialize, Serialize};

/// A delta representing changes to engine state.
///
/// StateDelta captures only the fields that changed between two state versions,
/// enabling incremental updates instead of full state snapshots.
///
/// # Version Tracking
///
/// Each delta includes version numbers to ensure consistency:
/// - `from_version`: The state version before the changes
/// - `to_version`: The state version after the changes
///
/// The receiver can verify the delta applies to their current state by
/// checking that their version matches `from_version`. If versions don't
/// match, a full state sync is required.
///
/// # Example
///
/// ```no_run
/// use keyrx_core::engine::state::delta::{StateDelta, DeltaChange};
/// use keyrx_core::engine::KeyCode;
///
/// let delta = StateDelta {
///     from_version: 42,
///     to_version: 43,
///     changes: vec![DeltaChange::KeyPressed(KeyCode::A)],
/// };
///
/// // Check if this delta is a no-op
/// assert!(!delta.is_empty());
///
/// // Check if we should use full sync instead
/// assert!(!delta.should_use_full_sync());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateDelta {
    /// State version before these changes.
    pub from_version: u64,
    /// State version after these changes.
    pub to_version: u64,
    /// List of changes in this delta.
    pub changes: Vec<DeltaChange>,
}

impl StateDelta {
    /// Create a new state delta.
    pub fn new(from_version: u64, to_version: u64, changes: Vec<DeltaChange>) -> Self {
        Self {
            from_version,
            to_version,
            changes,
        }
    }

    /// Returns true if this delta contains no changes.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Returns true if sending a full state sync would be more efficient.
    ///
    /// This heuristic compares the number of changes to a threshold.
    /// If more than 50% of the state fields are changing, it's more
    /// efficient to send the full state instead.
    ///
    /// Note: This is a simple heuristic. A more accurate implementation
    /// would compare actual serialized sizes, but that requires benchmarking
    /// to determine the right threshold.
    pub fn should_use_full_sync(&self) -> bool {
        // Rough heuristic: if we have more than 10 changes, use full sync
        // This threshold can be tuned based on profiling data
        self.changes.len() > 10
    }

    /// Get the number of changes in this delta.
    pub fn change_count(&self) -> usize {
        self.changes.len()
    }
}

/// A single change in a state delta.
///
/// Each variant represents one type of state change that can occur.
/// These are designed to be compact and efficient to serialize.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeltaChange {
    // === Key State Changes ===
    /// A key was pressed.
    KeyPressed(KeyCode),
    /// A key was released.
    KeyReleased(KeyCode),
    /// All keys were released.
    AllKeysReleased,

    // === Layer State Changes ===
    /// A layer was activated (pushed onto the stack).
    LayerActivated(LayerId),
    /// A layer was deactivated (removed from the stack).
    LayerDeactivated(LayerId),
    /// The entire layer stack was updated.
    LayerStackChanged {
        /// New layer stack in priority order.
        layers: Vec<LayerId>,
    },

    // === Modifier State Changes ===
    /// A modifier was activated.
    ModifierChanged {
        /// Modifier ID (0-254 for custom, 255 reserved).
        id: u8,
        /// Whether the modifier is now active.
        active: bool,
    },
    /// All modifiers were cleared.
    AllModifiersCleared,

    // === Pending Decision Changes ===
    /// A pending decision was added.
    PendingAdded {
        /// Unique ID for this pending decision.
        id: u32,
    },
    /// A pending decision was resolved.
    PendingResolved {
        /// ID of the resolved decision.
        id: u32,
    },
    /// All pending decisions were cleared.
    AllPendingCleared,

    // === Version Updates ===
    /// State version changed (used when multiple mutations occur atomically).
    VersionChanged {
        /// New version number.
        version: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_delta_creation() {
        let delta = StateDelta::new(10, 11, vec![DeltaChange::KeyPressed(KeyCode::A)]);

        assert_eq!(delta.from_version, 10);
        assert_eq!(delta.to_version, 11);
        assert_eq!(delta.change_count(), 1);
        assert!(!delta.is_empty());
        assert!(!delta.should_use_full_sync());
    }

    #[test]
    fn empty_delta() {
        let delta = StateDelta::new(5, 5, vec![]);

        assert!(delta.is_empty());
        assert_eq!(delta.change_count(), 0);
        assert!(!delta.should_use_full_sync());
    }

    #[test]
    fn large_delta_suggests_full_sync() {
        let changes = (0..15)
            .map(|i| DeltaChange::ModifierChanged {
                id: i as u8,
                active: true,
            })
            .collect();

        let delta = StateDelta::new(0, 1, changes);

        assert!(!delta.is_empty());
        assert!(delta.should_use_full_sync());
        assert_eq!(delta.change_count(), 15);
    }

    #[test]
    fn delta_change_variants() {
        let changes = vec![
            DeltaChange::KeyPressed(KeyCode::A),
            DeltaChange::KeyReleased(KeyCode::B),
            DeltaChange::AllKeysReleased,
            DeltaChange::LayerActivated(1),
            DeltaChange::LayerDeactivated(2),
            DeltaChange::LayerStackChanged {
                layers: vec![0, 1, 2],
            },
            DeltaChange::ModifierChanged {
                id: 5,
                active: true,
            },
            DeltaChange::AllModifiersCleared,
            DeltaChange::PendingAdded { id: 42 },
            DeltaChange::PendingResolved { id: 43 },
            DeltaChange::AllPendingCleared,
            DeltaChange::VersionChanged { version: 100 },
        ];

        let delta = StateDelta::new(0, 1, changes);
        assert_eq!(delta.change_count(), 12);
    }

    #[test]
    fn delta_serialization() {
        let delta = StateDelta::new(
            42,
            43,
            vec![
                DeltaChange::KeyPressed(KeyCode::A),
                DeltaChange::LayerActivated(1),
                DeltaChange::ModifierChanged {
                    id: 5,
                    active: true,
                },
            ],
        );

        let json = serde_json::to_string(&delta).expect("serializes");
        let deserialized: StateDelta = serde_json::from_str(&json).expect("deserializes");

        assert_eq!(delta, deserialized);
    }

    #[test]
    fn delta_change_equality() {
        let change1 = DeltaChange::KeyPressed(KeyCode::A);
        let change2 = DeltaChange::KeyPressed(KeyCode::A);
        let change3 = DeltaChange::KeyPressed(KeyCode::B);

        assert_eq!(change1, change2);
        assert_ne!(change1, change3);
    }

    #[test]
    fn modifier_change() {
        let change = DeltaChange::ModifierChanged {
            id: 10,
            active: true,
        };

        let json = serde_json::to_string(&change).expect("serializes");
        let deserialized: DeltaChange = serde_json::from_str(&json).expect("deserializes");

        assert_eq!(change, deserialized);
    }

    #[test]
    fn layer_stack_change() {
        let change = DeltaChange::LayerStackChanged {
            layers: vec![0, 3, 7],
        };

        let json = serde_json::to_string(&change).expect("serializes");
        let deserialized: DeltaChange = serde_json::from_str(&json).expect("deserializes");

        assert_eq!(change, deserialized);
    }

    #[test]
    fn pending_changes() {
        let add = DeltaChange::PendingAdded { id: 123 };
        let resolve = DeltaChange::PendingResolved { id: 456 };

        assert_ne!(add, resolve);

        let json1 = serde_json::to_string(&add).expect("serializes");
        let json2 = serde_json::to_string(&resolve).expect("serializes");

        let deserialized1: DeltaChange = serde_json::from_str(&json1).expect("deserializes");
        let deserialized2: DeltaChange = serde_json::from_str(&json2).expect("deserializes");

        assert_eq!(add, deserialized1);
        assert_eq!(resolve, deserialized2);
    }
}
