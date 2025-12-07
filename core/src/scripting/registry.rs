//! Registry for storing key remappings defined by scripts.

use crate::engine::layout::{Layout, LayoutCompositor, LayoutMetadata};
use crate::engine::{
    ComboRegistry, HoldAction, KeyCode, LayerAction, LayerId, LayerStack, Modifier, ModifierState,
    RemapAction, TimingConfig, VirtualModifiers,
};
use std::collections::HashMap;
use std::sync::OnceLock;

const DEFAULT_LAYOUT_ID: &str = "default";
static FALLBACK_LAYOUT: OnceLock<Layout> = OnceLock::new();

/// Central storage for script-defined key behaviors.
///
/// The registry maps physical keys to actions (remap, block, pass) and tap-hold
/// bindings. Unmapped keys default to Pass (unchanged passthrough).
#[derive(Debug, Clone)]
pub struct RemapRegistry {
    mappings: HashMap<KeyCode, RemapAction>,
    tap_holds: HashMap<KeyCode, TapHoldBinding>,
    combos: ComboRegistry,
    layouts: LayoutCompositor,
    modifier_names: HashMap<String, u8>,
    modifiers: ModifierState,
    next_modifier_id: u16,
    timing: TimingConfig,
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
        let mut layouts = LayoutCompositor::new();
        layouts.add_layout(
            Layout::new(LayoutMetadata::new(DEFAULT_LAYOUT_ID, "Default")),
            0,
        );
        Self {
            mappings: HashMap::new(),
            tap_holds: HashMap::new(),
            combos: ComboRegistry::new(),
            layouts,
            modifier_names: HashMap::new(),
            modifiers: ModifierState::new(),
            next_modifier_id: 0,
            timing: TimingConfig::default(),
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

    /// Iterate over all explicit key mappings (remap/block/pass).
    pub fn mappings(&self) -> impl Iterator<Item = (KeyCode, RemapAction)> + '_ {
        self.mappings.iter().map(|(key, action)| (*key, *action))
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

    /// Define a named virtual modifier, returning its ID.
    pub fn define_modifier(&mut self, name: &str) -> Result<u8, String> {
        self.define_modifier_with_id(name, None)
    }

    /// Define a named virtual modifier with an explicit ID (used by scripting runtime for determinism).
    pub fn define_modifier_with_id(
        &mut self,
        name: &str,
        explicit_id: Option<u8>,
    ) -> Result<u8, String> {
        let normalized = Self::normalize_modifier_name(name)?;
        if let Some(&existing) = self.modifier_names.get(&normalized) {
            return Ok(existing);
        }

        let id = if let Some(id) = explicit_id {
            if id as u16 > VirtualModifiers::MAX_ID as u16 {
                return Err(format!(
                    "modifier id {} exceeds maximum {}",
                    id,
                    VirtualModifiers::MAX_ID
                ));
            }
            id
        } else {
            if self.next_modifier_id > VirtualModifiers::MAX_ID as u16 {
                return Err(format!(
                    "maximum virtual modifiers reached (0-{})",
                    VirtualModifiers::MAX_ID
                ));
            }
            let id = self.next_modifier_id as u8;
            self.next_modifier_id = self.next_modifier_id.saturating_add(1);
            id
        };

        self.modifier_names.insert(normalized, id);
        if self.next_modifier_id <= id as u16 {
            self.next_modifier_id = id as u16 + 1;
        }
        Ok(id)
    }

    /// Look up a modifier ID by name.
    pub fn modifier_id(&self, name: &str) -> Option<u8> {
        let normalized = Self::normalize_modifier_name(name).ok()?;
        self.modifier_names.get(&normalized).copied()
    }

    /// Activate a virtual modifier by ID.
    pub fn activate_modifier(&mut self, id: u8) {
        self.modifiers.activate(Modifier::Virtual(id));
    }

    /// Deactivate a virtual modifier by ID.
    pub fn deactivate_modifier(&mut self, id: u8) {
        self.modifiers.deactivate(Modifier::Virtual(id));
    }

    /// Arm a one-shot virtual modifier by ID.
    pub fn one_shot_modifier(&mut self, id: u8) {
        self.modifiers.arm_one_shot(Modifier::Virtual(id));
    }

    /// Check if a virtual modifier is active (including one-shot).
    pub fn is_modifier_active(&self, id: u8) -> bool {
        self.modifiers.is_active(Modifier::Virtual(id))
    }

    /// Get the modifier state snapshot.
    pub fn modifier_state(&self) -> ModifierState {
        self.modifiers
    }

    /// Get the current timing configuration.
    pub fn timing_config(&self) -> &TimingConfig {
        &self.timing
    }

    /// Replace the timing configuration.
    pub fn set_timing_config(&mut self, timing: TimingConfig) {
        self.timing = timing;
    }

    /// Get modifier names (for syncing preview state).
    pub fn modifier_names(&self) -> &HashMap<String, u8> {
        &self.modifier_names
    }

    /// Next modifier ID to assign (for syncing preview state).
    pub fn next_modifier_id(&self) -> u16 {
        self.next_modifier_id
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

    /// Define or update a layer by name, returning its ID.
    pub fn define_layer(&mut self, name: &str, transparent: bool) -> Result<LayerId, String> {
        let normalized = Self::normalize_layer_name(name)?;
        Ok(self
            .default_layers_mut()
            .define_or_update_named(&normalized, transparent))
    }

    /// Map a key to an action within a named layer.
    pub fn map_layer(
        &mut self,
        layer_name: &str,
        key: KeyCode,
        action: LayerAction,
    ) -> Result<(), String> {
        let normalized = Self::normalize_layer_name(layer_name)?;
        let layer_id = self
            .default_layers()
            .layer_id_by_name(&normalized)
            .ok_or_else(|| format!("layer '{}' is not defined", normalized))?;

        if !self
            .default_layers_mut()
            .set_mapping_for_layer(layer_id, key, action)
        {
            return Err(format!("failed to set mapping for layer '{}'", normalized));
        }

        Ok(())
    }

    /// Push a named layer onto the active stack.
    pub fn push_layer(&mut self, name: &str) -> Result<bool, String> {
        let normalized = Self::normalize_layer_name(name)?;
        let layer_id = self
            .default_layers()
            .layer_id_by_name(&normalized)
            .ok_or_else(|| format!("layer '{}' is not defined", normalized))?;
        Ok(self.default_layers_mut().push(layer_id))
    }

    /// Toggle a named layer on/off.
    pub fn toggle_layer(&mut self, name: &str) -> Result<bool, String> {
        let normalized = Self::normalize_layer_name(name)?;
        let layer_id = self
            .default_layers()
            .layer_id_by_name(&normalized)
            .ok_or_else(|| format!("layer '{}' is not defined", normalized))?;
        Ok(self.default_layers_mut().toggle(layer_id))
    }

    /// Pop the top-most non-base layer.
    pub fn pop_layer(&mut self) -> Option<LayerId> {
        self.default_layers_mut().pop()
    }

    /// Check if a layer is active by name.
    pub fn is_layer_active(&self, name: &str) -> Result<bool, String> {
        let normalized = Self::normalize_layer_name(name)?;
        Ok(self.default_layers().is_active_by_name(&normalized))
    }

    /// Get the layer stack.
    pub fn layers(&self) -> &LayerStack {
        self.default_layers()
    }

    /// Look up a layer ID by name.
    pub fn layer_id(&self, name: &str) -> Option<LayerId> {
        let normalized = Self::normalize_layer_name(name).ok()?;
        self.default_layers().layer_id_by_name(&normalized)
    }

    /// Clear all mappings.
    pub fn clear(&mut self) {
        self.mappings.clear();
        self.tap_holds.clear();
        self.combos = ComboRegistry::new();
        self.layouts.clear();
        self.layouts.add_layout(
            Layout::new(LayoutMetadata::new(DEFAULT_LAYOUT_ID, "Default")),
            0,
        );
        self.modifier_names.clear();
        self.modifiers = ModifierState::new();
        self.next_modifier_id = 0;
        self.timing = TimingConfig::default();
    }

    /// Get the number of active mappings.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
            && self.tap_holds.is_empty()
            && self.combos.is_empty()
            && self.default_layers().is_empty()
            && self.modifier_names.is_empty()
            && self.modifiers == ModifierState::new()
            && self.timing == TimingConfig::default()
    }

    /// Access layout compositor (including default layout).
    pub fn layouts(&self) -> &LayoutCompositor {
        &self.layouts
    }

    /// Mutably access layout compositor (including default layout).
    pub fn layouts_mut(&mut self) -> &mut LayoutCompositor {
        &mut self.layouts
    }

    /// Define or update a layout with priority.
    pub fn define_layout(&mut self, metadata: LayoutMetadata, priority: i32) {
        self.layouts.add_layout(Layout::new(metadata), priority);
    }

    /// Remove a layout by id (default layout cannot be removed).
    pub fn remove_layout(&mut self, id: &str) -> bool {
        if id == DEFAULT_LAYOUT_ID {
            return false;
        }
        self.layouts.remove_layout(id)
    }

    /// Enable a layout by id.
    pub fn enable_layout(&mut self, id: &str) -> bool {
        self.layouts.enable_layout(id)
    }

    /// Disable a layout by id (default layout stays enabled).
    pub fn disable_layout(&mut self, id: &str) -> bool {
        if id == DEFAULT_LAYOUT_ID {
            return false;
        }
        self.layouts.disable_layout(id)
    }

    /// Set layout priority (higher wins). Returns false if layout is unknown.
    pub fn set_layout_priority(&mut self, id: &str, priority: i32) -> bool {
        self.layouts.set_priority(id, priority)
    }

    fn default_layout(&self) -> &Layout {
        self.layouts
            .layout(DEFAULT_LAYOUT_ID)
            .or_else(|| {
                self.layouts
                    .active_layouts()
                    .next()
                    .map(|layout| layout.layout())
            })
            .unwrap_or_else(|| {
                FALLBACK_LAYOUT
                    .get_or_init(|| Layout::new(LayoutMetadata::new(DEFAULT_LAYOUT_ID, "Default")))
            })
    }

    fn default_layout_mut(&mut self) -> &mut Layout {
        if self.layouts.layout(DEFAULT_LAYOUT_ID).is_none() {
            self.layouts.add_layout(
                Layout::new(LayoutMetadata::new(DEFAULT_LAYOUT_ID, "Default")),
                0,
            );
        }
        if let Some(layout) = self.layouts.layout_mut(DEFAULT_LAYOUT_ID) {
            layout
        } else {
            unreachable!("default layout must exist")
        }
    }

    fn default_layers(&self) -> &LayerStack {
        self.default_layout().layers()
    }

    fn default_layers_mut(&mut self) -> &mut LayerStack {
        self.default_layout_mut().layers_mut()
    }

    fn normalize_layer_name(name: &str) -> Result<String, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("layer name cannot be empty".into());
        }
        if trimmed.contains(':') {
            return Err("layer name cannot contain ':'".into());
        }
        Ok(trimmed.to_string())
    }

    fn normalize_modifier_name(name: &str) -> Result<String, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("modifier name cannot be empty".into());
        }
        if trimmed.contains(':') {
            return Err("modifier name cannot contain ':'".into());
        }
        Ok(trimmed.to_string())
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
        registry.define_modifier("hyper").unwrap();
        registry.activate_modifier(0);
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.tap_holds().count(), 1);
        assert_eq!(registry.combos().len(), 1);
        assert!(registry.is_modifier_active(0));
        assert_eq!(registry.modifier_id("hyper"), Some(0));

        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.lookup(KeyCode::A), RemapAction::Pass);
        assert!(registry.tap_holds().next().is_none());
        assert_eq!(registry.combos().len(), 0);
        assert!(registry.modifier_id("hyper").is_none());
        assert!(!registry.is_modifier_active(0));
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

    #[test]
    fn layer_define_and_mapping_are_stored() {
        let mut registry = RemapRegistry::new();
        registry.define_layer("nav", true).unwrap();
        registry
            .map_layer("nav", KeyCode::A, LayerAction::Remap(KeyCode::Escape))
            .unwrap();
        assert!(registry.push_layer("nav").unwrap());

        let action = registry.layers().lookup(KeyCode::A);
        assert_eq!(action, Some(&LayerAction::Remap(KeyCode::Escape)));
        assert!(registry.is_layer_active("NAV").unwrap());
    }

    #[test]
    fn layer_operations_require_defined_layer() {
        let mut registry = RemapRegistry::new();
        assert!(registry.push_layer("nav").is_err());
        assert!(registry.toggle_layer("nav").is_err());
        assert!(registry
            .map_layer("nav", KeyCode::A, LayerAction::Block)
            .is_err());

        registry.define_layer("nav", false).unwrap();
        assert!(registry.push_layer("nav").unwrap());
        assert!(registry.toggle_layer("nav").unwrap());
    }

    #[test]
    fn define_modifier_assigns_stable_ids() {
        let mut registry = RemapRegistry::new();
        let first = registry.define_modifier("hyper").unwrap();
        let second = registry.define_modifier("meh").unwrap();
        let repeat = registry.define_modifier("hyper").unwrap();

        assert_eq!(first, 0);
        assert_eq!(second, 1);
        assert_eq!(repeat, first);
        assert_eq!(registry.modifier_id("hyper"), Some(0));
        assert_eq!(registry.modifier_id("meh"), Some(1));
    }

    #[test]
    fn modifier_activation_and_one_shot_state() {
        let mut registry = RemapRegistry::new();
        let id = registry.define_modifier("hyper").unwrap();

        registry.activate_modifier(id);
        assert!(registry.modifier_state().is_active(Modifier::Virtual(id)));

        registry.deactivate_modifier(id);
        assert!(!registry.modifier_state().is_active(Modifier::Virtual(id)));

        registry.one_shot_modifier(id);
        let mut snapshot = registry.modifier_state();
        assert!(snapshot.is_active(Modifier::Virtual(id)));
        assert!(snapshot.consume_one_shot(Modifier::Virtual(id)));
        assert!(!snapshot.is_active(Modifier::Virtual(id)));
    }
}
