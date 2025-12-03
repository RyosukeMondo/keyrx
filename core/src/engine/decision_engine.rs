//! Decision engine algorithms for tap-hold, combo, and layer action processing.
//!
//! This module contains the core decision-making logic extracted from the
//! advanced engine, providing pure functions for handling timing-based decisions.

use crate::engine::{
    layer_actions::{self, LayerActionContext},
    DecisionResolution, DecisionType, HoldAction, InputEvent, KeyCode, KeyStateTracker,
    LayerAction, LayerStack, Modifier, ModifierState, OutputAction,
};

/// Result of handling a layer action.
#[derive(Debug)]
pub struct LayerActionResult {
    /// Output actions to emit.
    pub outputs: Vec<OutputAction>,
    /// Whether the action consumed the event.
    pub consumed: bool,
}

/// Determine the decision type from a layer action.
pub fn decision_type_from_action(action: &LayerAction) -> DecisionType {
    match action {
        LayerAction::Remap(_) => DecisionType::Remap,
        LayerAction::Block => DecisionType::Block,
        LayerAction::TapHold { .. } => DecisionType::Tap, // Will resolve to Tap or Hold later
        LayerAction::LayerPush(_) | LayerAction::LayerPop | LayerAction::LayerToggle(_) => {
            DecisionType::Layer
        }
        LayerAction::ModifierActivate(_)
        | LayerAction::ModifierDeactivate(_)
        | LayerAction::ModifierOneShot(_) => DecisionType::Block, // Modifier actions block input
        LayerAction::Pass => DecisionType::PassThrough,
    }
}

/// Check if an event triggers safe mode (Ctrl+Alt+Shift+Escape).
pub fn check_safe_mode_toggle(event: &InputEvent, key_state: &KeyStateTracker) -> bool {
    if !event.pressed || event.key != KeyCode::Escape {
        return false;
    }

    let has_ctrl =
        key_state.is_pressed(KeyCode::LeftCtrl) || key_state.is_pressed(KeyCode::RightCtrl);
    let has_alt = key_state.is_pressed(KeyCode::LeftAlt) || key_state.is_pressed(KeyCode::RightAlt);
    let has_shift =
        key_state.is_pressed(KeyCode::LeftShift) || key_state.is_pressed(KeyCode::RightShift);

    has_ctrl && has_alt && has_shift
}

/// Create a pass-through output action from an event.
pub fn pass_through_event(event: &InputEvent) -> OutputAction {
    if event.pressed {
        OutputAction::KeyDown(event.key)
    } else {
        OutputAction::KeyUp(event.key)
    }
}

/// Result of processing decision resolutions.
#[derive(Debug, Default)]
pub struct ResolutionResult {
    /// Output actions to emit.
    pub outputs: Vec<OutputAction>,
    /// Whether events were consumed.
    pub consumed: bool,
    /// Whether to skip layer action processing.
    pub skip_layer_actions: bool,
    /// Keys to add to blocked releases.
    pub block_releases: Vec<KeyCode>,
    /// Keys to remove from blocked releases.
    pub unblock_releases: Vec<KeyCode>,
    /// Hold actions to activate (key, action).
    pub hold_activations: Vec<(KeyCode, HoldAction)>,
    /// Layer actions to execute.
    pub layer_actions: Vec<LayerAction>,
}

/// Process decision resolutions into output actions and state changes.
///
/// This function is mostly pure - it returns the state changes needed rather than
/// mutating state directly. The caller applies the state changes.
pub fn process_resolutions(
    resolutions: Vec<DecisionResolution>,
    key_state: &KeyStateTracker,
) -> ResolutionResult {
    let mut result = ResolutionResult::default();

    for resolution in resolutions {
        match resolution {
            DecisionResolution::Tap { key, .. } => {
                result.outputs.push(OutputAction::KeyDown(key));
                result.outputs.push(OutputAction::KeyUp(key));
                result.consumed = true;
            }
            DecisionResolution::Hold { key, action, .. } => {
                result.block_releases.push(key);
                result.hold_activations.push((key, action));
                result.consumed = true;
            }
            DecisionResolution::Consume(key) => {
                result.unblock_releases.push(key);
                result.outputs.push(OutputAction::Block);
                result.consumed = true;
                result.skip_layer_actions = true;
            }
            DecisionResolution::ComboTriggered(action) => {
                result.layer_actions.push(action);
                result.consumed = true;
            }
            DecisionResolution::ComboTimeout(keys) => {
                for key in keys {
                    if key_state.is_pressed(key) {
                        result.outputs.push(OutputAction::KeyDown(key));
                        result.consumed = true;
                    }
                }
            }
        }
    }

    result
}

