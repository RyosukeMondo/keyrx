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
use crate::engine::transitions::log::TransitionEntry;
use crate::engine::transitions::StateTransition;
use crate::engine::{DecisionType, EngineTracer, InputEvent, LayerAction, OutputAction};
use crate::identity::DeviceIdentity;
use crate::traits::ScriptRuntime;

impl<S> AdvancedEngine<S>
where
    S: ScriptRuntime,
{
    /// Check if an event should be processed or passed through based on device state.
    ///
    /// This method implements the revolutionary mapping pipeline's first stage:
    /// device resolution and per-device passthrough mode.
    ///
    /// Returns `Some(true)` if the event should be passed through (remap disabled).
    /// Returns `Some(false)` if the event should be processed (remap enabled).
    /// Returns `None` if device registry is not configured or device not found.
    pub(crate) fn should_passthrough_device(&self, event: &InputEvent) -> Option<bool> {
        let registry = self.device_registry.as_ref()?;

        // Extract device identity from event
        let vendor_id = event.vendor_id?;
        let product_id = event.product_id?;
        let serial_number = event.serial_number.as_ref()?;

        let identity = DeviceIdentity::new(vendor_id, product_id, serial_number.clone());

        // Try to get device state with a non-blocking read
        let device_state = registry.try_get_device_state(&identity)?;

        // Return true if remap is disabled (passthrough), false if enabled (process)
        Some(!device_state.remap_enabled)
    }

    /// Validate and apply a state transition.
    ///
    /// This checks if the transition is valid from the current state,
    /// applies it through the state graph, and updates the current state kind.
    /// When transition logging is enabled, this also captures before/after
    /// state snapshots and records the transition.
    pub(crate) fn validate_transition(
        &mut self,
        transition: StateTransition,
    ) -> Result<(), String> {
        // Capture state before transition (for logging)
        #[cfg(feature = "transition-logging")]
        let state_before = (&self.state).into();
        #[cfg(feature = "transition-logging")]
        let start = std::time::Instant::now();

        // Validate the transition is allowed
        if !self
            .state_graph
            .is_valid(self.current_state_kind, &transition)
        {
            return Err(format!(
                "Invalid transition {:?} from state {}",
                transition,
                self.current_state_kind.name()
            ));
        }

        // Apply the transition and update state kind
        let result = match self.state_graph.apply(self.current_state_kind, &transition) {
            Ok(new_state) => {
                self.current_state_kind = new_state;
                Ok(())
            }
            Err(e) => Err(format!("Transition validation failed: {}", e)),
        };

        // Log the transition when enabled
        #[cfg(feature = "transition-logging")]
        if result.is_ok() {
            let state_after = (&self.state).into();
            let duration_ns = start.elapsed().as_nanos() as u64;
            let wall_time_us = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64;

            let entry = TransitionEntry::new(
                transition,
                state_before,
                state_after,
                wall_time_us,
                duration_ns,
            );
            self.transition_log.push(entry);
        }

        result
    }

    /// Update the current state kind based on the actual engine state.
    ///
    /// This should be called after state changes that don't have explicit
    /// transitions (like key releases that may change from Typing to Idle).
    pub(crate) fn update_state_kind(&mut self) {
        use crate::engine::transitions::StateKind;

        // Use StateKind::from_engine_state to infer the kind from actual state
        let inferred = StateKind::from_engine_state(&self.state);

        // Only update if different and the current state is an active input state
        // Session and system states (Recording, Replaying, etc.) are managed explicitly
        if inferred != self.current_state_kind && self.current_state_kind.is_active_input() {
            self.current_state_kind = inferred;
        }
    }

    /// Process a single event with optional tracing.
    ///
    /// When a tracer is provided, spans are emitted for input reception,
    /// decision making, and output generation. This enables detailed
    /// performance analysis and debugging via OpenTelemetry.
    pub fn process_event_traced(
        &mut self,
        event: InputEvent,
        tracer: Option<&EngineTracer>,
    ) -> Vec<OutputAction> {
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
                return vec![OutputAction::PassThrough];
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
            return validation.early_output;
        }

        // Step 3: Resolve decisions (combos, pending, layers).
        let decision = self.resolve_decision(&event);

        // Step 4: Apply decision outcomes.
        let result = apply_decision(&event, &mut self.blocked_releases, decision);

        // Step 5: Update state kind based on actual state after processing
        self.update_state_kind();

        // Step 6: Emit tracing spans.
        trace_event(
            tracer,
            &event,
            result.decision_type,
            start_time,
            &self.default_layers().active_layer_ids(),
            &result.outputs,
        );

        result.outputs
    }

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
