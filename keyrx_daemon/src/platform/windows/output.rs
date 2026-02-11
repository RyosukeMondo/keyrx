use crate::platform::windows::inject::EventInjector;
use crate::platform::{DeviceError, OutputDevice};
use keyrx_core::runtime::event::KeyEvent;

pub struct WindowsKeyboardOutput {
    injector: EventInjector,
}

impl WindowsKeyboardOutput {
    pub fn new() -> Self {
        Self {
            injector: EventInjector,
        }
    }

    #[allow(dead_code)]
    pub fn destroy(&mut self) -> Result<(), DeviceError> {
        // Nothing special to destroy for Windows SendInput implementation
        Ok(())
    }

    #[must_use]
    pub fn is_destroyed(&self) -> bool {
        false
    }
}

impl OutputDevice for WindowsKeyboardOutput {
    fn inject_event(&mut self, event: KeyEvent) -> Result<(), DeviceError> {
        self.injector
            .inject(&event)
            .map_err(DeviceError::InjectionFailed)
    }
}

impl Default for WindowsKeyboardOutput {
    fn default() -> Self {
        Self::new()
    }
}