/// Activate a hold action and return the outputs.
///
/// Returns the outputs and optionally state changes to apply.
pub fn activate_hold_action(
    action: &HoldAction,
    modifiers: &mut ModifierState,
    layers: &mut LayerStack,
) -> Vec<OutputAction> {
    match action {
        HoldAction::Key(key) => vec![OutputAction::KeyDown(*key)],
        HoldAction::Modifier(id) => {
            modifiers.activate(Modifier::Virtual(*id));
            vec![OutputAction::Block]
        }
        HoldAction::Layer(layer) => {
            layers.push(*layer);
            vec![OutputAction::Block]
        }
    }
}

/// Handle a layer action for a given event.
///
/// Returns the outputs and whether the event was consumed.
/// This is a thin wrapper around the unified `apply_layer_action`.
pub fn handle_layer_action(
    event: &InputEvent,
    action: &LayerAction,
    modifiers: &mut ModifierState,
    layers: &mut LayerStack,
) -> LayerActionResult {
    let ctx = LayerActionContext::with_event(event);
    let result = layer_actions::apply_layer_action(action, ctx, modifiers, layers);
    LayerActionResult {
        outputs: result.outputs,
        consumed: result.consumed,
    }
}

/// Execute a layer action without event context (for combos).
///
/// This is a thin wrapper around the unified `apply_layer_action`.
pub fn execute_layer_action(
    action: &LayerAction,
    modifiers: &mut ModifierState,
    layers: &mut LayerStack,
) -> Vec<OutputAction> {
    let ctx = LayerActionContext::without_event();
    layer_actions::apply_layer_action(action, ctx, modifiers, layers).outputs
}

/// Result of combo processing.
#[derive(Debug, Default)]
pub struct ComboResult {
    /// Whether keys should be blocked for pending combo.
    pub blocked: bool,
    /// Immediate outputs (if combo triggered).
    pub outputs: Vec<OutputAction>,
    /// Whether the pending queue should be cleared.
    pub clear_pending: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    fn key_down(key: KeyCode, ts: u64) -> InputEvent {
        InputEvent::key_down(key, ts)
    }

    fn key_up(key: KeyCode, ts: u64) -> InputEvent {
        InputEvent::key_up(key, ts)
    }

    #[test]
    fn decision_type_remap() {
        let action = LayerAction::Remap(KeyCode::B);
        assert_eq!(decision_type_from_action(&action), DecisionType::Remap);
    }

    #[test]
    fn decision_type_block() {
        let action = LayerAction::Block;
        assert_eq!(decision_type_from_action(&action), DecisionType::Block);
    }

    #[test]
    fn decision_type_tap_hold() {
        let action = LayerAction::TapHold {
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        };
        assert_eq!(decision_type_from_action(&action), DecisionType::Tap);
    }

    #[test]
    fn decision_type_layer_push() {
        let action = LayerAction::LayerPush(1);
        assert_eq!(decision_type_from_action(&action), DecisionType::Layer);
    }

    #[test]
    fn decision_type_pass() {
        let action = LayerAction::Pass;
        assert_eq!(
            decision_type_from_action(&action),
            DecisionType::PassThrough
        );
    }

    #[test]
    fn safe_mode_toggle_requires_all_modifiers() {
        let mut key_state = KeyStateTracker::new();
        let event = key_down(KeyCode::Escape, 0);

        // No modifiers - should not toggle
        assert!(!check_safe_mode_toggle(&event, &key_state));

        // Only Ctrl
        key_state.key_down(KeyCode::LeftCtrl, 0, false);
        assert!(!check_safe_mode_toggle(&event, &key_state));

        // Ctrl + Alt
        key_state.key_down(KeyCode::LeftAlt, 0, false);
        assert!(!check_safe_mode_toggle(&event, &key_state));

        // All three
        key_state.key_down(KeyCode::LeftShift, 0, false);
        assert!(check_safe_mode_toggle(&event, &key_state));
    }

    #[test]
    fn safe_mode_toggle_not_on_key_up() {
        let mut key_state = KeyStateTracker::new();
        key_state.key_down(KeyCode::LeftCtrl, 0, false);
        key_state.key_down(KeyCode::LeftAlt, 0, false);
        key_state.key_down(KeyCode::LeftShift, 0, false);

        let event = key_up(KeyCode::Escape, 0);
        assert!(!check_safe_mode_toggle(&event, &key_state));
    }

    #[test]
    fn pass_through_event_key_down() {
        let event = key_down(KeyCode::A, 0);
        assert_eq!(
            pass_through_event(&event),
            OutputAction::KeyDown(KeyCode::A)
        );
    }

    #[test]
    fn pass_through_event_key_up() {
        let event = key_up(KeyCode::A, 0);
        assert_eq!(pass_through_event(&event), OutputAction::KeyUp(KeyCode::A));
    }

