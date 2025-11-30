//! Windows input driver using WH_KEYBOARD_LL.
//!
//! This module implements keyboard capture and injection on Windows using:
//! - WH_KEYBOARD_LL (low-level keyboard hook) for capturing input
//! - SendInput API for injecting remapped keys
//!
//! # Thread Model
//!
//! The hook must be installed from a thread with a message pump. The `HookManager`
//! spawns a dedicated thread that:
//! 1. Installs the keyboard hook via `SetWindowsHookExW`
//! 2. Runs a message pump via `GetMessageW`/`DispatchMessageW`
//! 3. Uninstalls the hook on shutdown
//!
//! # Permissions
//!
//! Low-level keyboard hooks generally don't require administrator privileges,
//! but some antivirus software may block them. If hook installation fails,
//! try adding an exception for the KeyRx executable.

use crate::drivers::DeviceInfo;
use crate::engine::{InputEvent, KeyCode, OutputAction};
use crate::error::WindowsDriverError;
use crate::traits::InputSource;
use anyhow::Result;
use async_trait::async_trait;
use crossbeam_channel::{Receiver, Sender};
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{debug, error, trace, warn};
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetLastError, PeekMessageW, PostThreadMessageW,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG,
    PM_REMOVE, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

/// Thread-local storage for the event sender used by the hook callback.
///
/// This is necessary because the hook callback is a C-style function pointer
/// that cannot capture any context. We store the sender in thread-local storage
/// and access it from within the callback.
thread_local! {
    static HOOK_SENDER: RefCell<Option<Sender<InputEvent>>> = const { RefCell::new(None) };
}

/// Global storage for the hook thread's thread ID.
///
/// This is used to post WM_QUIT to the hook thread when shutting down.
/// We use an atomic because it needs to be accessed from multiple threads.
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);

/// Low-level keyboard hook manager.
///
/// Manages the lifecycle of a Windows keyboard hook, including installation,
/// event capture, and cleanup.
pub struct HookManager {
    /// The hook handle returned by SetWindowsHookExW.
    hook_handle: Option<HHOOK>,
    /// Flag to signal the message pump to stop.
    running: Arc<AtomicBool>,
}

impl HookManager {
    /// Create a new HookManager.
    ///
    /// The hook is not installed until `install()` is called.
    pub fn new(running: Arc<AtomicBool>) -> Self {
        Self {
            hook_handle: None,
            running,
        }
    }

    /// Install the low-level keyboard hook.
    ///
    /// This must be called from a thread that will run a message pump,
    /// as hook callbacks are dispatched via the Windows message queue.
    ///
    /// # Arguments
    ///
    /// * `sender` - Channel sender for keyboard events
    ///
    /// # Errors
    ///
    /// Returns `WindowsDriverError::HookInstallFailed` if the hook cannot be installed.
    pub fn install(&mut self, sender: Sender<InputEvent>) -> Result<(), WindowsDriverError> {
        // Store the sender in thread-local storage for the callback
        HOOK_SENDER.with(|s| {
            *s.borrow_mut() = Some(sender);
        });

        // Install the low-level keyboard hook
        // SAFETY: We pass null for hmod (current process) and 0 for thread ID (all threads)
        let hook = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_proc),
                HINSTANCE::default(),
                0,
            )
        };

        match hook {
            Ok(handle) => {
                debug!("Keyboard hook installed successfully");
                self.hook_handle = Some(handle);
                Ok(())
            }
            Err(e) => {
                error!("Failed to install keyboard hook: {}", e);
                // Clear the sender since we failed
                HOOK_SENDER.with(|s| {
                    *s.borrow_mut() = None;
                });
                Err(WindowsDriverError::hook_install_failed(e.code().0 as u32))
            }
        }
    }

    /// Uninstall the keyboard hook.
    ///
    /// This should be called before the thread exits to properly clean up.
    pub fn uninstall(&mut self) {
        if let Some(handle) = self.hook_handle.take() {
            // SAFETY: We're passing a valid hook handle that we received from SetWindowsHookExW
            let result = unsafe { UnhookWindowsHookEx(handle) };
            if result.is_err() {
                warn!("Failed to unhook keyboard hook");
            } else {
                debug!("Keyboard hook uninstalled");
            }
        }

        // Clear the thread-local sender
        HOOK_SENDER.with(|s| {
            *s.borrow_mut() = None;
        });
    }

    /// Check if the hook is currently installed.
    pub fn is_installed(&self) -> bool {
        self.hook_handle.is_some()
    }

    /// Get the running flag.
    pub fn running(&self) -> &Arc<AtomicBool> {
        &self.running
    }

    /// Run the Windows message loop.
    ///
    /// This function processes messages from the Windows message queue, which is
    /// required for the low-level keyboard hook to receive callbacks. The loop
    /// continues until:
    /// - The `running` flag is set to `false`
    /// - A `WM_QUIT` message is received
    ///
    /// # Thread Safety
    ///
    /// This must be called from the same thread that called `install()`.
    /// The message loop will sleep for 1ms between iterations when no messages
    /// are pending to avoid busy-waiting.
    pub fn run_message_loop(&self) {
        // Store the thread ID so we can post WM_QUIT from another thread
        // SAFETY: GetCurrentThreadId is safe to call and returns the current thread's ID
        let thread_id = unsafe { GetCurrentThreadId() };
        HOOK_THREAD_ID.store(thread_id, Ordering::SeqCst);
        debug!("Starting Windows message loop on thread {}", thread_id);
        let mut msg = MSG::default();

        while self.running.load(Ordering::SeqCst) {
            // Use PeekMessageW with PM_REMOVE to check for messages without blocking
            // SAFETY: We pass valid pointers and use the correct flags
            let has_message = unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.as_bool();

            if has_message {
                // Check for WM_QUIT to allow graceful shutdown
                if msg.message == WM_QUIT {
                    debug!("Received WM_QUIT, exiting message loop");
                    break;
                }

                // Translate and dispatch the message
                // SAFETY: msg is a valid MSG structure filled by PeekMessageW
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            } else {
                // No messages pending, sleep briefly to avoid busy-waiting
                // This keeps CPU usage low while still being responsive
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }

        // Clear the thread ID on exit
        HOOK_THREAD_ID.store(0, Ordering::SeqCst);
        debug!("Windows message loop stopped");
    }
}

