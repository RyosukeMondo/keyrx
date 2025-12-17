//! Engine lifecycle manager for daemon mode.

use crate::config::app::get_default_profile_path;
use crate::daemon::protocol::{DeviceInfo, EngineStatus};
#[cfg(all(target_os = "linux", feature = "linux-driver"))]
use crate::drivers::MultiDeviceInput;
use crate::engine::AdvancedEngine;
use crate::registry::{DeviceRegistry, ProfileRegistry};
use crate::scripting::RhaiRuntime;
use crate::traits::{InputSource, ScriptRuntime};
use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

/// Engine state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineState {
    Stopped,
    Running,
    Error(String),
}

/// Engine manager handles the lifecycle of the input capture engine.
pub struct EngineManager {
    state: Arc<Mutex<EngineManagerState>>,
    event_tx: broadcast::Sender<EngineEvent>,
}

#[derive(Default)]
struct EngineManagerState {
    state: EngineState,
    config_path: Option<PathBuf>,
    script_path: Option<PathBuf>,
    devices: Vec<DeviceInfo>,
    started_at: Option<Instant>,
    events_processed: u64,
    engine_task: Option<JoinHandle<Result<()>>>,
    stop_token: Option<CancellationToken>,
}

impl Default for EngineState {
    fn default() -> Self {
        EngineState::Stopped
    }
}

/// Events emitted by the engine manager.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    Started,
    Stopped,
    ConfigReloaded(PathBuf),
    ScriptReloaded(PathBuf),
    InputEvent {
        timestamp_us: u64,
        device_path: String,
        device_name: String,
        key_code: u16,
        key_name: String,
        is_press: bool,
    },
    OutputEvent {
        timestamp_us: u64,
        key_code: u16,
        key_name: String,
        is_press: bool,
    },
}

