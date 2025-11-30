//! Registry for storing key remappings defined by scripts.

use crate::engine::{ComboRegistry, HoldAction, KeyCode, LayerAction, RemapAction};
use std::collections::HashMap;

/// Central storage for script-defined key behaviors.
///
/// The registry maps physical keys to actions (remap, block, pass) and tap-hold
/// bindings. Unmapped keys default to Pass (unchanged passthrough).
#[derive(Debug, Clone)]
pub struct RemapRegistry {
    mappings: HashMap<KeyCode, RemapAction>,
    tap_holds: HashMap<KeyCode, TapHoldBinding>,
    combos: ComboRegistry,
}

/// A tap-hold binding configured via script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TapHoldBinding {
    pub tap: KeyCode,
    pub hold: HoldAction,
}

impl RemapRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            tap_holds: HashMap::new(),
            combos: ComboRegistry::new(),
        }
    }

    /// Remap a key to another key.
    ///
    /// When `from` is pressed, it will be translated to `to`.
    pub fn remap(&mut self, from: KeyCode, to: KeyCode) {
        self.mappings.insert(from, RemapAction::Remap(to));
    }

    /// Block a key (consume it without passing through).
    pub fn block(&mut self, key: KeyCode) {
        self.mappings.insert(key, RemapAction::Block);
    }

    /// Pass a key through unchanged.
    ///
    /// This explicitly marks a key as passthrough, removing any existing mapping.
    pub fn pass(&mut self, key: KeyCode) {
        self.mappings.insert(key, RemapAction::Pass);
    }

    /// Register a tap-hold behavior for a key.
    pub fn register_tap_hold(&mut self, key: KeyCode, tap: KeyCode, hold: HoldAction) {
        self.tap_holds.insert(key, TapHoldBinding { tap, hold });
    }

    /// Register a combo definition.
    ///
    /// Returns true if the combo key count is valid (2-4) and stored.
    pub fn register_combo(&mut self, keys: &[KeyCode], action: LayerAction) -> bool {
        self.combos.register(keys, action)
    }

    /// Iterate over all combo definitions.
    pub fn combos(&self) -> &ComboRegistry {
        &self.combos
    }

    /// Look up the action for a key.
    ///
    /// Returns the mapped action, or `RemapAction::Pass` for unmapped keys.
    pub fn lookup(&self, key: KeyCode) -> RemapAction {
        self.mappings
            .get(&key)
            .copied()
            .unwrap_or(RemapAction::Pass)
    }

    /// Look up a tap-hold binding for a key, if configured.
    pub fn tap_hold(&self, key: KeyCode) -> Option<&TapHoldBinding> {
        self.tap_holds.get(&key)
    }

    /// Iterate over all tap-hold bindings.
    pub fn tap_holds(&self) -> impl Iterator<Item = (&KeyCode, &TapHoldBinding)> {
        self.tap_holds.iter()
    }

    /// Clear all mappings.
    pub fn clear(&mut self) {
        self.mappings.clear();
        self.tap_holds.clear();
        self.combos = ComboRegistry::new();
    }

    /// Get the number of active mappings.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty() && self.tap_holds.is_empty() && self.combos.is_empty()
    }
}

impl Default for RemapRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_registry_is_empty() {
        let registry = RemapRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.tap_holds().next().is_none());
    }

    #[test]
    fn unmapped_key_returns_pass() {
        let registry = RemapRegistry::new();
        assert_eq!(registry.lookup(KeyCode::A), RemapAction::Pass);
        assert_eq!(registry.lookup(KeyCode::CapsLock), RemapAction::Pass);
    }

    #[test]
    fn remap_a_to_b() {
        let mut registry = RemapRegistry::new();
        registry.remap(KeyCode::A, KeyCode::B);

        assert_eq!(registry.lookup(KeyCode::A), RemapAction::Remap(KeyCode::B));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn block_key() {
        let mut registry = RemapRegistry::new();
        registry.block(KeyCode::CapsLock);

        assert_eq!(registry.lookup(KeyCode::CapsLock), RemapAction::Block);
    }

    #[test]
    fn pass_removes_existing_mapping() {
        let mut registry = RemapRegistry::new();
        registry.block(KeyCode::CapsLock);
        assert_eq!(registry.lookup(KeyCode::CapsLock), RemapAction::Block);

        registry.pass(KeyCode::CapsLock);
        assert_eq!(registry.lookup(KeyCode::CapsLock), RemapAction::Pass);
    }

    #[test]
    fn clear_removes_all_mappings() {
        let mut registry = RemapRegistry::new();
        registry.remap(KeyCode::A, KeyCode::B);
        registry.block(KeyCode::CapsLock);
        registry.register_tap_hold(KeyCode::CapsLock, KeyCode::Escape, HoldAction::Modifier(1));
        registry.register_combo(
            &[KeyCode::A, KeyCode::B],
            LayerAction::Remap(KeyCode::Escape),
        );
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.tap_holds().count(), 1);
        assert_eq!(registry.combos().len(), 1);

        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.lookup(KeyCode::A), RemapAction::Pass);
        assert!(registry.tap_holds().next().is_none());
        assert_eq!(registry.combos().len(), 0);
    }

    #[test]
    fn remap_overwrites_previous() {
        let mut registry = RemapRegistry::new();
        registry.remap(KeyCode::A, KeyCode::B);
        registry.remap(KeyCode::A, KeyCode::C);

        assert_eq!(registry.lookup(KeyCode::A), RemapAction::Remap(KeyCode::C));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn default_is_empty() {
        let registry = RemapRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn tap_hold_binding_is_registered_and_retrievable() {
        let mut registry = RemapRegistry::new();
        registry.register_tap_hold(
            KeyCode::CapsLock,
            KeyCode::Escape,
            HoldAction::Key(KeyCode::LeftCtrl),
        );

        let binding = registry.tap_hold(KeyCode::CapsLock).unwrap();
        assert_eq!(binding.tap, KeyCode::Escape);
        assert_eq!(binding.hold, HoldAction::Key(KeyCode::LeftCtrl));
    }

    #[test]
    fn tap_hold_overwrites_previous_binding() {
        let mut registry = RemapRegistry::new();
        registry.register_tap_hold(
            KeyCode::CapsLock,
            KeyCode::Escape,
            HoldAction::Key(KeyCode::LeftCtrl),
        );
        registry.register_tap_hold(KeyCode::CapsLock, KeyCode::A, HoldAction::Modifier(2));

        let binding = registry.tap_hold(KeyCode::CapsLock).unwrap();
        assert_eq!(binding.tap, KeyCode::A);
        assert_eq!(binding.hold, HoldAction::Modifier(2));
        assert_eq!(registry.tap_holds().count(), 1);
    }

    #[test]
    fn combo_registration_is_stored_and_matches() {
        let mut registry = RemapRegistry::new();
        assert!(registry.register_combo(
            &[KeyCode::A, KeyCode::B],
            LayerAction::Remap(KeyCode::Escape)
        ));

        let action = registry.combos().find(&[KeyCode::B, KeyCode::A]);
        assert_eq!(action, Some(&LayerAction::Remap(KeyCode::Escape)));
    }

    #[test]
    fn combo_registration_rejects_invalid_counts() {
        let mut registry = RemapRegistry::new();
        assert!(!registry.register_combo(&[KeyCode::A], LayerAction::Block));
        assert!(!registry.register_combo(
            &[KeyCode::A, KeyCode::B, KeyCode::C, KeyCode::D, KeyCode::E],
            LayerAction::Block
        ));
        assert!(registry.combos().is_empty());
    }
}
