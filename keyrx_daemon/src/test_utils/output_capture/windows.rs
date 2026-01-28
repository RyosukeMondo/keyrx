//! Windows-specific implementation of OutputCapture using low-level keyboard hooks.

use std::time::Duration;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use keyrx_core::runtime::event::KeyEvent;

use crate::platform::windows::keycode::vk_to_keycode;
use crate::test_utils::VirtualDeviceError;

/// Marker for events injected by the daemon
const DAEMON_OUTPUT_MARKER: usize = 0x4441454D; // "DAEM"

// Thread-local sender for Windows keyboard hook events.
//
// Using thread-local storage instead of a global static allows multiple
// OutputCapture instances to coexist safely, each in its own thread.
// This prevents conflicts when running tests concurrently.
thread_local! {
    static SENDER_TLS: std::cell::RefCell<Option<crossbeam_channel::Sender<KeyEvent>>> =
        const { std::cell::RefCell::new(None) };
}

/// Captures output events from the daemon's virtual keyboard.
///
/// On Windows, this uses a low-level keyboard hook to intercept events
/// that have been marked with the daemon's output marker.
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::test_utils::OutputCapture;
/// use std::time::Duration;
///
/// // Set up the keyboard hook
/// let capture = OutputCapture::find_by_name(
///     "keyrx Virtual Keyboard",
///     Duration::from_secs(5)
/// )?;
///
/// println!("Capturing from: {}", capture.name());
/// ```
pub struct OutputCapture {
    /// Name of the device.
    name: String,
    /// Channel for receiving events from the hook.
    receiver: crossbeam_channel::Receiver<KeyEvent>,
    /// Thread handle for the message loop.
    msg_thread: Option<std::thread::JoinHandle<()>>,
    /// Thread ID of the message loop.
    thread_id: u32,
    /// Buffered events from previous fetch_events call.
    /// When fetch_events returns multiple key events, we store extras here.
    event_buffer: Vec<KeyEvent>,
}

