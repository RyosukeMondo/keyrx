//! Engine run command.
//!
//! This module implements the `keyrx run` command for running the keyboard remapping engine.
//! It supports multiple input drivers (Linux, Windows, mock) and provides session recording
//! and OpenTelemetry tracing capabilities.
//!
//! # Submodules
//!
//! - [`setup`]: Engine initialization, configuration loading, device definitions
//! - [`execution`]: Main event loops, platform-specific execution, signal handling

mod execution;
mod setup;

use super::run_builder::RuntimeBuilder;
use super::run_recorder::RecordingContext;
use crate::cli::{Command, CommandContext, CommandResult, OutputFormat, OutputWriter};
use crate::config::Config;
use crate::engine::{AdvancedEngine, EngineTracer, EventRecorder, InputEvent, TimingConfig};
use crate::scripting::RhaiRuntime;
use crate::traits::InputSource;
use anyhow::Result;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::instrument;

pub use execution::DeviceRuntime;

/// Run the engine in headless mode.
pub struct RunCommand {
    pub script_path: Option<PathBuf>,
    pub debug: bool,
    pub use_mock: bool,
    pub device_path: Option<PathBuf>,
    pub output: OutputWriter,
    /// Optional limit for mock run duration (used for tests to avoid hanging).
    pub mock_run_limit: Option<Duration>,
    /// Optional path to record session to (.krx file).
    pub record_path: Option<PathBuf>,
    /// Optional path to export OpenTelemetry traces to.
    pub trace_path: Option<PathBuf>,
    /// Runtime configuration loaded from file.
    pub config: Option<Config>,
    /// Validate script and exit immediately without running engine.
    pub validate_only: bool,
    /// Disable script cache for this run.
    pub disable_cache: bool,
    /// Clear script cache before running.
    pub clear_cache: bool,
}

impl RunCommand {
    pub fn new(
        script_path: Option<PathBuf>,
        debug: bool,
        use_mock: bool,
        device_path: Option<PathBuf>,
        format: OutputFormat,
    ) -> Self {
        Self {
            script_path,
            debug,
            use_mock,
            device_path,
            output: OutputWriter::new(format),
            mock_run_limit: None,
            record_path: None,
            trace_path: None,
            config: None,
            validate_only: false,
            disable_cache: false,
            clear_cache: false,
        }
    }

    /// Set validate-only mode (load script, validate, then exit).
    pub fn with_validate_only(mut self, validate_only: bool) -> Self {
        self.validate_only = validate_only;
        self
    }

    /// Set the path for session recording.
    pub fn with_record_path(mut self, path: Option<PathBuf>) -> Self {
        self.record_path = path;
        self
    }

    /// Set the path for OpenTelemetry trace export.
    pub fn with_trace_path(mut self, path: Option<PathBuf>) -> Self {
        self.trace_path = path;
        self
    }

    /// Configure cache handling behavior.
    pub fn with_cache_options(mut self, disable_cache: bool, clear_cache: bool) -> Self {
        self.disable_cache = disable_cache;
        self.clear_cache = clear_cache;
        self
    }

    /// Set an optional maximum runtime for mock runs (primarily for tests).
    pub fn with_mock_run_limit(mut self, duration: Duration) -> Self {
        self.mock_run_limit = Some(duration);
        self
    }

    /// Set the runtime configuration.
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Get the timing configuration from loaded config, or use defaults.
    pub(crate) fn timing_config_from_config(&self) -> TimingConfig {
        if let Some(ref config) = self.config {
            let defaults = TimingConfig::default();
            TimingConfig {
                tap_timeout_ms: config.timing.tap_timeout_ms,
                combo_timeout_ms: config.timing.combo_timeout_ms,
                hold_delay_ms: config.timing.hold_delay_ms,
                eager_tap: defaults.eager_tap,
                permissive_hold: defaults.permissive_hold,
                retro_tap: defaults.retro_tap,
            }
        } else {
            TimingConfig::default()
        }
    }

    /// Main entry point for running the engine.
    pub async fn run(&self) -> Result<()> {
        let builder = RuntimeBuilder::new(self.script_path.clone(), self.debug, &self.output)
            .with_cache_control(self.disable_cache, self.clear_cache);
        builder.init_debug_logging()?;

        if self.validate_only {
            self.output.success("Validating script...");
            let _runtime = builder.prepare_runtime()?;
            self.output.success("Script validation successful");
            return Ok(());
        }

        let profile_registry = self.prepare_profile_registry().await;
        let device_definitions = self.load_device_definitions();
        self.output.success("Starting KeyRx engine...");

        if self.use_mock {
            let runtime = builder.prepare_runtime()?;
            self.run_with_mock(
                runtime,
                &builder,
                profile_registry,
                device_definitions.clone(),
            )
            .await
        } else {
            #[cfg(all(target_os = "linux", feature = "linux-driver"))]
            {
                self.run_with_platform_driver_deferred_init(
                    &builder,
                    profile_registry.clone(),
                    device_definitions.clone(),
                )
                .await
            }
            #[cfg(not(all(target_os = "linux", feature = "linux-driver")))]
            {
                let runtime = builder.prepare_runtime()?;
                self.run_with_platform_driver(
                    runtime,
                    &builder,
                    profile_registry,
                    device_definitions,
                )
                .await
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(
        level = "trace",
        skip(
            self,
            engine,
            input,
            events,
            last_timestamp,
            recorder,
            seq,
            tracer,
            device_runtime
        ),
        fields(event_count = events.len())
    )]
    pub(crate) async fn process_events<I: InputSource>(
        &self,
        engine: &mut AdvancedEngine<RhaiRuntime>,
        input: &mut I,
        events: Vec<InputEvent>,
        last_timestamp: &mut u64,
        recorder: &mut Option<EventRecorder>,
        seq: &mut u64,
        tracer: Option<&EngineTracer>,
        mut device_runtime: Option<&mut DeviceRuntime>,
    ) -> Result<()> {
        for event in events {
            if let Some(runtime) = device_runtime.as_deref_mut() {
                runtime.register_event_device(&event, &self.output).await;
            }

            if event.timestamp_us > *last_timestamp {
                for action in engine.tick(event.timestamp_us) {
                    input.send_output(action).await?;
                }
                *last_timestamp = event.timestamp_us;
            }

            let process_start = Instant::now();
            let outputs = engine.process_event_traced(event.clone(), tracer);

            let mut ctx = RecordingContext::new(recorder, seq);
            ctx.record_event(&event, &outputs, engine, process_start);

            for action in outputs {
                input.send_output(action).await?;
            }
        }
        Ok(())
    }
}

impl Command for RunCommand {
    fn name(&self) -> &str {
        "run"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        let result = tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(self.run())
        });

        match result {
            Ok(()) => CommandResult::success(()),
            Err(err) => {
                use crate::cli::HasExitCode;
                let exit_code = err.exit_code();
                CommandResult::failure(exit_code, format!("{err:#}"))
            }
        }
    }
}
