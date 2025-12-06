//! Linux input driver built on evdev (capture) and uinput (injection).
//! Permissions: read `/dev/input/event*`, write `/dev/uinput`, and load the `uinput` kernel module.
//! Architecture: blocking evdev reader thread feeds the async engine via `crossbeam_channel`, while the uinput writer injects remapped output.
mod device_info;
mod discovery;
mod helpers;
mod keymap;
mod reader;
mod writer;

/// Safety wrappers for Linux driver operations.
///
/// Contains safe abstractions over evdev and uinput operations.
pub mod safety;
use crate::config::UINPUT_PATH;
use crate::drivers::common::cache::{KeymapCache, LruKeymapCache};
use crate::drivers::{DeviceInfo, KeyInjector};
use crate::engine::{InputEvent, OutputAction};
use crate::errors::{driver::*, KeyrxError};
use crate::metrics::MetricsCollector;
use crate::traits::InputSource;
use crate::{bail_keyrx, keyrx_err};
use async_trait::async_trait;
use crossbeam_channel::Sender;
use device_info::try_get_keyboard_info;
pub use discovery::list_keyboards;
use reader::EvdevReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use tracing::{debug, warn};
pub use writer::UinputWriter;
const UINPUT_NOT_FOUND_HELP: &str = "uinput device not found at /dev/uinput\n\n\
 Remediation:\n  \
 1. Load the uinput kernel module: sudo modprobe uinput\n  \
 2. If that fails, check if uinput is built into your kernel\n  \
 3. Ensure your kernel supports CONFIG_INPUT_UINPUT";

#[allow(dead_code)]
const UINPUT_PERMISSION_DENIED_HELP: &str = "Permission denied accessing /dev/uinput\n\n\
 Remediation:\n  \
 1. Add your user to the 'input' group: sudo usermod -aG input $USER\n  \
 2. Create a udev rule for uinput access:\n     \
    echo 'KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"input\"' | \
    sudo tee /etc/udev/rules.d/99-uinput.rules\n  \
 3. Reload udev rules: sudo udevadm control --reload-rules && \
    sudo udevadm trigger\n  \
 4. Log out and log back in for group changes to take effect\n  \
 5. Alternatively, run with sudo (not recommended for regular use)";

#[allow(dead_code)]
const UINPUT_ACCESS_FAILED_HELP: &str = "Remediation:\n  \
 1. Check device permissions: ls -la /dev/uinput\n  \
 2. Check if you have read/write access to the device\n  \
 3. Ensure the uinput module is loaded: lsmod | grep uinput";
pub struct LinuxInput {
    reader_handle: Option<JoinHandle<()>>,
    injector: Box<dyn KeyInjector>,
    rx: crossbeam_channel::Receiver<InputEvent>,
    tx: Sender<InputEvent>,
    running: Arc<AtomicBool>,
    device_path: PathBuf,
    device_info: DeviceInfo,
    panic_error: Arc<AtomicBool>,
    metrics: Arc<dyn MetricsCollector>,
    cache: Arc<LruKeymapCache>,
}
impl LinuxInput {
    pub fn new(device_path: Option<PathBuf>) -> Result<Self, KeyrxError> {
        let injector = UinputWriter::new()
            .map_err(|e| e.with_context("operation", "Failed to create uinput writer"))?;
        Self::new_with_injector(device_path, Box::new(injector))
    }

    pub fn new_with_metrics(
        device_path: Option<PathBuf>,
        metrics: Arc<dyn MetricsCollector>,
    ) -> Result<Self, KeyrxError> {
        let injector = UinputWriter::new()
            .map_err(|e| e.with_context("operation", "Failed to create uinput writer"))?;
        Self::new_with_injector_and_metrics(device_path, Box::new(injector), metrics)
    }

    pub fn new_with_injector(
        device_path: Option<PathBuf>,
        injector: Box<dyn KeyInjector>,
    ) -> Result<Self, KeyrxError> {
        Self::new_with_injector_and_metrics(
            device_path,
            injector,
            crate::metrics::default_noop_collector(),
        )
    }

