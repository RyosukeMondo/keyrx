//! Builder pattern for creating PendingOp test instances.
//!
//! This fixture provides a fluent builder API to reduce boilerplate when
//! constructing PendingOp instances in tests.
//!
//! ## Usage
//!
//! ```rust
//! use fixtures::operations::OperationBuilder;
//! use keyrx_core::engine::KeyCode;
//!
//! // Simple remap
//! let op = OperationBuilder::new()
//!     .remap(KeyCode::A, KeyCode::B)
//!     .build();
//!
//! // Block a key
//! let op = OperationBuilder::new()
//!     .block(KeyCode::Escape)
//!     .build();
//!
//! // Combo
//! let op = OperationBuilder::new()
//!     .combo(vec![KeyCode::LeftCtrl, KeyCode::C], LayerAction::Pass)
//!     .build();
//! ```

use keyrx_core::engine::{HoldAction, KeyCode, LayerAction};
use keyrx_core::scripting::builtins::{LayerMapAction, PendingOp, TimingUpdate};

/// Builder for creating PendingOp instances in tests.
///
/// This builder provides a fluent API for constructing various types of
/// pending operations without repetitive boilerplate.
#[derive(Debug)]
pub struct OperationBuilder {
    op: Option<PendingOp>,
}

impl OperationBuilder {
    /// Create a new OperationBuilder.
    pub fn new() -> Self {
        Self { op: None }
    }

    /// Create a remap operation.
    pub fn remap(mut self, from: KeyCode, to: KeyCode) -> Self {
        self.op = Some(PendingOp::Remap { from, to });
        self
    }

    /// Create a block operation.
    pub fn block(mut self, key: KeyCode) -> Self {
        self.op = Some(PendingOp::Block { key });
        self
    }

    /// Create a pass operation.
    pub fn pass(mut self, key: KeyCode) -> Self {
        self.op = Some(PendingOp::Pass { key });
        self
    }

    /// Create a tap-hold operation.
    pub fn tap_hold(mut self, key: KeyCode, tap: KeyCode, hold: HoldAction) -> Self {
        self.op = Some(PendingOp::TapHold { key, tap, hold });
        self
    }

    /// Create a combo operation.
    pub fn combo(mut self, keys: Vec<KeyCode>, action: LayerAction) -> Self {
        self.op = Some(PendingOp::Combo { keys, action });
        self
    }

    /// Create a layer definition operation.
    pub fn layer_define(mut self, name: impl Into<String>, transparent: bool) -> Self {
        self.op = Some(PendingOp::LayerDefine {
            name: name.into(),
            transparent,
        });
        self
    }

    /// Create a layer mapping operation.
    pub fn layer_map(mut self, layer: impl Into<String>, key: KeyCode, action: LayerMapAction) -> Self {
        self.op = Some(PendingOp::LayerMap {
            layer: layer.into(),
            key,
            action,
        });
        self
    }

    /// Create a layer push operation.
    pub fn layer_push(mut self, name: impl Into<String>) -> Self {
        self.op = Some(PendingOp::LayerPush { name: name.into() });
        self
    }

    /// Create a layer toggle operation.
    pub fn layer_toggle(mut self, name: impl Into<String>) -> Self {
        self.op = Some(PendingOp::LayerToggle { name: name.into() });
        self
    }

    /// Create a layer pop operation.
    pub fn layer_pop(mut self) -> Self {
        self.op = Some(PendingOp::LayerPop);
        self
    }

    /// Create a define modifier operation.
    pub fn define_modifier(mut self, name: impl Into<String>, id: u8) -> Self {
        self.op = Some(PendingOp::DefineModifier {
            name: name.into(),
            id,
        });
        self
    }

    /// Create a modifier activate operation.
    pub fn modifier_activate(mut self, name: impl Into<String>, id: u8) -> Self {
        self.op = Some(PendingOp::ModifierActivate {
            name: name.into(),
            id,
        });
        self
    }

