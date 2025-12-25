pub mod hook;
pub mod inject;
pub mod input;
pub mod keycode;
pub mod output;
#[cfg(test)]
mod tests;
pub mod tray;

use crate::platform::InputDevice;
pub use input::WindowsKeyboardInput;
pub use output::WindowsKeyboardOutput;

#[cfg(target_os = "windows")]
pub struct WindowsPlatform {
    pub input: WindowsKeyboardInput,
    #[allow(dead_code)]
    pub output: WindowsKeyboardOutput,
}

#[cfg(target_os = "windows")]
impl WindowsPlatform {
    pub fn new() -> Self {
        Self {
            input: WindowsKeyboardInput::new(),
            output: WindowsKeyboardOutput::new(),
        }
    }

    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Grab the input device to start the keyboard hook
        self.input
            .grab()
            .map_err(|e: crate::platform::DeviceError| e.to_string())?;
        Ok(())
    }

    pub fn process_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // For Windows, the message loop is typically run on the main thread.
        // This method can be used to process any internal event queues if needed.
        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl Default for WindowsPlatform {
    fn default() -> Self {
        Self::new()
    }
}
