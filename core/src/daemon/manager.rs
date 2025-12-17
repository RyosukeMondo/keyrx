//! Engine lifecycle manager for daemon mode.

use crate::daemon::protocol::{DeviceInfo, EngineStatus};
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

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

struct EngineManagerState {
    state: EngineState,
    config_path: Option<PathBuf>,
    script_path: Option<PathBuf>,
    devices: Vec<DeviceInfo>,
    started_at: Option<Instant>,
    events_processed: u64,
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
                state: EngineState::Stopped,
                config_path: None,
                script_path: None,
                devices: Vec::new(),
                started_at: None,
                events_processed: 0,
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
        use_all_devices: bool,
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
            use_all_devices = use_all_devices,
            "Starting engine"
        );

        // TODO: Actually start the engine here
        // For now, this is a placeholder that simulates starting the engine

        // Discover devices
        let devices = if use_all_devices {
            self.discover_devices().await?
        } else {
            Vec::new()
        };

        info!(
            service = "keyrx",
            event = "devices_discovered",
            count = devices.len(),
            "Discovered devices"
        );

        // Update state
        state.state = EngineState::Running;
        state.config_path = config_path;
        state.script_path = script_path;
        state.devices = devices.clone();
        state.started_at = Some(Instant::now());
        state.events_processed = 0;

        // Emit event
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

        // TODO: Actually stop the engine here

        // Update state
        state.state = EngineState::Stopped;
        state.devices.clear();
        state.started_at = None;

        // Emit event
        let _ = self.event_tx.send(EngineEvent::Stopped);

        Ok(())
    }

    /// Reload configuration and/or script.
    pub async fn reload(
        &self,
        config_path: Option<PathBuf>,
        script_path: Option<PathBuf>,
    ) -> Result<()> {
        let mut state = self.state.lock().await;

        if state.state != EngineState::Running {
            return Err(anyhow!("Engine is not running"));
        }

        info!(
            service = "keyrx",
            event = "engine_reload",
            config_path = ?config_path,
            script_path = ?script_path,
            "Reloading configuration"
        );

        // --- Start of atomic update ---

        // 1. Clone the current state to be able to restore it on failure
        let old_config_path = state.config_path.clone();
        let old_script_path = state.script_path.clone();

        let mut new_config_path = state.config_path.clone();
        let mut new_script_path = state.script_path.clone();

        if let Some(path) = config_path {
            new_config_path = Some(path);
        }
        if let Some(path) = script_path {
            new_script_path = Some(path);
        }

        // 2. Simulate validation/compilation
        if let Some(ref path) = new_script_path {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains("fail") {
                    warn!(
                        service = "keyrx",
                        event = "script_compilation_failed",
                        path = %path.display(),
                        "Simulated script compilation failure"
                    );
                    // No need to rollback here as we haven't modified the state yet
                    return Err(anyhow!(
                        "Simulated script compilation failed for {}",
                        path.display()
                    ));
                }
            }
        }

        // --- All checks passed, commit the new state ---
        state.config_path = new_config_path.clone();
        state.script_path = new_script_path.clone();

        // 5. If everything succeeded, send notifications
        if state.config_path != old_config_path {
            if let Some(path) = new_config_path {
                let _ = self.event_tx.send(EngineEvent::ConfigReloaded(path));
            }
        }

        if state.script_path != old_script_path {
            if let Some(path) = new_script_path {
                let _ = self.event_tx.send(EngineEvent::ScriptReloaded(path));
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
