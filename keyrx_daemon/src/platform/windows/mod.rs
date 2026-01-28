pub mod device_map;
pub mod inject;
pub mod input;
pub mod key_blocker;
pub mod keycode;
pub mod output;
pub mod platform_state;
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
    key_blocker: Option<key_blocker::KeyBlocker>,
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
            key_blocker: None,
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

        // Install low-level keyboard hook for blocking remapped keys
        // This hook runs BEFORE Raw Input in the Windows message chain
        match key_blocker::KeyBlocker::new() {
            Ok(blocker) => {
                log::info!("✓ Key blocker installed successfully");
                self.key_blocker = Some(blocker);
            }
            Err(e) => {
                log::warn!("Failed to install key blocker: {}. Remapping may show double inputs.", e);
                // Continue without blocker - daemon still works but keys won't be blocked
            }
        }

        // Initialize global platform state for cross-layer communication
        // This allows the profile service to configure key blocking
        platform_state::PlatformState::initialize(self.key_blocker.take());

        // Note: key_blocker is now moved into platform_state, so we can't use self.key_blocker anymore
        // This is intentional - all access should go through platform_state from now on

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

    /// Configure key blocker based on active profile mappings
    ///
    /// Extracts all source keys from the profile and tells the key blocker
    /// to block those scan codes. This prevents double input (original + remapped).
    ///
    /// # Arguments
    ///
    /// * `config` - The active profile configuration, or None to clear all blocking
    pub fn configure_key_blocking(
        &mut self,
        config: Option<&keyrx_core::config::ConfigRoot>,
    ) -> Result<(), String> {
        let Some(blocker) = &self.key_blocker else {
            log::warn!("Key blocker not available, skipping configuration");
            return Ok(());
        };

        // Clear all existing blocked keys first
        blocker.clear_all();

        let Some(config) = config else {
            log::info!("✓ Key blocking cleared (no active profile)");
            return Ok(());
        };

        // Extract all source keys from all device mappings
        let mut blocked_count = 0;
        for device_config in &config.devices {
            for mapping in &device_config.mappings {
                self.extract_and_block_keys(mapping, blocker, &mut blocked_count);
            }
        }

        log::info!("✓ Configured key blocking: {} keys blocked", blocked_count);
        Ok(())
    }

    /// Recursively extract source keys from a mapping and block them
    fn extract_and_block_keys(
        &self,
        mapping: &keyrx_core::config::KeyMapping,
        blocker: &key_blocker::KeyBlocker,
        blocked_count: &mut usize,
    ) {
        use keyrx_core::config::{BaseKeyMapping, KeyMapping};

        match mapping {
            KeyMapping::Base(base) => {
                // Extract the source key from base mapping
                let source_key = match base {
                    BaseKeyMapping::Simple { from, .. } => *from,
                    BaseKeyMapping::Modifier { from, .. } => *from,
                    BaseKeyMapping::Lock { from, .. } => *from,
                    BaseKeyMapping::TapHold { from, .. } => *from,
                    BaseKeyMapping::ModifiedOutput { from, .. } => *from,
                };

                // Convert to scan code and block it
                if let Some(scan_code) = keycode::keycode_to_scancode(source_key) {
                    blocker.block_key(scan_code);
                    *blocked_count += 1;
                    log::trace!(
                        "Blocking {:?} (scan code: 0x{:04X})",
                        source_key,
                        scan_code
                    );
                } else {
                    log::warn!(
                        "Failed to convert {:?} to scan code, won't be blocked",
                        source_key
                    );
                }
            }
            KeyMapping::Conditional { mappings, .. } => {
                // Recursively process conditional mappings
                for base_mapping in mappings {
                    // Wrap BaseKeyMapping in KeyMapping::Base for recursion
                    let wrapped = KeyMapping::Base(base_mapping.clone());
                    self.extract_and_block_keys(&wrapped, blocker, blocked_count);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
impl Default for WindowsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert Windows-specific DeviceInfo to common platform DeviceInfo.
pub(crate) fn convert_device_info(device: &device_map::DeviceInfo) -> CommonDeviceInfo {
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
pub(crate) fn parse_vid_pid(path: &str) -> (u16, u16) {
    let mut vendor_id = 0;
    let mut product_id = 0;

    // Extract VID_XXXX and PID_YYYY from path like:
    // \\?\HID#VID_046D&PID_C52B&MI_00#...
    // Split by both # and & to handle all path segments
    for part in path.split(&['#', '&']) {
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

        self.init()
            .map_err(|e| PlatformError::InitializationFailed {
                reason: format!("Windows platform init failed: {}", e),
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
            crate::platform::DeviceError::InjectionFailed(msg) => PlatformError::InjectionFailed {
                reason: msg,
                suggestion: "Check Windows SendInput API permissions and event structure"
                    .to_string(),
            },
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
        if let Some(_hook) = crate::platform::recovery::recover_lock_with_context(
            &self.bridge_hook,
            "Windows platform shutdown (bridge hook)",
        )?
        .take()
        {
            // Hook cleanup should happen here if needed
            log::debug!("Released keyboard hook");
        }

        // Clean up bridge context
        if let Some(_context) = crate::platform::recovery::recover_lock_with_context(
            &self.bridge_context,
            "Windows platform shutdown (bridge context)",
        )?
        .take()
        {
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