impl Drop for HookManager {
    fn drop(&mut self) {
        self.uninstall();
    }
}

/// Key injector using the Windows SendInput API.
///
/// This struct provides the ability to inject keyboard events into the system
/// as if they came from a physical keyboard. Injected events are marked with
/// the LLKHF_INJECTED flag, allowing them to be distinguished from real input.
///
/// # Extended Keys
///
/// Some keys require the KEYEVENTF_EXTENDEDKEY flag to work correctly:
/// - Arrow keys (Up, Down, Left, Right)
/// - Navigation keys (Home, End, Page Up, Page Down, Insert, Delete)
/// - Numpad Enter (distinct from main Enter)
/// - Right-side modifiers (Right Ctrl, Right Alt)
/// - Print Screen, Pause, Num Lock
pub struct SendInputInjector;

impl SendInputInjector {
    /// Create a new key injector.
    ///
    /// The injector is stateless and can be created at any time.
    pub fn new() -> Self {
        Self
    }

    /// Inject a key press or release event.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to inject
    /// * `pressed` - `true` for key down, `false` for key up
    ///
    /// # Errors
    ///
    /// Returns `WindowsDriverError::SendInputFailed` if the injection fails.
    /// This can happen if:
    /// - Another application has blocked input injection
    /// - The system is in a secure state (e.g., UAC prompt)
    /// - The calling process lacks required privileges
    pub fn inject_key(&self, key: KeyCode, pressed: bool) -> Result<(), WindowsDriverError> {
        let vk_code = keycode_to_vk(key);

        // Determine flags
        let mut flags = KEYBD_EVENT_FLAGS::default();

        // Add KEYEVENTF_KEYUP for key release
        if !pressed {
            flags |= KEYEVENTF_KEYUP;
        }

        // Add KEYEVENTF_EXTENDEDKEY for extended keys
        if is_extended_key(key) {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }

        // Build the KEYBDINPUT structure
        let kbd_input = KEYBDINPUT {
            wVk: VIRTUAL_KEY(vk_code),
            wScan: 0, // Let Windows determine scan code from virtual key
            dwFlags: flags,
            time: 0,        // System will fill in the time
            dwExtraInfo: 0, // No extra info
        };

        // Build the INPUT structure with the keyboard input
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 { ki: kbd_input },
        };

        // Call SendInput with a single input event
        // SAFETY: We pass a valid INPUT structure with correct size
        let inputs = [input];
        let result = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };

        if result == 0 {
            // SendInput failed, get the error code
            let error_code = unsafe { GetLastError() };
            error!("SendInput failed with error code: {:?}", error_code);
            Err(WindowsDriverError::send_input_failed(error_code.0))
        } else {
            debug!(
                "Injected key {:?} {} (vk={:#x}, extended={})",
                key,
                if pressed { "down" } else { "up" },
                vk_code,
                is_extended_key(key)
            );
            Ok(())
        }
    }

    /// Inject a key press followed by a key release (a complete key tap).
    ///
    /// This is a convenience method for injecting a full key press cycle.
    pub fn inject_key_tap(&self, key: KeyCode) -> Result<(), WindowsDriverError> {
        self.inject_key(key, true)?;
        self.inject_key(key, false)
    }
}

