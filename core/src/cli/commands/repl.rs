//! Interactive REPL command.

use crate::cli::{OutputFormat, OutputWriter};
use crate::engine::{AdvancedEngine, EngineState, InputEvent, KeyCode, LayerAction, RemapAction};
use crate::scripting::{RemapRegistry, RhaiRuntime};
use crate::traits::ScriptRuntime;
use anyhow::{Context, Result};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::path::PathBuf;

const HISTORY_FILE: &str = ".keyrx_repl_history";

/// Interactive REPL for KeyRx.
pub struct ReplCommand {
    output: OutputWriter,
}

impl ReplCommand {
    pub fn new(format: OutputFormat) -> Self {
        Self {
            output: OutputWriter::new(format),
        }
    }

    pub fn run(&self) -> Result<()> {
        let mut editor = DefaultEditor::new().context("Failed to initialize readline")?;
        let history_path = dirs_home().map(|p| p.join(HISTORY_FILE));

        if let Some(ref path) = history_path {
            let _ = editor.load_history(path);
        }

        let mut session = ReplSession::new()?;
        self.print_welcome();

        loop {
            let prompt = session.prompt();
            match editor.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    let _ = editor.add_history_entry(line);

                    match self.process_command(&mut session, line) {
                        CommandResult::Continue => {}
                        CommandResult::Exit => break,
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C (use 'exit' or 'quit' to leave)");
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

        if let Some(ref path) = history_path {
            let _ = editor.save_history(path);
        }

        Ok(())
    }

    fn print_welcome(&self) {
        println!("KeyRx REPL v{}", env!("CARGO_PKG_VERSION"));
        println!("Type 'help' for available commands, 'exit' to quit.\n");
    }

    fn process_command(&self, session: &mut ReplSession, line: &str) -> CommandResult {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let cmd = parts[0].to_lowercase();
        let args = parts.get(1).copied().unwrap_or("");

        match cmd.as_str() {
            "help" | "?" => {
                self.cmd_help();
                CommandResult::Continue
            }
            "exit" | "quit" | "q" => CommandResult::Exit,
            "load" | "load_script" => {
                self.cmd_load(session, args);
                CommandResult::Continue
            }
            "reload" => {
                self.cmd_reload(session);
                CommandResult::Continue
            }
            "simulate" | "sim" => {
                self.cmd_simulate(session, args);
                CommandResult::Continue
            }
            "state" => {
                self.cmd_state(session);
                CommandResult::Continue
            }
            "layers" => {
                self.cmd_layers(session);
                CommandResult::Continue
            }
            "eval" => {
                self.cmd_eval(session, args);
                CommandResult::Continue
            }
            "reset" => {
                self.cmd_reset(session);
                CommandResult::Continue
            }
            "timing" => {
                self.cmd_timing(session);
                CommandResult::Continue
            }
            _ => {
                // Try to evaluate as Rhai expression
                if line.contains('(') || line.contains('=') || line.contains(';') {
                    self.cmd_eval(session, line);
                } else {
                    println!(
                        "Unknown command: '{}'. Type 'help' for available commands.",
                        cmd
                    );
                }
                CommandResult::Continue
            }
        }
    }

    fn cmd_help(&self) {
        println!(
            r#"Available commands:
  help, ?           Show this help message
  load <path>       Load a Rhai script file
  reload            Reload the current script
  simulate <keys>   Simulate key events (e.g., "A,B,CapsLock")
  sim <keys>        Alias for simulate
  state             Show current engine state (JSON)
  layers            List defined layers and their status
  timing            Show timing configuration
  eval <code>       Evaluate Rhai expression
  reset             Reset engine to initial state
  exit, quit, q     Exit the REPL

Examples:
  load examples/capslock.rhai
  simulate A,B,C
  sim CapsLock:hold:300
  eval remap("A", "B")
  eval get_timing()
"#
        );
    }

    fn cmd_load(&self, session: &mut ReplSession, path: &str) {
        let path = path.trim();
        if path.is_empty() {
            println!("Usage: load <script_path>");
            return;
        }

        match session.load_script(path) {
            Ok(()) => println!("Loaded: {}", path),
            Err(e) => println!("Error: {}", e),
        }
    }

    fn cmd_reload(&self, session: &mut ReplSession) {
        match &session.script_path.clone() {
            Some(path) => {
                let path_str = path.display().to_string();
                match session.load_script(&path_str) {
                    Ok(()) => println!("Reloaded: {}", path_str),
                    Err(e) => println!("Error: {}", e),
                }
            }
            None => println!("No script loaded. Use 'load <path>' first."),
        }
    }

    fn cmd_simulate(&self, session: &mut ReplSession, keys: &str) {
        let keys = keys.trim();
        if keys.is_empty() {
            println!("Usage: simulate <keys>");
            println!("Example: simulate A,B,CapsLock:hold:300");
            return;
        }

        match session.simulate(keys) {
            Ok(results) => {
                for result in &results {
                    let direction = if result.pressed { "↓" } else { "↑" };
                    println!(
                        "  {} {} → {}",
                        direction,
                        result.input,
                        result.outputs.join(", ")
                    );
                }
                println!();
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    fn cmd_state(&self, session: &mut ReplSession) {
        let state = session.snapshot();
        match self.output.format() {
            OutputFormat::Json => {
                if let Err(e) = self.output.data(&state) {
                    println!("Error formatting state: {}", e);
                }
            }
            OutputFormat::Human => {
                println!("Engine State:");
                println!("  Safe mode: {}", state.safe_mode);
                println!("  Pressed keys: {:?}", state.pressed_keys);
                println!("  Active layers: {:?}", state.layers.active_layers());
                println!("  Pending decisions: {}", state.pending.len());
                if !state.pending.is_empty() {
                    for p in &state.pending {
                        println!("    - {:?}", p);
                    }
                }
            }
        }
    }

    fn cmd_layers(&self, session: &mut ReplSession) {
        let state = session.snapshot();
        let active = state.layers.active_layers();

        println!("Active Layers (bottom to top):");
        if active.is_empty() {
            println!("  (none)");
        } else {
            for name in &active {
                println!("  - {}", name);
            }
        }
    }

    fn cmd_timing(&self, session: &mut ReplSession) {
        let state = session.snapshot();
        let timing = &state.timing;

        println!("Timing Configuration:");
        println!("  tap_timeout_ms:    {}", timing.tap_timeout_ms);
        println!("  combo_timeout_ms:  {}", timing.combo_timeout_ms);
        println!("  hold_delay_ms:     {}", timing.hold_delay_ms);
        println!("  eager_tap:         {}", timing.eager_tap);
        println!("  permissive_hold:   {}", timing.permissive_hold);
        println!("  retro_tap:         {}", timing.retro_tap);
    }

    fn cmd_eval(&self, session: &mut ReplSession, code: &str) {
        let code = code.trim();
        if code.is_empty() {
            println!("Usage: eval <rhai_code>");
            return;
        }

        match session.eval(code) {
            Ok(result) => {
                if !result.is_empty() && result != "()" {
                    println!("=> {}", result);
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    fn cmd_reset(&self, session: &mut ReplSession) {
        match session.reset() {
            Ok(()) => println!("Engine reset to initial state."),
            Err(e) => println!("Error: {}", e),
        }
    }
}

enum CommandResult {
    Continue,
    Exit,
}

/// Holds REPL session state.
struct ReplSession {
    runtime: RhaiRuntime,
    engine: AdvancedEngine<RhaiRuntime>,
    script_path: Option<PathBuf>,
}

impl ReplSession {
    fn new() -> Result<Self> {
        let runtime = RhaiRuntime::new()?;
        let registry = runtime.registry().clone();
        let engine = create_engine(&registry, RhaiRuntime::new()?);

        Ok(Self {
            runtime,
            engine,
            script_path: None,
        })
    }

    fn prompt(&self) -> String {
        match &self.script_path {
            Some(p) => {
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default();
                format!("keyrx ({})> ", name)
            }
            None => "keyrx> ".to_string(),
        }
    }

    fn load_script(&mut self, path: &str) -> Result<()> {
        let path_buf = PathBuf::from(path);

        // Create fresh runtime and load script
        let mut runtime = RhaiRuntime::new()?;
        runtime
            .load_file(path)
            .with_context(|| format!("Failed to load '{}'", path))?;
        runtime.run_script()?;

        if runtime.has_hook("on_init") {
            runtime.call_hook("on_init")?;
        }

        // Rebuild engine with new registry
        let registry = runtime.registry().clone();
        self.engine = create_engine(&registry, RhaiRuntime::new()?);
        self.runtime = runtime;
        self.script_path = Some(path_buf);

        Ok(())
    }

    fn simulate(&mut self, keys: &str) -> Result<Vec<SimulationResult>> {
        let events = parse_input_keys(keys)?;
        let mut results = Vec::new();

        for event in events {
            let outputs = self.engine.process_event(event.clone());
            let output_strings: Vec<String> = outputs.iter().map(|o| format!("{:?}", o)).collect();

            results.push(SimulationResult {
                input: event.key.name(),
                outputs: if output_strings.is_empty() {
                    vec!["PASS".to_string()]
                } else {
                    output_strings
                },
                pressed: event.pressed,
            });
        }

        Ok(results)
    }

    fn snapshot(&self) -> EngineState {
        self.engine.snapshot()
    }

    fn eval(&mut self, code: &str) -> Result<String> {
        // Execute on the runtime
        self.runtime.execute(code)?;

        // Rebuild engine to pick up any changes
        let registry = self.runtime.registry().clone();
        self.engine = create_engine(&registry, RhaiRuntime::new()?);

        Ok("()".to_string())
    }

    fn reset(&mut self) -> Result<()> {
        // Reload from current script or create fresh
        if let Some(path) = self.script_path.clone() {
            self.load_script(&path.display().to_string())
        } else {
            *self = Self::new()?;
            Ok(())
        }
    }
}

struct SimulationResult {
    input: String,
    outputs: Vec<String>,
    pressed: bool,
}

fn create_engine(registry: &RemapRegistry, runtime: RhaiRuntime) -> AdvancedEngine<RhaiRuntime> {
    let mut engine = AdvancedEngine::new(runtime, registry.timing_config().clone());

    // Seed layer stack with registry-defined layers and mappings
    let mut layers = registry.layers().clone();
    if let Some(base_id) = layers.layer_id_by_name("base") {
        for (key, action) in registry.mappings() {
            if let Some(layer_action) = to_layer_action(action) {
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

fn parse_input_keys(keys: &str) -> Result<Vec<InputEvent>> {
    let mut events = Vec::new();
    let mut timestamp = 0u64;
    const GAP_US: u64 = 1_000;

    for token in keys.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }

        let mut parts = token.split(':');
        let key_name = parts.next().unwrap_or("");
        let key = KeyCode::from_name(key_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown key: '{}'", key_name))?;

        let mut hold_us = GAP_US;
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
        timestamp = timestamp.saturating_add(GAP_US);
    }

    Ok(events)
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}
