//! Engine run command.

use crate::cli::{OutputFormat, OutputWriter};
use crate::discovery::{DeviceId, DeviceRegistry, DiscoveryReason, RegistryEntry, RegistryStatus};
use crate::drivers::DeviceInfo;
use crate::engine::{AdvancedEngine, InputEvent, LayerAction, RemapAction};
use crate::mocks::MockInput;
use crate::scripting::{RemapRegistry, RhaiRuntime};
use crate::traits::{InputSource, ScriptRuntime};
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, instrument, warn};
use tracing_subscriber::{fmt, prelude::*, util::SubscriberInitExt, EnvFilter};

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
        }
    }

    /// Set an optional maximum runtime for mock runs (primarily for tests).
    pub fn with_mock_run_limit(mut self, duration: Duration) -> Self {
        self.mock_run_limit = Some(duration);
        self
    }

    pub async fn run(&self) -> Result<()> {
        // Initialize tracing if debug mode
        if self.debug {
            self.init_debug_logging()?;
            debug!(
                service = "keyrx",
                event = "debug_mode_enabled",
                component = "cli_run",
                format = "json",
                "Debug logging enabled"
            );
        }

        self.output.success("Starting KeyRx engine...");

        let runtime = self.prepare_runtime()?;

        // Run with appropriate input source
        if self.use_mock {
            self.run_with_mock(runtime).await
        } else {
            self.run_with_platform_driver(runtime).await
        }
    }

    fn prepare_runtime(&self) -> Result<RhaiRuntime> {
        let mut runtime = RhaiRuntime::new()?;

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

        Ok(runtime)
    }

    fn init_debug_logging(&self) -> Result<()> {
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

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| anyhow!("failed to initialize tracing subscriber: {e}"))?;

        Ok(())
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
                let reason_text = self.describe_reason(reason);
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

    fn describe_reason(&self, reason: &DiscoveryReason) -> String {
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

    async fn run_with_mock(&self, runtime: RhaiRuntime) -> Result<()> {
        self.output
            .success("Using mock input (no real keyboard interception)");

        let mut input = MockInput::new();
        let registry = runtime.registry().clone();
        let mut engine = self.build_engine(runtime, registry);
        input.start().await?;

        self.output.success("Engine started. Press Ctrl+C to stop.");
        info!(
            service = "keyrx",
            event = "engine_started",
            component = "cli_run",
            driver = "mock",
            "Engine running with mock input"
        );

        // Set up Ctrl+C handler
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
            self.process_events_with_engine(&mut engine, &mut input, events, &mut last_timestamp)
                .await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        input.stop().await?;
        self.output.success("Engine stopped.");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn run_with_platform_driver(&self, runtime: RhaiRuntime) -> Result<()> {
        self.output.success("Using Linux input driver");

        // Show which device we're using and initialize input
        let mut input = self.initialize_linux_input()?;
        let device_info = input.device_info().clone();
        let profile_entry = self.load_device_profile(&device_info);
        self.report_profile_status(&device_info, &profile_entry);

        let registry = runtime.registry().clone();
        let mut engine = self.build_engine(runtime, registry);
        input.start().await?;

        self.output.success("Engine started. Press Ctrl+C to stop.");
        info!(
            service = "keyrx",
            event = "engine_started",
            component = "cli_run",
            driver = "linux",
            "Engine running with Linux input driver"
        );

        // Set up graceful shutdown with signal handlers
        let running = self.setup_signal_handlers()?;

        // Run event loop until interrupted
        let mut last_timestamp = 0u64;
        while running.load(Ordering::SeqCst) {
            let events = input.poll_events().await?;
            if !events.is_empty() {
                self.process_events_with_engine(
                    &mut engine,
                    &mut input,
                    events,
                    &mut last_timestamp,
                )
                .await?;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        self.output.success("Signal received, stopping...");
        input.stop().await?;
        self.output.success("Engine stopped.");
        Ok(())
    }

    /// Initialize the Linux input driver with the configured device path.
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

    /// Set up signal handlers for graceful shutdown on SIGINT and SIGTERM.
    #[cfg(target_os = "linux")]
    fn setup_signal_handlers(&self) -> Result<Arc<AtomicBool>> {
        // Set up graceful shutdown using signal-hook for SIGINT and SIGTERM
        // This ensures clean keyboard release even when killed by systemd/init
        let running = Arc::new(AtomicBool::new(true));

        // Register signal handlers for both SIGINT (Ctrl+C) and SIGTERM (kill/systemd)
        // signal-hook uses a single Arc<AtomicBool> and sets it to false on signal
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
    async fn run_with_platform_driver(&self, runtime: RhaiRuntime) -> Result<()> {
        let input = self.init_windows_input()?;
        let mut input = input;
        let registry = runtime.registry().clone();
        let mut engine = self.build_engine(runtime, registry);
        input.start().await?;

        self.output.success("Engine started. Press Ctrl+C to stop.");
        info!(
            service = "keyrx",
            event = "engine_started",
            component = "cli_run",
            driver = "windows",
            "Engine running with Windows input driver"
        );

        // Set up graceful shutdown
        let running = Arc::new(AtomicBool::new(true));
        Self::spawn_ctrl_c_flag(running.clone());

        // Run event loop until interrupted
        let mut last_timestamp = 0u64;
        while running.load(Ordering::SeqCst) {
            let events = input.poll_events().await?;
            if !events.is_empty() {
                self.process_events_with_engine(
                    &mut engine,
                    &mut input,
                    events,
                    &mut last_timestamp,
                )
                .await?;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        input.stop().await?;
        self.output.success("Engine stopped.");
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    async fn run_with_platform_driver(&self, runtime: RhaiRuntime) -> Result<()> {
        self.output
            .warning("No platform driver available for this OS, falling back to mock input");
        self.run_with_mock(runtime).await
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

    fn build_engine(
        &self,
        runtime: RhaiRuntime,
        registry: RemapRegistry,
    ) -> AdvancedEngine<RhaiRuntime> {
        let mut engine = AdvancedEngine::new(runtime, registry.timing_config().clone());

        // Seed layer mappings and tap-holds into the base layer.
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

        // Seed combos and modifiers.
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

    #[instrument(
        level = "trace",
        skip(self, engine, input, events, last_timestamp),
        fields(event_count = events.len())
    )]
    async fn process_events_with_engine<I: InputSource>(
        &self,
        engine: &mut AdvancedEngine<RhaiRuntime>,
        input: &mut I,
        events: Vec<InputEvent>,
        last_timestamp: &mut u64,
    ) -> Result<()> {
        for event in events {
            if event.timestamp_us > *last_timestamp {
                for action in engine.tick(event.timestamp_us) {
                    input.send_output(action).await?;
                }
                *last_timestamp = event.timestamp_us;
            }

            let outputs = engine.process_event(event);
            for action in outputs {
                input.send_output(action).await?;
            }
        }
        Ok(())
    }

    fn to_layer_action(action: RemapAction) -> Option<LayerAction> {
        match action {
            RemapAction::Remap(target) => Some(LayerAction::Remap(target)),
            RemapAction::Block => Some(LayerAction::Block),
            RemapAction::Pass => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{KeyCode, RemapAction};
    use std::fs;
    use std::path::PathBuf;
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

        let cmd = RunCommand::new(Some(script_path), false, true, None, OutputFormat::Human);
        let runtime = cmd.prepare_runtime().expect("runtime should load script");

        assert_eq!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        );
        assert_eq!(runtime.lookup_remap(KeyCode::CapsLock), RemapAction::Block);
    }

    #[test]
    fn prepare_runtime_errors_on_invalid_path() {
        let cmd = RunCommand::new(
            Some(PathBuf::from("/not/a/real/script.rhai")),
            false,
            true,
            None,
            OutputFormat::Human,
        );

        let result = cmd.prepare_runtime();
        assert!(result.is_err());
    }
}