    pub fn new_with_injector_and_metrics(
        device_path: Option<PathBuf>,
        injector: Box<dyn KeyInjector>,
        metrics: Arc<dyn MetricsCollector>,
    ) -> Result<Self, KeyrxError> {
        // Determine device path: use provided or auto-detect
        let (device_path, device_info) = match device_path {
            Some(path) => {
                debug!(
                    service = "keyrx",
                    event = "linux_device_provided",
                    component = "linux_input",
                    path = %path.display(),
                    "Using specified device"
                );
                let info = try_get_keyboard_info(&path).unwrap_or_else(|| {
                    DeviceInfo::new(path.clone(), path.display().to_string(), 0, 0, true)
                });
                (path, info)
            }
            None => {
                debug!(
                    service = "keyrx",
                    event = "linux_device_autodetect",
                    component = "linux_input",
                    "Auto-detecting keyboard device"
                );
                let info = Self::find_first_keyboard()?;
                (info.path.clone(), info)
            }
        };
        // Create the event channel
        let (tx, rx) = crossbeam_channel::unbounded();
        let running = Arc::new(AtomicBool::new(false));
        let panic_error = Arc::new(AtomicBool::new(false));

        // Initialize keymap cache with capacity for 256 entries
        // This should cover all standard keys with room for device-specific mappings
        let cache = match LruKeymapCache::new(256) {
            Some(cache) => Arc::new(cache),
            None => {
                // This should never happen with a non-zero capacity
                bail_keyrx!(
                    DRIVER_INIT_FAILED,
                    reason = "Failed to create keymap cache".to_string()
                );
            }
        };

        debug!(
            service = "keyrx",
            event = "linux_input_created",
            component = "linux_input",
            path = %device_path.display(),
            "LinuxInput created for device"
        );
        Ok(Self {
            reader_handle: None,
            injector,
            rx,
            tx,
            running,
            device_path,
            device_info,
            panic_error,
            metrics,
            cache,
        })
    }
    fn find_first_keyboard() -> Result<DeviceInfo, KeyrxError> {
        let mut keyboards = list_keyboards()?;

        // Filter out keyrx's own virtual keyboard to prevent self-detection
        keyboards.retain(|dev| !dev.name.contains("KeyRx Virtual Keyboard"));

        if keyboards.is_empty() {
            return Err(keyrx_err!(
                DRIVER_DEVICE_NOT_FOUND,
                device = "keyboard".to_string(),
                reason = "No keyboard devices found. Check permissions: ls -la /dev/input/event*"
                    .to_string()
            ));
        }
        let device = &keyboards[0];
        debug!(
            service = "keyrx",
            event = "linux_device_detected",
            component = "linux_input",
            device_name = device.name,
            path = %device.path.display(),
            "Auto-detected keyboard device"
        );
        Ok(device.clone())
    }
    pub fn list_devices() -> Result<Vec<DeviceInfo>, KeyrxError> {
        list_keyboards()
    }
    pub fn device_path(&self) -> &Path {
        &self.device_path
    }
    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }
    pub fn receiver(&self) -> &crossbeam_channel::Receiver<InputEvent> {
        &self.rx
    }
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    fn check_uinput_accessible() -> Result<(), KeyrxError> {
        let path = Path::new(UINPUT_PATH);
        if !path.exists() {
            bail_keyrx!(EVDEV_UINPUT_CREATE_FAILED, reason = UINPUT_NOT_FOUND_HELP);
        }
        // Check if readable/writable by attempting to open for read
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
        {
            Ok(_) => {
                debug!(
                    service = "keyrx",
                    event = "uinput_accessible",
                    component = "linux_input",
                    path = UINPUT_PATH,
                    "Successfully accessed uinput device"
                );
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                bail_keyrx!(DRIVER_PERMISSION_DENIED, device = UINPUT_PATH)
            }
            Err(e) => bail_keyrx!(
                LINUX_DEVICE_NODE_ERROR,
                path = format!("{}: {}", UINPUT_PATH, e)
            ),
        }
    }
}

#[async_trait]
impl InputSource for LinuxInput {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>, KeyrxError> {
        self.fail_if_reader_panicked()?;
        if self.is_inactive() {
            self.log_poll_when_inactive();
            bail_keyrx!(
                DRIVER_DEVICE_DISCONNECTED,
                device = "keyboard input (stopped by emergency exit or manual stop)"
            );
        }

        #[cfg(feature = "otel-tracing")]
        let poll_span = tracing::trace_span!(
            "driver.poll_events",
            driver = "linux",
            device = %self.device_path.display(),
            running = self.running.load(Ordering::Relaxed)
        );
        #[cfg(feature = "otel-tracing")]
        let _poll_guard = poll_span.enter();