impl Default for SendInputInjector {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a key is an extended key that requires KEYEVENTF_EXTENDEDKEY.
///
/// Extended keys are those on the enhanced keyboard that were not on the
/// original IBM PC/XT 83-key keyboard. These include:
/// - Right-side modifiers (Right Alt, Right Ctrl)
/// - Navigation cluster (Insert, Delete, Home, End, Page Up, Page Down)
/// - Arrow keys
/// - Numpad Enter, Numpad /, Print Screen, Pause, Num Lock
fn is_extended_key(key: KeyCode) -> bool {
    matches!(
        key,
        // Navigation keys
        KeyCode::Insert
            | KeyCode::Delete
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::PageUp
            | KeyCode::PageDown
            // Arrow keys
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            // Right-side modifiers
            | KeyCode::RightCtrl
            | KeyCode::RightAlt
            // Numpad keys that need extended flag
            | KeyCode::NumpadEnter
            | KeyCode::NumpadDivide
            // Other extended keys
            | KeyCode::PrintScreen
            | KeyCode::Pause
            | KeyCode::NumLock
            // Windows keys
            | KeyCode::LeftMeta
            | KeyCode::RightMeta
    )
}

/// Low-level keyboard hook callback.
///
/// This function is called by Windows for every keyboard event. It must complete
/// quickly (within ~100ms per Windows requirements) or keyboard input will lag.
///
/// # Safety
///
/// This is an unsafe extern function called by Windows. The `lparam` must be
/// a valid pointer to `KBDLLHOOKSTRUCT` when `ncode >= 0`.
unsafe extern "system" fn low_level_keyboard_proc(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // If ncode < 0, we must call CallNextHookEx and return its result
    if ncode < 0 {
        return CallNextHookEx(HHOOK::default(), ncode, wparam, lparam);
    }

    // Extract keyboard info from lparam
    // SAFETY: When ncode >= 0, lparam is guaranteed to be a valid KBDLLHOOKSTRUCT pointer
    let kb_struct = &*(lparam.0 as *const KBDLLHOOKSTRUCT);

    // Determine if this is a key press or release
    let pressed = matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
    let _is_keyup = matches!(wparam.0 as u32, WM_KEYUP | WM_SYSKEYUP);

    // Convert virtual key code to KeyCode
    let vk_code = kb_struct.vkCode as u16;
    let key = vk_to_keycode(vk_code);

    // Get timestamp (Windows provides milliseconds, we store as-is for now)
    let timestamp = kb_struct.time as u64;

    // Create the input event
    let event = InputEvent {
        key,
        pressed,
        timestamp,
    };

    // Send the event via the thread-local sender
    HOOK_SENDER.with(|s| {
        if let Some(sender) = s.borrow().as_ref() {
            // Non-blocking send - if the channel is full, we drop the event
            // rather than blocking the hook callback
            if sender.try_send(event).is_err() {
                // Channel full or disconnected, event dropped
            }
        }
    });

    // Pass the event to the next hook in the chain
    // TODO: In future, we'll return 1 to block events based on script decisions
    CallNextHookEx(HHOOK::default(), ncode, wparam, lparam)
}

/// Windows input source using low-level keyboard hook.
///
/// `WindowsInput` coordinates keyboard event capture via a low-level keyboard hook
/// and key injection via `SendInput`. It implements the `InputSource` trait for
/// integration with the KeyRx remapping engine.
///
/// # Architecture
///
/// ```text
/// ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
/// │  Physical KB    │────▶│  Hook Thread     │────▶│  Engine         │
/// │  (WH_KEYBOARD_LL)     │  (message pump)  │     │  (async)        │
/// └─────────────────┘     └──────────────────┘     └─────────────────┘
///                                                          │
///                                                          ▼
/// ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
/// │  Applications   │◀────│  SendInput       │◀────│  Remap Logic    │
/// │                 │     │                  │     │                 │
/// └─────────────────┘     └──────────────────┘     └─────────────────┘
/// ```
///
/// # Thread Model
///
/// - The hook and message pump run in a dedicated thread (required by Windows)
/// - Events are sent via a crossbeam channel to the async engine
/// - The `running` flag is shared via `Arc<AtomicBool>` for clean shutdown
/// - WM_QUIT is posted to the hook thread to stop the message pump
pub struct WindowsInput {
    /// Handle to the hook thread (set after start() is called).
    hook_thread: Option<JoinHandle<()>>,
    /// Receiver for events from the hook callback.
    rx: Receiver<InputEvent>,
    /// Sender for events (held to pass to the hook thread).
    tx: Sender<InputEvent>,
    /// Shared flag to signal shutdown.
    running: Arc<AtomicBool>,
    /// Key injector for sending output.
    injector: SendInputInjector,
}

impl WindowsInput {
    /// Create a new Windows input source.
    ///
    /// This initializes the input source but does not start the hook.
    /// Call [`start()`](Self::start) to begin capturing keyboard events.
    ///
    /// # Returns
    ///
    /// Returns a new `WindowsInput` instance ready to be started.
    pub fn new() -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let running = Arc::new(AtomicBool::new(false));
        let injector = SendInputInjector::new();

        debug!("WindowsInput created");

        Ok(Self {
            hook_thread: None,
            rx,
            tx,
            running,
            injector,
        })
    }

    /// Check if the driver is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get the event receiver.
    ///
    /// This can be used for direct access to the event channel, though
    /// typically events should be accessed via `poll_events()`.
    pub fn receiver(&self) -> &Receiver<InputEvent> {
        &self.rx
    }
}

impl Default for WindowsInput {
    fn default() -> Self {
        Self::new().expect("WindowsInput::new should not fail")
    }
}

impl Drop for WindowsInput {
    fn drop(&mut self) {
        // Ensure the driver is stopped and hook is released on drop.
        // This is critical for graceful cleanup even on panics or unexpected termination.
        if self.running.load(Ordering::Relaxed) {
            debug!("WindowsInput::drop - stopping driver...");
            self.running.store(false, Ordering::Relaxed);

            // Post WM_QUIT to the hook thread to break out of the message loop
            let thread_id = HOOK_THREAD_ID.load(Ordering::SeqCst);
            if thread_id != 0 {
                // SAFETY: PostThreadMessageW is safe to call with a valid thread ID
                let result =
                    unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
                if result.is_err() {
                    warn!("WindowsInput::drop - Failed to post WM_QUIT to hook thread");
                }
            }

            // Wait for the hook thread to finish
            if let Some(handle) = self.hook_thread.take() {
                debug!("WindowsInput::drop - waiting for hook thread...");
                match handle.join() {
                    Ok(()) => debug!("WindowsInput::drop - hook thread finished cleanly"),
                    Err(e) => warn!("WindowsInput::drop - hook thread panicked: {:?}", e),
                }
            }

            // Drain any remaining events
            while self.rx.try_recv().is_ok() {}

            debug!("WindowsInput::drop - cleanup complete");
        }
    }
}

