//! Unit tests for layer action application.

use keyrx_core::engine::{
    layer_actions::{apply_layer_action, LayerActionContext, LayerActionResult},
    state::{LayerStack, ModifierState},
    HoldAction, InputEvent, KeyCode, Layer, LayerAction, Modifier, OutputAction,
};

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
