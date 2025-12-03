//! Engine run command.

use super::run_builder::RuntimeBuilder;
use super::run_recorder::{RecordingContext, RecordingManager};
use super::run_tracer::TracingManager;
use crate::cli::{OutputFormat, OutputWriter};
use crate::discovery::{DeviceId, DeviceRegistry, DiscoveryReason, RegistryEntry, RegistryStatus};
use crate::drivers::DeviceInfo;
use crate::engine::{AdvancedEngine, EngineTracer, EventRecorder, InputEvent};
use crate::mocks::MockInput;
use crate::scripting::RhaiRuntime;
use crate::traits::InputSource;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, instrument, warn};

#[cfg(target_os = "linux")]
use crate::drivers::LinuxInput;

#[cfg(target_os = "linux")]
use signal_hook::consts::{SIGINT, SIGTERM};
#[cfg(target_os = "linux")]
use signal_hook::flag;

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
        }
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

    /// Set an optional maximum runtime for mock runs (primarily for tests).
    pub fn with_mock_run_limit(mut self, duration: Duration) -> Self {
        self.mock_run_limit = Some(duration);
        self
    }

    pub async fn run(&self) -> Result<()> {
        let builder = RuntimeBuilder::new(self.script_path.clone(), self.debug, &self.output);
        builder.init_debug_logging()?;

        self.output.success("Starting KeyRx engine...");

        let runtime = builder.prepare_runtime()?;

        if self.use_mock {
            self.run_with_mock(runtime, &builder).await
        } else {
            self.run_with_platform_driver(runtime, &builder).await
        }
    }

    fn load_device_profile(&self, device_info: &DeviceInfo) -> RegistryEntry {
        let mut registry = DeviceRegistry::new();
        registry.load_or_default(DeviceId::new(device_info.vendor_id, device_info.product_id))
    }

    fn report_profile_status(&self, device_info: &DeviceInfo, entry: &RegistryEntry) {
        match &entry.status {
            RegistryStatus::Ready => {
                self.output.success(&format!(
                    "Loaded device profile for {:04x}:{:04x} from {}",
                    device_info.vendor_id,
                    device_info.product_id,
                    entry.path.display()
                ));
            }
            RegistryStatus::NeedsDiscovery(reason) => {
                let reason_text = describe_reason(reason);
                self.output.warning(&format!(
                    "Using default profile for {:04x}:{:04x} ({}). Discovery recommended: {}",
                    device_info.vendor_id, device_info.product_id, device_info.name, reason_text
                ));
                warn!(
                    service = "keyrx",
                    event = "discovery_prompt",
                    component = "cli_run",
                    vendor_id = device_info.vendor_id,
                    product_id = device_info.product_id,
                    path = %entry.path.display(),
                    reason = ?reason,
                    "Profile missing or invalid, discovery needed"
                );
            }
        }
    }

    async fn run_with_mock(
        &self,
        runtime: RhaiRuntime,
        builder: &RuntimeBuilder<'_>,
    ) -> Result<()> {
        self.output
            .success("Using mock input (no real keyboard interception)");

        let mut input = MockInput::new();
        let registry = runtime.registry().clone();
        let script_path_str = self.script_path.as_ref().map(|p| p.display().to_string());
        let timing_config = registry.timing_config().clone();
        let mut engine = builder.build_engine(runtime, registry);

        let recording_mgr = RecordingManager::new(self.record_path.clone(), &self.output);
        let mut recorder =
            recording_mgr.create_recorder(&engine, script_path_str, timing_config)?;
        let mut seq = 0u64;

        let tracing_mgr = TracingManager::new(self.trace_path.clone(), &self.output);
        let tracer = tracing_mgr.create_tracer()?;

        input.start().await?;

        self.output.success("Engine started. Press Ctrl+C to stop.");
        info!(
            service = "keyrx",
            event = "engine_started",
            component = "cli_run",
            driver = "mock",
            "Engine running with mock input"
        );

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            r.store(false, Ordering::SeqCst);
        });

        let start = Instant::now();
        let mut last_timestamp = 0u64;
        while running.load(Ordering::SeqCst) {
            if let Some(limit) = self.mock_run_limit {
                if start.elapsed() >= limit {
                    running.store(false, Ordering::SeqCst);
                    break;
                }
            }

            let events = input.poll_events().await?;
            self.process_events(
                &mut engine,
                &mut input,
                events,
                &mut last_timestamp,
                &mut recorder,
                &mut seq,
                tracer.as_ref(),
            )
            .await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        input.stop().await?;
        recording_mgr.finish_recording(recorder)?;
        tracing_mgr.finish_tracing(tracer);
        self.output.success("Engine stopped.");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn run_with_platform_driver(
        &self,
        runtime: RhaiRuntime,
        builder: &RuntimeBuilder<'_>,
    ) -> Result<()> {
        self.output.success("Using Linux input driver");

        let mut input = self.initialize_linux_input()?;
        let device_info = input.device_info().clone();
        let profile_entry = self.load_device_profile(&device_info);
        self.report_profile_status(&device_info, &profile_entry);

        let registry = runtime.registry().clone();
        let script_path_str = self.script_path.as_ref().map(|p| p.display().to_string());
        let timing_config = registry.timing_config().clone();
        let mut engine = builder.build_engine(runtime, registry);

        let recording_mgr = RecordingManager::new(self.record_path.clone(), &self.output);
        let mut recorder =
            recording_mgr.create_recorder(&engine, script_path_str, timing_config)?;
        let mut seq = 0u64;

        let tracing_mgr = TracingManager::new(self.trace_path.clone(), &self.output);
        let tracer = tracing_mgr.create_tracer()?;

        input.start().await?;

        self.output.success("Engine started. Press Ctrl+C to stop.");
        info!(
            service = "keyrx",
            event = "engine_started",
            component = "cli_run",
            driver = "linux",
            "Engine running with Linux input driver"
        );

        let running = self.setup_signal_handlers()?;

        let mut last_timestamp = 0u64;
        while running.load(Ordering::SeqCst) {
            let events = input.poll_events().await?;
            if !events.is_empty() {
                self.process_events(
                    &mut engine,
                    &mut input,
                    events,
                    &mut last_timestamp,
                    &mut recorder,
                    &mut seq,
                    tracer.as_ref(),
                )
                .await?;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        self.output.success("Signal received, stopping...");
        input.stop().await?;
        recording_mgr.finish_recording(recorder)?;
        tracing_mgr.finish_tracing(tracer);
        self.output.success("Engine stopped.");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn initialize_linux_input(&self) -> Result<LinuxInput> {
        if let Some(ref device) = self.device_path {
            self.output
                .success(&format!("Using device: {}", device.display()));
        } else {
            self.output.success("Auto-detecting keyboard device...");
        }

        match LinuxInput::new(self.device_path.clone()) {
            Ok(input) => {
                self.output.success(&format!(
                    "Opened keyboard: {}",
                    input.device_path().display()
                ));
                Ok(input)
            }
            Err(e) => {
                self.output
                    .error(&format!("Failed to initialize driver: {e:#}"));
                Err(e)
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn setup_signal_handlers(&self) -> Result<Arc<AtomicBool>> {
        let running = Arc::new(AtomicBool::new(true));

        flag::register(SIGINT, running.clone())
            .map_err(|e| anyhow::anyhow!("Failed to register SIGINT handler: {e}"))?;
        flag::register(SIGTERM, running.clone())
            .map_err(|e| anyhow::anyhow!("Failed to register SIGTERM handler: {e}"))?;

        debug!(
            service = "keyrx",
            event = "signal_handlers_registered",
            component = "cli_run",
            signals = "SIGINT,SIGTERM",
            "Signal handlers registered"
        );

        Ok(running)
    }

    #[cfg(target_os = "windows")]
    async fn run_with_platform_driver(
        &self,
        runtime: RhaiRuntime,
        builder: &RuntimeBuilder<'_>,
    ) -> Result<()> {
        let mut input = self.init_windows_input()?;
        let registry = runtime.registry().clone();
        let script_path_str = self.script_path.as_ref().map(|p| p.display().to_string());
        let timing_config = registry.timing_config().clone();
        let mut engine = builder.build_engine(runtime, registry);

        let recording_mgr = RecordingManager::new(self.record_path.clone(), &self.output);
        let mut recorder =
            recording_mgr.create_recorder(&engine, script_path_str, timing_config)?;
        let mut seq = 0u64;

        let tracing_mgr = TracingManager::new(self.trace_path.clone(), &self.output);
        let tracer = tracing_mgr.create_tracer()?;

        input.start().await?;

        self.output.success("Engine started. Press Ctrl+C to stop.");
        info!(
            service = "keyrx",
            event = "engine_started",
            component = "cli_run",
            driver = "windows",
            "Engine running with Windows input driver"
        );

        let running = Arc::new(AtomicBool::new(true));
        Self::spawn_ctrl_c_flag(running.clone());

        let mut last_timestamp = 0u64;
        while running.load(Ordering::SeqCst) {
            let events = input.poll_events().await?;
            if !events.is_empty() {
                self.process_events(
                    &mut engine,
                    &mut input,
                    events,
                    &mut last_timestamp,
                    &mut recorder,
                    &mut seq,
                    tracer.as_ref(),
                )
                .await?;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        input.stop().await?;
        recording_mgr.finish_recording(recorder)?;
        tracing_mgr.finish_tracing(tracer);
        self.output.success("Engine stopped.");
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    async fn run_with_platform_driver(
        &self,
        runtime: RhaiRuntime,
        builder: &RuntimeBuilder<'_>,
    ) -> Result<()> {
        self.output
            .warning("No platform driver available for this OS, falling back to mock input");
        self.run_with_mock(runtime, builder).await
    }

    #[cfg(target_os = "windows")]
    fn init_windows_input(&self) -> Result<crate::drivers::WindowsInput> {
        use crate::drivers::WindowsInput;

        self.output.success("Using Windows input driver");
        if self.device_path.is_some() {
            self.output.warning(
                "Note: --device is ignored on Windows (uses global keyboard hook for all keyboards)",
            );
        }

        match WindowsInput::new() {
            Ok(input) => {
                self.output
                    .success("Initialized Windows low-level keyboard hook");
                Ok(input)
            }
            Err(e) => {
                self.output
                    .error(&format!("Failed to initialize driver: {e:#}"));
                Err(e)
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn spawn_ctrl_c_flag(running: Arc<AtomicBool>) {
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            running.store(false, Ordering::SeqCst);
        });
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(
        level = "trace",
        skip(self, engine, input, events, last_timestamp, recorder, seq, tracer),
        fields(event_count = events.len())
    )]
    async fn process_events<I: InputSource>(
        &self,
        engine: &mut AdvancedEngine<RhaiRuntime>,
        input: &mut I,
        events: Vec<InputEvent>,
        last_timestamp: &mut u64,
        recorder: &mut Option<EventRecorder>,
        seq: &mut u64,
        tracer: Option<&EngineTracer>,
    ) -> Result<()> {
        for event in events {
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

fn describe_reason(reason: &DiscoveryReason) -> String {
    match reason {
        DiscoveryReason::MissingProfile => "no profile found on disk".to_string(),
        DiscoveryReason::ParseError => "stored profile is corrupt".to_string(),
        DiscoveryReason::SchemaMismatch { expected, found } => format!(
            "profile schema mismatch (expected {}, found {})",
            expected, found
        ),
        DiscoveryReason::IoError(msg) => format!("I/O error loading profile: {msg}"),
    }
}

// Tests for RuntimeBuilder are in run_builder.rs
