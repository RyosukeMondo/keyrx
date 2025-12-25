use crate::platform::{DeviceError, InputDevice};
use crossbeam_channel::Receiver;
use keyrx_core::runtime::event::KeyEvent;

/// A keyboard input device backed by a channel receiver.
///
/// This struct receives events from the central `RawInputManager` which routes
/// WM_INPUT messages to the appropriate device channel.
pub struct WindowsKeyboardInput {
    receiver: Receiver<KeyEvent>,
}

impl WindowsKeyboardInput {
    /// Creates a new input device that reads from the given receiver.
    pub fn new(receiver: Receiver<KeyEvent>) -> Self {
        Self { receiver }
    }

    pub fn is_grabbed(&self) -> bool {
        true // Raw Input is always "grabbed" (listening)
    }
}

impl InputDevice for WindowsKeyboardInput {
    fn next_event(&mut self) -> Result<KeyEvent, DeviceError> {
        self.receiver.try_recv().map_err(|e| match e {
            crossbeam_channel::TryRecvError::Empty => DeviceError::EndOfStream,
            crossbeam_channel::TryRecvError::Disconnected => DeviceError::EndOfStream,
        })
    }

    fn grab(&mut self) -> Result<(), DeviceError> {
        // Raw Input "grab" is implicit via RIDEV_INPUTSINK registered globally.
        // We could technically filter events if "not grabbed", but typically
        // the daemon always wants to process events if it's running.
        // For per-device grab: we already have the stream.
        Ok(())
    }

    fn release(&mut self) -> Result<(), DeviceError> {
        // Stop processing?
        Ok(())
    }
}
