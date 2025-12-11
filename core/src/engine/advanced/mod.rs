//! Advanced remapping engine that orchestrates state, layer logic, and
//! timing-based decisions (tap-hold, combos).
//!
//! # Module Organization
//!
//! - [`mod.rs`] - Core `AdvancedEngine` struct, constructor, and public API
//! - [`combos`] - Combo detection and resolution logic
//! - [`processing`] - Event processing and decision resolution

mod combos;
mod processing;

use crate::engine::state::EngineState;
use crate::engine::transitions::log::TransitionLog;
use crate::engine::transitions::{StateGraph, StateKind};
use crate::engine::{
    ComboRegistry, DecisionQueue, InputEvent, KeyCode, LayerStack, Layout, LayoutCompositor,
    LayoutMetadata, ModifierCoordinator, ModifierState, OutputAction, PendingDecision,
    TimingConfig,
};
use crate::registry::device::DeviceRegistry;
use crate::traits::{KeyStateProvider, ScriptRuntime};
use std::collections::HashSet;
use std::sync::OnceLock;

// Re-export submodule items for internal use only
// All public API is exposed through AdvancedEngine methods

const DEFAULT_LAYOUT_ID: &str = "default";
static FALLBACK_LAYOUT: OnceLock<Layout> = OnceLock::new();

/// View adapter for KeyState that implements KeyStateProvider.
///
/// This provides a read-only view of the unified state's key tracking
/// that's compatible with code expecting KeyStateTracker.
pub struct KeyStateView<'a>(pub(crate) &'a EngineState);

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

/// Extended engine with timing-based decisions.
pub struct AdvancedEngine<S>
where
    S: ScriptRuntime,
{
    pub(crate) _script: S,

    // Unified state - this is the canonical approach
    pub(crate) state: EngineState,

    // State graph for transition validation
    pub(crate) state_graph: StateGraph,
    pub(crate) current_state_kind: StateKind,

    // Transition logging for debugging
    pub(crate) transition_log: TransitionLog,

    // Layout compositor with a default layout (replaces legacy single stack)
    pub(crate) layouts: LayoutCompositor,
    pub(crate) modifier_coordinator: ModifierCoordinator,

    // Decisions
    pub(crate) pending: DecisionQueue,
    pub(crate) combos: ComboRegistry,
    pub(crate) blocked_releases: HashSet<KeyCode>,

    // Config
    pub(crate) timing: TimingConfig,
    pub(crate) safe_mode: bool,
    pub(crate) _running: bool,

    // Revolutionary mapping pipeline (optional)
    /// Device registry for per-device configuration
    pub(crate) device_registry: Option<DeviceRegistry>,
}