    /// Create a modifier deactivate operation.
    pub fn modifier_deactivate(mut self, name: impl Into<String>, id: u8) -> Self {
        self.op = Some(PendingOp::ModifierDeactivate {
            name: name.into(),
            id,
        });
        self
    }

    /// Create a modifier one-shot operation.
    pub fn modifier_one_shot(mut self, name: impl Into<String>, id: u8) -> Self {
        self.op = Some(PendingOp::ModifierOneShot {
            name: name.into(),
            id,
        });
        self
    }

    /// Create a set timing operation.
    pub fn set_timing(mut self, update: TimingUpdate) -> Self {
        self.op = Some(PendingOp::SetTiming(update));
        self
    }

    /// Build the PendingOp.
    ///
    /// # Panics
    ///
    /// Panics if no operation was configured before calling build().
    /// This is intentional for test code - tests should fail fast if
    /// the builder is used incorrectly.
    pub fn build(self) -> PendingOp {
        self.op.expect("OperationBuilder: must call a builder method (remap, block, etc.) before build()")
    }
}

impl Default for OperationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remap_builder() {
        let op = OperationBuilder::new()
            .remap(KeyCode::A, KeyCode::B)
            .build();

        match op {
            PendingOp::Remap { from, to } => {
                assert_eq!(from, KeyCode::A);
                assert_eq!(to, KeyCode::B);
            }
            _ => panic!("Expected Remap operation"),
        }
    }

    #[test]
    fn test_block_builder() {
        let op = OperationBuilder::new().block(KeyCode::Escape).build();

        match op {
            PendingOp::Block { key } => {
                assert_eq!(key, KeyCode::Escape);
            }
            _ => panic!("Expected Block operation"),
        }
    }

    #[test]
    fn test_pass_builder() {
        let op = OperationBuilder::new().pass(KeyCode::Space).build();

        match op {
            PendingOp::Pass { key } => {
                assert_eq!(key, KeyCode::Space);
            }
            _ => panic!("Expected Pass operation"),
        }
    }

    #[test]
    fn test_combo_builder() {
        let keys = vec![KeyCode::LeftCtrl, KeyCode::C];
        let action = LayerAction::Pass;
        let op = OperationBuilder::new().combo(keys.clone(), action).build();

        match op {
            PendingOp::Combo { keys: k, action: a } => {
                assert_eq!(k, keys);
                assert!(matches!(a, LayerAction::Pass));
            }
            _ => panic!("Expected Combo operation"),
        }
    }

    #[test]
    fn test_layer_define_builder() {
        let op = OperationBuilder::new()
            .layer_define("custom", true)
            .build();

        match op {
            PendingOp::LayerDefine { name, transparent } => {
                assert_eq!(name, "custom");
                assert!(transparent);
            }
            _ => panic!("Expected LayerDefine operation"),
        }
    }

    #[test]
    fn test_layer_push_builder() {
        let op = OperationBuilder::new().layer_push("nav").build();

        match op {
            PendingOp::LayerPush { name } => {
                assert_eq!(name, "nav");
            }
            _ => panic!("Expected LayerPush operation"),
        }
    }

    #[test]
    fn test_layer_pop_builder() {
        let op = OperationBuilder::new().layer_pop().build();

        assert!(matches!(op, PendingOp::LayerPop));
    }

    #[test]
    fn test_define_modifier_builder() {
        let op = OperationBuilder::new()
            .define_modifier("super", 1)
            .build();

        match op {
            PendingOp::DefineModifier { name, id } => {
                assert_eq!(name, "super");
                assert_eq!(id, 1);
            }
            _ => panic!("Expected DefineModifier operation"),
        }
    }

    #[test]
    #[should_panic(expected = "must call a builder method")]
    fn test_build_without_operation() {
        OperationBuilder::new().build();
    }
}
