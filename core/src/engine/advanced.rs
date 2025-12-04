//! Advanced remapping engine that orchestrates state, layer logic, and
//! timing-based decisions (tap-hold, combos).

use crate::engine::decision_engine::{
    self, activate_hold_action, decision_type_from_action, handle_layer_action, process_resolutions,
};
use crate::engine::processing::{
    apply_decision, trace_event, validate_and_check_safe_mode, DecisionResult,
};
use crate::engine::state::EngineState as UnifiedEngineState;
use crate::engine::transitions::{StateGraph, StateKind, StateTransition};
use crate::engine::{
    ComboDef, ComboRegistry, DecisionQueue, DecisionResolution, DecisionType, EngineTracer,
    InputEvent, KeyCode, LayerAction, LayerStack, ModifierState, OutputAction, PendingDecision,
    PendingDecisionState, TimingConfig,
};
use crate::traits::{KeyStateProvider, ScriptRuntime};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// View adapter for KeyState that implements KeyStateProvider.
///
/// This provides a read-only view of the unified state's key tracking
/// that's compatible with code expecting KeyStateTracker.
pub struct KeyStateView<'a>(&'a UnifiedEngineState);

impl KeyStateProvider for KeyStateView<'_> {
    fn is_pressed(&self, key: KeyCode) -> bool {
        self.0.is_key_pressed(key)
    }

    fn press(&mut self, _key: KeyCode, _timestamp_us: u64, _is_repeat: bool) -> bool {
        // KeyStateView is read-only; mutations should use EngineState::apply()
        unreachable!("KeyStateView is read-only; use EngineState::apply() to mutate state")
    }

    fn release(&mut self, _key: KeyCode) -> Option<u64> {
        // KeyStateView is read-only; mutations should use EngineState::apply()
        unreachable!("KeyStateView is read-only; use EngineState::apply() to mutate state")
    }

    fn press_time(&self, key: KeyCode) -> Option<u64> {
        self.0.key_press_time(key)
    }

    fn pressed_keys(&self) -> Box<dyn Iterator<Item = KeyCode> + '_> {
        Box::new(self.0.pressed_keys())
    }
}

/// Single pressed key with timestamp for snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PressedKeyState {
    pub key: KeyCode,
    pub pressed_at: u64,
}

/// Serializable snapshot of engine state for GUI/FFI inspection.
///
/// Deprecated: This will be replaced by StateSnapshot from the unified state module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStateSnapshot {
    pub pressed_keys: Vec<PressedKeyState>,
    pub modifiers: ModifierState,
    pub layers: LayerStack,
    pub pending: Vec<PendingDecisionState>,
    pub timing: TimingConfig,
    pub safe_mode: bool,
}

/// Extended engine with timing-based decisions.
pub struct AdvancedEngine<S>
where
    S: ScriptRuntime,
{
    _script: S,

    // Unified state - this is the new approach
    state: UnifiedEngineState,

    // State graph for transition validation
    state_graph: StateGraph,
    current_state_kind: StateKind,

    // Legacy compatibility layer during migration
    layers_compat: LayerStack, // For layer definitions and lookups

    // Decisions
    pending: DecisionQueue,
    combos: ComboRegistry,
    blocked_releases: HashSet<KeyCode>,

    // Config
    timing: TimingConfig,
    safe_mode: bool,
    _running: bool,
}

