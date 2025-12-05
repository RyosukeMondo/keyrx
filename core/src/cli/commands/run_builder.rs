//! Runtime and engine building for the run command.

use crate::cli::{OutputFormat, OutputWriter};
use crate::config::script_cache_dir;
use crate::engine::{AdvancedEngine, LayerAction, RemapAction};
use crate::scripting::cache::ScriptCache;
use crate::scripting::{RemapRegistry, RhaiRuntime};
use crate::traits::ScriptRuntime;
use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;
use tracing::{debug, info};
use tracing_subscriber::{fmt, prelude::*, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Serialize)]
struct CacheMetricsOutput {
    hits: u64,
    misses: u64,
    hit_rate: f64,
    evictions: u64,
    entries: usize,
    size_bytes: usize,
    max_size_bytes: usize,
    startup_micros_saved: u64,
    startup_ms_saved: f64,
}

/// Builder for preparing script runtime and engine.
pub struct RuntimeBuilder<'a> {
    script_path: Option<PathBuf>,
    debug: bool,
    output: &'a OutputWriter,
    disable_cache: bool,
    clear_cache: bool,
}

impl<'a> RuntimeBuilder<'a> {
    /// Create a new runtime builder.
    pub fn new(script_path: Option<PathBuf>, debug: bool, output: &'a OutputWriter) -> Self {
        Self {
            script_path,
            debug,
            output,
            disable_cache: false,
            clear_cache: false,
        }
    }

    /// Configure caching behavior for the runtime.
    pub fn with_cache_control(mut self, disable_cache: bool, clear_cache: bool) -> Self {
        self.disable_cache = disable_cache;
        self.clear_cache = clear_cache;
        self
    }

    /// Initialize debug logging if enabled.
    pub fn init_debug_logging(&self) -> Result<()> {
        if !self.debug {
            return Ok(());
        }

        let env_filter =
            EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("debug"))?;

        let fmt_layer = fmt::layer()
            .json()
            .flatten_event(true)
            .with_target(true)
            .with_level(true)
            .with_timer(fmt::time::SystemTime)
            .with_current_span(true)
            .with_span_list(true);

        // Try to initialize, but ignore error if already initialized
        // (main function may have already set up tracing)
        let _ = tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init();

        debug!(
            service = "keyrx",
            event = "debug_mode_enabled",
            component = "cli_run",
            format = "json",
            "Debug logging enabled"
        );

