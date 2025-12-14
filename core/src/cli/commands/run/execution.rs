//! Execution logic for the run command.
//!
//! This module handles:
//! - Main event loops for different input modes
//! - Platform-specific driver initialization and execution
//! - Signal handling for graceful shutdown
//! - Device runtime management

use super::setup::identity_from_event;
use super::RunCommand;
use crate::cli::commands::run_builder::RuntimeBuilder;
use crate::cli::commands::run_recorder::RecordingManager;
use crate::cli::commands::run_tracer::TracingManager;
use crate::cli::OutputWriter;
use crate::definitions::DeviceDefinitionLibrary;
use crate::engine::InputEvent;
use crate::identity::DeviceIdentity;
use crate::mocks::MockInput;
use crate::registry::{
    DeviceBinding, DeviceBindings, DeviceEvent, DeviceRegistry, ProfileRegistry,
};
use crate::scripting::RhaiRuntime;
use crate::traits::InputSource;
use crate::traits::ScriptRuntime;
use anyhow::Result;
use chrono::Utc;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
#[cfg(all(target_os = "linux", feature = "linux-driver"))]
use tracing::debug;
use tracing::info;

#[cfg(all(target_os = "linux", feature = "linux-driver"))]
use crate::drivers::LinuxInput;

#[cfg(all(target_os = "linux", feature = "linux-driver"))]
use signal_hook::consts::{SIGINT, SIGTERM};
#[cfg(all(target_os = "linux", feature = "linux-driver"))]
use signal_hook::flag;

/// Runtime state for device management during engine execution.
pub struct DeviceRuntime {
    registry: DeviceRegistry,
    bindings: DeviceBindings,
    seen_devices: HashSet<DeviceIdentity>,
    #[allow(dead_code)]
    events_rx: tokio::sync::mpsc::UnboundedReceiver<DeviceEvent>,
}

impl DeviceRuntime {
    /// Create a new device runtime.
    pub fn new(output: &OutputWriter) -> Self {
        let (registry, events_rx) = DeviceRegistry::new();
        let mut bindings = DeviceBindings::new();

        if let Err(error) = bindings.load() {
            output.warning(&format!(
                "Failed to load device bindings, starting fresh: {error}"
            ));
            bindings.clear();
        }

        Self {
            registry,
            bindings,
            seen_devices: HashSet::new(),
            events_rx,
        }
    }

    /// Register a device from an input event if it has identity information.
    pub async fn register_event_device(&mut self, event: &InputEvent, output: &OutputWriter) {
        if let Some(identity) = identity_from_event(event) {
            self.register_identity(identity, output).await;
        }
    }

    /// Register a device identity with the registry and apply bindings.
    pub async fn register_identity(&mut self, identity: DeviceIdentity, output: &OutputWriter) {
        if !self.seen_devices.insert(identity.clone()) {
            return;
        }

        let _ = self.registry.register_device(identity.clone()).await;

        let binding = self.bindings.get_binding(&identity).cloned();
        let binding_to_apply = binding.unwrap_or_else(|| {
            let mut default_binding = DeviceBinding::new();
            default_binding.remap_enabled = true;
            default_binding
        });

        if let Err(error) = self
            .registry
            .set_remap_enabled(&identity, binding_to_apply.remap_enabled)
            .await
        {
            output.warning(&format!(
                "Failed to set remap state for {}: {error}",
                identity
            ));
        }

        if let Some(profile_id) = binding_to_apply.profile_id.clone() {
            if let Err(error) = self
                .registry
                .assign_profile(&identity, profile_id.clone())
                .await
            {
                output.warning(&format!(
                    "Failed to assign profile {} to {}: {error}",
                    profile_id, identity
                ));
            }
        } else {
            let _ = self.registry.unassign_profile(&identity).await;
        }

        if let Some(label) = binding_to_apply.user_label.clone() {
            if let Err(error) = self
                .registry
                .set_user_label(&identity, Some(label.clone()))
                .await
            {
                output.warning(&format!("Failed to set label for {}: {error}", identity));
            }
        }

        self.bindings.set_binding(identity, binding_to_apply);
    }

