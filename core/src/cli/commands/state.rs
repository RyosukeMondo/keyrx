//! State inspection command.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::engine::{AdvancedEngine, LayerAction, PressedKey, RemapAction, StateSnapshot};
use crate::scripting::{RemapRegistry, RhaiRuntime};
use crate::traits::ScriptRuntime;
use anyhow::{anyhow, Context};
use serde::Serialize;
use std::path::PathBuf;

/// Inspect current engine state.
pub struct StateCommand {
    pub show_layers: bool,
    pub show_modifiers: bool,
    pub show_pending: bool,
    pub script_path: Option<PathBuf>,
    pub output: OutputWriter,
}

/// Human-oriented view of the engine state with optional sections.
#[derive(Serialize)]
struct StateView {
    pressed_keys: Vec<PressedKey>,
    version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_layers: Option<Vec<u16>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_layer: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    standard_modifiers: Option<crate::engine::state::StandardModifiers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    virtual_modifiers: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pending_count: Option<usize>,
}

impl StateView {
    fn from_state(
        state: StateSnapshot,
        show_layers: bool,
        show_modifiers: bool,
        show_pending: bool,
    ) -> Self {
        Self {
            pressed_keys: state.pressed_keys,
            version: state.version,
            active_layers: show_layers.then_some(state.active_layers),
            base_layer: show_layers.then_some(state.base_layer),
            standard_modifiers: show_modifiers.then_some(state.standard_modifiers),
            virtual_modifiers: show_modifiers.then_some(state.virtual_modifiers),
            pending_count: show_pending.then_some(state.pending_count),
        }
    }
}

impl StateCommand {
    pub fn new(
        show_layers: bool,
        show_modifiers: bool,
        show_pending: bool,
        script_path: Option<PathBuf>,
        format: OutputFormat,
    ) -> Self {
        Self {
            show_layers,
            show_modifiers,
            show_pending,
            script_path,
            output: OutputWriter::new(format),
        }
    }

    /// Collect the current engine state snapshot.
    pub fn collect_state(&self) -> anyhow::Result<StateSnapshot> {
        let runtime = self.prepare_runtime()?;
        let registry = runtime.registry().clone();
        let engine = self.create_engine(&registry, runtime);
        Ok(engine.snapshot())
    }

    pub fn run(&self) -> CommandResult<()> {
        let state = match self.collect_state() {
            Ok(s) => s,
            Err(e) => {
                return CommandResult::failure(
                    ExitCode::GeneralError,
                    format!("Failed to collect state: {}", e),
                )
            }
        };

        let result = if matches!(
            self.output.format(),
            OutputFormat::Json | OutputFormat::Yaml
        ) {
            // JSON mode returns the full StateSnapshot for programmatic consumption.
            self.output.data(&state)
        } else {
            // Human mode can optionally hide sections to reduce noise.
            let view = StateView::from_state(
                state,
                self.show_layers,
                self.show_modifiers,
                self.show_pending,
            );
            self.output.data(&view)
        };

        if let Err(e) = result {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to output state: {}", e),
            );
        }

        CommandResult::success(())
    }

    fn prepare_runtime(&self) -> anyhow::Result<RhaiRuntime> {
        let mut runtime = RhaiRuntime::new()?;

        if let Some(path) = &self.script_path {
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid UTF-8 in path: {:?}", path))?;
            runtime
                .load_file(path_str)
                .with_context(|| format!("Failed to compile script '{}'", path.display()))?;
            runtime.run_script()?;

            if runtime.has_hook("on_init") {
                runtime.call_hook("on_init")?;
            }
        }

        Ok(runtime)
    }

    fn create_engine(
        &self,
        registry: &RemapRegistry,
        runtime: RhaiRuntime,
    ) -> AdvancedEngine<RhaiRuntime> {
        let mut engine = AdvancedEngine::new(runtime, registry.timing_config().clone());

        // Seed layouts with mappings and tap-holds.
        *engine.layouts_mut() = registry.layouts().clone();
        {
            let layers = engine.layers_mut();
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
        }

        // Seed combos.
        for combo in registry.combos().all() {
            engine
                .combos_mut()
                .register(&combo.keys, combo.action.clone());
        }

        // Seed modifiers.
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
}

impl Command for StateCommand {
    fn name(&self) -> &str {
        "state"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        self.run()
    }
}
