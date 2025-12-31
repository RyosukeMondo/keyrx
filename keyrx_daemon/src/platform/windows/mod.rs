pub mod device_map;
pub mod inject;
pub mod input;
pub mod keycode;
pub mod output;
pub mod rawinput;
#[cfg(test)]
mod tests;
pub mod tray;

use std::sync::{Arc, Mutex};

use crossbeam_channel::{unbounded, Sender};
use keyrx_core::runtime::KeyEvent;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
};

pub use input::WindowsKeyboardInput;
pub use output::WindowsKeyboardOutput;

use self::device_map::DeviceMap;
use self::rawinput::{BridgeContextHandle, RawInputManager};
use crate::platform::{DeviceInfo as CommonDeviceInfo, Platform, PlatformError, PlatformResult};

#[cfg(target_os = "windows")]
pub struct WindowsPlatform {
    pub input: WindowsKeyboardInput,
    _sender: Sender<KeyEvent>,
    device_map: DeviceMap,
    raw_input_manager: Option<RawInputManager>,
    bridge_context: Arc<Mutex<Option<BridgeContextHandle>>>,
    bridge_hook: Arc<Mutex<Option<isize>>>,
}

#[cfg(target_os = "windows")]
impl WindowsPlatform {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            input: WindowsKeyboardInput::new(receiver),
            _sender: sender,
            device_map: DeviceMap::new(),
            raw_input_manager: None,
            bridge_context: Arc::new(Mutex::new(None)),
            bridge_hook: Arc::new(Mutex::new(None)),
        }
    }

    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Enumerate initial devices
        self.device_map.enumerate()?;

        // Create Raw Input Manager (creates window + registers devices)
        // Must be done on the same thread that pumps messages (this thread)
        let manager = RawInputManager::new(
            self.device_map.clone(),
            self._sender.clone(),
            self.bridge_context.clone(),
            self.bridge_hook.clone(),
        )?;
        self.raw_input_manager = Some(manager);

        Ok(())
    }

    pub fn process_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            // process all pending messages
            while PeekMessageW(&mut msg, 0 as _, 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl Default for WindowsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert Windows-specific DeviceInfo to common platform DeviceInfo.
fn convert_device_info(device: &device_map::DeviceInfo) -> CommonDeviceInfo {
    // Parse vendor/product IDs from the path if possible
    // Windows device path format: \\?\HID#VID_XXXX&PID_YYYY#...
    let (vendor_id, product_id) = parse_vid_pid(&device.path);

    CommonDeviceInfo {
        id: device.device_id(),
        name: format!("Keyboard ({})", device.device_id()),
        path: device.path.clone(),
        vendor_id,
        product_id,
    }
}

/// Parse VID and PID from Windows device path.
fn parse_vid_pid(path: &str) -> (u16, u16) {
    let mut vendor_id = 0;
    let mut product_id = 0;

    // Extract VID_XXXX and PID_YYYY from path like:
    // \\?\HID#VID_046D&PID_C52B&MI_00#...
    for part in path.split('&') {
        if let Some(vid) = part.strip_prefix("VID_") {
            if let Ok(vid_val) = u16::from_str_radix(&vid[..4.min(vid.len())], 16) {
                vendor_id = vid_val;
            }
        }
        if let Some(pid) = part.strip_prefix("PID_") {
            if let Ok(pid_val) = u16::from_str_radix(&pid[..4.min(pid.len())], 16) {
                product_id = pid_val;
            }
        }
    }

    (vendor_id, product_id)
}

#[cfg(target_os = "windows")]
impl Platform for WindowsPlatform {
    fn initialize(&mut self) -> PlatformResult<()> {
        log::info!("Initializing Windows platform");

        self.init().map_err(|e| {
            PlatformError::InitializationFailed(format!("Windows platform init failed: {}", e))
        })?;

        log::info!("Windows platform initialized");
        Ok(())
    }

    fn capture_input(&mut self) -> PlatformResult<KeyEvent> {
        // Process pending Windows messages
        self.process_events().map_err(|e| {
            PlatformError::Io(std::io::Error::other(format!(
                "Failed to process Windows events: {}",
                e
            )))
        })?;

        // Try to receive event from input channel
        use crate::platform::InputDevice;
        self.input.next_event().map_err(|e| match e {
            crate::platform::DeviceError::EndOfStream => {
                PlatformError::DeviceNotFound("No events available".to_string())
            }
            crate::platform::DeviceError::Io(io_err) => PlatformError::Io(io_err),
            crate::platform::DeviceError::PermissionDenied(msg) => {
                PlatformError::PermissionDenied(msg)
            }
            _ => PlatformError::Io(std::io::Error::other(format!("Input error: {}", e))),
        })
    }

    fn inject_output(&mut self, event: KeyEvent) -> PlatformResult<()> {
        use crate::platform::OutputDevice;
        let mut output = WindowsKeyboardOutput::new();
        output.inject_event(event).map_err(|e| match e {
            crate::platform::DeviceError::InjectionFailed(msg) => {
                PlatformError::InjectionFailed(msg)
            }
            crate::platform::DeviceError::Io(io_err) => PlatformError::Io(io_err),
            _ => PlatformError::Io(std::io::Error::other(format!("Injection error: {}", e))),
        })
    }

    fn list_devices(&self) -> PlatformResult<Vec<CommonDeviceInfo>> {
        let devices = self.device_map.all();
        Ok(devices.iter().map(convert_device_info).collect())
    }

    fn shutdown(&mut self) -> PlatformResult<()> {
        log::info!("Shutting down Windows platform");

        // Clean up Raw Input Manager
        if let Some(_manager) = self.raw_input_manager.take() {
            // RawInputManager cleanup happens on drop
            log::debug!("Released Raw Input Manager");
        }

        // Clean up hooks
        if let Some(_hook) = self.bridge_hook.lock().unwrap().take() {
            // Hook cleanup should happen here if needed
            log::debug!("Released keyboard hook");
        }

        // Clean up bridge context
        if let Some(_context) = self.bridge_context.lock().unwrap().take() {
            log::debug!("Released bridge context");
        }

        log::info!("Windows platform shutdown complete");
        Ok(())
    }
}

// SAFETY: WindowsPlatform is Send + Sync because:
// - The Windows message loop runs on a single thread (the main thread)
// - Arc<Mutex<>> fields provide safe concurrent access
// - crossbeam_channel is thread-safe
// - device_map uses Arc<RwLock<>> internally
//
// IMPORTANT: This implementation assumes single-threaded usage for the message loop.
// The Platform trait requires Send + Sync for the daemon event loop and web API handlers,
// but the actual Windows message processing must occur on the thread that created the window.
// If multi-threaded access to WindowsPlatform becomes necessary, the message loop handling
// will need to be refactored to use a dedicated message pump thread.
#[cfg(target_os = "windows")]
unsafe impl Send for WindowsPlatform {}
#[cfg(target_os = "windows")]
unsafe impl Sync for WindowsPlatform {}
