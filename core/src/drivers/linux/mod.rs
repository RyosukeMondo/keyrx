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
use crate::drivers::{DeviceInfo, KeyInjector};
use crate::engine::{InputEvent, OutputAction};
use crate::traits::InputSource;
use anyhow::{bail, Context, Result};
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
}
impl LinuxInput {
    pub fn new(device_path: Option<PathBuf>) -> Result<Self> {
        let injector = UinputWriter::new().context("Failed to create uinput writer")?;
        Self::new_with_injector(device_path, Box::new(injector))
    }
    pub fn new_with_injector(
        device_path: Option<PathBuf>,
        injector: Box<dyn KeyInjector>,
    ) -> Result<Self> {
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
        })
    }
    fn find_first_keyboard() -> Result<DeviceInfo> {
        let keyboards = list_keyboards()?;
        if keyboards.is_empty() {
            bail!(
                "No keyboard devices found\n\n\
                 Remediation:\n  \
                 1. Ensure a keyboard is connected\n  \
                 2. Check permissions: ls -la /dev/input/event*\n  \
                 3. Add user to input group: sudo usermod -aG input $USER\n  \
                 4. Log out and back in for group changes to take effect"
            );
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
    pub fn list_devices() -> Result<Vec<DeviceInfo>> {
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
    fn check_uinput_accessible() -> Result<()> {
        let path = Path::new(UINPUT_PATH);
        if !path.exists() {
            bail!(UINPUT_NOT_FOUND_HELP);
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
                bail!(UINPUT_PERMISSION_DENIED_HELP)
            }
            Err(e) => bail!("Failed to access {UINPUT_PATH}: {e}\n\n{UINPUT_ACCESS_FAILED_HELP}"),
        }
    }
}
#[async_trait]
impl InputSource for LinuxInput {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        self.fail_if_reader_panicked()?;
        if self.is_inactive() {
            self.log_poll_when_inactive();
            return Ok(vec![]);
        }

        let mut events = Vec::new();
        while let Some(event) = self.next_event()? {
            events.push(event);
        }

        self.log_polled_events(events.len());
        Ok(events)
    }
    async fn send_output(&mut self, action: OutputAction) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            self.log_inactive_send();
            return Ok(());
        }

        match action {
            OutputAction::KeyDown(key) => self.inject_key_action(key, true, "linux_key_down")?,
            OutputAction::KeyUp(key) => self.inject_key_action(key, false, "linux_key_up")?,
            OutputAction::KeyTap(key) => self.tap_key(key)?,
            OutputAction::Block => self.log_block_action(),
            OutputAction::PassThrough => self.log_passthrough_action(),
        }
        Ok(())
    }
    async fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            self.log_start_skipped();
            return Ok(());
        }

        self.prepare_start()
            .context("Failed to start Linux input source")?;
        let mut reader = self.build_reader()?;
        reader.grab().context("Failed to grab keyboard device")?;
        self.spawn_reader(reader);
        self.log_started();
        Ok(())
    }
    async fn stop(&mut self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            self.log_stop_skipped();
            return Ok(());
        }

        self.running.store(false, Ordering::Relaxed);
        self.join_reader_thread();
        self.drain_events();
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
