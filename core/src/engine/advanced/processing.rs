//! Event processing and decision resolution for the advanced engine.
//!
//! This module handles:
//! - Event processing pipeline (validation, state updates, decisions)
//! - Device passthrough logic
//! - State transition validation
//! - Layer action handling

use super::{AdvancedEngine, KeyStateView};
use crate::engine::decision_engine::{decision_type_from_action, handle_layer_action};
use crate::engine::processing::{
    apply_decision, trace_event, validate_and_check_safe_mode, DecisionResult,
};
use crate::engine::transitions::StateTransition;
use crate::engine::{
    DecisionType, EngineTracer, InputEvent, LayerAction, OutputAction, ProcessingResult,
};
use crate::traits::ScriptRuntime;

impl<S> AdvancedEngine<S>
where
    S: ScriptRuntime,
{
    // ... (should_passthrough_device, validate_transition, update_state_kind remain unchanged) ...

    /// Process a single event with optional tracing.
    ///
    /// When a tracer is provided, spans are emitted for input reception,
    /// decision making, and output generation. This enables detailed
    /// performance analysis and debugging via OpenTelemetry.
    pub fn process_event_traced(
        &mut self,
        event: InputEvent,
        tracer: Option<&EngineTracer>,
    ) -> ProcessingResult {
        let start_time = std::time::Instant::now();
        #[cfg(feature = "otel-tracing")]
        let event_span = tracing::info_span!(
            "engine.process_event",
            key = ?event.key,
            pressed = event.pressed,
            timestamp_us = event.timestamp_us,
            device_id = event.device_id.as_deref().unwrap_or(""),
            vendor_id = event.vendor_id.unwrap_or_default(),
            product_id = event.product_id.unwrap_or_default(),
            is_repeat = event.is_repeat,
            is_synthetic = event.is_synthetic,
            scan_code = event.scan_code as u64,
            serial = event
                .serial_number
                .as_deref()
                .unwrap_or(""),
        );
        #[cfg(feature = "otel-tracing")]
        let _event_span_guard = event_span.enter();

        let active_layers_before = self.default_layers().active_layer_ids();

        // Step 0: Revolutionary mapping pipeline - device resolution and passthrough
        // Check if this device has remapping disabled
        if let Some(should_pass) = self.should_passthrough_device(&event) {
            if should_pass {
                // Device has remap_enabled=false, pass through immediately
                tracing::debug!(
                    service = "keyrx",
                    event = "device_passthrough",
                    component = "advanced_engine",
                    vendor_id = event.vendor_id,
                    product_id = event.product_id,
                    "Device remapping disabled, passing through event"
                );
                return ProcessingResult {
                    outputs: vec![OutputAction::PassThrough],
                    decision_type: DecisionType::PassThrough,
                    active_layers: active_layers_before,
                    latency_us: start_time.elapsed().as_micros() as u64,
                };
            }
            // Device has remap_enabled=true, continue with normal processing
        }
        // If device not found or registry not configured, continue with normal processing

        // Step 1: Validate the transition through the state graph
        let transition = if event.pressed {
            StateTransition::KeyPressed {
                key: event.key,
                timestamp: event.timestamp_us,
            }
        } else {
            StateTransition::KeyReleased {
                key: event.key,
                timestamp: event.timestamp_us,
            }
        };

        // Validate the transition (log errors but continue processing for compatibility)
        if let Err(_e) = self.validate_transition(transition) {
            #[cfg(debug_assertions)]
            tracing::warn!("State transition validation warning: {}", _e);
        }

        // Step 2: Update key state using the unified state's mutation API.
        let key_mutation = if event.pressed {
            crate::engine::state::Mutation::KeyDown {
                key: event.key,
                timestamp_us: event.timestamp_us,
                is_repeat: event.is_repeat,
            }
        } else {
            crate::engine::state::Mutation::KeyUp {
                key: event.key,
                timestamp_us: event.timestamp_us,
            }
        };

        // Apply the key state mutation (ignore errors for keys already pressed/not pressed)
        let _ = self.state.apply(key_mutation);

        // Step 2: Validate and check safe mode using the unified state
        let validation =
            validate_and_check_safe_mode(&event, &KeyStateView(&self.state), self.safe_mode);

        if validation.safe_mode_toggled {
            self.state.pending_mut().clear();
            self.safe_mode = !self.safe_mode;
        }
        if validation.early_return {
            return ProcessingResult {
                outputs: validation.early_output,
                decision_type: DecisionType::PassThrough,
                active_layers: active_layers_before,
                latency_us: start_time.elapsed().as_micros() as u64,
            };
        }

        // Step 3: Resolve decisions (combos, pending, layers).
        let decision = self.resolve_decision(&event);

        // Step 4: Apply decision outcomes.
        let result = apply_decision(&event, &mut self.blocked_releases, decision);

        // Step 5: Update state kind based on actual state after processing
        self.update_state_kind();

        // Get active layers after processing (though typically decisions depend on layers before)
        // For tracing, we usually want layers active during decision.
        // Using `active_layers_before` seems appropriate for decision context.

        // Step 6: Emit tracing spans.
        trace_event(
            tracer,
            &event,
            result.decision_type,
            start_time,
            &active_layers_before,
            &result.outputs,
        );

        ProcessingResult {
            outputs: result.outputs,
            decision_type: result.decision_type,
            active_layers: active_layers_before,
            latency_us: start_time.elapsed().as_micros() as u64,
        }
    }
    // ...

    /// Resolve all decisions for an event (combos, pending, layers).
    pub(crate) fn resolve_decision(&mut self, event: &InputEvent) -> DecisionResult {
        // Mark other tap-hold decisions as interrupted when another key is pressed.
        if event.pressed {
            self.state.pending_mut().mark_interrupted(event.key);
        }

        let mut result = DecisionResult::default();

        // Combo tracking + pending resolutions.
        let (blocked_for_combo, combo_outputs) = if event.pressed {
            self.enqueue_combos(event)
        } else {
            (false, Vec::new())
        };
        if !combo_outputs.is_empty() {
            result.consumed = true;
            result.decision_type = DecisionType::Combo;
            result.outputs.extend(combo_outputs);
        }
        result.blocked_for_combo = blocked_for_combo;

        let resolutions = self.state.pending_mut().resolve_on_event(event);
        let (resolved_outputs, resolved_consumed, skip_layer_actions) =
            self.handle_resolutions(resolutions, Some(event));
        result.outputs.extend(resolved_outputs);
        result.consumed |= resolved_consumed;
        result.skip_layer_actions = skip_layer_actions;

        // Layer lookup and action execution using the default layout
        if !result.skip_layer_actions {
            if let Some(action) = self.default_layers().lookup(event.key).cloned() {
                let (handled_outputs, handled) = self.handle_layer_action(event, action.clone());
                result.outputs.extend(handled_outputs);
                if handled {
                    result.consumed = true;
                    result.decision_type = decision_type_from_action(&action);
                }
            }
        }

        result
    }

    /// Handle a layer action for an event.
    ///
    /// TapHold actions are special-cased to use the decision queue.
    /// Other actions are handled via decision_engine.
    pub(crate) fn handle_layer_action(
        &mut self,
        event: &InputEvent,
        action: LayerAction,
    ) -> (Vec<OutputAction>, bool) {
        // TapHold requires special handling with DecisionQueue
        if let LayerAction::TapHold { tap, hold } = &action {
            if event.pressed {
                let (_, eager) = self.state.pending_mut().add_tap_hold(
                    event.key,
                    event.timestamp_us,
                    *tap,
                    hold.clone(),
                );
                let mut outputs = Vec::new();
                if let Some(resolution) = eager {
                    let (eager_outputs, _, _) =
                        self.handle_resolutions(vec![resolution], Some(event));
                    outputs.extend(eager_outputs);
                }
                outputs.push(OutputAction::Block);
                return (outputs, true);
            } else {
                return (vec![OutputAction::Block], true);
            }
        }

        // Use the decision_engine helper for other actions
        let (modifiers, layers) = self.modifier_and_default_layers();
        let result = handle_layer_action(event, &action, modifiers, layers);
        (result.outputs, result.consumed)
    }
}