#[async_trait]
impl InputSource for WindowsInput {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        if !self.running.load(Ordering::Relaxed) {
            trace!("poll_events called while not running");
            return Ok(vec![]);
        }

        // Non-blocking receive from the channel
        // Collect all available events without blocking
        let mut events = Vec::new();
        loop {
            match self.rx.try_recv() {
                Ok(event) => {
                    trace!(
                        "Received event: {:?} {}",
                        event.key,
                        if event.pressed { "down" } else { "up" }
                    );
                    events.push(event);
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    // No more events available
                    break;
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    // Channel closed - hook thread has stopped
                    error!("Event channel disconnected - hook thread may have crashed");
                    self.running.store(false, Ordering::Relaxed);
                    anyhow::bail!("Input hook disconnected unexpectedly");
                }
            }
        }

        if !events.is_empty() {
            debug!("poll_events returning {} events", events.len());
        }

        Ok(events)
    }

    async fn send_output(&mut self, action: OutputAction) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            trace!("send_output called while not running");
            return Ok(());
        }

        match action {
            OutputAction::KeyDown(key) => {
                debug!("Sending key down: {:?}", key);
                self.injector.inject_key(key, true)?;
            }
            OutputAction::KeyUp(key) => {
                debug!("Sending key up: {:?}", key);
                self.injector.inject_key(key, false)?;
            }
            OutputAction::KeyTap(key) => {
                debug!("Sending key tap: {:?}", key);
                self.injector.inject_key(key, true)?;
                self.injector.inject_key(key, false)?;
            }
            OutputAction::Block => {
                // Block does nothing - the original event is already captured
                // and won't be passed through unless we explicitly emit it
                trace!("Blocking key (no action needed)");
            }
            OutputAction::PassThrough => {
                // PassThrough is handled by the engine - it re-emits the original key
                // For the driver, this is a no-op since the engine will call
                // KeyDown/KeyUp for the original key if needed
                trace!("PassThrough (no action needed)");
            }
        }

        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            warn!("WindowsInput already running");
            return Ok(());
        }

        debug!("Starting WindowsInput...");

        // Set running flag before spawning thread
        self.running.store(true, Ordering::Relaxed);

        // Clone what we need for the thread
        let running = self.running.clone();
        let tx = self.tx.clone();

        // Spawn the hook thread
        let handle = thread::spawn(move || {
            debug!("Hook thread started");

            // Create the hook manager
            let mut hook_manager = HookManager::new(running.clone());

            // Install the hook
            match hook_manager.install(tx) {
                Ok(()) => {
                    debug!("Keyboard hook installed successfully");
                }
                Err(e) => {
                    error!("Failed to install keyboard hook: {:?}", e);
                    running.store(false, Ordering::Relaxed);
                    return;
                }
            }

            // Run the message loop (blocks until WM_QUIT or running=false)
            hook_manager.run_message_loop();

            // Uninstall the hook (also done in Drop, but explicit is clearer)
            hook_manager.uninstall();

            debug!("Hook thread finished");
        });

        self.hook_thread = Some(handle);

        // Give the hook thread a moment to start and install the hook
        // This is a simple approach; a more robust solution would use a channel
        // to signal that the hook was successfully installed
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Check if the thread is still running (hook installation succeeded)
        if !self.running.load(Ordering::Relaxed) {
            // Hook installation failed, wait for thread to finish
            if let Some(handle) = self.hook_thread.take() {
                let _ = handle.join();
            }
            anyhow::bail!("Failed to start keyboard hook");
        }

        debug!("WindowsInput started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            debug!("WindowsInput already stopped");
            return Ok(());
        }

        debug!("Stopping WindowsInput...");

        // Signal the hook thread to stop
        self.running.store(false, Ordering::Relaxed);

        // Post WM_QUIT to the hook thread to break out of the message loop
        let thread_id = HOOK_THREAD_ID.load(Ordering::SeqCst);
        if thread_id != 0 {
            // SAFETY: PostThreadMessageW is safe to call with a valid thread ID
            let result = unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
            if result.is_err() {
                warn!("Failed to post WM_QUIT to hook thread");
            } else {
                debug!("Posted WM_QUIT to hook thread {}", thread_id);
            }
        }

        // Wait for the hook thread to finish
        if let Some(handle) = self.hook_thread.take() {
            debug!("Waiting for hook thread to finish...");
            match handle.join() {
                Ok(()) => {
                    debug!("Hook thread finished cleanly");
                }
                Err(e) => {
                    error!("Hook thread panicked: {:?}", e);
                    // Continue with cleanup even if thread panicked
                }
            }
        }

        // Drain any remaining events from the channel
        while self.rx.try_recv().is_ok() {
            // Discard remaining events
        }

        debug!("WindowsInput stopped successfully");
        Ok(())
    }
}