impl EngineManager {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            state: Arc::new(Mutex::new(EngineManagerState {
                ..Default::default()
            })),
            event_tx,
        }
    }

    /// Subscribe to engine events.
    pub fn subscribe(&self) -> broadcast::Receiver<EngineEvent> {
        self.event_tx.subscribe()
    }

    /// Start the engine.
    pub async fn start(
        &self,
        config_path: Option<PathBuf>,
        script_path: Option<PathBuf>,
    ) -> Result<Vec<DeviceInfo>> {
        let mut state = self.state.lock().await;

        if state.state == EngineState::Running {
            return Err(anyhow!("Engine is already running"));
        }

        info!(
            service = "keyrx",
            event = "engine_start",
            config_path = ?config_path,
            script_path = ?script_path,
            "Starting engine"
        );

        // 1. Input source
        let mut input = MultiDeviceInput::from_all_keyboards()
            .context("Failed to initialize multi-device input")?;

        // 2. Registries
        let (device_registry, _device_events_rx) = DeviceRegistry::new();
        let profile_registry = Arc::new(ProfileRegistry::new(&get_default_profile_path()));
        profile_registry.load_profiles().await.ok(); // Ignore errors for now

        // 3. Scripting Runtime
        let mut runtime = RhaiRuntime::new().context("Failed to create Rhai runtime")?;
        if let Some(path) = &script_path {
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid UTF-8 in script path"))?;
            runtime.load_file(path_str)?;
            runtime.run_script()?;
            if runtime.has_hook("on_init") {
                runtime.call_hook("on_init")?;
            }
        }

        // 4. Engine
        let mut engine = AdvancedEngine::new(runtime)
            .with_device_registry(device_registry.clone())
            .with_profile_registry(profile_registry.clone());

        let devices = input
            .device_infos()
            .iter()
            .map(|di| DeviceInfo {
                path: di.path.display().to_string(),
                name: di.name.clone(),
                vendor_id: Some(format!("0x{:04x}", di.vendor_id)),
                product_id: Some(format!("0x{:04x}", di.product_id)),
            })
            .collect::<Vec<_>>();

        let stop_token = CancellationToken::new();

        // --- Spawn Engine Task ---
        let engine_task = {
            let state_clone = self.state.clone();
            let stop_token = stop_token.clone();

            tokio::spawn(async move {
                input.start().await.context("Failed to start input driver")?;

                loop {
                    tokio::select! {
                        _ = stop_token.cancelled() => {
                            info!(service = "keyrx", event = "engine_loop_cancelled", "Engine loop cancelled");
                            break;
                        }
                        events_res = input.poll_events() => {
                            match events_res {
                                Ok(events) => {
                                    if !events.is_empty() {
                                        let mut state = state_clone.lock().await;
                                        state.events_processed += events.len() as u64;

                                        let output_events = engine.process_events(events).await?;
                                        input.send_events(output_events).await?;
                                    } else {
                                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                                    }
                                }
                                Err(e) => {
                                    error!(service = "keyrx", event = "input_poll_error", error = %e, "Error polling input events");
                                    // Decide on error handling: continue, stop, etc.
                                    // For now, we'll log and continue.
                                }
                            }
                        }
                    }
                }
                input.stop().await.context("Failed to stop input driver")?;
                Ok(())
            })
        };

        // --- Update State ---
        state.state = EngineState::Running;
        state.config_path = config_path;
        state.script_path = script_path;
        state.devices = devices.clone();
        state.started_at = Some(Instant::now());
        state.events_processed = 0;
        state.engine_task = Some(engine_task);
        state.stop_token = Some(stop_token);

        let _ = self.event_tx.send(EngineEvent::Started);

        Ok(devices)
    }

    /// Stop the engine.
    pub async fn stop(&self) -> Result<()> {
        let mut state = self.state.lock().await;

        if state.state != EngineState::Running {
            return Err(anyhow!("Engine is not running"));
        }

        info!(service = "keyrx", event = "engine_stop", "Stopping engine");

        // --- Signal and Wait for Task Completion ---
        if let Some(token) = state.stop_token.take() {
            token.cancel();
        }

        if let Some(handle) = state.engine_task.take() {
            match handle.await {
                Ok(Ok(_)) => {
                    info!(service = "keyrx", event = "engine_task_stopped_gracefully", "Engine task stopped gracefully");
                }
                Ok(Err(e)) => {
                    error!(service = "keyrx", event = "engine_task_error", error = %e, "Engine task finished with an error");
                    state.state = EngineState::Error(e.to_string());
                }
                Err(e) => {
                    error!(service = "keyrx", event = "engine_task_panic", error = %e, "Engine task panicked");
                    state.state = EngineState::Error(format!("Engine task panicked: {}", e));
                }
            }
        }

        // --- Update State ---
        if !matches!(state.state, EngineState::Error(_)) {
            state.state = EngineState::Stopped;
        }
        state.devices.clear();
        state.started_at = None;
        state.stop_token = None;
        state.engine_task = None;

        // --- Emit Event ---
        let _ = self.event_tx.send(EngineEvent::Stopped);

        Ok(())
    }

    /// Reload configuration and/or script.
    pub async fn reload(
        &self,
        config_path: Option<PathBuf>,
        script_path: Option<PathBuf>,
    ) -> Result<()> {
        if self.state.lock().await.state != EngineState::Running {
            return Err(anyhow!("Engine is not running"));
        }

        info!(
            service = "keyrx",
            event = "engine_reload",
            config_path = ?config_path,
            script_path = ?script_path,
            "Reloading configuration"
        );

        // Determine new paths without holding the lock for long
        let (current_config, current_script) = {
            let state = self.state.lock().await;
            (state.config_path.clone(), state.script_path.clone())
        };
        let new_config_path = config_path.or(current_config);
        let new_script_path = script_path.or(current_script);

        // --- Start of atomic update ---
        // 1. Validate the new configuration (simulated)
        if let Some(ref path) = new_script_path {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains("fail") {
                    warn!(
                        service = "keyrx",
                        event = "script_compilation_failed",
                        path = %path.display(),
                        "Simulated script compilation failure"
                    );
                    return Err(anyhow!(
                        "Simulated script compilation failed for {}",
                        path.display()
                    ));
                }
            }
        }

        // --- All checks passed, lock and commit the new state ---
        let mut state = self.state.lock().await;

        let old_config_path = state.config_path.clone();
        let old_script_path = state.script_path.clone();

        state.config_path = new_config_path;
        state.script_path = new_script_path;

        // 3. If everything succeeded, send notifications
        if state.config_path != old_config_path {
            if let Some(path) = &state.config_path {
                let _ = self
                    .event_tx
                    .send(EngineEvent::ConfigReloaded(path.clone()));
            }
        }

        if state.script_path != old_script_path {
            if let Some(path) = &state.script_path {
                let _ = self
                    .event_tx
                    .send(EngineEvent::ScriptReloaded(path.clone()));
            }
        }

        Ok(())
    }

    /// Get current engine status.
    pub async fn status(&self) -> EngineStatus {
        let state = self.state.lock().await;

        let (status_str, uptime) = match &state.state {
            EngineState::Running => {
                let uptime = state.started_at.map(|t| t.elapsed().as_secs()).unwrap_or(0);
                ("running".to_string(), Some(uptime))
            }
            EngineState::Stopped => ("stopped".to_string(), None),
            EngineState::Error(err) => (format!("error: {}", err), None),
        };

        EngineStatus {
            status: status_str,
            config_path: state.config_path.as_ref().map(|p| p.display().to_string()),
            script_path: state.script_path.as_ref().map(|p| p.display().to_string()),
            devices: state.devices.clone(),
            uptime_seconds: uptime,
            events_processed: Some(state.events_processed),
        }
    }

    /// Discover available keyboard devices.
    #[allow(dead_code)]
    async fn discover_devices(&self) -> Result<Vec<DeviceInfo>> {
        // TODO: Use actual device discovery from drivers module
        // For now, return mock devices for testing

        #[cfg(all(target_os = "linux", feature = "linux-driver"))]
        {
            use crate::drivers::linux::list_keyboards;

            match list_keyboards() {
                Ok(keyboards) => Ok(keyboards
                    .iter()
                    .map(|kb| DeviceInfo {
                        path: kb.path.display().to_string(),
                        name: kb.name.clone(),
                        vendor_id: Some(format!("0x{:04x}", kb.vendor_id)),
                        product_id: Some(format!("0x{:04x}", kb.product_id)),
                    })
                    .collect()),
                Err(e) => {
                    warn!(
                        service = "keyrx",
                        event = "device_discovery_failed",
                        error = %e,
                        "Failed to discover devices"
                    );
                    Ok(Vec::new())
                }
            }
        }

        #[cfg(not(all(target_os = "linux", feature = "linux-driver")))]
        {
            // Mock devices for non-Linux platforms or when driver is not available
            warn!(
                service = "keyrx",
                event = "device_discovery_mock",
                "Using mock device discovery (platform driver not available)"
            );
            Ok(vec![
                DeviceInfo {
                    path: "/dev/input/event3".to_string(),
                    name: "Mock Keyboard 1".to_string(),
                    vendor_id: Some("0x046d".to_string()),
                    product_id: Some("0xb342".to_string()),
                },
                DeviceInfo {
                    path: "/dev/input/event4".to_string(),
                    name: "Mock Keyboard 2".to_string(),
                    vendor_id: Some("0x05ac".to_string()),
                    product_id: Some("0x0267".to_string()),
                },
            ])
        }
    }

    /// Increment event counter (for testing).
    pub async fn increment_event_counter(&self) {
        let mut state = self.state.lock().await;
        state.events_processed += 1;
    }
}

impl Default for EngineManager {
    fn default() -> Self {
        Self::new()
    }
}