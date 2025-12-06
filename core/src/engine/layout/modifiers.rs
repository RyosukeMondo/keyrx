//! Cross-layout modifier coordination with scoping and sharing rules.
//!
//! This module tracks modifiers that should be visible across multiple layouts
//! while honoring layout-specific scopes and temporary (held) modifiers. It
//! provides helpers to merge shared/global modifiers with layout-scoped ones so
//! each layout gets the correct active set.

use std::collections::HashMap;

use super::LayoutId;
use crate::engine::state::ModifierSet;

/// Scope for a modifier across layouts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModifierScope {
    /// Modifier is visible to every layout.
    Global,
    /// Modifier only applies to a specific layout id.
    Layout(LayoutId),
    /// Modifier is global but only while held (cleared separately).
    Temporary,
}

/// A modifier with scope and stickiness metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrossLayoutModifier {
    pub id: u8,
    pub scope: ModifierScope,
    /// Sticky modifiers persist until explicitly cleared.
    pub sticky: bool,
}

impl CrossLayoutModifier {
    /// Create a non-sticky modifier with the provided scope.
    pub fn new(id: u8, scope: ModifierScope) -> Self {
        Self {
            id,
            scope,
            sticky: false,
        }
    }

    /// Create a sticky modifier with the provided scope.
    pub fn sticky(id: u8, scope: ModifierScope) -> Self {
        Self {
            id,
            scope,
            sticky: true,
        }
    }
}

/// Tracks modifiers that should be shared across layouts with scoped overrides.
#[derive(Debug, Clone, Default)]
pub struct ModifierCoordinator {
    global: ModifierSet,
    sticky_global: ModifierSet,
    temporary: ModifierSet,
    layout_scoped: HashMap<LayoutId, ModifierSet>,
    sticky_layout_scoped: HashMap<LayoutId, ModifierSet>,
}

impl ModifierCoordinator {
    /// Create an empty coordinator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Activate a modifier with its scope.
    pub fn activate(&mut self, modifier: CrossLayoutModifier) {
        match (modifier.scope, modifier.sticky) {
            (ModifierScope::Global, false) => self.global.add(modifier.id),
            (ModifierScope::Global, true) => self.sticky_global.add(modifier.id),
            (ModifierScope::Temporary, _) => self.temporary.add(modifier.id),
            (ModifierScope::Layout(layout_id), sticky) => {
                let set = if sticky {
                    self.sticky_layout_scoped.entry(layout_id).or_default()
                } else {
                    self.layout_scoped.entry(layout_id).or_default()
                };
                set.add(modifier.id);
            }
        }
    }

    /// Release a non-sticky modifier in the given scope.
    ///
    /// Sticky modifiers stay active until `clear_sticky` is called.
    pub fn release(&mut self, modifier: &CrossLayoutModifier) -> bool {
        match &modifier.scope {
            ModifierScope::Global => self.global_remove(modifier.id),
            ModifierScope::Temporary => Self::remove_from_set(&mut self.temporary, modifier.id),
            ModifierScope::Layout(layout_id) => self.remove_from_layout(layout_id, modifier.id),
        }
    }

    /// Clear a sticky modifier in the specified scope.
    pub fn clear_sticky(&mut self, id: u8, scope: &ModifierScope) -> bool {
        match scope {
            ModifierScope::Global => self.sticky_global_remove(id),
            ModifierScope::Temporary => Self::remove_from_set(&mut self.temporary, id),
            ModifierScope::Layout(layout_id) => self.remove_from_sticky_layout(layout_id, id),
        }
    }

    /// Clear all temporary (held) modifiers.
    pub fn clear_temporary(&mut self) {
        self.temporary.clear();
    }

    /// Clear modifiers for a specific layout (both sticky and non-sticky).
    pub fn clear_layout(&mut self, layout_id: &str) {
        self.layout_scoped.remove(layout_id);
        self.sticky_layout_scoped.remove(layout_id);
    }