/// Convert Windows virtual key code to KeyRx KeyCode.
/// This is a stub mapping - full implementation post-MVP.
#[allow(dead_code)]
fn vk_to_keycode(vk: u16) -> KeyCode {
    // Windows virtual key codes (from WinUser.h / VK_* constants)
    match vk {
        // Letters A-Z (0x41-0x5A)
        0x41 => KeyCode::A,
        0x42 => KeyCode::B,
        0x43 => KeyCode::C,
        0x44 => KeyCode::D,
        0x45 => KeyCode::E,
        0x46 => KeyCode::F,
        0x47 => KeyCode::G,
        0x48 => KeyCode::H,
        0x49 => KeyCode::I,
        0x4A => KeyCode::J,
        0x4B => KeyCode::K,
        0x4C => KeyCode::L,
        0x4D => KeyCode::M,
        0x4E => KeyCode::N,
        0x4F => KeyCode::O,
        0x50 => KeyCode::P,
        0x51 => KeyCode::Q,
        0x52 => KeyCode::R,
        0x53 => KeyCode::S,
        0x54 => KeyCode::T,
        0x55 => KeyCode::U,
        0x56 => KeyCode::V,
        0x57 => KeyCode::W,
        0x58 => KeyCode::X,
        0x59 => KeyCode::Y,
        0x5A => KeyCode::Z,
        // Numbers 0-9 (0x30-0x39)
        0x30 => KeyCode::Key0,
        0x31 => KeyCode::Key1,
        0x32 => KeyCode::Key2,
        0x33 => KeyCode::Key3,
        0x34 => KeyCode::Key4,
        0x35 => KeyCode::Key5,
        0x36 => KeyCode::Key6,
        0x37 => KeyCode::Key7,
        0x38 => KeyCode::Key8,
        0x39 => KeyCode::Key9,
        // Function keys F1-F12 (0x70-0x7B)
        0x70 => KeyCode::F1,
        0x71 => KeyCode::F2,
        0x72 => KeyCode::F3,
        0x73 => KeyCode::F4,
        0x74 => KeyCode::F5,
        0x75 => KeyCode::F6,
        0x76 => KeyCode::F7,
        0x77 => KeyCode::F8,
        0x78 => KeyCode::F9,
        0x79 => KeyCode::F10,
        0x7A => KeyCode::F11,
        0x7B => KeyCode::F12,
        // Modifier keys
        0x10 => KeyCode::LeftShift,  // VK_SHIFT (use left by default)
        0xA0 => KeyCode::LeftShift,  // VK_LSHIFT
        0xA1 => KeyCode::RightShift, // VK_RSHIFT
        0x11 => KeyCode::LeftCtrl,   // VK_CONTROL (use left by default)
        0xA2 => KeyCode::LeftCtrl,   // VK_LCONTROL
        0xA3 => KeyCode::RightCtrl,  // VK_RCONTROL
        0x12 => KeyCode::LeftAlt,    // VK_MENU (use left by default)
        0xA4 => KeyCode::LeftAlt,    // VK_LMENU
        0xA5 => KeyCode::RightAlt,   // VK_RMENU
        0x5B => KeyCode::LeftMeta,   // VK_LWIN
        0x5C => KeyCode::RightMeta,  // VK_RWIN
        // Navigation keys
        0x26 => KeyCode::Up,       // VK_UP
        0x28 => KeyCode::Down,     // VK_DOWN
        0x25 => KeyCode::Left,     // VK_LEFT
        0x27 => KeyCode::Right,    // VK_RIGHT
        0x24 => KeyCode::Home,     // VK_HOME
        0x23 => KeyCode::End,      // VK_END
        0x21 => KeyCode::PageUp,   // VK_PRIOR
        0x22 => KeyCode::PageDown, // VK_NEXT
        // Editing keys
        0x2D => KeyCode::Insert,    // VK_INSERT
        0x2E => KeyCode::Delete,    // VK_DELETE
        0x08 => KeyCode::Backspace, // VK_BACK
        // Whitespace
        0x20 => KeyCode::Space, // VK_SPACE
        0x09 => KeyCode::Tab,   // VK_TAB
        0x0D => KeyCode::Enter, // VK_RETURN
        // Lock keys
        0x14 => KeyCode::CapsLock,   // VK_CAPITAL
        0x90 => KeyCode::NumLock,    // VK_NUMLOCK
        0x91 => KeyCode::ScrollLock, // VK_SCROLL
        // Escape area
        0x1B => KeyCode::Escape,      // VK_ESCAPE
        0x2C => KeyCode::PrintScreen, // VK_SNAPSHOT
        0x13 => KeyCode::Pause,       // VK_PAUSE
        // Punctuation and symbols
        0xC0 => KeyCode::Grave,        // VK_OEM_3 (` ~)
        0xBD => KeyCode::Minus,        // VK_OEM_MINUS (- _)
        0xBB => KeyCode::Equal,        // VK_OEM_PLUS (= +)
        0xDB => KeyCode::LeftBracket,  // VK_OEM_4 ([ {)
        0xDD => KeyCode::RightBracket, // VK_OEM_6 (] })
        0xDC => KeyCode::Backslash,    // VK_OEM_5 (\ |)
        0xBA => KeyCode::Semicolon,    // VK_OEM_1 (; :)
        0xDE => KeyCode::Apostrophe,   // VK_OEM_7 (' ")
        0xBC => KeyCode::Comma,        // VK_OEM_COMMA (, <)
        0xBE => KeyCode::Period,       // VK_OEM_PERIOD (. >)
        0xBF => KeyCode::Slash,        // VK_OEM_2 (/ ?)
        // Numpad keys
        0x60 => KeyCode::Numpad0,        // VK_NUMPAD0
        0x61 => KeyCode::Numpad1,        // VK_NUMPAD1
        0x62 => KeyCode::Numpad2,        // VK_NUMPAD2
        0x63 => KeyCode::Numpad3,        // VK_NUMPAD3
        0x64 => KeyCode::Numpad4,        // VK_NUMPAD4
        0x65 => KeyCode::Numpad5,        // VK_NUMPAD5
        0x66 => KeyCode::Numpad6,        // VK_NUMPAD6
        0x67 => KeyCode::Numpad7,        // VK_NUMPAD7
        0x68 => KeyCode::Numpad8,        // VK_NUMPAD8
        0x69 => KeyCode::Numpad9,        // VK_NUMPAD9
        0x6B => KeyCode::NumpadAdd,      // VK_ADD
        0x6D => KeyCode::NumpadSubtract, // VK_SUBTRACT
        0x6A => KeyCode::NumpadMultiply, // VK_MULTIPLY
        0x6F => KeyCode::NumpadDivide,   // VK_DIVIDE
        0x6E => KeyCode::NumpadDecimal,  // VK_DECIMAL
        // Note: NumpadEnter shares VK_RETURN (0x0D) but has extended flag
        // Media keys
        0xAF => KeyCode::VolumeUp,       // VK_VOLUME_UP
        0xAE => KeyCode::VolumeDown,     // VK_VOLUME_DOWN
        0xAD => KeyCode::VolumeMute,     // VK_VOLUME_MUTE
        0xB3 => KeyCode::MediaPlayPause, // VK_MEDIA_PLAY_PAUSE
        0xB2 => KeyCode::MediaStop,      // VK_MEDIA_STOP
        0xB0 => KeyCode::MediaNext,      // VK_MEDIA_NEXT_TRACK
        0xB1 => KeyCode::MediaPrev,      // VK_MEDIA_PREV_TRACK
        // Unknown
        _ => KeyCode::Unknown(vk),
    }
}