    #[test]
    fn resolution_tap_emits_down_up() {
        let resolutions = vec![DecisionResolution::Tap {
            key: KeyCode::Escape,
            was_eager: false,
        }];
        let key_state = KeyStateTracker::new();

        let result = process_resolutions(resolutions, &key_state);
        assert_eq!(
            result.outputs,
            vec![
                OutputAction::KeyDown(KeyCode::Escape),
                OutputAction::KeyUp(KeyCode::Escape)
            ]
        );
        assert!(result.consumed);
    }

    #[test]
    fn resolution_hold_records_key_to_block() {
        let resolutions = vec![DecisionResolution::Hold {
            key: KeyCode::CapsLock,
            action: HoldAction::Key(KeyCode::LeftCtrl),
            from_eager: false,
        }];
        let key_state = KeyStateTracker::new();

        let result = process_resolutions(resolutions, &key_state);
        assert!(result.consumed);
        assert_eq!(result.block_releases, vec![KeyCode::CapsLock]);
        assert_eq!(result.hold_activations.len(), 1);
    }

    #[test]
    fn resolution_consume_sets_skip_layer_actions() {
        let resolutions = vec![DecisionResolution::Consume(KeyCode::A)];
        let key_state = KeyStateTracker::new();

        let result = process_resolutions(resolutions, &key_state);
        assert!(result.skip_layer_actions);
        assert!(result.consumed);
        assert_eq!(result.unblock_releases, vec![KeyCode::A]);
    }

    #[test]
    fn resolution_combo_timeout_emits_for_pressed_keys() {
        let mut key_state = KeyStateTracker::new();
        key_state.key_down(KeyCode::A, 0, false);

        let resolutions = vec![DecisionResolution::ComboTimeout(smallvec![
            KeyCode::A,
            KeyCode::B
        ])];

        let result = process_resolutions(resolutions, &key_state);
        // Only A is pressed, so only A should emit
        assert_eq!(result.outputs, vec![OutputAction::KeyDown(KeyCode::A)]);
        assert!(result.consumed);
    }

    #[test]
    fn activate_hold_key_action() {
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        let action = HoldAction::Key(KeyCode::LeftCtrl);

        let outputs = activate_hold_action(&action, &mut modifiers, &mut layers);
        assert_eq!(outputs, vec![OutputAction::KeyDown(KeyCode::LeftCtrl)]);
    }

    #[test]
    fn activate_hold_layer_action() {
        use crate::engine::Layer;

        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        // Define the layer first
        let layer2 = Layer::with_id(2, "layer2");
        layers.define_layer(layer2);

        let action = HoldAction::Layer(2);

        let outputs = activate_hold_action(&action, &mut modifiers, &mut layers);
        assert_eq!(outputs, vec![OutputAction::Block]);
        assert!(layers.is_active(2));
    }

    #[test]
    fn handle_layer_action_remap() {
        let event = key_down(KeyCode::A, 0);
        let action = LayerAction::Remap(KeyCode::B);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();

        let result = handle_layer_action(&event, &action, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::KeyDown(KeyCode::B)]);
        assert!(result.consumed);
    }

    #[test]
    fn handle_layer_action_remap_release() {
        let event = key_up(KeyCode::A, 100);
        let action = LayerAction::Remap(KeyCode::B);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();

        let result = handle_layer_action(&event, &action, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::KeyUp(KeyCode::B)]);
        assert!(result.consumed);
    }

    #[test]
    fn handle_layer_action_pass() {
        let event = key_down(KeyCode::A, 0);
        let action = LayerAction::Pass;
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();

        let result = handle_layer_action(&event, &action, &mut modifiers, &mut layers);
        assert_eq!(result.outputs, vec![OutputAction::PassThrough]);
        assert!(!result.consumed);
    }

    #[test]
    fn execute_layer_action_remap() {
        let action = LayerAction::Remap(KeyCode::B);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();

        let outputs = execute_layer_action(&action, &mut modifiers, &mut layers);
        assert_eq!(
            outputs,
            vec![
                OutputAction::KeyDown(KeyCode::B),
                OutputAction::KeyUp(KeyCode::B)
            ]
        );
    }

    #[test]
    fn execute_layer_action_layer_toggle() {
        use crate::engine::Layer;

        let action = LayerAction::LayerToggle(1);
        let mut modifiers = ModifierState::new();
        let mut layers = LayerStack::new();
        // Define the layer first
        let layer1 = Layer::with_id(1, "layer1");
        layers.define_layer(layer1);

        let outputs = execute_layer_action(&action, &mut modifiers, &mut layers);
        assert_eq!(outputs, vec![OutputAction::Block]);
        assert!(layers.is_active(1));
    }
}
