#![allow(dead_code)]

use std::collections::HashMap;

use smallvec::SmallVec;

use crate::engine::{KeyCode, LayerAction};

const MIN_COMBO_KEYS: usize = 2;
const MAX_COMBO_KEYS: usize = 4;

/// A combo definition with its normalized key set and resulting action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComboDef {
    /// Canonical, deduplicated keys for the combo.
    pub keys: SmallVec<[KeyCode; 4]>,
    /// Action to perform when the combo is matched.
    pub action: LayerAction,
}

/// Registry of combo definitions supporting order-independent lookup.
#[derive(Debug, Default, Clone)]
pub struct ComboRegistry {
    combos: HashMap<SmallVec<[KeyCode; 4]>, LayerAction>,
}

impl ComboRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            combos: HashMap::new(),
        }
    }

    /// Register a new combo. Keys are normalized (deduped + sorted) to ensure
    /// order-independent matching. Returns false if the key count is invalid.
    pub fn register(&mut self, keys: &[KeyCode], action: LayerAction) -> bool {
        let normalized = normalize_keys(keys);
        if normalized.len() < MIN_COMBO_KEYS || normalized.len() > MAX_COMBO_KEYS {
            return false;
        }

        self.combos.insert(normalized, action);
        true
    }

    /// Find a combo action for the given keys, ignoring order.
    pub fn find(&self, keys: &[KeyCode]) -> Option<&LayerAction> {
        let normalized = normalize_keys(keys);
        self.combos.get(&normalized)
    }

    /// Return all registered combos (useful for inspection and diagnostics).
    pub fn all(&self) -> impl Iterator<Item = ComboDef> + '_ {
        self.combos.iter().map(|(keys, action)| ComboDef {
            keys: keys.clone(),
            action: action.clone(),
        })
    }

    /// Number of registered combos.
    pub fn len(&self) -> usize {
        self.combos.len()
    }

    /// True if no combos are registered.
    pub fn is_empty(&self) -> bool {
        self.combos.is_empty()
    }
}

fn normalize_keys(keys: &[KeyCode]) -> SmallVec<[KeyCode; 4]> {
    let mut unique = SmallVec::<[KeyCode; 4]>::new();
    for key in keys.iter().copied() {
        if !unique.contains(&key) {
            unique.push(key);
        }
    }
    // Sort by display name to obtain a stable ordering for HashMap keys.
    unique.sort_by_key(|key| key.name());
    unique
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_and_matches_two_key_combo_order_independent() {
        let mut registry = ComboRegistry::new();
        assert!(registry.register(
            &[KeyCode::A, KeyCode::B],
            LayerAction::Remap(KeyCode::Escape)
        ));

        let action = registry.find(&[KeyCode::B, KeyCode::A]);
        assert_eq!(action, Some(&LayerAction::Remap(KeyCode::Escape)));
    }

    #[test]
    fn registers_three_key_combo_and_matches_any_order() {
        let mut registry = ComboRegistry::new();
        assert!(registry.register(&[KeyCode::Q, KeyCode::W, KeyCode::E], LayerAction::Block));

        let action = registry.find(&[KeyCode::E, KeyCode::Q, KeyCode::W]);
        assert_eq!(action, Some(&LayerAction::Block));
    }

    #[test]
    fn rejects_invalid_key_counts() {
        let mut registry = ComboRegistry::new();
        assert!(!registry.register(&[KeyCode::A], LayerAction::Block));
        assert!(!registry.register(
            &[KeyCode::A, KeyCode::B, KeyCode::C, KeyCode::D, KeyCode::E],
            LayerAction::Block
        ));
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn duplicates_are_deduped_and_still_match() {
        let mut registry = ComboRegistry::new();
        assert!(registry.register(
            &[KeyCode::A, KeyCode::A, KeyCode::B],
            LayerAction::LayerPush(1)
        ));

        // Should only store two unique keys.
        let stored: Vec<ComboDef> = registry.all().collect();
        assert_eq!(stored.len(), 1);
        assert_eq!(
            stored[0].keys,
            SmallVec::<[KeyCode; 4]>::from_vec(vec![KeyCode::A, KeyCode::B])
        );

        // Matching with duplicates or different order still works.
        let action = registry.find(&[KeyCode::B, KeyCode::A, KeyCode::A]);
        assert_eq!(action, Some(&LayerAction::LayerPush(1)));
    }

    #[test]
    fn find_returns_none_for_unknown_combo() {
        let mut registry = ComboRegistry::new();
        registry.register(
            &[KeyCode::A, KeyCode::B],
            LayerAction::Remap(KeyCode::Escape),
        );

        assert!(registry.find(&[KeyCode::C, KeyCode::D]).is_none());
    }
}