/// Convert KeyRx KeyCode to Windows virtual key code.
/// This is a stub mapping - full implementation post-MVP.
#[allow(dead_code)]
fn keycode_to_vk(key: KeyCode) -> u16 {
    match key {
        // Letters A-Z
        KeyCode::A => 0x41,
        KeyCode::B => 0x42,
        KeyCode::C => 0x43,
        KeyCode::D => 0x44,
        KeyCode::E => 0x45,
        KeyCode::F => 0x46,
        KeyCode::G => 0x47,
        KeyCode::H => 0x48,
        KeyCode::I => 0x49,
        KeyCode::J => 0x4A,
        KeyCode::K => 0x4B,
        KeyCode::L => 0x4C,
        KeyCode::M => 0x4D,
        KeyCode::N => 0x4E,
        KeyCode::O => 0x4F,
        KeyCode::P => 0x50,
        KeyCode::Q => 0x51,
        KeyCode::R => 0x52,
        KeyCode::S => 0x53,
        KeyCode::T => 0x54,
        KeyCode::U => 0x55,
        KeyCode::V => 0x56,
        KeyCode::W => 0x57,
        KeyCode::X => 0x58,
        KeyCode::Y => 0x59,
        KeyCode::Z => 0x5A,
        // Numbers 0-9
        KeyCode::Key0 => 0x30,
        KeyCode::Key1 => 0x31,
        KeyCode::Key2 => 0x32,
        KeyCode::Key3 => 0x33,
        KeyCode::Key4 => 0x34,
        KeyCode::Key5 => 0x35,
        KeyCode::Key6 => 0x36,
        KeyCode::Key7 => 0x37,
        KeyCode::Key8 => 0x38,
        KeyCode::Key9 => 0x39,
        // Function keys F1-F12
        KeyCode::F1 => 0x70,
        KeyCode::F2 => 0x71,
        KeyCode::F3 => 0x72,
        KeyCode::F4 => 0x73,
        KeyCode::F5 => 0x74,
        KeyCode::F6 => 0x75,
        KeyCode::F7 => 0x76,
        KeyCode::F8 => 0x77,
        KeyCode::F9 => 0x78,
        KeyCode::F10 => 0x79,
        KeyCode::F11 => 0x7A,
        KeyCode::F12 => 0x7B,
        // Modifier keys
        KeyCode::LeftShift => 0xA0,
        KeyCode::RightShift => 0xA1,
        KeyCode::LeftCtrl => 0xA2,
        KeyCode::RightCtrl => 0xA3,
        KeyCode::LeftAlt => 0xA4,
        KeyCode::RightAlt => 0xA5,
        KeyCode::LeftMeta => 0x5B,
        KeyCode::RightMeta => 0x5C,
        // Navigation
        KeyCode::Up => 0x26,
        KeyCode::Down => 0x28,
        KeyCode::Left => 0x25,
        KeyCode::Right => 0x27,
        KeyCode::Home => 0x24,
        KeyCode::End => 0x23,
        KeyCode::PageUp => 0x21,
        KeyCode::PageDown => 0x22,
        // Editing
        KeyCode::Insert => 0x2D,
        KeyCode::Delete => 0x2E,
        KeyCode::Backspace => 0x08,
        // Whitespace
        KeyCode::Space => 0x20,
        KeyCode::Tab => 0x09,
        KeyCode::Enter => 0x0D,
        // Lock keys
        KeyCode::CapsLock => 0x14,
        KeyCode::NumLock => 0x90,
        KeyCode::ScrollLock => 0x91,
        // Escape area
        KeyCode::Escape => 0x1B,
        KeyCode::PrintScreen => 0x2C,
        KeyCode::Pause => 0x13,
        // Punctuation
        KeyCode::Grave => 0xC0,
        KeyCode::Minus => 0xBD,
        KeyCode::Equal => 0xBB,
        KeyCode::LeftBracket => 0xDB,
        KeyCode::RightBracket => 0xDD,
        KeyCode::Backslash => 0xDC,
        KeyCode::Semicolon => 0xBA,
        KeyCode::Apostrophe => 0xDE,
        KeyCode::Comma => 0xBC,
        KeyCode::Period => 0xBE,
        KeyCode::Slash => 0xBF,
        // Numpad
        KeyCode::Numpad0 => 0x60,
        KeyCode::Numpad1 => 0x61,
        KeyCode::Numpad2 => 0x62,
        KeyCode::Numpad3 => 0x63,
        KeyCode::Numpad4 => 0x64,
        KeyCode::Numpad5 => 0x65,
        KeyCode::Numpad6 => 0x66,
        KeyCode::Numpad7 => 0x67,
        KeyCode::Numpad8 => 0x68,
        KeyCode::Numpad9 => 0x69,
        KeyCode::NumpadAdd => 0x6B,
        KeyCode::NumpadSubtract => 0x6D,
        KeyCode::NumpadMultiply => 0x6A,
        KeyCode::NumpadDivide => 0x6F,
        KeyCode::NumpadEnter => 0x0D, // Same as Enter, distinguished by extended flag
        KeyCode::NumpadDecimal => 0x6E,
        // Media
        KeyCode::VolumeUp => 0xAF,
        KeyCode::VolumeDown => 0xAE,
        KeyCode::VolumeMute => 0xAD,
        KeyCode::MediaPlayPause => 0xB3,
        KeyCode::MediaStop => 0xB2,
        KeyCode::MediaNext => 0xB0,
        KeyCode::MediaPrev => 0xB1,
        // Unknown - return 0
        KeyCode::Unknown(vk) => vk,
    }
}