    /// View of modifiers shared across every layout.
    pub fn shared_modifiers(&self) -> ModifierSet {
        let mut combined = ModifierSet::new();
        self.merge_set(&mut combined, &self.global);
        self.merge_set(&mut combined, &self.sticky_global);
        self.merge_set(&mut combined, &self.temporary);
        combined
    }

    /// View of modifiers for a specific layout, merged with shared scopes.
    pub fn modifiers_for_layout(&self, layout_id: &str) -> ModifierSet {
        let mut combined = self.shared_modifiers();
        if let Some(scoped) = self.layout_scoped.get(layout_id) {
            self.merge_set(&mut combined, scoped);
        }
        if let Some(sticky) = self.sticky_layout_scoped.get(layout_id) {
            self.merge_set(&mut combined, sticky);
        }
        combined
    }

    fn merge_set(&self, target: &mut ModifierSet, source: &ModifierSet) {
        for id in source.active_ids() {
            target.add(id);
        }
    }

    fn remove_from_set(set: &mut ModifierSet, id: u8) -> bool {
        let existed = set.contains(id);
        set.remove(id);
        existed
    }

    fn remove_from_layout(&mut self, layout_id: &LayoutId, id: u8) -> bool {
        self.layout_scoped
            .get_mut(layout_id)
            .map(|set| Self::remove_from_set(set, id))
            .unwrap_or(false)
    }

    fn remove_from_sticky_layout(&mut self, layout_id: &LayoutId, id: u8) -> bool {
        self.sticky_layout_scoped
            .get_mut(layout_id)
            .map(|set| Self::remove_from_set(set, id))
            .unwrap_or(false)
    }

    fn global_remove(&mut self, id: u8) -> bool {
        let removed_global = Self::remove_from_set(&mut self.global, id);
        let removed_sticky = self.sticky_global_remove(id);
        removed_global || removed_sticky
    }

    fn sticky_global_remove(&mut self, id: u8) -> bool {
        Self::remove_from_set(&mut self.sticky_global, id)
    }
}

#[cfg(test)]
mod tests {
    use super::{CrossLayoutModifier, ModifierCoordinator, ModifierScope};

    #[test]
    fn propagates_global_modifiers_to_all_layouts() {
        let mut coordinator = ModifierCoordinator::new();
        coordinator.activate(CrossLayoutModifier::new(1, ModifierScope::Global));

        let coding = coordinator.modifiers_for_layout("coding");
        let gaming = coordinator.modifiers_for_layout("gaming");

        assert!(coding.contains(1));
        assert!(gaming.contains(1));
    }

    #[test]
    fn scopes_modifiers_to_specific_layouts() {
        let mut coordinator = ModifierCoordinator::new();
        coordinator.activate(CrossLayoutModifier::new(
            2,
            ModifierScope::Layout("coding".into()),
        ));

        let coding = coordinator.modifiers_for_layout("coding");
        let gaming = coordinator.modifiers_for_layout("gaming");

        assert!(coding.contains(2));
        assert!(!gaming.contains(2));
    }

    #[test]
    fn releases_non_sticky_modifiers() {
        let mut coordinator = ModifierCoordinator::new();
        let modifier = CrossLayoutModifier::new(3, ModifierScope::Global);
        coordinator.activate(modifier.clone());

        assert!(coordinator.modifiers_for_layout("any").contains(3));
        assert!(coordinator.release(&modifier));
        assert!(!coordinator.modifiers_for_layout("any").contains(3));
    }

    #[test]
    fn sticky_modifiers_require_explicit_clear() {
        let mut coordinator = ModifierCoordinator::new();
        let sticky = CrossLayoutModifier::sticky(4, ModifierScope::Layout("studio".into()));
        coordinator.activate(sticky.clone());

        assert!(coordinator.modifiers_for_layout("studio").contains(4));
        assert!(!coordinator.release(&sticky));
        assert!(coordinator.modifiers_for_layout("studio").contains(4));

        assert!(coordinator.clear_sticky(4, &sticky.scope));
        assert!(!coordinator.modifiers_for_layout("studio").contains(4));
    }
}
