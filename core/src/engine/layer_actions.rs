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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{HoldAction, KeyCode, Layer};

    fn key_down(key: KeyCode, ts: u64) -> InputEvent {
        InputEvent::key_down(key, ts)
    }

    fn key_up(key: KeyCode, ts: u64) -> InputEvent {
        InputEvent::key_up(key, ts)
    }

    #[test]
    fn apply_remap_with_key_down() {
        let event = key_down(KeyCode::A, 0);
        let action = LayerAction::Remap(KeyCode::B);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::with_event(&event);

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::KeyDown(KeyCode::B)]);
        assert!(result.consumed);
    }

    #[test]
    fn apply_remap_with_key_up() {
        let event = key_up(KeyCode::A, 100);
        let action = LayerAction::Remap(KeyCode::B);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::with_event(&event);

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::KeyUp(KeyCode::B)]);
        assert!(result.consumed);
    }

    #[test]
    fn apply_remap_without_event_emits_tap() {
        let action = LayerAction::Remap(KeyCode::B);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::without_event();

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(
            result.outputs,
            vec![
                OutputAction::KeyDown(KeyCode::B),
                OutputAction::KeyUp(KeyCode::B)
            ]
        );
        assert!(result.consumed);
    }

    #[test]
    fn apply_block_action() {
        let event = key_down(KeyCode::A, 0);
        let action = LayerAction::Block;
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::with_event(&event);

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(result.consumed);
    }

    #[test]
    fn apply_tap_hold_with_event_blocks() {
        let event = key_down(KeyCode::A, 0);
        let action = LayerAction::TapHold {
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        };
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::with_event(&event);

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(result.consumed);
    }

    #[test]
    fn apply_tap_hold_without_event_empty() {
        let action = LayerAction::TapHold {
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        };
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::without_event();

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert!(result.outputs.is_empty());
        assert!(result.consumed);
    }

    #[test]
    fn apply_layer_push_action() {
        let event = key_down(KeyCode::A, 0);
        let action = LayerAction::LayerPush(2);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        layers.define_layer(Layer::with_id(2, "layer2"));
        let ctx = LayerActionContext::with_event(&event);

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(result.consumed);
        assert!(layers.is_active(2));
    }

    #[test]
    fn apply_layer_pop_action() {
        let event = key_down(KeyCode::A, 0);
        let action = LayerAction::LayerPop;
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        layers.define_layer(Layer::with_id(1, "layer1"));
        layers.push(1);
        let ctx = LayerActionContext::with_event(&event);

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(result.consumed);
        assert!(!layers.is_active(1));
    }

    #[test]
    fn apply_layer_toggle_action() {
        let action = LayerAction::LayerToggle(1);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        layers.define_layer(Layer::with_id(1, "layer1"));
        let ctx = LayerActionContext::without_event();

        // First toggle: activates
        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(layers.is_active(1));

        // Second toggle: deactivates
        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(!layers.is_active(1));
    }

    #[test]
    fn apply_modifier_activate_action() {
        let event = key_down(KeyCode::A, 0);
        let action = LayerAction::ModifierActivate(1);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::with_event(&event);

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(result.consumed);
        assert!(modifiers.is_active(Modifier::Virtual(1)));
    }

    #[test]
    fn apply_modifier_deactivate_action() {
        let event = key_down(KeyCode::A, 0);
        let action = LayerAction::ModifierDeactivate(1);
        let mut modifiers = ModifierState::new();
        modifiers.activate(Modifier::Virtual(1));
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::with_event(&event);

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(!modifiers.is_active(Modifier::Virtual(1)));
    }

    #[test]
    fn apply_modifier_one_shot_action() {
        let event = key_down(KeyCode::A, 0);
        let action = LayerAction::ModifierOneShot(1);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::with_event(&event);

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(result.consumed);
    }

    #[test]
    fn apply_pass_action() {
        let event = key_down(KeyCode::A, 0);
        let action = LayerAction::Pass;
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::with_event(&event);

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::PassThrough]);
        assert!(!result.consumed);
    }

    // Additional edge case tests

    #[test]
    fn layer_action_context_construction() {
        let event = key_down(KeyCode::A, 0);

        let ctx_with = LayerActionContext::with_event(&event);
        assert!(ctx_with.event.is_some());

        let ctx_without = LayerActionContext::without_event();
        assert!(ctx_without.event.is_none());
    }

    #[test]
    fn layer_action_result_helpers() {
        let block_result = LayerActionResult::block();
        assert_eq!(block_result.outputs, vec![OutputAction::Block]);
        assert!(block_result.consumed);

        let pass_result = LayerActionResult::pass();
        assert_eq!(pass_result.outputs, vec![OutputAction::PassThrough]);
        assert!(!pass_result.consumed);
    }

    #[test]
    fn apply_remap_key_up_without_event_still_taps() {
        // When no event context, remap always produces a tap
        let action = LayerAction::Remap(KeyCode::Space);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::without_event();

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(
            result.outputs,
            vec![
                OutputAction::KeyDown(KeyCode::Space),
                OutputAction::KeyUp(KeyCode::Space)
            ]
        );
    }

    #[test]
    fn apply_layer_push_idempotent() {
        let action = LayerAction::LayerPush(3);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        layers.define_layer(Layer::with_id(3, "layer3"));
        let ctx = LayerActionContext::without_event();

        // First push
        let _ = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert!(layers.is_active(3));

        // Second push - layer still active
        let _ = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert!(layers.is_active(3));
    }

    #[test]
    fn apply_modifier_activate_twice_idempotent() {
        let action = LayerAction::ModifierActivate(5);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::without_event();

        // First activation
        let _ = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert!(modifiers.is_active(Modifier::Virtual(5)));

        // Second activation - still active
        let _ = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert!(modifiers.is_active(Modifier::Virtual(5)));
    }

    #[test]
    fn apply_modifier_deactivate_when_not_active() {
        let action = LayerAction::ModifierDeactivate(7);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::without_event();

        // Deactivate when not active - should not panic
        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(!modifiers.is_active(Modifier::Virtual(7)));
    }

    #[test]
    fn apply_layer_pop_when_empty() {
        let action = LayerAction::LayerPop;
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::without_event();

        // Pop when empty - should not panic
        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(result.consumed);
    }

    #[test]
    fn apply_block_without_event_context() {
        let action = LayerAction::Block;
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::without_event();

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        assert!(result.consumed);
    }

    #[test]
    fn apply_pass_without_event_context() {
        let action = LayerAction::Pass;
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let ctx = LayerActionContext::without_event();

        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::PassThrough]);
        assert!(!result.consumed);
    }

    #[test]
    fn apply_layer_toggle_undefined_layer() {
        let action = LayerAction::LayerToggle(99);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        // Don't define layer 99
        let ctx = LayerActionContext::without_event();

        // Should not panic, just not activate
        let result = apply_layer_action(&action, ctx, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::Block]);
        // Layer 99 was never defined so toggle won't activate it
        assert!(!layers.is_active(99));
    }
}