        let mut events = Vec::new();
        while let Some(event) = self.next_event()? {
            #[cfg(feature = "otel-tracing")]
            let event_span = tracing::trace_span!(
                "driver.input_event",
                driver = "linux",
                key = ?event.key,
                pressed = event.pressed,
                timestamp_us = event.timestamp_us,
                device_id = event.device_id.as_deref().unwrap_or(""),
                is_repeat = event.is_repeat,
                is_synthetic = event.is_synthetic,
                scan_code = event.scan_code as u64,
            );
            #[cfg(feature = "otel-tracing")]
            let _event_guard = event_span.enter();
            events.push(event);
        }

        self.log_polled_events(events.len());
        Ok(events)
    }

    async fn send_output(&mut self, action: OutputAction) -> Result<(), KeyrxError> {
        if !self.running.load(Ordering::Relaxed) {
            self.log_inactive_send();
            return Ok(());
        }

        #[cfg(feature = "otel-tracing")]
        let action_label = format!("{:?}", &action);
        #[cfg(feature = "otel-tracing")]
        let send_span = tracing::trace_span!(
            "driver.send_output",
            driver = "linux",
            device = %self.device_path.display(),
            action = %action_label
        );
        #[cfg(feature = "otel-tracing")]
        let _send_guard = send_span.enter();

        match action {
            OutputAction::KeyDown(key) => self.inject_key_action(key, true, "linux_key_down")?,
            OutputAction::KeyUp(key) => self.inject_key_action(key, false, "linux_key_up")?,
            OutputAction::KeyTap(key) => self.tap_key(key)?,
            OutputAction::Block => self.log_block_action(),
            OutputAction::PassThrough => self.log_passthrough_action(),
        }
        Ok(())
    }

    async fn start(&mut self) -> Result<(), KeyrxError> {
        if self.running.load(Ordering::Relaxed) {
            self.log_start_skipped();
            return Ok(());
        }

        self.prepare_start()
            .map_err(|e| keyrx_err!(DRIVER_INIT_FAILED, reason = e.to_string()))?;
        let mut reader = self.build_reader()?;
        reader
            .grab()
            .map_err(|e| keyrx_err!(EVDEV_DEVICE_GRAB_FAILED, device = e.to_string()))?;
        self.spawn_reader(reader);
        self.log_started();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), KeyrxError> {
        if !self.running.load(Ordering::Relaxed) {
            self.log_stop_skipped();
            return Ok(());
        }

        self.running.store(false, Ordering::Relaxed);
        self.join_reader_thread();
        self.drain_events();

        // Invalidate cache entries for this device
        let device_id = self.device_path.to_string_lossy();
        self.cache.invalidate_device(&device_id);
        debug!(
            service = "keyrx",
            event = "linux_cache_invalidated",
            component = "linux_input",
            device_id = %device_id,
            "Invalidated cache entries for device"
        );

        self.log_stopped();
        Ok(())
    }
}

impl Drop for LinuxInput {
    fn drop(&mut self) {
        // Ensure the driver is stopped and keyboard is released on drop.
        // This is critical for graceful cleanup even on panics or unexpected termination.
        if self.running.load(Ordering::Relaxed) {
            debug!(
                service = "keyrx",
                event = "linux_drop_stopping",
                component = "linux_input",
                "LinuxInput::drop - stopping driver"
            );
            self.running.store(false, Ordering::Relaxed);
            // Wait for the reader thread to finish
            if let Some(handle) = self.reader_handle.take() {
                debug!(
                    service = "keyrx",
                    event = "linux_drop_join_reader",
                    component = "linux_input",
                    "LinuxInput::drop - waiting for reader thread"
                );
                match handle.join() {
                    Ok(()) => debug!(
                        service = "keyrx",
                        event = "linux_drop_reader_stopped",
                        component = "linux_input",
                        status = "clean",
                        "LinuxInput::drop - reader thread finished cleanly"
                    ),
                    Err(e) => warn!(
                        service = "keyrx",
                        event = "linux_drop_reader_panic",
                        component = "linux_input",
                        error = ?e,
                        "LinuxInput::drop - reader thread panicked"
                    ),
                }
            }
            // Drain any remaining events
            while self.rx.try_recv().is_ok() {}

            // Invalidate cache entries for this device
            let device_id = self.device_path.to_string_lossy();
            self.cache.invalidate_device(&device_id);

            debug!(
                service = "keyrx",
                event = "linux_drop_complete",
                component = "linux_input",
                "LinuxInput::drop - cleanup complete"
            );
        }
    }
}
#[cfg(test)]
mod mod_tests;
