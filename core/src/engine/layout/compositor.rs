//! Composition of multiple layouts with priority-based resolution.
//!
//! The compositor keeps layouts ordered by priority and recency (most recent
//! wins on ties) so callers can resolve conflicts predictably. Layouts can be
//! enabled/disabled without removal, and a shared modifier set is tracked for
//! upcoming cross-layout modifier coordination.

use std::cmp::Ordering;

use super::{Layout, LayoutId};
use crate::engine::state::{LayerId, ModifierSet};

/// A layout registered with the compositor.
#[derive(Debug, Clone)]
pub struct ActiveLayout {
    layout: Layout,
    priority: i32,
    enabled: bool,
    order: u64,
}

impl ActiveLayout {
    fn new(layout: Layout, priority: i32, order: u64) -> Self {
        Self {
            layout,
            priority,
            enabled: true,
            order,
        }
    }

    /// Unique layout identifier.
    pub fn id(&self) -> &str {
        self.layout.id()
    }

    /// Layout priority (higher wins).
    pub fn priority(&self) -> i32 {
        self.priority
    }

    /// Whether this layout participates in resolution.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Borrow the underlying layout.
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Mutably borrow the underlying layout.
    pub fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }

    /// Active layer identifiers in priority order for this layout.
    pub fn layer_ids(&self) -> Vec<LayerId> {
        self.layout.active_layer_ids()
    }
}

/// Snapshot of a resolved layout used for lookups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLayout {
    pub id: LayoutId,
    pub priority: i32,
    pub layer_ids: Vec<LayerId>,
}

/// Manages active layouts and shared modifiers.
#[derive(Debug, Clone, Default)]
pub struct LayoutCompositor {
    layouts: Vec<ActiveLayout>,
    shared_modifiers: ModifierSet,
    sequence: u64,
}

impl LayoutCompositor {
    /// Create an empty compositor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a compositor with pre-seeded shared modifiers.
    pub fn with_shared_modifiers(shared_modifiers: ModifierSet) -> Self {
        Self {
            layouts: Vec::new(),
            shared_modifiers,
            sequence: 0,
        }
    }

    /// Borrow the shared modifier set.
    pub fn shared_modifiers(&self) -> &ModifierSet {
        &self.shared_modifiers
    }

    /// Mutably borrow the shared modifier set.
    pub fn shared_modifiers_mut(&mut self) -> &mut ModifierSet {
        &mut self.shared_modifiers
    }

    /// Number of tracked layouts (enabled or not).
    pub fn len(&self) -> usize {
        self.layouts.len()
    }

    /// True when no layouts are registered.
    pub fn is_empty(&self) -> bool {
        self.layouts.is_empty()
    }

    /// Add or replace a layout with the given priority.
    ///
    /// Re-adding a layout bumps its recency so it wins ties at the same priority.
    pub fn add_layout(&mut self, layout: Layout, priority: i32) {
        let id = layout.id().to_string();
        let order = self.next_order();
        if let Some(active) = self.layouts.iter_mut().find(|l| l.id() == id) {
            active.layout = layout;
            active.priority = priority;
            active.enabled = true;
            active.order = order;
        } else {
            self.layouts
                .push(ActiveLayout::new(layout, priority, order));
        }
        self.sort_layouts();
    }

    /// Remove a layout by id. Returns true if a layout was removed.
    pub fn remove_layout(&mut self, id: &str) -> bool {
        let before = self.layouts.len();
        self.layouts.retain(|layout| layout.id() != id);
        before != self.layouts.len()
    }

    /// Enable a layout. Returns false if the layout is unknown.
    ///
    /// Enabling bumps recency so the layout wins ties going forward.
    pub fn enable_layout(&mut self, id: &str) -> bool {
        self.set_layout_enabled(id, true)
    }

    /// Disable a layout. Returns false if the layout is unknown.
    pub fn disable_layout(&mut self, id: &str) -> bool {
        self.set_layout_enabled(id, false)
    }

    /// Update layout priority and reorder to respect recency on ties.
    pub fn set_priority(&mut self, id: &str, priority: i32) -> bool {
        if let Some(index) = self.layouts.iter().position(|l| l.id() == id) {
            let order = self.next_order();
            if let Some(layout) = self.layouts.get_mut(index) {
                layout.priority = priority;
                layout.order = order;
            }
            self.sort_layouts();
            true
        } else {
            false
        }
    }

    /// Borrow a layout by id.
    pub fn layout(&self, id: &str) -> Option<&Layout> {
        self.layouts
            .iter()
            .find(|l| l.id() == id)
            .map(|l| l.layout())
    }

    /// Mutably borrow a layout by id.
    pub fn layout_mut(&mut self, id: &str) -> Option<&mut Layout> {
        self.layouts
            .iter_mut()
            .find(|l| l.id() == id)
            .map(|l| l.layout_mut())
    }

