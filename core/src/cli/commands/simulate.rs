//! Simulate command for headless testing.

use crate::cli::{OutputFormat, OutputWriter};
use crate::engine::{
    AdvancedEngine, EngineState, InputEvent, KeyCode, LayerAction, OutputAction,
    PendingDecisionState, RemapAction,
};
use crate::scripting::{RemapRegistry, RhaiRuntime};
use crate::traits::ScriptRuntime;
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::PathBuf;

const DEFAULT_EVENT_GAP_US: u64 = 1_000;

#[derive(Debug)]
struct ParsedKey {
    key: KeyCode,
    hold_ms: Option<u64>,
}

/// Result of simulating a single key event.
#[derive(Debug, Serialize)]
pub struct SimulationResult {
    /// Original input key.
    pub input: String,
    /// Primary output action (for backward compatibility).
    pub output: String,
    /// All output actions produced for this input event.
    pub outputs: Vec<String>,
    /// Whether this was a key press or release.
    pub pressed: bool,
    /// Event timestamp in microseconds.
    pub timestamp_us: u64,
}

/// Complete simulation output.
#[derive(Debug, Serialize)]
pub struct SimulationOutput {
    /// Results for each input event.
    pub results: Vec<SimulationResult>,
    /// Total events processed.
    pub total: usize,
    /// Number of remapped events.
    pub remapped: usize,
    /// Number of blocked events.
    pub blocked: usize,
    /// Number of passed events.
    pub passed: usize,
    /// Pending decisions after simulation.
    pub pending: Vec<PendingDecisionState>,
    /// Active layers (top to bottom).
    pub active_layers: Vec<String>,
    /// Full engine state snapshot (for debugging).
    pub state: EngineState,
}

/// Simulate command for headless testing.
pub struct SimulateCommand {
    pub input_keys: String,
    pub script_path: Option<PathBuf>,
    pub hold_ms: Option<u64>,
    pub combo: bool,
    pub output: OutputWriter,
}

impl SimulateCommand {
    pub fn new(input_keys: String, script_path: Option<PathBuf>, format: OutputFormat) -> Self {
        Self {
            input_keys,
            script_path,
            hold_ms: None,
            combo: false,
            output: OutputWriter::new(format),
        }
    }

    /// Configure a default hold duration (milliseconds) for generated key-ups.
    pub fn with_hold_ms(mut self, hold_ms: Option<u64>) -> Self {
        self.hold_ms = hold_ms;
        self
    }

    /// Configure whether the input keys should be treated as a combo (simultaneous press/release).
    pub fn with_combo(mut self, combo: bool) -> Self {
        self.combo = combo;
        self
    }

    /// Parse comma-separated key names into InputEvents.
    ///
    /// Each key name is converted to a key-down and key-up event pair.
    pub fn parse_input(&self) -> Result<Vec<InputEvent>> {
        let parsed_keys = self.parse_keys()?;
        if parsed_keys.is_empty() {
            return Ok(Vec::new());
        }

        if self.combo {
            self.build_combo_events(parsed_keys)
        } else {
            self.build_sequence_events(parsed_keys)
        }
    }

    /// Execute simulation and return the output.
    ///
    /// This is the core simulation logic that returns the result directly,
    /// useful for testing without stdout capture.
    pub async fn execute(&self) -> Result<SimulationOutput> {
        // Parse input keys
        let events = self.parse_input()?;
        if events.is_empty() {
            bail!("No valid input keys provided");
        }

        // Create runtime and load script if provided
        let runtime = self.create_runtime()?;
        let registry = runtime.registry().clone();

        // Create engine with mocks
        let mut engine = self.create_engine(&registry, runtime);

        // Process events and collect results
        let (results, remapped, blocked, passed) = self.process_events(&mut engine, &events);
        let state = engine.snapshot();
        let active_layers = state
            .layers
            .active_layers()
            .into_iter()
            .map(str::to_string)
            .collect();

        Ok(SimulationOutput {
            total: results.len(),
            results,
            remapped,
            blocked,
            passed,
            pending: state.pending.clone(),
            active_layers,
            state,
        })
    }

