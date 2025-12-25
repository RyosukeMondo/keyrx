pub mod device_map;
pub mod inject;
pub mod input;
pub mod keycode;
pub mod output;
pub mod rawinput;
#[cfg(test)]
mod tests;
pub mod tray;

use crossbeam_channel::{unbounded, Sender};
use keyrx_core::runtime::KeyEvent;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
};

pub use input::WindowsKeyboardInput;
pub use output::WindowsKeyboardOutput;

use self::device_map::DeviceMap;
use self::rawinput::RawInputManager;

#[cfg(target_os = "windows")]
pub struct WindowsPlatform {
    pub input: WindowsKeyboardInput,
    _sender: Sender<KeyEvent>,
    device_map: DeviceMap,
    raw_input_manager: Option<RawInputManager>,
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
        }
    }

    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Enumerate initial devices
        self.device_map.enumerate()?;

        // Create Raw Input Manager (creates window + registers devices)
        // Must be done on the same thread that pumps messages (this thread)
        let manager = RawInputManager::new(self.device_map.clone(), self._sender.clone())?;
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
