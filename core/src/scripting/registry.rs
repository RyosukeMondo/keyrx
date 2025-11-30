//! Registry for storing key remappings defined by scripts.

use crate::engine::{KeyCode, RemapAction};
use std::collections::HashMap;

/// Central storage for script-defined key behaviors.
///
/// The registry maps physical keys to actions (remap, block, or pass).
/// Unmapped keys default to Pass (unchanged passthrough).
#[derive(Debug, Clone)]
pub struct RemapRegistry {
    mappings: HashMap<KeyCode, RemapAction>,
}

impl RemapRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
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

    /// Look up the action for a key.
    ///
    /// Returns the mapped action, or `RemapAction::Pass` for unmapped keys.
    pub fn lookup(&self, key: KeyCode) -> RemapAction {
        self.mappings
            .get(&key)
            .copied()
            .unwrap_or(RemapAction::Pass)
    }

    /// Clear all mappings.
    pub fn clear(&mut self) {
        self.mappings.clear();
    }

    /// Get the number of active mappings.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
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
        assert_eq!(registry.len(), 2);

        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.lookup(KeyCode::A), RemapAction::Pass);
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
}
