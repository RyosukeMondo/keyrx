//! Simulate command for headless testing.

use crate::cli::{OutputFormat, OutputWriter};
use crate::config::DEFAULT_EVENT_GAP_US;
use crate::engine::{
    AdvancedEngine, EngineState, HoldAction, InputEvent, KeyCode, LayerAction, OutputAction,
    PendingDecisionState, RemapAction, TimingConfig,
};
use crate::scripting::{RemapRegistry, RhaiRuntime, TapHoldBinding};
use crate::traits::ScriptRuntime;
use anyhow::{bail, Context, Result};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
struct ParsedKey {
    key: KeyCode,
    hold_ms: Option<u64>,
}

/// Information about a tap-hold key binding.
#[derive(Debug, Clone, Serialize)]
pub struct TapHoldInfo {
    /// Key output on tap (quick press and release).
    pub tap: String,
    /// Key/action on hold (press and hold beyond threshold).
    pub hold: String,
    /// Tap-hold threshold in milliseconds.
    pub threshold_ms: u32,
}

impl TapHoldInfo {
    fn from_binding(binding: &TapHoldBinding, timing: &TimingConfig) -> Self {
        let hold = match &binding.hold {
            HoldAction::Key(k) => k.name(),
            HoldAction::Modifier(id) => format!("Modifier({})", id),
            HoldAction::Layer(id) => format!("Layer({})", id),
        };
        Self {
            tap: binding.tap.name(),
            hold,
            threshold_ms: timing.tap_timeout_ms,
        }
    }
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
    /// Tap-hold info if this key has tap-hold behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tap_hold: Option<TapHoldInfo>,
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

        // Build tap-hold info map for result enrichment
        let timing = registry.timing_config().clone();
        let tap_hold_info: HashMap<KeyCode, TapHoldInfo> = registry
            .tap_holds()
            .map(|(key, binding)| (*key, TapHoldInfo::from_binding(binding, &timing)))
            .collect();

        // Create engine with mocks
        let mut engine = self.create_engine(&registry, runtime);