impl std::fmt::Debug for OutputCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputCapture")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl OutputCapture {
    /// Finds and opens an output device by name.
    ///
    /// On Windows, this sets up a low-level keyboard hook to capture events
    /// marked with the daemon's output marker.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the device to find (used for identification only)
    /// * `timeout` - Maximum time to wait for hook setup
    ///
    /// # Returns
    ///
    /// An `OutputCapture` instance connected to the hook, or an error.
    ///
    /// # Errors
    ///
    /// - [`VirtualDeviceError::Timeout`] if hook setup times out
    /// - [`VirtualDeviceError::CreationFailed`] if hook setup fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_daemon::test_utils::OutputCapture;
    /// use std::time::Duration;
    ///
    /// // Set up the keyboard hook
    /// let capture = OutputCapture::find_by_name(
    ///     "keyrx Virtual Keyboard",
    ///     Duration::from_secs(5)
    /// )?;
    ///
    /// println!("Capturing from: {}", capture.name());
    /// ```
    pub fn find_by_name(name: &str, _timeout: Duration) -> Result<Self, VirtualDeviceError> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let (setup_sender, setup_receiver) = crossbeam_channel::bounded(1);
        let name_clone = name.to_string();

        // Store the sender in thread-local storage to be accessed by the hook callback.
        SENDER_TLS.with(|s| {
            *s.borrow_mut() = Some(sender);
        });

        unsafe extern "system" fn hook_proc(
            code: i32,
            w_param: WPARAM,
            l_param: LPARAM,
        ) -> LRESULT {
            if code == HC_ACTION as i32 {
                let kbd_struct = unsafe { *(l_param as *const KBDLLHOOKSTRUCT) };
                if kbd_struct.dwExtraInfo == DAEMON_OUTPUT_MARKER {
                    if let Some(keycode) = vk_to_keycode(kbd_struct.vkCode as u16) {
                        let event = match w_param as u32 {
                            WM_KEYDOWN | WM_SYSKEYDOWN => Some(KeyEvent::Press(keycode)),
                            WM_KEYUP | WM_SYSKEYUP => Some(KeyEvent::Release(keycode)),
                            _ => None,
                        };
                        if let Some(event) = event {
                            SENDER_TLS.with(|s| {
                                if let Some(sender) = s.borrow().as_ref() {
                                    let _ = sender.try_send(event);
                                }
                            });
                        }
                    }
                }
            }
            unsafe { CallNextHookEx(std::ptr::null_mut(), code, w_param as _, l_param as _) }
        }

        let msg_thread = std::thread::spawn(move || unsafe {
            let h_mod = GetModuleHandleW(std::ptr::null());
            let thread_id = GetCurrentThreadId();
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), h_mod as _, 0);

            if hook.is_null() {
                let _ = setup_sender.send(Err("Failed to set hook".to_string()));
                return;
            }

            if setup_sender.send(Ok(thread_id)).is_err() {
                UnhookWindowsHookEx(hook);
                return;
            }

            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            UnhookWindowsHookEx(hook);
        });

        // Wait for hook setup success
        let thread_id = setup_receiver
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| VirtualDeviceError::Timeout {
                operation: "hook setup".to_string(),
                timeout_ms: 5000,
            })?
            .map_err(|e| VirtualDeviceError::CreationFailed { message: e })?;

        Ok(Self {
            name: name_clone,
            receiver,
            msg_thread: Some(msg_thread),
            thread_id,
            event_buffer: Vec::new(),
        })
    }

    /// Returns the name of the captured device.
    ///
    /// This is the name provided when creating the OutputCapture instance.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let capture = OutputCapture::find_by_name("test-keyboard", timeout)?;
    /// assert_eq!(capture.name(), "test-keyboard");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Reads the next keyboard event with a timeout.
    ///
    /// This method waits for events from the keyboard hook channel.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for an event
    ///
    /// # Returns
    ///
    /// - `Ok(Some(KeyEvent))` if a keyboard event was received
    /// - `Ok(None)` if the timeout expired without any keyboard events
    /// - `Err(VirtualDeviceError)` if the capture thread disconnected
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_daemon::test_utils::OutputCapture;
    /// use std::time::Duration;
    ///
    /// let mut capture = OutputCapture::find_by_name("keyrx Virtual Keyboard", timeout)?;
    ///
    /// match capture.next_event(Duration::from_millis(100))? {
    ///     Some(event) => println!("Got event: {:?}", event),
    ///     None => println!("Timeout - no events received"),
    /// }
    /// ```
    pub fn next_event(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<KeyEvent>, VirtualDeviceError> {
        // First check if we have buffered events (no need to poll)
        if !self.event_buffer.is_empty() {
            return Ok(Some(self.event_buffer.remove(0)));
        }

        match self.receiver.recv_timeout(timeout) {
            Ok(event) => Ok(Some(event)),
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => Ok(None),
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => Err(
                VirtualDeviceError::creation_failed("capture thread disconnected"),
            ),
        }
    }

    /// Collects keyboard events until the timeout expires.
    ///
    /// This method continues reading events until the specified timeout
    /// elapses with no new events. Events are returned in the order received.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Time to wait for additional events after receiving each event.
    ///   The timeout resets after each event, so this is effectively the
    ///   "idle timeout" for the collection.
    ///
    /// # Returns
    ///
    /// A vector of collected keyboard events (may be empty if timeout with no events).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_daemon::test_utils::OutputCapture;
    /// use std::time::Duration;
    ///
    /// let mut capture = OutputCapture::find_by_name("test-keyboard", timeout)?;
    ///
    /// // Collect all events that arrive within 100ms gaps
    /// let events = capture.collect_events(Duration::from_millis(100))?;
    /// println!("Collected {} events", events.len());
    /// ```
    pub fn collect_events(
        &mut self,
        timeout: Duration,
    ) -> Result<Vec<KeyEvent>, VirtualDeviceError> {
        let mut events = Vec::new();

        loop {
            match self.next_event(timeout)? {
                Some(event) => {
                    events.push(event);
                    // Continue collecting with fresh timeout
                }
                None => {
                    // Timeout expired, return collected events
                    return Ok(events);
                }
            }
        }
    }

    /// Drains and discards all pending events from the device.
    ///
    /// This is useful before starting a test to ensure no stale events
    /// from previous operations affect the test results.
    ///
    /// # Returns
    ///
    /// The number of events that were drained (for debugging/logging).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_daemon::test_utils::OutputCapture;
    ///
    /// let mut capture = OutputCapture::find_by_name("test-keyboard", timeout)?;
    ///
    /// // Clear any pending events before starting test
    /// let drained = capture.drain()?;
    /// if drained > 0 {
    ///     println!("Drained {} stale events", drained);
    /// }
    /// ```
    pub fn drain(&mut self) -> Result<usize, VirtualDeviceError> {
        // First clear any buffered events
        let mut count = self.event_buffer.len();
        self.event_buffer.clear();

        while self.receiver.try_recv().is_ok() {
            count += 1;
        }
        Ok(count)
    }
}

impl Drop for OutputCapture {
    fn drop(&mut self) {
        // Clear the thread-local sender
        SENDER_TLS.with(|s| {
            *s.borrow_mut() = None;
        });

        if let Some(msg_thread) = self.msg_thread.take() {
            unsafe {
                let _ = PostThreadMessageW(self.thread_id, WM_QUIT, 0 as _, 0 as _);
            }
            let _ = msg_thread.join();
        }
    }
}