impl<S> AdvancedEngine<S>
where
    S: ScriptRuntime,
{
    /// Create a new engine with injected dependencies and timing config.
    pub fn new(script: S, timing: TimingConfig) -> Self {
        Self {
            _script: script,
            state: UnifiedEngineState::new(timing.clone()),
            state_graph: StateGraph::new(),
            current_state_kind: StateKind::Idle,
            layers_compat: LayerStack::new(),
            pending: DecisionQueue::new(timing.clone()),
            combos: ComboRegistry::new(),
            blocked_releases: HashSet::new(),
            timing,
            safe_mode: false,
            _running: false,
        }
    }

    /// Mutable access to layer stack (useful for configuration in setup/tests).
    pub fn layers_mut(&mut self) -> &mut LayerStack {
        &mut self.layers_compat
    }

    /// Mutable access to combo registry for configuration.
    pub fn combos_mut(&mut self) -> &mut ComboRegistry {
        &mut self.combos
    }

    /// Get the current state kind.
    pub fn current_state_kind(&self) -> StateKind {
        self.current_state_kind
    }

    /// Validate and apply a state transition.
    ///
    /// This checks if the transition is valid from the current state,
    /// applies it through the state graph, and updates the current state kind.
    fn validate_transition(&mut self, transition: StateTransition) -> Result<(), String> {
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
        match self.state_graph.apply(self.current_state_kind, &transition) {
            Ok(new_state) => {
                self.current_state_kind = new_state;
                Ok(())
            }
            Err(e) => Err(format!("Transition validation failed: {}", e)),
        }
    }

    /// Update the current state kind based on the actual engine state.
    ///
    /// This should be called after state changes that don't have explicit
    /// transitions (like key releases that may change from Typing to Idle).
    fn update_state_kind(&mut self) {
        // Use StateKind::from_engine_state to infer the kind from actual state
        let inferred = StateKind::from_engine_state(&self.state);

        // Only update if different and the current state is an active input state
        // Session and system states (Recording, Replaying, etc.) are managed explicitly
        if inferred != self.current_state_kind && self.current_state_kind.is_active_input() {
            self.current_state_kind = inferred;
        }
    }

    /// Process a single event through all layers.
    pub fn process_event(&mut self, event: InputEvent) -> Vec<OutputAction> {
        self.process_event_traced(event, None)
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
        if let Err(e) = self.validate_transition(transition) {
            #[cfg(debug_assertions)]
            tracing::warn!("State transition validation warning: {}", e);
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
            self.pending.clear();
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
            &self.layers_compat.active_layer_ids(),
            &result.outputs,
        );

        result.outputs
    }

    /// Resolve all decisions for an event (combos, pending, layers).
    fn resolve_decision(&mut self, event: &InputEvent) -> DecisionResult {
        // Mark other tap-hold decisions as interrupted when another key is pressed.
        if event.pressed {
            self.pending.mark_interrupted(event.key);
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

        let resolutions = self.pending.check_event(event);
        let (resolved_outputs, resolved_consumed, skip_layer_actions) =
            self.handle_resolutions(resolutions, Some(event));
        result.outputs.extend(resolved_outputs);
        result.consumed |= resolved_consumed;
        result.skip_layer_actions = skip_layer_actions;

        // Layer lookup and action execution using the compat layer
        if !result.skip_layer_actions {
            if let Some(action) = self.layers_compat.lookup(event.key).cloned() {
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

    /// Check for timeout-based resolutions (tap-hold and combo windows).
    pub fn tick(&mut self, now_us: u64) -> Vec<OutputAction> {
        if self.safe_mode {
            return Vec::new();
        }

        let resolutions = self.pending.check_timeouts(now_us);
        let (outputs, _, _) = self.handle_resolutions(resolutions, None);
        outputs
    }

    /// Inspect key state.
    ///
    /// Returns a view of the unified state's key tracking.
    pub fn key_state(&self) -> KeyStateView<'_> {
        KeyStateView(&self.state)
    }

    /// Inspect modifier state.
    pub fn modifiers(&self) -> &ModifierState {
        // TODO: This should return a view of the unified state's modifiers
        // For now, we use the compat layer
        self.state.modifiers()
    }

    /// Mutable modifier state (used for configuration).
    pub fn modifiers_mut(&mut self) -> &mut ModifierState {
        // TODO: Direct mutation bypasses the unified state's mutation API
        // This should eventually use apply() mutations
        self.state.modifiers_mut()
    }

    /// Inspect layer stack.
    pub fn layers(&self) -> &LayerStack {
        &self.layers_compat
    }

    /// Inspect pending decisions.
    pub fn pending(&self) -> &[PendingDecision] {
        self.pending.pending()
    }

    /// Access timing config.
    pub fn timing_config(&self) -> &TimingConfig {
        &self.timing
    }

    /// Serializable snapshot of current engine state.
    ///
    /// DEPRECATED: This returns the legacy EngineStateSnapshot format.
    /// New code should use `state_snapshot()` to get the unified StateSnapshot.
    pub fn snapshot(&self) -> EngineStateSnapshot {
        let pressed_keys = self
            .state
            .pressed_keys()
            .filter_map(|key| {
                self.state
                    .key_press_time(key)
                    .map(|pressed_at| PressedKeyState { key, pressed_at })
            })
            .collect();

        EngineStateSnapshot {
            pressed_keys,
            modifiers: *self.state.modifiers(),
            layers: self.layers_compat.clone(),
            pending: self.pending.snapshot(),
            timing: self.timing.clone(),
            safe_mode: self.safe_mode,
        }
    }

    /// Get a state snapshot using the new unified StateSnapshot format.
    ///
    /// This is the preferred way to get state snapshots for new code.
    pub fn state_snapshot(&self) -> crate::engine::state::snapshot::StateSnapshot {
        (&self.state).into()
    }

    fn enqueue_combos(&mut self, event: &InputEvent) -> (bool, Vec<OutputAction>) {
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

    fn handle_resolutions(
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
        // TODO: Eventually use the unified state's mutation API instead
        for (_, action) in &result.hold_activations {
            let hold_outputs =
                activate_hold_action(action, self.state.modifiers_mut(), &mut self.layers_compat);
            outputs.extend(hold_outputs);
        }

        // Execute layer actions (from combo triggers)
        for action in result.layer_actions {
            outputs.extend(self.execute_layer_action_with_event(action, event));
        }

        (outputs, result.consumed, result.skip_layer_actions)
    }

    fn handle_layer_action(
        &mut self,
        event: &InputEvent,
        action: LayerAction,
    ) -> (Vec<OutputAction>, bool) {
        // TapHold requires special handling with DecisionQueue
        if let LayerAction::TapHold { tap, hold } = &action {
            if event.pressed {
                let (_, eager) =
                    self.pending
                        .add_tap_hold(event.key, event.timestamp_us, *tap, hold.clone());
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
        // TODO: Eventually use the unified state's mutation API instead
        let result = handle_layer_action(
            event,
            &action,
            self.state.modifiers_mut(),
            &mut self.layers_compat,
        );
        (result.outputs, result.consumed)
    }

    fn execute_layer_action_with_event(
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
        // TODO: Eventually use the unified state's mutation API instead
        decision_engine::execute_layer_action(
            &action,
            self.state.modifiers_mut(),
            &mut self.layers_compat,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{HoldAction, Layer};
    use crate::mocks::MockRuntime;

    fn test_engine() -> AdvancedEngine<MockRuntime> {
        AdvancedEngine::new(MockRuntime::default(), TimingConfig::default())
    }

    fn key_down(key: KeyCode, ts: u64) -> InputEvent {
        InputEvent::key_down(key, ts)
    }

    fn key_up(key: KeyCode, ts: u64) -> InputEvent {
        InputEvent::key_up(key, ts)
    }

    #[test]
    fn tap_hold_tap_path_emits_tap() {
        let mut engine = test_engine();
        let mut base = Layer::base();
        base.set_mapping(
            KeyCode::CapsLock,
            LayerAction::TapHold {
                tap: KeyCode::Escape,
                hold: HoldAction::Key(KeyCode::LeftCtrl),
            },
        );
        engine.layers_mut().define_layer(base);

        let output = engine.process_event(key_down(KeyCode::CapsLock, 0));
        assert_eq!(output, vec![OutputAction::Block]);

        let output = engine.process_event(key_up(KeyCode::CapsLock, 50_000));
        assert_eq!(
            output,
            vec![
                OutputAction::KeyDown(KeyCode::Escape),
                OutputAction::KeyUp(KeyCode::Escape),
                OutputAction::Block
            ]
        );
    }

    #[test]
    fn tap_hold_hold_path_emits_hold_action_on_timeout() {
        let mut engine = test_engine();
        let mut base = Layer::base();
        base.set_mapping(
            KeyCode::CapsLock,
            LayerAction::TapHold {
                tap: KeyCode::Escape,
                hold: HoldAction::Key(KeyCode::LeftCtrl),
            },
        );
        engine.layers_mut().define_layer(base);

        let output = engine.process_event(key_down(KeyCode::CapsLock, 0));
        assert_eq!(output, vec![OutputAction::Block]);

        let tick_out = engine.tick(250_000);
        assert_eq!(tick_out, vec![OutputAction::KeyDown(KeyCode::LeftCtrl)]);
    }

    #[test]
    fn combo_trigger_blocks_keys_and_emits_action() {
        let mut engine = test_engine();
        engine.combos_mut().register(
            &[KeyCode::A, KeyCode::B],
            LayerAction::Remap(KeyCode::Escape),
        );

        let out1 = engine.process_event(key_down(KeyCode::A, 0));
        assert!(out1.contains(&OutputAction::Block));

        let out2 = engine.process_event(key_down(KeyCode::B, 10_000));
        assert_eq!(
            out2,
            vec![
                OutputAction::KeyDown(KeyCode::Escape),
                OutputAction::KeyUp(KeyCode::Escape),
                OutputAction::Block
            ]
        );
    }

    #[test]
    fn safe_mode_toggle_passes_through_events() {
        let mut engine = test_engine();

        // Press chord Ctrl+Alt+Shift+Escape to toggle.
        let _ = engine.process_event(key_down(KeyCode::LeftCtrl, 0));
        let _ = engine.process_event(key_down(KeyCode::LeftAlt, 0));
        let _ = engine.process_event(key_down(KeyCode::LeftShift, 0));
        let out = engine.process_event(key_down(KeyCode::Escape, 0));
        assert_eq!(out, vec![OutputAction::PassThrough]);

        // Subsequent events should pass through while safe mode is active.
        let out2 = engine.process_event(key_down(KeyCode::A, 10_000));
        assert_eq!(out2, vec![OutputAction::PassThrough]);
    }

    #[test]
    fn snapshot_exposes_serializable_state() {
        let mut engine = test_engine();
        let mut base = Layer::base();
        base.set_mapping(
            KeyCode::CapsLock,
            LayerAction::TapHold {
                tap: KeyCode::Escape,
                hold: HoldAction::Key(KeyCode::LeftCtrl),
            },
        );
        engine.layers_mut().define_layer(base);

        let _ = engine.process_event(key_down(KeyCode::CapsLock, 100));

        let snapshot = engine.snapshot();
        assert!(!snapshot.safe_mode);
        assert!(snapshot
            .pressed_keys
            .iter()
            .any(|pk| pk.key == KeyCode::CapsLock && pk.pressed_at == 100));
        assert!(snapshot.layers.is_active(0));
        assert_eq!(snapshot.pending.len(), 1);

        serde_json::to_string(&snapshot).expect("engine state serializes");
    }

    #[test]
    fn combo_timeout_emits_single_release_path() {
        let mut engine = test_engine();
        engine.combos_mut().register(
            &[KeyCode::A, KeyCode::B],
            LayerAction::Remap(KeyCode::Escape),
        );

        let first = engine.process_event(key_down(KeyCode::A, 0));
        assert_eq!(first, vec![OutputAction::Block]);

        // Timeout should synthesize only a KeyDown for the still-pressed key.
        let timeout = engine.tick(60_000);
        assert_eq!(timeout, vec![OutputAction::KeyDown(KeyCode::A)]);

        // The real release should produce a single KeyUp, not a double release.
        let release = engine.process_event(key_up(KeyCode::A, 70_000));
        assert_eq!(release, vec![OutputAction::KeyUp(KeyCode::A)]);
    }

    #[test]
    fn combo_queue_saturation_does_not_block_events() {
        let mut engine = test_engine();
        engine.combos_mut().register(
            &[KeyCode::A, KeyCode::B],
            LayerAction::Remap(KeyCode::Escape),
        );

        for _ in 0..DecisionQueue::MAX_PENDING {
            let _ = engine
                .pending
                .add_combo(&[KeyCode::A, KeyCode::B], 0, LayerAction::Block);
        }

        // With a full queue, new combo-related keys should pass through instead of being blocked.
        let output = engine.process_event(key_down(KeyCode::A, 0));
        assert_eq!(output, vec![OutputAction::KeyDown(KeyCode::A)]);
    }

    #[test]
    fn combo_tap_hold_uses_event_timestamp() {
        let mut engine = test_engine();
        engine.combos_mut().register(
            &[KeyCode::A, KeyCode::B],
            LayerAction::TapHold {
                tap: KeyCode::Escape,
                hold: HoldAction::Key(KeyCode::LeftCtrl),
            },
        );

        let first = engine.process_event(key_down(KeyCode::A, 0));
        assert_eq!(first, vec![OutputAction::Block]);

        // Second key should enqueue tap-hold with the event's timestamp.
        let second = engine.process_event(key_down(KeyCode::B, 10_000));
        assert_eq!(second, vec![OutputAction::Block]);

        let pending = engine.pending.snapshot();
        let (key, pressed_at) = match pending.first() {
            Some(PendingDecisionState::TapHold {
                key, pressed_at, ..
            }) => (*key, *pressed_at),
            other => {
                unreachable!("expected TapHold pending, got {:?}", other)
            }
        };
        assert_eq!(key, KeyCode::B);
        assert_eq!(pressed_at, 10_000);

        // Resolve as hold via timeout to ensure the pending entry completes.
        let timeout = engine.tick(260_000);
        assert_eq!(timeout, vec![OutputAction::KeyDown(KeyCode::LeftCtrl)]);

        let release = engine.process_event(key_up(KeyCode::B, 300_000));
        assert_eq!(release, vec![OutputAction::Block]);
    }
}