    /// Create and initialize the script runtime.
    fn create_runtime(&self) -> Result<RhaiRuntime> {
        let mut runtime = RhaiRuntime::new()?;
        if let Some(path) = &self.script_path {
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in path: {:?}", path))?;
            runtime
                .load_file(path_str)
                .with_context(|| format!("Failed to compile script '{}'", path.display()))?;

            // Run top-level statements (e.g., remap/block/pass calls)
            runtime.run_script()?;

            // Call on_init if defined
            if runtime.has_hook("on_init") {
                runtime.call_hook("on_init")?;
            }
        }
        Ok(runtime)
    }

    /// Create the advanced engine with mocked input events.
    fn create_engine(
        &self,
        registry: &RemapRegistry,
        runtime: RhaiRuntime,
    ) -> AdvancedEngine<RhaiRuntime> {
        let mut engine = AdvancedEngine::new(runtime, registry.timing_config().clone());

        // Seed layer stack with registry-defined layers and mappings.
        let mut layers = registry.layers().clone();
        if let Some(base_id) = layers.layer_id_by_name("base") {
            for (key, action) in registry.mappings() {
                if let Some(layer_action) = Self::to_layer_action(action) {
                    layers.set_mapping_for_layer(base_id, key, layer_action);
                }
            }

            for (key, binding) in registry.tap_holds() {
                layers.set_mapping_for_layer(
                    base_id,
                    *key,
                    LayerAction::TapHold {
                        tap: binding.tap,
                        hold: binding.hold.clone(),
                    },
                );
            }
        }
        *engine.layers_mut() = layers;

        // Seed combos
        for combo in registry.combos().all() {
            engine
                .combos_mut()
                .register(&combo.keys, combo.action.clone());
        }

        // Seed modifiers
        engine
            .modifiers_mut()
            .clone_from(&registry.modifier_state());

        engine
    }

    fn to_layer_action(action: RemapAction) -> Option<LayerAction> {
        match action {
            RemapAction::Remap(target) => Some(LayerAction::Remap(target)),
            RemapAction::Block => Some(LayerAction::Block),
            RemapAction::Pass => None,
        }
    }

    /// Process events through the engine and collect results (including timeout ticks).
    fn process_events(
        &self,
        engine: &mut AdvancedEngine<RhaiRuntime>,
        events: &[InputEvent],
    ) -> (Vec<SimulationResult>, usize, usize, usize) {
        let mut results = Vec::new();
        let mut remapped = 0;
        let mut blocked = 0;
        let mut passed = 0;
        let mut last_timestamp = events.first().map(|e| e.timestamp_us).unwrap_or(0);

        for (idx, event) in events.iter().enumerate() {
            let mut outputs = Vec::new();
            if idx > 0 && event.timestamp_us > last_timestamp {
                outputs.extend(engine.tick(event.timestamp_us));
            }

            outputs.extend(engine.process_event(event.clone()));
            self.record_result(
                event,
                outputs,
                &mut results,
                &mut remapped,
                &mut blocked,
                &mut passed,
            );
            last_timestamp = event.timestamp_us;
        }

        (results, remapped, blocked, passed)
    }