/// List all keyboard devices available on the system.
///
/// On Windows, this returns a single entry representing the system keyboard.
/// Windows uses a global low-level keyboard hook (WH_KEYBOARD_LL) which
/// intercepts all keyboard input regardless of which physical device generated it.
///
/// # Errors
///
/// This function currently always succeeds on Windows.
pub fn list_keyboards() -> Result<Vec<DeviceInfo>> {
    // Windows uses a global keyboard hook that captures all keyboard input.
    // We return a single "virtual" device representing the system keyboard.
    // Full HID device enumeration could be added later for device-specific handling.
    Ok(vec![DeviceInfo::new(
        PathBuf::from("\\\\?\\HID#System#Keyboard"),
        "System Keyboard (Global Hook)".to_string(),
        0, // Vendor ID not available via global hook
        0, // Product ID not available via global hook
        true,
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_input_default() {
        let input = WindowsInput::default();
        assert!(!input.is_running());
    }

    #[test]
    fn windows_input_new() {
        let input = WindowsInput::new().unwrap();
        assert!(!input.is_running());
    }

    #[test]
    fn windows_input_has_receiver() {
        let input = WindowsInput::new().unwrap();
        // Verify we can access the receiver (channel is empty initially)
        assert!(input.receiver().try_recv().is_err());
    }

    #[test]
    fn hook_manager_new() {
        let running = Arc::new(AtomicBool::new(true));
        let manager = HookManager::new(running.clone());
        assert!(!manager.is_installed());
        assert!(manager.running().load(Ordering::SeqCst));
    }

    #[test]
    fn hook_manager_running_flag() {
        let running = Arc::new(AtomicBool::new(true));
        let manager = HookManager::new(running.clone());

        // Check initial state
        assert!(manager.running().load(Ordering::SeqCst));

        // Modify the flag
        running.store(false, Ordering::SeqCst);
        assert!(!manager.running().load(Ordering::SeqCst));
    }

    #[test]
    fn hook_manager_uninstall_when_not_installed() {
        let running = Arc::new(AtomicBool::new(true));
        let mut manager = HookManager::new(running);
        // Should not panic when uninstalling a hook that was never installed
        manager.uninstall();
        assert!(!manager.is_installed());
    }

    #[test]
    fn list_keyboards_returns_system_keyboard() {
        let keyboards = list_keyboards().unwrap();
        assert_eq!(keyboards.len(), 1);
        assert!(keyboards[0].is_keyboard());
        assert!(keyboards[0].name().contains("System Keyboard"));
    }

    #[test]
    fn vk_keycode_mapping_letters() {
        assert_eq!(vk_to_keycode(0x41), KeyCode::A);
        assert_eq!(vk_to_keycode(0x5A), KeyCode::Z);
    }

    #[test]
    fn vk_keycode_mapping_numbers() {
        assert_eq!(vk_to_keycode(0x30), KeyCode::Key0);
        assert_eq!(vk_to_keycode(0x39), KeyCode::Key9);
    }

    #[test]
    fn vk_keycode_mapping_function_keys() {
        assert_eq!(vk_to_keycode(0x70), KeyCode::F1);
        assert_eq!(vk_to_keycode(0x7B), KeyCode::F12);
    }

    #[test]
    fn vk_keycode_mapping_modifiers() {
        assert_eq!(vk_to_keycode(0xA0), KeyCode::LeftShift);
        assert_eq!(vk_to_keycode(0xA1), KeyCode::RightShift);
        assert_eq!(vk_to_keycode(0xA2), KeyCode::LeftCtrl);
        assert_eq!(vk_to_keycode(0xA3), KeyCode::RightCtrl);
        assert_eq!(vk_to_keycode(0xA4), KeyCode::LeftAlt);
        assert_eq!(vk_to_keycode(0xA5), KeyCode::RightAlt);
    }

    #[test]
    fn vk_keycode_mapping_special() {
        assert_eq!(vk_to_keycode(0x1B), KeyCode::Escape);
        assert_eq!(vk_to_keycode(0x14), KeyCode::CapsLock);
        assert_eq!(vk_to_keycode(0x20), KeyCode::Space);
        assert_eq!(vk_to_keycode(0x0D), KeyCode::Enter);
        assert_eq!(vk_to_keycode(0x09), KeyCode::Tab);
    }

    #[test]
    fn vk_keycode_mapping_unknown() {
        assert_eq!(vk_to_keycode(0xFF), KeyCode::Unknown(0xFF));
    }

    #[test]
    fn keycode_to_vk_roundtrip() {
        let keys = vec![
            KeyCode::A,
            KeyCode::Z,
            KeyCode::Key0,
            KeyCode::F1,
            KeyCode::Escape,
            KeyCode::CapsLock,
            KeyCode::Space,
        ];
        for key in keys {
            let vk = keycode_to_vk(key);
            let back = vk_to_keycode(vk);
            assert_eq!(key, back, "Roundtrip failed for {:?}", key);
        }
    }

    #[test]
    fn send_input_injector_creation() {
        let injector = SendInputInjector::new();
        // Injector is stateless, just verify it can be created
        drop(injector);

        // Also test Default impl
        let _injector: SendInputInjector = Default::default();
    }

    #[test]
    fn extended_key_detection_navigation() {
        // Navigation keys should be extended
        assert!(is_extended_key(KeyCode::Insert));
        assert!(is_extended_key(KeyCode::Delete));
        assert!(is_extended_key(KeyCode::Home));
        assert!(is_extended_key(KeyCode::End));
        assert!(is_extended_key(KeyCode::PageUp));
        assert!(is_extended_key(KeyCode::PageDown));
    }

    #[test]
    fn extended_key_detection_arrows() {
        // Arrow keys should be extended
        assert!(is_extended_key(KeyCode::Up));
        assert!(is_extended_key(KeyCode::Down));
        assert!(is_extended_key(KeyCode::Left));
        assert!(is_extended_key(KeyCode::Right));
    }

    #[test]
    fn extended_key_detection_modifiers() {
        // Right-side modifiers should be extended
        assert!(is_extended_key(KeyCode::RightCtrl));
        assert!(is_extended_key(KeyCode::RightAlt));
        assert!(is_extended_key(KeyCode::LeftMeta));
        assert!(is_extended_key(KeyCode::RightMeta));

        // Left-side modifiers should NOT be extended (except meta)
        assert!(!is_extended_key(KeyCode::LeftCtrl));
        assert!(!is_extended_key(KeyCode::LeftAlt));
        assert!(!is_extended_key(KeyCode::LeftShift));
        assert!(!is_extended_key(KeyCode::RightShift));
    }

    #[test]
    fn extended_key_detection_numpad() {
        // NumpadEnter and NumpadDivide are extended
        assert!(is_extended_key(KeyCode::NumpadEnter));
        assert!(is_extended_key(KeyCode::NumpadDivide));

        // Other numpad keys are NOT extended
        assert!(!is_extended_key(KeyCode::Numpad0));
        assert!(!is_extended_key(KeyCode::Numpad5));
        assert!(!is_extended_key(KeyCode::NumpadAdd));
        assert!(!is_extended_key(KeyCode::NumpadMultiply));
    }

    #[test]
    fn extended_key_detection_regular_keys() {
        // Regular keys should NOT be extended
        assert!(!is_extended_key(KeyCode::A));
        assert!(!is_extended_key(KeyCode::Space));
        assert!(!is_extended_key(KeyCode::Enter));
        assert!(!is_extended_key(KeyCode::Tab));
        assert!(!is_extended_key(KeyCode::Escape));
        assert!(!is_extended_key(KeyCode::CapsLock));
        assert!(!is_extended_key(KeyCode::F1));
        assert!(!is_extended_key(KeyCode::Key1));
    }

    #[test]
    fn extended_key_detection_special() {
        // PrintScreen, Pause, NumLock are extended
        assert!(is_extended_key(KeyCode::PrintScreen));
        assert!(is_extended_key(KeyCode::Pause));
        assert!(is_extended_key(KeyCode::NumLock));

        // ScrollLock and CapsLock are NOT extended
        assert!(!is_extended_key(KeyCode::ScrollLock));
        assert!(!is_extended_key(KeyCode::CapsLock));
    }
}
