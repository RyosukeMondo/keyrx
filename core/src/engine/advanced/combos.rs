//! Combo detection and resolution logic for the advanced engine.
//!
//! This module handles:
//! - Combo enqueuing when keys are pressed
//! - Immediate combo triggering when pressed keys match
//! - Combo timeout resolution

use super::{AdvancedEngine, KeyStateView};
use crate::engine::decision_engine::{self, activate_hold_action, process_resolutions};
use crate::engine::{ComboDef, DecisionResolution, InputEvent, LayerAction, OutputAction};
use crate::traits::ScriptRuntime;

impl<S> AdvancedEngine<S>
where
    S: ScriptRuntime,
{
    /// Enqueue combos that include the pressed key.
    ///
    /// Returns a tuple of:
    /// - `blocked`: Whether any events should be blocked while waiting for combo completion
    /// - `outputs`: Any immediate outputs from combo triggers
    pub(crate) fn enqueue_combos(&mut self, event: &InputEvent) -> (bool, Vec<OutputAction>) {
        // Get currently pressed keys from unified state
        let pressed_keys: Vec<_> = self.state.pressed_keys().collect();
        let mut blocked = false;
        let mut outputs = Vec::new();

        for ComboDef { keys, action } in self.combos.all() {
            if keys.contains(&event.key)
                && self
                    .pending
                    .add_combo(&keys, event.timestamp_us, action.clone())
            {
                blocked = true;
            }
        }

        // If current pressed keys already match a combo, trigger immediately.
        if let Some(action) = self.combos.find(&pressed_keys) {
            self.pending.clear();
            let immediate = self.execute_layer_action_with_event(action.clone(), Some(event));
            if !immediate.is_empty() {
                blocked = true;
                outputs.extend(immediate);
            }
        }

        (blocked, outputs)
    }

    /// Handle decision resolutions from pending queue.
    ///
    /// Processes resolutions from combo timeouts, tap-hold decisions, etc.
    /// Returns:
    /// - `outputs`: Actions to emit
    /// - `consumed`: Whether the event was consumed
    /// - `skip_layer_actions`: Whether to skip layer action lookup
    pub(crate) fn handle_resolutions(
        &mut self,
        resolutions: Vec<DecisionResolution>,
        event: Option<&InputEvent>,
    ) -> (Vec<OutputAction>, bool, bool) {
        // Use the unified state's key tracking via the view adapter
        let result = process_resolutions(resolutions, &KeyStateView(&self.state));
        let mut outputs = result.outputs;

        // Apply state changes
        for key in result.block_releases {
            self.blocked_releases.insert(key);
        }
        for key in result.unblock_releases {
            self.blocked_releases.remove(&key);
        }

        // Activate hold actions
        for (_, action) in &result.hold_activations {
            let (modifiers, layers) = self.modifier_and_default_layers();
            let hold_outputs = activate_hold_action(action, modifiers, layers);
            outputs.extend(hold_outputs);
        }

        // Execute layer actions (from combo triggers)
        for action in result.layer_actions {
            outputs.extend(self.execute_layer_action_with_event(action, event));
        }

        (outputs, result.consumed, result.skip_layer_actions)
    }

    /// Execute a layer action with optional event context.
    ///
    /// Handles TapHold actions specially by enqueuing them in the decision queue.
    /// Other actions are executed immediately via decision_engine.
    pub(crate) fn execute_layer_action_with_event(
        &mut self,
        action: LayerAction,
        event: Option<&InputEvent>,
    ) -> Vec<OutputAction> {
        // TapHold from combo needs special handling with DecisionQueue
        if let LayerAction::TapHold { tap, hold } = &action {
            if let Some(evt) = event {
                let (_, eager) =
                    self.pending
                        .add_tap_hold(evt.key, evt.timestamp_us, *tap, hold.clone());
                return eager
                    .map(|res| self.handle_resolutions(vec![res], Some(evt)).0)
                    .unwrap_or_default();
            }
            return Vec::new();
        }

        // Use decision_engine helper for other actions
        let (modifiers, layers) = self.modifier_and_default_layers();
        decision_engine::execute_layer_action(&action, modifiers, layers)
    }
}