    fn record_result(
        &self,
        event: &InputEvent,
        outputs: Vec<OutputAction>,
        results: &mut Vec<SimulationResult>,
        remapped: &mut usize,
        blocked: &mut usize,
        passed: &mut usize,
    ) {
        let mut output_strings = Vec::new();
        let mut saw_block = false;
        let mut saw_remap = false;
        let mut saw_pass = false;

        for action in outputs {
            match action {
                OutputAction::KeyDown(k) | OutputAction::KeyUp(k) => {
                    if k == event.key {
                        saw_pass = true;
                    } else {
                        saw_remap = true;
                    }
                    output_strings.push(k.name());
                }
                OutputAction::KeyTap(k) => {
                    saw_remap = true;
                    output_strings.push(format!("Tap({})", k.name()));
                }
                OutputAction::Block => {
                    saw_block = true;
                    output_strings.push("BLOCKED".to_string());
                }
                OutputAction::PassThrough => {
                    saw_pass = true;
                    output_strings.push(event.key.name());
                }
            }
        }

        if output_strings.is_empty() {
            *passed += 1;
        } else if saw_block {
            *blocked += 1;
        } else if saw_remap {
            *remapped += 1;
        } else if saw_pass {
            *passed += 1;
        }

        let primary = output_strings
            .first()
            .cloned()
            .unwrap_or_else(|| "NO_OUTPUT".to_string());

        results.push(SimulationResult {
            input: event.key.name(),
            output: primary,
            outputs: output_strings,
            pressed: event.pressed,
            timestamp_us: event.timestamp_us,
        });
    }

    fn parse_keys(&self) -> Result<Vec<ParsedKey>> {
        let mut keys = Vec::new();

        for token in self.input_keys.split(',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }

            let mut parts = token.split(':');
            let key_name = parts
                .next()
                .ok_or_else(|| anyhow::anyhow!("Empty key entry"))?;

            let key = KeyCode::from_name(key_name)
                .ok_or_else(|| anyhow::anyhow!("Unknown key: '{}'", key_name))?;

            let mut hold_ms = None;
            if let Some(kind) = parts.next() {
                if kind.eq_ignore_ascii_case("hold") {
                    let value = parts
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("Missing hold duration for '{}'", token))?;
                    let parsed = value
                        .parse::<u64>()
                        .with_context(|| format!("Invalid hold duration '{}'", value))?;
                    if parsed == 0 {
                        bail!("hold duration must be greater than 0ms");
                    }
                    hold_ms = Some(parsed);
                } else {
                    bail!("Unsupported token segment '{}' in '{}'", kind, token);
                }

                if parts.next().is_some() {
                    bail!("Too many segments in '{}'", token);
                }
            }

            keys.push(ParsedKey { key, hold_ms });
        }

        Ok(keys)
    }

    fn build_sequence_events(&self, keys: Vec<ParsedKey>) -> Result<Vec<InputEvent>> {
        let mut events = Vec::new();
        let mut timestamp = 0u64;

        for parsed in keys {
            let hold_us = self.resolve_hold_us(&parsed);
            events.push(InputEvent::key_down(parsed.key, timestamp));
            timestamp = timestamp.saturating_add(hold_us);
            events.push(InputEvent::key_up(parsed.key, timestamp));
            timestamp = timestamp.saturating_add(DEFAULT_EVENT_GAP_US);
        }

        Ok(events)
    }

    fn build_combo_events(&self, keys: Vec<ParsedKey>) -> Result<Vec<InputEvent>> {
        let mut events = Vec::new();

        for parsed in &keys {
            events.push(InputEvent::key_down(parsed.key, 0));
        }

        for parsed in keys {
            let hold_us = self.resolve_hold_us(&parsed);
            events.push(InputEvent::key_up(parsed.key, hold_us));
        }

        // Ensure deterministic ordering (down events first when timestamps match).
        events.sort_by(|a, b| {
            a.timestamp_us
                .cmp(&b.timestamp_us)
                .then_with(|| b.pressed.cmp(&a.pressed))
        });

        Ok(events)
    }

    fn resolve_hold_us(&self, parsed: &ParsedKey) -> u64 {
        parsed
            .hold_ms
            .or(self.hold_ms)
            .map(|ms| ms.saturating_mul(1_000))
            .unwrap_or(DEFAULT_EVENT_GAP_US)
    }

    /// Run the simulation and output results.
    pub async fn run(&self) -> Result<()> {
        let output = self.execute().await?;
        self.output.data(&output)?;
        Ok(())
    }
}
