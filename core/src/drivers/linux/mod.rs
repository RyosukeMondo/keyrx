//! Linux input driver built on evdev (capture) and uinput (injection).
//! Permissions: read `/dev/input/event*`, write `/dev/uinput`, and load the `uinput` kernel module.
//! Architecture: blocking evdev reader thread feeds the async engine via `crossbeam_channel`, while the uinput writer injects remapped output.
mod device_info;
mod discovery;
mod keymap;
mod reader;
mod writer;
use crate::drivers::{DeviceInfo, KeyInjector};
use crate::engine::{InputEvent, OutputAction};
use crate::traits::InputSource;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use crossbeam_channel::Sender;
pub use discovery::list_keyboards;
use reader::EvdevReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use tracing::{debug, error, trace, warn};
pub use writer::UinputWriter;
const UINPUT_PATH: &str = "/dev/uinput";
pub struct LinuxInput {
    reader_handle: Option<JoinHandle<()>>,
    injector: Box<dyn KeyInjector>,
    rx: crossbeam_channel::Receiver<InputEvent>,
    tx: Sender<InputEvent>,
    running: Arc<AtomicBool>,
    device_path: PathBuf,
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
        let device_path = match device_path {
            Some(path) => {
                debug!(
                    service = "keyrx",
                    event = "linux_device_provided",
                    component = "linux_input",
                    path = %path.display(),
                    "Using specified device"
                );
                path
            }
            None => {
                debug!(
                    service = "keyrx",
                    event = "linux_device_autodetect",
                    component = "linux_input",
                    "Auto-detecting keyboard device"
                );
                Self::find_first_keyboard()?
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
            panic_error,
        })
    }
    fn find_first_keyboard() -> Result<PathBuf> {
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
        Ok(device.path.clone())
    }
    pub fn list_devices() -> Result<Vec<DeviceInfo>> {
        list_keyboards()
    }
    pub fn device_path(&self) -> &Path {
        &self.device_path
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
            bail!(
                "uinput device not found at {UINPUT_PATH}\n\n\
                 Remediation:\n  \
                 1. Load the uinput kernel module: sudo modprobe uinput\n  \
                 2. If that fails, check if uinput is built into your kernel\n  \
                 3. Ensure your kernel supports CONFIG_INPUT_UINPUT"
            );
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
                bail!(
                    "Permission denied accessing {UINPUT_PATH}\n\n\
                     Remediation:\n  \
                     1. Add your user to the 'input' group: sudo usermod -aG input $USER\n  \
                     2. Create a udev rule for uinput access:\n     \
                        echo 'KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"input\"' | \
                        sudo tee /etc/udev/rules.d/99-uinput.rules\n  \
                     3. Reload udev rules: sudo udevadm control --reload-rules && \
                        sudo udevadm trigger\n  \
                     4. Log out and log back in for group changes to take effect\n  \
                     5. Alternatively, run with sudo (not recommended for regular use)"
                );
            }
            Err(e) => {
                bail!(
                    "Failed to access {UINPUT_PATH}: {e}\n\n\
                     Remediation:\n  \
                     1. Check device permissions: ls -la {UINPUT_PATH}\n  \
                     2. Check if you have read/write access to the device\n  \
                     3. Ensure the uinput module is loaded: lsmod | grep uinput"
                );
            }
        }
    }
}
#[async_trait]
impl InputSource for LinuxInput {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        // Check if the reader thread panicked
        if self.panic_error.load(Ordering::SeqCst) {
            error!(
                service = "keyrx",
                event = "linux_reader_panic_detected",
                component = "linux_input",
                "poll_events called after reader thread panic"
            );
            self.running.store(false, Ordering::Relaxed);
            bail!("Input reader thread panicked - keyboard has been ungrabbed for safety");
        }
        if !self.running.load(Ordering::Relaxed) {
            trace!(
                service = "keyrx",
                event = "linux_poll_events_inactive",
                component = "linux_input",
                "poll_events called while not running"
            );
            return Ok(vec![]);
        }
        // Non-blocking receive from the channel
        // Collect all available events without blocking
        let mut events = Vec::new();
        loop {
            match self.rx.try_recv() {
                Ok(event) => {
                    trace!(
                        service = "keyrx",
                        event = "linux_input_event_received",
                        component = "linux_input",
                        key = ?event.key,
                        pressed = event.pressed,
                        "Received input event"
                    );
                    events.push(event);
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    // No more events available
                    break;
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    // Channel closed - reader thread has stopped
                    // Check if it was due to a panic
                    if self.panic_error.load(Ordering::SeqCst) {
                        error!(
                            service = "keyrx",
                            event = "linux_channel_disconnected",
                            component = "linux_input",
                            reason = "reader_panic",
                            "Event channel disconnected due to reader thread panic"
                        );
                        self.running.store(false, Ordering::Relaxed);
                        bail!(
                            "Input reader thread panicked - keyboard has been ungrabbed for safety"
                        );
                    }
                    error!(
                        service = "keyrx",
                        event = "linux_channel_disconnected",
                        component = "linux_input",
                        reason = "unexpected_disconnect",
                        "Event channel disconnected - reader thread may have crashed"
                    );
                    self.running.store(false, Ordering::Relaxed);
                    bail!("Input reader disconnected unexpectedly");
                }
            }
        }
        if !events.is_empty() {
            debug!(
                service = "keyrx",
                event = "linux_poll_events",
                component = "linux_input",
                count = events.len(),
                "Returning polled events"
            );
        }
        Ok(events)
    }
    async fn send_output(&mut self, action: OutputAction) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            trace!(
                service = "keyrx",
                event = "linux_send_output_inactive",
                component = "linux_input",
                "send_output called while not running"
            );
            return Ok(());
        }
        match action {
            OutputAction::KeyDown(key) => {
                debug!(
                    service = "keyrx",
                    event = "linux_key_down",
                    component = "linux_input",
                    key = ?key,
                    "Sending key down"
                );
                self.injector.inject(key, true)?;
            }
            OutputAction::KeyUp(key) => {
                debug!(
                    service = "keyrx",
                    event = "linux_key_up",
                    component = "linux_input",
                    key = ?key,
                    "Sending key up"
                );
                self.injector.inject(key, false)?;
            }
            OutputAction::KeyTap(key) => {
                debug!(
                    service = "keyrx",
                    event = "linux_key_tap",
                    component = "linux_input",
                    key = ?key,
                    "Sending key tap"
                );
                self.injector.inject(key, true)?;
                self.injector.inject(key, false)?;
            }
            OutputAction::Block => {
                // Block does nothing - the original event is already grabbed
                // and won't be passed through unless we explicitly emit it
                trace!(
                    service = "keyrx",
                    event = "linux_block_action",
                    component = "linux_input",
                    "Blocking key (no action needed)"
                );
            }
            OutputAction::PassThrough => {
                // PassThrough is handled by the engine - it re-emits the original key
                // For the driver, this is a no-op since the engine will call
                // KeyDown/KeyUp for the original key if needed
                trace!(
                    service = "keyrx",
                    event = "linux_passthrough_action",
                    component = "linux_input",
                    "PassThrough (no action needed)"
                );
            }
        }
        Ok(())
    }
    async fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            warn!(
                service = "keyrx",
                event = "linux_start_skipped",
                component = "linux_input",
                reason = "already_running",
                "LinuxInput already running"
            );
            return Ok(());
        }
        // Verify uinput is accessible before starting
        Self::check_uinput_accessible().context("Failed to start Linux input source")?;
        // Reset panic error flag for fresh start
        self.panic_error.store(false, Ordering::SeqCst);
        // Set running flag before spawning thread
        self.running.store(true, Ordering::Relaxed);
        // Create the evdev reader
        let mut reader = EvdevReader::new(
            self.device_path.clone(),
            self.tx.clone(),
            self.running.clone(),
            self.panic_error.clone(),
        )
        .context("Failed to create evdev reader")?;
        // Grab exclusive access to the keyboard
        reader.grab().context("Failed to grab keyboard device")?;
        // Spawn the reader thread
        let handle = reader.spawn();
        self.reader_handle = Some(handle);
        debug!(
            service = "keyrx",
            event = "linux_started",
            component = "linux_input",
            path = %self.device_path.display(),
            "LinuxInput started successfully"
        );
        Ok(())
    }
    async fn stop(&mut self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            debug!(
                service = "keyrx",
                event = "linux_stop_skipped",
                component = "linux_input",
                reason = "already_stopped",
                "LinuxInput already stopped"
            );
            return Ok(());
        }
        debug!(
            service = "keyrx",
            event = "linux_stopping",
            component = "linux_input",
            "Stopping LinuxInput"
        );
        // Signal the reader thread to stop
        self.running.store(false, Ordering::Relaxed);
        // Wait for the reader thread to finish
        if let Some(handle) = self.reader_handle.take() {
            debug!(
                service = "keyrx",
                event = "linux_join_reader",
                component = "linux_input",
                "Waiting for reader thread to finish"
            );
            match handle.join() {
                Ok(()) => {
                    debug!(
                        service = "keyrx",
                        event = "linux_reader_stopped",
                        component = "linux_input",
                        status = "clean",
                        "Reader thread finished cleanly"
                    );
                }
                Err(e) => {
                    error!(
                        service = "keyrx",
                        event = "linux_reader_panic",
                        component = "linux_input",
                        error = ?e,
                        "Reader thread panicked"
                    );
                    // Continue with cleanup even if thread panicked
                }
            }
        }
        // Drain any remaining events from the channel
        while self.rx.try_recv().is_ok() {
            // Discard remaining events
        }
        debug!(
            service = "keyrx",
            event = "linux_stopped",
            component = "linux_input",
            "LinuxInput stopped successfully"
        );
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