impl<S> AdvancedEngine<S>
where
    S: ScriptRuntime,
{
    /// Create a new engine with injected dependencies and timing config.
    pub fn new(script: S, timing: TimingConfig) -> Self {
        let mut layouts = LayoutCompositor::new();
        layouts.add_layout(
            Layout::new(LayoutMetadata::new(DEFAULT_LAYOUT_ID, "Default")),
            0,
        );

        Self {
            _script: script,
            state: EngineState::new(timing.clone()),
            state_graph: StateGraph::new(),
            current_state_kind: StateKind::Idle,
            transition_log: TransitionLog::default(),
            layouts,
            modifier_coordinator: ModifierCoordinator::new(),
            pending: DecisionQueue::new(timing.clone()),
            combos: ComboRegistry::new(),
            blocked_releases: HashSet::new(),
            timing,
            safe_mode: false,
            _running: false,
            device_registry: None,
        }
    }

    /// Access the default layout used for single-layout compatibility.
    pub(crate) fn default_layout(&self) -> &Layout {
        self.layouts
            .layout(DEFAULT_LAYOUT_ID)
            .or_else(|| {
                self.layouts
                    .active_layouts()
                    .next()
                    .map(|layout| layout.layout())
            })
            .unwrap_or_else(|| {
                FALLBACK_LAYOUT
                    .get_or_init(|| Layout::new(LayoutMetadata::new(DEFAULT_LAYOUT_ID, "Default")))
            })
    }

    /// Mutably access the default layout, recreating it if missing.
    #[allow(clippy::unwrap_used)]
    pub(crate) fn default_layout_mut(&mut self) -> &mut Layout {
        if self.layouts.layout(DEFAULT_LAYOUT_ID).is_none() {
            self.layouts.add_layout(
                Layout::new(LayoutMetadata::new(DEFAULT_LAYOUT_ID, "Default")),
                0,
            );
        }
        self.layouts.layout_mut(DEFAULT_LAYOUT_ID).unwrap()
    }

    /// Convenience for accessing the default layout's layers.
    pub(crate) fn default_layers(&self) -> &LayerStack {
        self.default_layout().layers()
    }

    /// Convenience for mutating the default layout's layers.
    pub(crate) fn default_layers_mut(&mut self) -> &mut LayerStack {
        self.default_layout_mut().layers_mut()
    }

    /// Borrow modifier state and the default layout's layers together.
    #[allow(clippy::unwrap_used)]
    pub(crate) fn modifier_and_default_layers(&mut self) -> (&mut ModifierState, &mut LayerStack) {
        let (state, layouts) = (&mut self.state, &mut self.layouts);

        if layouts.layout(DEFAULT_LAYOUT_ID).is_none() {
            layouts.add_layout(
                Layout::new(LayoutMetadata::new(DEFAULT_LAYOUT_ID, "Default")),
                0,
            );
        }

        let layers = layouts
            .layout_mut(DEFAULT_LAYOUT_ID)
            .map(|layout| layout.layers_mut())
            .unwrap();

        let modifiers = state.modifiers_mut();

        (modifiers, layers)
    }

    /// Set the device registry for per-device revolutionary mapping.
    ///
    /// When a device registry is set, the engine will check per-device
    /// configuration before processing events:
    /// - If a device has remap_enabled=false, events are passed through
    /// - If a device has an assigned profile, profile mappings are used
    /// - Otherwise, default layer-based remapping is used
    pub fn with_device_registry(mut self, registry: DeviceRegistry) -> Self {
        self.device_registry = Some(registry);
        self
    }

    /// Get a reference to the device registry, if configured.
    pub fn device_registry(&self) -> Option<&DeviceRegistry> {
        self.device_registry.as_ref()
    }

    /// Mutable access to layer stack (useful for configuration in setup/tests).
    pub fn layers_mut(&mut self) -> &mut LayerStack {
        self.default_layers_mut()
    }

    /// Mutable access to combo registry for configuration.
    pub fn combos_mut(&mut self) -> &mut ComboRegistry {
        &mut self.combos
    }

    /// Get the current state kind.
    pub fn current_state_kind(&self) -> StateKind {
        self.current_state_kind
    }

    /// Get a reference to the transition log.
    ///
    /// The transition log records all state transitions with before/after
    /// state snapshots, timing information, and metadata. This is useful
    /// for debugging, replay, and analysis.
    ///
    /// When the `transition-logging` feature is disabled, this returns
    /// a zero-sized stub that has no overhead.
    pub fn transition_log(&self) -> &TransitionLog {
        &self.transition_log
    }

    /// Get a mutable reference to the transition log.
    ///
    /// This allows clearing the log or adjusting its configuration.
    pub fn transition_log_mut(&mut self) -> &mut TransitionLog {
        &mut self.transition_log
    }

    /// Inspect key state.
    ///
    /// Returns a view of the unified state's key tracking.
    pub fn key_state(&self) -> KeyStateView<'_> {
        KeyStateView(&self.state)
    }

    /// Inspect modifier state.
    pub fn modifiers(&self) -> &ModifierState {
        self.state.modifiers()
    }

    /// Mutable modifier state (used for configuration).
    pub fn modifiers_mut(&mut self) -> &mut ModifierState {
        self.state.modifiers_mut()
    }

    /// Inspect layer stack.
    pub fn layers(&self) -> &LayerStack {
        self.default_layers()
    }

    /// Inspect the layout compositor (including multiple layouts).
    pub fn layouts(&self) -> &LayoutCompositor {
        &self.layouts
    }

    /// Mutate the layout compositor.
    pub fn layouts_mut(&mut self) -> &mut LayoutCompositor {
        &mut self.layouts
    }

    /// Access cross-layout modifier coordination.
    pub fn modifier_coordinator(&self) -> &ModifierCoordinator {
        &self.modifier_coordinator
    }

    /// Mutate cross-layout modifier coordination.
    pub fn modifier_coordinator_mut(&mut self) -> &mut ModifierCoordinator {
        &mut self.modifier_coordinator
    }

    /// Inspect pending decisions.
    pub fn pending(&self) -> &[PendingDecision] {
        self.pending.pending()
    }

    /// Access timing config.
    pub fn timing_config(&self) -> &TimingConfig {
        &self.timing
    }

    /// Get a serializable snapshot of current engine state.
    ///
    /// Returns a StateSnapshot suitable for FFI, debugging, and persistence.
    pub fn snapshot(&self) -> crate::engine::state::snapshot::StateSnapshot {
        crate::engine::state::snapshot::StateSnapshot::with_layouts(
            &self.state,
            &self.layouts,
            Some(&self.modifier_coordinator),
        )
    }

    /// Process a single event through all layers.
    pub fn process_event(&mut self, event: InputEvent) -> Vec<OutputAction> {
        self.process_event_traced(event, None)
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
}

#[cfg(test)]
mod tests;
