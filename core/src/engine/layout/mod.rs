//! Layout abstraction that wraps a layer stack with metadata.
//!
//! This module decouples layout identity/metadata from the underlying
//! `LayerStack` so layouts can be composed and managed independently.

pub mod compositor;
pub mod modifiers;

use serde::{Deserialize, Serialize};

use crate::engine::state::{Layer, LayerId, LayerStack};
pub use compositor::{ActiveLayout, LayoutCompositor, ResolvedLayout};
pub use modifiers::{CrossLayoutModifier, ModifierCoordinator, ModifierScope};

/// Stable identifier for a layout.
pub type LayoutId = String;

/// Metadata describing a layout independent of its layers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutMetadata {
    /// Stable layout identifier (slug or UUID).
    pub id: LayoutId,
    /// Human-friendly name shown in UIs/logs.
    pub name: String,
    /// Optional description for diagnostics or UX.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Free-form tags for grouping/filtering.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

impl LayoutMetadata {
    /// Create metadata with required fields and empty optional fields.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            tags: Vec::new(),
        }
    }

    /// Add a tag if it's non-empty and not already present.
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !tag.is_empty() && !self.tags.iter().any(|existing| existing == &tag) {
            self.tags.push(tag);
        }
    }
}

/// A layout ties metadata to a stack of layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    metadata: LayoutMetadata,
    layers: LayerStack,
}

impl Layout {
    /// Create a new layout with default base layer.
    pub fn new(metadata: LayoutMetadata) -> Self {
        Self {
            metadata,
            layers: LayerStack::new(),
        }
    }

    /// Create a new layout with a custom base layer.
    pub fn with_base_layer(metadata: LayoutMetadata, base_layer: Layer) -> Self {
        Self {
            metadata,
            layers: LayerStack::with_base_layer(base_layer),
        }
    }

    /// Access layout metadata.
    pub fn metadata(&self) -> &LayoutMetadata {
        &self.metadata
    }

    /// Mutably access layout metadata.
    pub fn metadata_mut(&mut self) -> &mut LayoutMetadata {
        &mut self.metadata
    }

    /// Stable layout identifier.
    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    /// Human-friendly layout name.
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Get immutable access to the underlying layer stack.
    pub fn layers(&self) -> &LayerStack {
        &self.layers
    }

    /// Get mutable access to the underlying layer stack.
    pub fn layers_mut(&mut self) -> &mut LayerStack {
        &mut self.layers
    }

    /// Define or replace a layer inside this layout.
    pub fn define_layer(&mut self, layer: Layer) {
        self.layers.define_layer(layer);
    }

    /// Return the active layer names from base to top.
    pub fn active_layer_names(&self) -> Vec<String> {
        self.layers.active_layer_names()
    }

    /// Return the active layer identifiers in priority order.
    pub fn active_layer_ids(&self) -> Vec<LayerId> {
        self.layers
            .active_layer_ids()
            .into_iter()
            .map(|id| id as LayerId)
            .collect()
    }

    /// Base layer id for this layout.
    pub fn base_layer(&self) -> LayerId {
        self.layers.active_layer_ids().first().copied().unwrap_or(0) as LayerId
    }
}

#[cfg(test)]
mod tests {
    use super::{Layout, LayoutMetadata};
    use crate::drivers::keycodes::KeyCode;
    use crate::engine::state::{Layer, LayerAction};

    #[test]
    fn creates_layout_with_defaults() {
        let metadata = LayoutMetadata::new("coding", "Coding");
        let layout = Layout::new(metadata.clone());

        assert_eq!(layout.id(), "coding");
        assert_eq!(layout.name(), "Coding");
        assert_eq!(layout.metadata().description, None);
        assert_eq!(layout.active_layer_names(), vec!["base".to_string()]);
        assert_eq!(layout.base_layer(), 0);
        assert_eq!(layout.metadata(), &metadata);
    }

    #[test]
    fn defines_layers_and_tracks_activation() {
        let metadata = LayoutMetadata::new("gaming", "Gaming");
        let mut layout = Layout::new(metadata);

        let mut nav_layer = Layer::with_id(1, "nav");
        nav_layer.set_mapping(KeyCode::Unknown(1), LayerAction::LayerPush(2));
        layout.define_layer(nav_layer);

        layout.layers_mut().push(1);
        assert_eq!(layout.active_layer_names(), vec!["base", "nav"]);
        assert!(layout.layers().is_active(1));
    }

    #[test]
    fn mutates_metadata_tags() {
        let mut metadata = LayoutMetadata::new("studio", "Studio");
        metadata.add_tag("live");
        metadata.add_tag("mixing");
        metadata.add_tag("live"); // dedupe

        let layout = Layout::new(metadata.clone());

        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(layout.metadata().tags, metadata.tags);
    }
}