    /// Iterator over layouts in resolution order (highest priority/most recent first).
    pub fn active_layouts(&self) -> impl Iterator<Item = &ActiveLayout> {
        self.layouts.iter().filter(|l| l.enabled)
    }

    /// Produce a snapshot of active layouts and their layer ids in resolution order.
    pub fn resolved_layouts(&self) -> Vec<ResolvedLayout> {
        self.active_layouts()
            .map(|layout| ResolvedLayout {
                id: layout.id().to_string(),
                priority: layout.priority,
                layer_ids: layout.layer_ids(),
            })
            .collect()
    }

    /// Clear all layouts and shared modifiers.
    pub fn clear(&mut self) {
        self.layouts.clear();
        self.shared_modifiers.clear();
        self.sequence = 0;
    }

    fn set_layout_enabled(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(index) = self.layouts.iter().position(|l| l.id() == id) {
            let order = enabled.then(|| self.next_order());
            if let Some(layout) = self.layouts.get_mut(index) {
                layout.enabled = enabled;
                if let Some(order) = order {
                    layout.order = order;
                }
            }
            if enabled {
                self.sort_layouts();
            }
            true
        } else {
            false
        }
    }

    fn next_order(&mut self) -> u64 {
        let current = self.sequence;
        self.sequence = self.sequence.saturating_add(1);
        current
    }

    fn sort_layouts(&mut self) {
        self.layouts
            .sort_by(|a, b| match b.priority.cmp(&a.priority) {
                Ordering::Equal => b.order.cmp(&a.order),
                other => other,
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::keycodes::KeyCode;
    use crate::engine::layout::LayoutMetadata;
    use crate::engine::state::{Layer, LayerAction};

    #[test]
    fn adds_and_orders_by_priority_and_recency() {
        let mut compositor = LayoutCompositor::new();
        compositor.add_layout(Layout::new(LayoutMetadata::new("low", "Low")), 1);
        compositor.add_layout(Layout::new(LayoutMetadata::new("high-1", "High 1")), 5);
        compositor.add_layout(Layout::new(LayoutMetadata::new("high-2", "High 2")), 5);

        let ids: Vec<_> = compositor
            .resolved_layouts()
            .into_iter()
            .map(|l| l.id)
            .collect();
        assert_eq!(ids, vec!["high-2", "high-1", "low"]);
    }

    #[test]
    fn priority_updates_and_recency_affect_order() {
        let mut compositor = LayoutCompositor::new();
        compositor.add_layout(Layout::new(LayoutMetadata::new("one", "One")), 1);
        compositor.add_layout(Layout::new(LayoutMetadata::new("two", "Two")), 1);

        let first_order: Vec<_> = compositor
            .resolved_layouts()
            .into_iter()
            .map(|l| l.id)
            .collect();
        assert_eq!(first_order, vec!["two", "one"]);

        assert!(compositor.set_priority("one", 1));
        let reordered: Vec<_> = compositor
            .resolved_layouts()
            .into_iter()
            .map(|l| l.id)
            .collect();
        assert_eq!(reordered, vec!["one", "two"]);
    }

    #[test]
    fn enable_and_disable_layouts_without_removal() {
        let mut compositor = LayoutCompositor::new();
        compositor.add_layout(Layout::new(LayoutMetadata::new("primary", "Primary")), 2);
        compositor.add_layout(
            Layout::new(LayoutMetadata::new("secondary", "Secondary")),
            1,
        );

        assert!(compositor.disable_layout("primary"));
        let ids: Vec<_> = compositor
            .resolved_layouts()
            .into_iter()
            .map(|l| l.id)
            .collect();
        assert_eq!(ids, vec!["secondary"]);

        assert!(compositor.enable_layout("primary"));
        let ids_after_enable: Vec<_> = compositor
            .resolved_layouts()
            .into_iter()
            .map(|l| l.id)
            .collect();
        assert_eq!(ids_after_enable, vec!["primary", "secondary"]);
    }

    #[test]
    fn resolved_layouts_surface_layer_ids() {
        let mut compositor = LayoutCompositor::new();

        let mut base = Layout::new(LayoutMetadata::new("base", "Base"));
        let mut overlay = Layer::with_id(1, "overlay");
        overlay.set_mapping(KeyCode::A, LayerAction::Remap(KeyCode::B));
        base.define_layer(overlay);
        base.layers_mut().push(1);

        let mut fn_layout = Layout::new(LayoutMetadata::new("fn", "Fn"));
        let mut fn_layer = Layer::with_id(2, "fn");
        fn_layer.transparent = true;
        fn_layout.define_layer(fn_layer);
        fn_layout.layers_mut().push(2);

        compositor.add_layout(base, 1);
        compositor.add_layout(fn_layout, 3);

        let resolved = compositor.resolved_layouts();
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].id, "fn");
        assert_eq!(resolved[0].layer_ids, vec![0, 2]);
        assert_eq!(resolved[1].id, "base");
        assert_eq!(resolved[1].layer_ids, vec![0, 1]);
    }
}