    /// Persist device bindings to storage.
    pub async fn persist(&mut self, output: &OutputWriter) {
        for state in self.registry.list_devices().await {
            let binding = DeviceBinding {
                profile_id: state.profile_id.clone(),
                remap_enabled: state.remap_enabled,
                user_label: state.identity.user_label.clone(),
                bound_at: Some(Utc::now().to_rfc3339()),
            };
            self.bindings.set_binding(state.identity.clone(), binding);
        }

        if let Err(error) = self.bindings.save() {
            output.warning(&format!("Failed to save device bindings: {error}"));
        } else {
            output.success("Saved device bindings");
        }
    }
}

impl RunCommand {
    /// Run the engine with mock input (for testing or platforms without drivers).
    pub(crate) async fn run_with_mock(
        &self,
        runtime: RhaiRuntime,
        builder: &RuntimeBuilder<'_>,
        profile_registry: Arc<ProfileRegistry>,
        device_definitions: Arc<DeviceDefinitionLibrary>,
    ) -> Result<()> {
        self.output
            .warning("Mock mode: Engine running without keyboard capture");
        self.output
            .warning("No input will be processed. Press Ctrl+C to exit.");
        self.output
            .warning("Hint: Use 'keyrx simulate' for interactive testing");

        let mut input = MockInput::new();
        let device_runtime = DeviceRuntime::new(&self.output);
        let _runtime_guard = self.install_revolutionary_runtime(
            &device_runtime.registry,
            &profile_registry,
            &device_definitions,
        )?;
        let mut registry = runtime.registry().clone();

        let timing_config = self.timing_config_from_config();
        registry.set_timing_config(timing_config.clone());

        let script_path_str = self.script_path.as_ref().map(|p| p.display().to_string());
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
                None,
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

    #[cfg(all(target_os = "linux", feature = "linux-driver"))]
    pub(crate) async fn run_with_platform_driver_deferred_init(
        &self,
        builder: &RuntimeBuilder<'_>,
        profile_registry: Arc<ProfileRegistry>,
        device_definitions: Arc<DeviceDefinitionLibrary>,
    ) -> Result<()> {
        self.output.success("Using Linux input driver");

        let mut input = self.initialize_linux_input()?;
        let device_info = input.device_info().clone();

        let mut device_runtime = DeviceRuntime::new(&self.output);
        let _runtime_guard = self.install_revolutionary_runtime(
            &device_runtime.registry,
            &profile_registry,
            &device_definitions,
        )?;
        if let Ok(serial) = crate::identity::linux::extract_serial_number(input.device_path()) {
            let identity =
                DeviceIdentity::new(device_info.vendor_id, device_info.product_id, serial);
            device_runtime
                .register_identity(identity, &self.output)
                .await;
        }

        let mut runtime = RhaiRuntime::new()?;
        if let Some(path) = &self.script_path {
            self.output
                .success(&format!("Loading script: {}", path.display()));
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in path: {:?}", path))?;
            runtime.load_file(path_str)?;
            runtime.run_script()?;

            if runtime.has_hook("on_init") {
                runtime.call_hook("on_init")?;
                self.output.success("Script initialized (on_init called)");
            }
        }

        let mut registry = runtime.registry().clone();

        let timing_config = self.timing_config_from_config();
        registry.set_timing_config(timing_config.clone());

        let script_path_str = self.script_path.as_ref().map(|p| p.display().to_string());
        let mut engine = builder
            .build_engine(runtime, registry)
            .with_device_registry(device_runtime.registry.clone());

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
        let exit_reason = loop {
            if !running.load(Ordering::SeqCst) {
                break "signal";
            }

            match input.poll_events().await {
                Ok(events) => {
                    if !events.is_empty() {
                        self.process_events(
                            &mut engine,
                            &mut input,
                            events,
                            &mut last_timestamp,
                            &mut recorder,
                            &mut seq,
                            tracer.as_ref(),
                            Some(&mut device_runtime),
                        )
                        .await?;
                    } else {
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                }
                Err(e) if e.code() == "DRIVER_DEVICE_DISCONNECTED" => {
                    break "emergency_exit";
                }
                Err(e) => return Err(e.into()),
            }
        };

        if exit_reason == "signal" {
            self.output.success("Signal received, stopping...");
        } else {
            self.output.success("Emergency exit triggered, stopping...");
        }
        input.stop().await?;
        device_runtime.persist(&self.output).await;
        recording_mgr.finish_recording(recorder)?;
        tracing_mgr.finish_tracing(tracer);
        self.output.success("Engine stopped.");
        Ok(())
    }

    #[cfg(all(target_os = "linux", feature = "linux-driver"))]
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
                Err(anyhow::anyhow!("{}", e))
            }
        }
    }

    #[cfg(all(target_os = "linux", feature = "linux-driver"))]
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

    #[cfg(all(target_os = "windows", feature = "windows-driver"))]
    pub(crate) async fn run_with_platform_driver(
        &self,
        runtime: RhaiRuntime,
        builder: &RuntimeBuilder<'_>,
        profile_registry: Arc<ProfileRegistry>,
        device_definitions: Arc<DeviceDefinitionLibrary>,
    ) -> Result<()> {
        let mut input = self.init_windows_input()?;
        let mut registry = runtime.registry().clone();

        let mut device_runtime = DeviceRuntime::new(&self.output);
        let _runtime_guard = self.install_revolutionary_runtime(
            &device_runtime.registry,
            &profile_registry,
            &device_definitions,
        )?;

        let timing_config = self.timing_config_from_config();
        registry.set_timing_config(timing_config.clone());

        let script_path_str = self.script_path.as_ref().map(|p| p.display().to_string());
        let mut engine = builder
            .build_engine(runtime, registry)
            .with_device_registry(device_runtime.registry.clone());

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
        let exit_reason = loop {
            if !running.load(Ordering::SeqCst) {
                break "signal";
            }

            match input.poll_events().await {
                Ok(events) => {
                    if !events.is_empty() {
                        self.process_events(
                            &mut engine,
                            &mut input,
                            events,
                            &mut last_timestamp,
                            &mut recorder,
                            &mut seq,
                            tracer.as_ref(),
                            Some(&mut device_runtime),
                        )
                        .await?;
                    } else {
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                }
                Err(e) if e.code() == "DRIVER_DEVICE_DISCONNECTED" => {
                    break "emergency_exit";
                }
                Err(e) => return Err(e.into()),
            }
        };

        if exit_reason == "signal" {
            self.output.success("Signal received, stopping...");
        } else {
            self.output.success("Emergency exit triggered, stopping...");
        }
        input.stop().await?;
        device_runtime.persist(&self.output).await;
        recording_mgr.finish_recording(recorder)?;
        tracing_mgr.finish_tracing(tracer);
        self.output.success("Engine stopped.");
        Ok(())
    }

    #[cfg(not(any(
        all(target_os = "linux", feature = "linux-driver"),
        all(target_os = "windows", feature = "windows-driver")
    )))]
    pub(crate) async fn run_with_platform_driver(
        &self,
        runtime: RhaiRuntime,
        builder: &RuntimeBuilder<'_>,
        profile_registry: Arc<ProfileRegistry>,
        device_definitions: Arc<DeviceDefinitionLibrary>,
    ) -> Result<()> {
        self.output
            .warning("No platform driver available for this OS, falling back to mock input");
        self.run_with_mock(runtime, builder, profile_registry, device_definitions)
            .await
    }

    #[cfg(all(target_os = "windows", feature = "windows-driver"))]
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

    #[cfg(all(target_os = "windows", feature = "windows-driver"))]
    fn spawn_ctrl_c_flag(running: Arc<AtomicBool>) {
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            running.store(false, Ordering::SeqCst);
        });
    }
}
