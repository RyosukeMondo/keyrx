//! Advanced remapping engine that orchestrates state, layer logic, and
//! timing-based decisions (tap-hold, combos).

use crate::engine::{
    ComboDef, ComboRegistry, DecisionQueue, DecisionResolution, HoldAction, InputEvent, KeyCode,
    KeyStateTracker, LayerAction, LayerStack, Modifier, ModifierState, OutputAction,
    PendingDecision, TimingConfig,
};
use crate::traits::{InputSource, ScriptRuntime};

/// Extended engine with timing-based decisions.
pub struct AdvancedEngine<I, S>
where
    I: InputSource,
    S: ScriptRuntime,
{
    _input: I,
    _script: S,

    // State
    key_state: KeyStateTracker,
    modifiers: ModifierState,
    layers: LayerStack,

    // Decisions
    pending: DecisionQueue,
    combos: ComboRegistry,

    // Config
    timing: TimingConfig,
    safe_mode: bool,
    _running: bool,
}

impl<I, S> AdvancedEngine<I, S>
where
    I: InputSource,
    S: ScriptRuntime,
{
    /// Create a new engine with injected dependencies and timing config.
    pub fn new(input: I, script: S, timing: TimingConfig) -> Self {
        Self {
            _input: input,
            _script: script,
            key_state: KeyStateTracker::new(),
            modifiers: ModifierState::new(),
            layers: LayerStack::new(),
            pending: DecisionQueue::new(timing.clone()),
            combos: ComboRegistry::new(),
            timing,
            safe_mode: false,
            _running: false,
        }
    }

    /// Mutable access to layer stack (useful for configuration in setup/tests).
    pub fn layers_mut(&mut self) -> &mut LayerStack {
        &mut self.layers
    }

    /// Mutable access to combo registry for configuration.
    pub fn combos_mut(&mut self) -> &mut ComboRegistry {
        &mut self.combos
    }

    /// Process a single event through all layers.
    pub fn process_event(&mut self, event: InputEvent) -> Vec<OutputAction> {
        if event.is_synthetic {
            return vec![OutputAction::PassThrough];
        }

        // Step 2: Update key state.
        if event.pressed {
            self.key_state
                .key_down(event.key, event.timestamp_us, event.is_repeat);
        } else {
            self.key_state.key_up(event.key);
        }

        // Step 3: Safe mode toggle (Ctrl+Alt+Shift+Escape).
        if self.check_safe_mode_toggle(&event) {
            self.pending.clear();
            self.safe_mode = !self.safe_mode;
            return vec![OutputAction::PassThrough];
        }

        if self.safe_mode {
            return vec![OutputAction::PassThrough];
        }

        let mut outputs = Vec::new();
        let mut consumed = false;

        // Step 4/5: Combo tracking + pending resolutions.
        let (blocked_for_combo, combo_outputs) = if event.pressed {
            self.enqueue_combos(&event)
        } else {
            (false, Vec::new())
        };
        if !combo_outputs.is_empty() {
            consumed = true;
            outputs.extend(combo_outputs);
        }

        let resolutions = self.pending.check_event(&event);
        let (resolved_outputs, resolved_consumed) = self.handle_resolutions(resolutions);
        outputs.extend(resolved_outputs);
        consumed |= resolved_consumed;

        // Step 7-9: Layer lookup and action execution.
        if let Some(action) = self.layers.lookup(event.key).cloned() {
            let (handled_outputs, handled) = self.handle_layer_action(&event, action);
            outputs.extend(handled_outputs);
            consumed |= handled;
        }

        if blocked_for_combo {
            outputs.push(OutputAction::Block);
            consumed = true;
        }

        if !consumed {
            outputs.push(self.pass_through_event(&event));
        }

        outputs
    }

    /// Check for timeout-based resolutions (tap-hold and combo windows).
    pub fn tick(&mut self, now_us: u64) -> Vec<OutputAction> {
        if self.safe_mode {
            return Vec::new();
        }

        let resolutions = self.pending.check_timeouts(now_us);
        let (outputs, _) = self.handle_resolutions(resolutions);
        outputs
    }

    /// Inspect key state.
    pub fn key_state(&self) -> &KeyStateTracker {
        &self.key_state
    }

    /// Inspect modifier state.
    pub fn modifiers(&self) -> &ModifierState {
        &self.modifiers
    }

    /// Inspect layer stack.
    pub fn layers(&self) -> &LayerStack {
        &self.layers
    }

    /// Inspect pending decisions.
    pub fn pending(&self) -> &[PendingDecision] {
        self.pending.pending()
    }

    /// Access timing config.
    pub fn timing_config(&self) -> &TimingConfig {
        &self.timing
    }

    fn check_safe_mode_toggle(&self, event: &InputEvent) -> bool {
        if !event.pressed || event.key != KeyCode::Escape {
            return false;
        }

        let has_ctrl = self.key_state.is_pressed(KeyCode::LeftCtrl)
            || self.key_state.is_pressed(KeyCode::RightCtrl);
        let has_alt = self.key_state.is_pressed(KeyCode::LeftAlt)
            || self.key_state.is_pressed(KeyCode::RightAlt);
        let has_shift = self.key_state.is_pressed(KeyCode::LeftShift)
            || self.key_state.is_pressed(KeyCode::RightShift);

        has_ctrl && has_alt && has_shift
    }

    fn enqueue_combos(&mut self, event: &InputEvent) -> (bool, Vec<OutputAction>) {
        let pressed_keys: Vec<_> = self.key_state.pressed_keys().collect();
        let mut blocked = false;
        let mut outputs = Vec::new();

        for ComboDef { keys, action } in self.combos.all() {
            if keys.contains(&event.key) {
                blocked = true;
                let _ = self
                    .pending
                    .add_combo(&keys, event.timestamp_us, action.clone());
            }
        }

        // If current pressed keys already match a combo, trigger immediately.
        if let Some(action) = self.combos.find(&pressed_keys) {
            let immediate = self.execute_layer_action(action.clone());
            self.pending.clear();
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
    ) -> (Vec<OutputAction>, bool) {
        let mut outputs = Vec::new();
        let mut consumed = false;

        for resolution in resolutions {
            match resolution {
                DecisionResolution::Tap { key, .. } => {
                    outputs.push(OutputAction::KeyDown(key));
                    outputs.push(OutputAction::KeyUp(key));
                    consumed = true;
                }
                DecisionResolution::Hold { action, .. } => {
                    let hold_outputs = self.activate_hold_action(action);
                    outputs.extend(hold_outputs);
                    consumed = true;
                }
                DecisionResolution::ComboTriggered(action) => {
                    outputs.extend(self.execute_layer_action(action));
                    consumed = true;
                }
                DecisionResolution::ComboTimeout(keys) => {
                    for key in keys {
                        outputs.push(OutputAction::KeyDown(key));
                        outputs.push(OutputAction::KeyUp(key));
                    }
                    consumed = true;
                }
            }
        }

        (outputs, consumed)
    }

    fn handle_layer_action(
        &mut self,
        event: &InputEvent,
        action: LayerAction,
    ) -> (Vec<OutputAction>, bool) {
        let mut outputs = Vec::new();
        let mut consumed = true;

        match action {
            LayerAction::Remap(target) => {
                outputs.push(if event.pressed {
                    OutputAction::KeyDown(target)
                } else {
                    OutputAction::KeyUp(target)
                });
            }
            LayerAction::Block => outputs.push(OutputAction::Block),
            LayerAction::TapHold { tap, hold } => {
                if event.pressed {
                    let (_, eager) =
                        self.pending
                            .add_tap_hold(event.key, event.timestamp_us, tap, hold.clone());
                    if let Some(resolution) = eager {
                        let (eager_outputs, _) = self.handle_resolutions(vec![resolution]);
                        outputs.extend(eager_outputs);
                    }
                    outputs.push(OutputAction::Block);
                } else {
                    outputs.push(OutputAction::Block);
                }
            }
            LayerAction::LayerPush(id) => {
                self.layers.push(id);
                outputs.push(OutputAction::Block);
            }
            LayerAction::LayerPop => {
                self.layers.pop();
                outputs.push(OutputAction::Block);
            }
            LayerAction::LayerToggle(id) => {
                self.layers.toggle(id);
                outputs.push(OutputAction::Block);
            }
            LayerAction::ModifierActivate(id) => {
                self.modifiers.activate(Modifier::Virtual(id));
                outputs.push(OutputAction::Block);
            }
            LayerAction::ModifierDeactivate(id) => {
                self.modifiers.deactivate(Modifier::Virtual(id));
                outputs.push(OutputAction::Block);
            }
            LayerAction::ModifierOneShot(id) => {
                self.modifiers.arm_one_shot(Modifier::Virtual(id));
                outputs.push(OutputAction::Block);
            }
            LayerAction::Pass => {
                outputs.push(OutputAction::PassThrough);
                consumed = false;
            }
        }

        (outputs, consumed)
    }

    fn activate_hold_action(&mut self, action: HoldAction) -> Vec<OutputAction> {
        match action {
            HoldAction::Key(key) => vec![OutputAction::KeyDown(key)],
            HoldAction::Modifier(id) => {
                self.modifiers.activate(Modifier::Virtual(id));
                vec![OutputAction::Block]
            }
            HoldAction::Layer(layer) => {
                self.layers.push(layer);
                vec![OutputAction::Block]
            }
        }
    }

    fn execute_layer_action(&mut self, action: LayerAction) -> Vec<OutputAction> {
        match action {
            LayerAction::Remap(target) => {
                vec![OutputAction::KeyDown(target), OutputAction::KeyUp(target)]
            }
            LayerAction::Block => vec![OutputAction::Block],
            LayerAction::TapHold { tap, hold } => {
                let (_, eager) = self.pending.add_tap_hold(KeyCode::Unknown(0), 0, tap, hold);
                eager
                    .map(|res| self.handle_resolutions(vec![res]).0)
                    .unwrap_or_default()
            }
            LayerAction::LayerPush(id) => {
                self.layers.push(id);
                vec![OutputAction::Block]
            }
            LayerAction::LayerPop => {
                self.layers.pop();
                vec![OutputAction::Block]
            }
            LayerAction::LayerToggle(id) => {
                self.layers.toggle(id);
                vec![OutputAction::Block]
            }
            LayerAction::ModifierActivate(id) => {
                self.modifiers.activate(Modifier::Virtual(id));
                vec![OutputAction::Block]
            }
            LayerAction::ModifierDeactivate(id) => {
                self.modifiers.deactivate(Modifier::Virtual(id));
                vec![OutputAction::Block]
            }
            LayerAction::ModifierOneShot(id) => {
                self.modifiers.arm_one_shot(Modifier::Virtual(id));
                vec![OutputAction::Block]
            }
            LayerAction::Pass => vec![OutputAction::PassThrough],
        }
    }

    fn pass_through_event(&self, event: &InputEvent) -> OutputAction {
        if event.pressed {
            OutputAction::KeyDown(event.key)
        } else {
            OutputAction::KeyUp(event.key)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Layer;
    use crate::mocks::{MockInput, MockRuntime};

    fn test_engine() -> AdvancedEngine<MockInput, MockRuntime> {
        AdvancedEngine::new(
            MockInput::new(),
            MockRuntime::default(),
            TimingConfig::default(),
        )
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
}