        // Process events and collect results
        let (results, remapped, blocked, passed) =
            self.process_events(&mut engine, &events, &tap_hold_info);
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
        tap_hold_info: &HashMap<KeyCode, TapHoldInfo>,
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
            Self::record_result(
                event,
                outputs,
                tap_hold_info,
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
        event: &InputEvent,
        outputs: Vec<OutputAction>,
        tap_hold_info: &HashMap<KeyCode, TapHoldInfo>,
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

        // Include tap-hold info if this key has tap-hold behavior (only for key-down events)
        let tap_hold = if event.pressed {
            tap_hold_info.get(&event.key).cloned()
        } else {
            None
        };

        results.push(SimulationResult {
            input: event.key.name(),
            output: primary,
            outputs: output_strings,
            pressed: event.pressed,
            timestamp_us: event.timestamp_us,
            tap_hold,
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

    /// Run interactive simulation mode (REPL-style).
    pub fn run_interactive(script_path: Option<PathBuf>, format: OutputFormat) -> Result<()> {
        let output = OutputWriter::new(format);
        let mut session = InteractiveSession::new(script_path)?;

        let mut editor = DefaultEditor::new().context("Failed to initialize readline")?;

        println!("KeyRx Interactive Simulation");
        println!("Type key names to simulate, 'help' for commands, 'quit' to exit.\n");

        if let Some(path) = &session.script_path {
            println!("Loaded script: {}\n", path.display());
        }

        loop {
            match editor.readline("simulate> ") {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    let _ = editor.add_history_entry(line);

                    match line.to_lowercase().as_str() {
                        "quit" | "exit" | "q" => break,
                        "help" | "?" => {
                            Self::print_interactive_help();
                        }
                        "state" => {
                            session.print_state(&output)?;
                        }
                        "reset" => {
                            session.reset()?;
                            println!("Engine reset to initial state.");
                        }
                        _ => {
                            session.simulate_and_print(line)?;
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C (use 'quit' to exit)");
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            }
        }

        Ok(())
    }

    fn print_interactive_help() {
        println!(
            r#"Interactive Simulation Commands:
  <key>           Simulate a key press/release (e.g., "A", "CapsLock")
  <key>:hold:<ms> Simulate a key with specific hold duration (e.g., "A:hold:300")
  <k1>,<k2>,...   Simulate multiple keys in sequence (e.g., "A,B,C")
  state           Show current engine state
  reset           Reset engine to initial state
  help, ?         Show this help message
  quit, exit, q   Exit interactive mode

Examples:
  A               Simulate pressing and releasing A
  CapsLock:hold:300  Hold CapsLock for 300ms (triggers hold behavior)
  A,B,C           Simulate A, then B, then C in sequence
"#
        );
    }
}

/// Holds interactive session state.
struct InteractiveSession {
    engine: AdvancedEngine<RhaiRuntime>,
    tap_hold_info: HashMap<KeyCode, TapHoldInfo>,
    script_path: Option<PathBuf>,
}

impl InteractiveSession {
    fn new(script_path: Option<PathBuf>) -> Result<Self> {
        let mut runtime = RhaiRuntime::new()?;

        if let Some(path) = &script_path {
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in path: {:?}", path))?;
            runtime
                .load_file(path_str)
                .with_context(|| format!("Failed to compile script '{}'", path.display()))?;
            runtime.run_script()?;

            if runtime.has_hook("on_init") {
                runtime.call_hook("on_init")?;
            }
        }

        let registry = runtime.registry().clone();
        let timing = registry.timing_config().clone();

        let tap_hold_info: HashMap<KeyCode, TapHoldInfo> = registry
            .tap_holds()
            .map(|(key, binding)| (*key, TapHoldInfo::from_binding(binding, &timing)))
            .collect();

        let engine = Self::create_engine(&registry, RhaiRuntime::new()?);

        Ok(Self {
            engine,
            tap_hold_info,
            script_path,
        })
    }

    fn create_engine(
        registry: &RemapRegistry,
        runtime: RhaiRuntime,
    ) -> AdvancedEngine<RhaiRuntime> {
        let mut engine = AdvancedEngine::new(runtime, registry.timing_config().clone());

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

        for combo in registry.combos().all() {
            engine
                .combos_mut()
                .register(&combo.keys, combo.action.clone());
        }

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

    fn simulate_and_print(&mut self, keys: &str) -> Result<()> {
        let events = self.parse_input_keys(keys)?;

        for event in events {
            let outputs = self.engine.process_event(event.clone());
            let direction = if event.pressed { "↓" } else { "↑" };

            let output_str = if outputs.is_empty() {
                "PASS".to_string()
            } else {
                outputs
                    .iter()
                    .map(Self::format_output)
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            // Show tap-hold info for key-down events if applicable
            if event.pressed {
                if let Some(th) = self.tap_hold_info.get(&event.key) {
                    println!(
                        "  {} {} → {} [tap-hold: tap={}, hold={}, threshold={}ms]",
                        direction,
                        event.key.name(),
                        output_str,
                        th.tap,
                        th.hold,
                        th.threshold_ms
                    );
                    continue;
                }
            }

            println!("  {} {} → {}", direction, event.key.name(), output_str);
        }
        println!();

        Ok(())
    }

    fn format_output(action: &OutputAction) -> String {
        match action {
            OutputAction::KeyDown(k) => format!("{}↓", k.name()),
            OutputAction::KeyUp(k) => format!("{}↑", k.name()),
            OutputAction::KeyTap(k) => format!("Tap({})", k.name()),
            OutputAction::Block => "BLOCKED".to_string(),
            OutputAction::PassThrough => "PASS".to_string(),
        }
    }

    fn parse_input_keys(&self, keys: &str) -> Result<Vec<InputEvent>> {
        let mut events = Vec::new();
        let mut timestamp = 0u64;

        for token in keys.split(',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }

            let mut parts = token.split(':');
            let key_name = parts.next().unwrap_or("");
            let key = KeyCode::from_name(key_name)
                .ok_or_else(|| anyhow::anyhow!("Unknown key: '{}'", key_name))?;

            let mut hold_us = DEFAULT_EVENT_GAP_US;
            if let Some(kind) = parts.next() {
                if kind.eq_ignore_ascii_case("hold") {
                    if let Some(ms_str) = parts.next() {
                        let ms: u64 = ms_str.parse().context("Invalid hold duration")?;
                        hold_us = ms.saturating_mul(1_000);
                    }
                }
            }

            events.push(InputEvent::key_down(key, timestamp));
            timestamp = timestamp.saturating_add(hold_us);
            events.push(InputEvent::key_up(key, timestamp));
            timestamp = timestamp.saturating_add(DEFAULT_EVENT_GAP_US);
        }

        Ok(events)
    }

    fn print_state(&self, output: &OutputWriter) -> Result<()> {
        let state = self.engine.snapshot();
        output.data(&state)?;
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        let script_path = self.script_path.clone();
        *self = Self::new(script_path)?;
        Ok(())
    }
}