        Ok(())
    }

    fn report_cache_metrics(&self, runtime: &RhaiRuntime) {
        let Some(cache) = runtime.script_cache() else {
            return;
        };

        let stats = cache.stats();
        if stats.hits + stats.misses == 0 {
            return;
        }

        let saved_ms = stats.startup_micros_saved as f64 / 1000.0;
        match self.output.format() {
            OutputFormat::Json => {
                let summary = CacheMetricsOutput {
                    hits: stats.hits,
                    misses: stats.misses,
                    hit_rate: stats.hit_rate(),
                    evictions: stats.evictions,
                    entries: stats.entries,
                    size_bytes: stats.size_bytes,
                    max_size_bytes: stats.max_size_bytes,
                    startup_micros_saved: stats.startup_micros_saved,
                    startup_ms_saved: saved_ms,
                };
                if let Err(error) = self.output.data(&summary) {
                    self.output
                        .warning(&format!("Failed to write cache metrics: {error}"));
                }
            }
            OutputFormat::Human => {
                self.output.success(&format!(
                    "Script cache: {:.1}% hit rate (hits: {}, misses: {}), entries: {}, size: {} / {} bytes, startup saved: {:.2}ms",
                    stats.hit_rate(),
                    stats.hits,
                    stats.misses,
                    stats.entries,
                    stats.size_bytes,
                    stats.max_size_bytes,
                    saved_ms
                ));
            }
        }
    }

    /// Prepare the script runtime by loading script and calling on_init hook.
    pub fn prepare_runtime(&self) -> Result<RhaiRuntime> {
        if self.clear_cache {
            ScriptCache::new(script_cache_dir()).clear();
            self.output.success("Cleared script cache");
        }

        let mut runtime = RhaiRuntime::new()?;

        if self.disable_cache {
            runtime.disable_cache();
            self.output.warning("Script cache disabled for this run");
        }

        if let Some(path) = &self.script_path {
            self.output
                .success(&format!("Loading script: {}", path.display()));
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in path: {:?}", path))?;
            runtime.load_file(path_str)?;

            debug!(
                service = "keyrx",
                event = "run_script_start",
                component = "cli_run",
                script = %path.display(),
                "Running script top-level statements"
            );
            runtime.run_script()?;

            if runtime.has_hook("on_init") {
                debug!(
                    service = "keyrx",
                    event = "script_on_init",
                    component = "cli_run",
                    script = %path.display(),
                    "Calling on_init() hook"
                );
                runtime.call_hook("on_init")?;
                self.output.success("Script initialized (on_init called)");
            }
        }

        self.report_cache_metrics(&runtime);

        Ok(runtime)
    }

    /// Build the advanced engine from a runtime and registry.
    pub fn build_engine(
        &self,
        runtime: RhaiRuntime,
        registry: RemapRegistry,
    ) -> AdvancedEngine<RhaiRuntime> {
        let mut engine = AdvancedEngine::new(runtime, registry.timing_config().clone());

        // Seed layer mappings and tap-holds into the base layer.
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

        // Seed combos and modifiers.
        for combo in registry.combos().all() {
            engine
                .combos_mut()
                .register(&combo.keys, combo.action.clone());
        }
        engine
            .modifiers_mut()
            .clone_from(&registry.modifier_state());

        info!(
            service = "keyrx",
            event = "engine_built",
            component = "cli_run",
            "Engine built with registry mappings"
        );

        engine
    }
}

/// Convert a RemapAction to a LayerAction if applicable.
fn to_layer_action(action: RemapAction) -> Option<LayerAction> {
    match action {
        RemapAction::Remap(target) => Some(LayerAction::Remap(target)),
        RemapAction::Block => Some(LayerAction::Block),
        RemapAction::Pass => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::OutputFormat;
    use crate::engine::KeyCode;
    use crate::traits::ScriptRuntime;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn prepare_runtime_loads_script_and_on_init() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("script.rhai");

        fs::write(
            &script_path,
            r#"
remap("A", "B");

fn on_init() {
    block("CapsLock");
}
"#,
        )
        .unwrap();

        let output = OutputWriter::new(OutputFormat::Human);
        let builder = RuntimeBuilder::new(Some(script_path), false, &output);
        let runtime = builder
            .prepare_runtime()
            .expect("runtime should load script");

        assert_eq!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        );
        assert_eq!(runtime.lookup_remap(KeyCode::CapsLock), RemapAction::Block);
    }

    #[test]
    fn prepare_runtime_errors_on_invalid_path() {
        let output = OutputWriter::new(OutputFormat::Human);
        let builder = RuntimeBuilder::new(
            Some(PathBuf::from("/not/a/real/script.rhai")),
            false,
            &output,
        );

        let result = builder.prepare_runtime();
        assert!(result.is_err());
    }

    #[test]
    fn prepare_runtime_disables_cache_when_requested() {
        let output = OutputWriter::new(OutputFormat::Human);
        let builder = RuntimeBuilder::new(None, false, &output).with_cache_control(true, false);

        let runtime = builder.prepare_runtime().expect("runtime should build");
        assert!(runtime.script_cache().is_none());
    }

    #[test]
    fn to_layer_action_converts_remap() {
        assert_eq!(
            to_layer_action(RemapAction::Remap(KeyCode::A)),
            Some(LayerAction::Remap(KeyCode::A))
        );
    }

    #[test]
    fn to_layer_action_converts_block() {
        assert_eq!(
            to_layer_action(RemapAction::Block),
            Some(LayerAction::Block)
        );
    }

    #[test]
    fn to_layer_action_returns_none_for_pass() {
        assert_eq!(to_layer_action(RemapAction::Pass), None);
    }
}
