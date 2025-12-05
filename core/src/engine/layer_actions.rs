//! Unified layer action handling.
//!
//! This module provides a single implementation for layer actions, replacing
//! the duplicate logic that existed in `handle_layer_action` and `execute_layer_action`.

use crate::engine::{InputEvent, LayerAction, LayerStack, Modifier, ModifierState, OutputAction};

/// Context for layer action execution.
///
/// When an event is present, remaps follow the event press state.
/// When no event is present (e.g., combo triggers), remaps emit tap (down+up).
#[derive(Debug, Clone, Copy)]
pub struct LayerActionContext<'a> {
    /// Optional event that triggered the action.
    pub event: Option<&'a InputEvent>,
}

impl<'a> LayerActionContext<'a> {
    /// Create context with an event.
    pub fn with_event(event: &'a InputEvent) -> Self {
        Self { event: Some(event) }
    }

    /// Create context without an event (for combos).
    pub fn without_event() -> Self {
        Self { event: None }
    }
}

/// Result of applying a layer action.
#[derive(Debug)]
pub struct LayerActionResult {
    /// Output actions to emit.
    pub outputs: Vec<OutputAction>,
    /// Whether the action consumed the event.
    pub consumed: bool,
}

impl LayerActionResult {
    fn new(outputs: Vec<OutputAction>, consumed: bool) -> Self {
        Self { outputs, consumed }
    }

    fn block() -> Self {
        Self::new(vec![OutputAction::Block], true)
    }

    fn pass() -> Self {
        Self::new(vec![OutputAction::PassThrough], false)
    }
}

/// Apply a layer action with optional event context.
///
/// This is the single unified implementation for layer actions.
/// - With event context: Remaps follow event press state
/// - Without event context: Remaps emit tap (down+up)
pub fn apply_layer_action(
    action: &LayerAction,
    ctx: LayerActionContext<'_>,
    modifiers: &mut ModifierState,
    layers: &mut LayerStack,
) -> LayerActionResult {
    match action {
        LayerAction::Remap(target) => apply_remap(*target, ctx),
        LayerAction::Block => LayerActionResult::block(),
        LayerAction::TapHold { .. } => apply_tap_hold(ctx),
        LayerAction::LayerPush(id) => apply_layer_push(*id, layers),
        LayerAction::LayerPop => apply_layer_pop(layers),
        LayerAction::LayerToggle(id) => apply_layer_toggle(*id, layers),
        LayerAction::ModifierActivate(id) => apply_modifier_activate(*id, modifiers),
        LayerAction::ModifierDeactivate(id) => apply_modifier_deactivate(*id, modifiers),
        LayerAction::ModifierOneShot(id) => apply_modifier_one_shot(*id, modifiers),
        LayerAction::Pass => LayerActionResult::pass(),
    }
}

fn apply_remap(target: crate::engine::KeyCode, ctx: LayerActionContext<'_>) -> LayerActionResult {
    let outputs = match ctx.event {
        Some(event) => {
            if event.pressed {
                vec![OutputAction::KeyDown(target)]
            } else {
                vec![OutputAction::KeyUp(target)]
            }
        }
        None => vec![OutputAction::KeyDown(target), OutputAction::KeyUp(target)],
    };
    LayerActionResult::new(outputs, true)
}

fn apply_tap_hold(ctx: LayerActionContext<'_>) -> LayerActionResult {
    // TapHold requires DecisionQueue interaction - caller handles this
    // With event context: return Block (actual handling in advanced.rs)
    // Without event context: return empty (combos can't trigger TapHold properly)
    if ctx.event.is_some() {
        LayerActionResult::block()
    } else {
        LayerActionResult::new(Vec::new(), true)
    }
}

fn apply_layer_push(id: crate::engine::LayerId, layers: &mut LayerStack) -> LayerActionResult {
    layers.push(id);
    LayerActionResult::block()
}

fn apply_layer_pop(layers: &mut LayerStack) -> LayerActionResult {
    layers.pop();
    LayerActionResult::block()
}

fn apply_layer_toggle(id: crate::engine::LayerId, layers: &mut LayerStack) -> LayerActionResult {
    layers.toggle(id);
    LayerActionResult::block()
}

fn apply_modifier_activate(id: u8, modifiers: &mut ModifierState) -> LayerActionResult {
    modifiers.activate(Modifier::Virtual(id));
    LayerActionResult::block()
}

fn apply_modifier_deactivate(id: u8, modifiers: &mut ModifierState) -> LayerActionResult {
    modifiers.deactivate(Modifier::Virtual(id));
    LayerActionResult::block()
}

fn apply_modifier_one_shot(id: u8, modifiers: &mut ModifierState) -> LayerActionResult {
    modifiers.arm_one_shot(Modifier::Virtual(id));
    LayerActionResult::block()
}
