//! Platform abstraction layer for keyboard input/output.
//!
//! This module provides a trait-based abstraction for platform-specific keyboard
//! input capture and output injection. The [`Platform`] trait defines the contract
//! for all platform implementations, enabling dependency injection and testability.
//!
//! # Architecture
//!
//! The module follows the dependency inversion principle:
//! - High-level daemon code depends on the [`Platform`] trait abstraction
//! - Platform-specific code (Linux, Windows) implements the trait
//! - Factory function [`create_platform()`] creates platform-specific instances
//!
//! # Usage
//!
//! ```no_run
//! use keyrx_daemon::platform::create_platform;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut platform = create_platform()?;
//!     platform.initialize()?;
//!
//!     loop {
//!         let event = platform.capture_input()?;
//!         // Process event...
//!         platform.inject_output(event)?;
//!     }
//! }
//! ```
//!
//! # System Tray Support
//!
//! The [`SystemTray`] trait provides a cross-platform interface for system tray
//! icons with menu support. Each platform implements this trait using native
//! tray APIs:
//! - Linux: Uses `ksni` crate for StatusNotifierItem/D-Bus protocol
//! - Windows: Uses `tray-icon` crate for native Windows tray API
//!
//! The tray provides "Reload Config" and "Exit" menu items via [`TrayControlEvent`].

use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::event::KeyEvent;
use thiserror::Error;

pub mod common;
pub mod recovery;
pub use common::{DeviceInfo, PlatformError, Result as PlatformResult};

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

pub mod mock;

#[cfg(target_os = "linux")]
#[allow(unused_imports)] // EvdevInput/UinputOutput will be used in task #17
pub use linux::{EvdevInput, LinuxPlatform, LinuxSystemTray, UinputOutput};

#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform;

#[cfg(target_os = "macos")]
pub use macos::MacosPlatform;

#[allow(unused_imports)] // Will be used in tasks #17-20
pub use mock::{MockInput, MockOutput};

/// Platform abstraction for keyboard input/output operations.
///
/// This trait provides a unified interface for platform-specific keyboard event
/// capture and injection. Implementations exist for Linux (evdev/uinput) and
/// Windows (Low-Level Hooks/SendInput).
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` to support concurrent access from
/// the daemon event loop and web API handlers.
///
/// # Object Safety
///
/// This trait is object-safe, meaning it can be used as `Box<dyn Platform>`.
/// This enables dependency injection and runtime polymorphism.
///
/// # Examples
///
/// ```no_run
/// use keyrx_daemon::platform::{create_platform, Platform};
/// use keyrx_core::runtime::event::KeyEvent;
///
/// fn run_event_loop(mut platform: Box<dyn Platform>) -> Result<(), Box<dyn std::error::Error>> {
///     platform.initialize()?;
///
///     loop {
///         let event = platform.capture_input()?;
///         // Process event through keyrx_core runtime...
///         platform.inject_output(event)?;
///     }
/// }
/// ```
///
/// # Platform Support
///
/// - **Linux**: Uses evdev for input capture, uinput for output injection
/// - **Windows**: Uses Low-Level Keyboard Hook for input, SendInput API for output
///
/// # Lifecycle
///
/// 1. Create platform instance via [`create_platform()`]
/// 2. Call [`initialize()`](Platform::initialize) to set up resources
/// 3. Call [`capture_input()`](Platform::capture_input) and [`inject_output()`](Platform::inject_output) in event loop
/// 4. Call [`shutdown()`](Platform::shutdown) to clean up resources
pub trait Platform: Send + Sync {
    /// Initializes platform-specific resources.
    ///
    /// This method must be called before [`capture_input()`](Platform::capture_input)
    /// or [`inject_output()`](Platform::inject_output). It performs platform-specific
    /// setup such as:
    /// - Opening device handles
    /// - Installing keyboard hooks
    /// - Creating virtual output devices
    ///
    /// # Errors
    ///
    /// - [`PlatformError::PermissionDenied`]: Insufficient privileges to access devices
    ///   - Linux: User not in `input` group or lacking CAP_SYS_ADMIN
    ///   - Windows: Process not running as administrator
    /// - [`PlatformError::InitializationFailed`]: Platform setup failed
    /// - [`PlatformError::DeviceNotFound`]: No suitable input devices found
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let mut platform = create_platform()?;
    /// platform.initialize()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn initialize(&mut self) -> PlatformResult<()>;

    /// Captures the next keyboard input event (blocking).
    ///
    /// This method blocks until an input event is available from any monitored
    /// device. The event represents a key press or release from a physical keyboard.
    ///
    /// # Returns
    ///
    /// - `Ok(KeyEvent)`: Successfully captured an input event
    /// - `Err(PlatformError)`: An error occurred during event capture
    ///
    /// # Errors
    ///
    /// - [`PlatformError::Io`]: I/O error reading from device
    /// - [`PlatformError::DeviceNotFound`]: Input device was disconnected
    ///
    /// # Performance
    ///
    /// This method should complete in <1ms under normal conditions to maintain
    /// low input latency.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let mut platform = create_platform()?;
    /// platform.initialize()?;
    ///
    /// loop {
    ///     let event = platform.capture_input()?;
    ///     println!("Captured: {:?}", event);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn capture_input(&mut self) -> PlatformResult<KeyEvent>;

    /// Injects a keyboard output event to the operating system.
    ///
    /// This method sends a synthetic keyboard event that appears to applications
    /// as if it came from a real keyboard. The event will be delivered to the
    /// currently focused application.
    ///
    /// # Arguments
    ///
    /// * `event` - The keyboard event to inject (press or release)
    ///
    /// # Errors
    ///
    /// - [`PlatformError::InjectionFailed`]: Failed to inject the event
    /// - [`PlatformError::Io`]: I/O error during injection
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_daemon::platform::create_platform;
    /// use keyrx_core::runtime::event::KeyEvent;
    /// use keyrx_core::config::KeyCode;
    ///
    /// let mut platform = create_platform()?;
    /// platform.initialize()?;
    ///
    /// // Inject 'A' key press and release
    /// platform.inject_output(KeyEvent::Press(KeyCode::A))?;
    /// platform.inject_output(KeyEvent::Release(KeyCode::A))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn inject_output(&mut self, event: KeyEvent) -> PlatformResult<()>;

    /// Lists all available input devices.
    ///
    /// Returns information about all keyboard input devices that can be used
    /// for key remapping. Each device includes metadata such as name, path,
    /// and USB identifiers.
    ///
    /// # Returns
    ///
    /// Vector of device information structures, one per detected device.
    ///
    /// # Errors
    ///
    /// - [`PlatformError::Io`]: Failed to enumerate devices
    /// - [`PlatformError::PermissionDenied`]: Insufficient privileges
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let platform = create_platform()?;
    /// let devices = platform.list_devices()?;
    ///
    /// for device in devices {
    ///     println!("Device: {} at {}", device.name, device.path);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>>;

    /// Cleans up platform resources and shuts down.
    ///
    /// This method should be called when the daemon is exiting to ensure proper
    /// cleanup of device handles, hooks, and other platform resources. After
    /// calling this method, the platform should not be used further.
    ///
    /// # Errors
    ///
    /// - [`PlatformError::Io`]: Failed to release resources
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let mut platform = create_platform()?;
    /// platform.initialize()?;
    ///
    /// // ... use platform ...
    ///
    /// platform.shutdown()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn shutdown(&mut self) -> PlatformResult<()>;
}

/// Creates a platform-specific implementation of the Platform trait.
///
/// This factory function returns the appropriate platform implementation based
/// on the target operating system. The returned instance is heap-allocated and
/// trait-object-compatible for maximum flexibility.
///
/// # Returns
///
/// - `Ok(Box<dyn Platform>)`: Platform-specific implementation
/// - `Err(PlatformError::Unsupported)`: Platform not supported
///
/// # Platform Selection
///
/// - **Linux**: Returns `LinuxPlatform`
/// - **Windows**: Returns `WindowsPlatform`
/// - **Other**: Returns `PlatformError::Unsupported`
///
/// # Examples
///
/// ```no_run
/// use keyrx_daemon::platform::create_platform;
///
/// let mut platform = create_platform()?;
/// platform.initialize()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Testing
///
/// For testing, you can create a mock implementation instead of using this factory:
///
/// ```
/// use keyrx_daemon::platform::{Platform, DeviceInfo};
/// use keyrx_core::runtime::event::KeyEvent;
///
/// struct MockPlatform;
///
/// impl Platform for MockPlatform {
///     fn initialize(&mut self) -> Result<(), keyrx_daemon::platform::PlatformError> {
///         Ok(())
///     }
///     # fn capture_input(&mut self) -> Result<KeyEvent, keyrx_daemon::platform::PlatformError> {
///     #     unimplemented!()
///     # }
///     # fn inject_output(&mut self, event: KeyEvent) -> Result<(), keyrx_daemon::platform::PlatformError> {
///     #     Ok(())
///     # }
///     # fn list_devices(&self) -> Result<Vec<DeviceInfo>, keyrx_daemon::platform::PlatformError> {
///     #     Ok(vec![])
///     # }
///     # fn shutdown(&mut self) -> Result<(), keyrx_daemon::platform::PlatformError> {
///     #     Ok(())
///     # }
///     // ... implement other methods
/// }
/// ```
pub fn create_platform() -> PlatformResult<Box<dyn Platform>> {
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::LinuxPlatform::new()))
    }

    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(windows::WindowsPlatform::new()))
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Err(PlatformError::Unsupported {
            operation: format!("Platform creation on {}", std::env::consts::OS),
        })
    }
}

/// Errors that can occur during device operations.
#[derive(Debug, Error)]
#[allow(dead_code)] // Will be used in tasks #14-20
pub enum DeviceError {
    /// Device was not found or could not be opened.
    #[error("device not found: {0}")]
    NotFound(String),

    /// Permission denied when accessing device.
    /// This typically occurs when the user lacks privileges to access input devices.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// End of event stream reached.
    /// For input devices, this indicates no more events are available.
    /// This is a normal termination condition, not an error.
    #[error("end of stream")]
    EndOfStream,

    /// Failed to inject event into output device.
    #[error("event injection failed: {0}")]
    InjectionFailed(String),

    /// I/O error occurred during device operation.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur during system tray operations.
#[derive(Debug, Error)]
#[allow(dead_code)] // Will be used by Linux tray implementation in task 4
pub enum TrayError {
    /// System tray is not available (e.g., headless server, no desktop environment).
    #[error("system tray not available: {0}")]
    NotAvailable(String),

    /// Failed to load tray icon.
    #[error("failed to load icon: {0}")]
    IconLoadFailed(String),

    /// General tray initialization or operation error.
    #[error("tray error: {0}")]
    Other(String),
}

/// Events generated by the system tray menu.
///
/// These events are sent when the user interacts with the tray icon's context menu.
/// Platform-specific tray implementations translate native menu events into these
/// cross-platform events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Will be used by Linux tray implementation in task 4
pub enum TrayControlEvent {
    /// User requested to reload the configuration.
    /// The daemon should re-read and apply the `.krx` config file.
    Reload,

    /// User requested to open the web UI configuration editor.
    /// The daemon should open the default browser to the web UI URL.
    OpenWebUI,

    /// User requested to exit the daemon.
    /// The daemon should perform a clean shutdown.
    Exit,
}

/// Result of a single iteration of event processing.
///
/// This enum indicates whether the event loop should continue normally,
/// reload configuration, or exit gracefully. It is used by platform
/// implementations to signal control events from the system tray or
/// other sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessResult {
    /// Continue processing events normally.
    Continue,
    /// User requested configuration reload (e.g., via tray menu).
    ReloadRequested,
    /// User requested exit (e.g., via tray menu).
    ExitRequested,
}

/// Cross-platform system tray interface.
///
/// This trait abstracts the system tray functionality, allowing platform-specific
/// implementations to provide native tray icons with menu support.
///
/// # Implementation Notes
///
/// - `new()` may fail if the system tray is unavailable (headless systems, minimal DE)
/// - `poll_event()` must be non-blocking (return `None` immediately if no events)
/// - `shutdown()` should release all tray resources
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::platform::{SystemTray, TrayControlEvent, TrayError};
///
/// fn run_with_tray<T: SystemTray>(mut tray: T) -> Result<(), TrayError> {
///     loop {
///         if let Some(event) = tray.poll_event() {
///             match event {
///                 TrayControlEvent::Reload => {
///                     println!("Reloading configuration...");
///                 }
///                 TrayControlEvent::OpenWebUI => {
///                     println!("Opening web UI...");
///                 }
///                 TrayControlEvent::Exit => {
///                     println!("Shutting down...");
///                     tray.shutdown()?;
///                     break;
///                 }
///             }
///         }
///         std::thread::sleep(std::time::Duration::from_millis(10));
///     }
///     Ok(())
/// }
/// ```
#[allow(dead_code)] // Will be used by Linux tray implementation in task 4
pub trait SystemTray {
    /// Creates a new system tray icon with menu.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: Tray icon created successfully
    /// - `Err(TrayError::NotAvailable)`: System tray not available
    /// - `Err(TrayError::IconLoadFailed)`: Failed to load tray icon
    /// - `Err(TrayError::Other)`: Other initialization error
    ///
    /// # Platform Notes
    ///
    /// - Linux: Requires a running desktop environment with StatusNotifierItem support
    /// - Windows: Requires the Windows message loop to be running
    fn new() -> Result<Self, TrayError>
    where
        Self: Sized;

    /// Polls for tray menu events without blocking.
    ///
    /// This method should return immediately, checking for any pending menu
    /// events without waiting.
    ///
    /// # Returns
    ///
    /// - `Some(TrayControlEvent)`: A menu event occurred
    /// - `None`: No pending events
    ///
    /// # Performance
    ///
    /// This method must complete in <1μs when no events are pending
    /// to avoid impacting keyboard event processing latency.
    fn poll_event(&self) -> Option<TrayControlEvent>;

    /// Releases all tray resources and removes the icon.
    ///
    /// After calling this method, the tray icon will no longer be visible
    /// and no more events will be generated.
    ///
    /// # Errors
    ///
    /// - `Err(TrayError::Other)`: Failed to release resources
    fn shutdown(&mut self) -> Result<(), TrayError>;
}

/// Input device trait for capturing keyboard events.
///
/// # Device Ownership
///
/// Input devices support exclusive access via the `grab()` and `release()` methods:
/// - `grab()`: Obtains exclusive access to the device, preventing other applications
///   from receiving events from this device.
/// - `release()`: Releases exclusive access, allowing other applications to receive events.
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::platform::{InputDevice, DeviceError};
///
/// fn process_input<I: InputDevice>(mut input: I) -> Result<(), DeviceError> {
///     input.grab()?;
///
///     loop {
///         match input.next_event() {
///             Ok(event) => println!("Event: {:?}", event),
///             Err(DeviceError::EndOfStream) => break,
///             Err(e) => return Err(e),
///         }
///     }
///
///     input.release()?;
///     Ok(())
/// }
/// ```
#[allow(dead_code)] // Will be implemented in tasks #14-16
pub trait InputDevice {
    /// Retrieves the next keyboard event from the device.
    ///
    /// # Returns
    ///
    /// - `Ok(KeyEvent)`: Successfully read an event
    /// - `Err(DeviceError::EndOfStream)`: No more events available (normal termination)
    /// - `Err(DeviceError::Io)`: I/O error occurred
    /// - `Err(DeviceError::PermissionDenied)`: Insufficient permissions
    fn next_event(&mut self) -> Result<KeyEvent, DeviceError>;

    /// Obtains exclusive access to the device.
    ///
    /// After calling this method, other applications will not receive events from
    /// this device until `release()` is called. This is essential for key remapping
    /// to prevent the original keystrokes from reaching applications.
    ///
    /// # Platform Notes
    ///
    /// - Linux: Uses `EVIOCGRAB` ioctl
    /// - Windows: Uses low-level keyboard hooks with event suppression
    /// - Mock: Sets an internal flag for testing
    ///
    /// # Errors
    ///
    /// - `DeviceError::PermissionDenied`: Insufficient privileges
    /// - `DeviceError::Io`: Underlying system call failed
    fn grab(&mut self) -> Result<(), DeviceError>;

    /// Releases exclusive access to the device.
    ///
    /// After calling this method, other applications can receive events from this device.
    ///
    /// # Errors
    ///
    /// - `DeviceError::Io`: Underlying system call failed
    fn release(&mut self) -> Result<(), DeviceError>;
}

/// Output device trait for injecting keyboard events.
///
/// Output devices emit synthetic keyboard events that appear to applications
/// as if they came from a real keyboard.
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::platform::{OutputDevice, DeviceError};
/// use keyrx_core::runtime::event::KeyEvent;
/// use keyrx_core::config::KeyCode;
///
/// fn inject_events<O: OutputDevice>(mut output: O) -> Result<(), DeviceError> {
///     output.inject_event(KeyEvent::Press(KeyCode::A))?;
///     output.inject_event(KeyEvent::Release(KeyCode::A))?;
///     Ok(())
/// }
/// ```
#[allow(dead_code)] // Will be implemented in tasks #15-16
pub trait OutputDevice {
    /// Injects a keyboard event into the output device.
    ///
    /// The injected event will be visible to all applications as if it came from
    /// a physical keyboard.
    ///
    /// # Platform Notes
    ///
    /// - Linux: Uses uinput subsystem to create virtual keyboard
    /// - Windows: Uses `SendInput` API
    /// - Mock: Appends event to internal buffer for testing
    ///
    /// # Errors
    ///
    /// - `DeviceError::InjectionFailed`: Failed to inject event
    /// - `DeviceError::Io`: Underlying system call failed
    fn inject_event(&mut self, event: KeyEvent) -> Result<(), DeviceError>;
}

// TODO: This legacy Platform enum will be removed in future tasks
// once all code is migrated to use the new Platform trait.
#[allow(dead_code)]
#[allow(clippy::large_enum_variant)] // LinuxPlatform is large but this enum is a placeholder
pub enum LegacyPlatform {
    #[cfg(target_os = "linux")]
    Linux(LinuxPlatform),
    #[cfg(target_os = "windows")]
    Windows(WindowsPlatform),
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    Unsupported,
}

impl LegacyPlatform {
    #[allow(dead_code)]
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        {
            LegacyPlatform::Linux(LinuxPlatform::new())
        }
        #[cfg(all(target_os = "windows", not(target_os = "linux")))]
        {
            LegacyPlatform::Windows(WindowsPlatform::new())
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            LegacyPlatform::Unsupported
        }
    }

    /// Initializes the platform with device configurations.
    ///
    /// # Arguments
    ///
    /// * `configs` - Slice of device configurations to initialize with
    ///
    /// # Errors
    ///
    /// Returns an error if platform initialization fails.
    #[allow(dead_code)]
    pub fn init(&mut self, configs: &[DeviceConfig]) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            #[cfg(target_os = "linux")]
            LegacyPlatform::Linux(p) => p.init(configs),
            #[cfg(target_os = "windows")]
            LegacyPlatform::Windows(p) => {
                // Windows platform doesn't use configs yet
                let _ = configs;
                p.init()
            }
            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            LegacyPlatform::Unsupported => {
                let _ = configs;
                Ok(())
            }
        }
    }

    /// Processes events from the platform (keyboard input and tray menu events).
    ///
    /// # Returns
    ///
    /// - `Ok(ProcessResult::Continue)`: Normal operation, continue processing
    /// - `Ok(ProcessResult::ReloadRequested)`: User requested config reload
    /// - `Ok(ProcessResult::ExitRequested)`: User requested exit
    /// - `Err(...)`: An error occurred during processing
    #[allow(dead_code)]
    pub fn process_events(&mut self) -> Result<ProcessResult, Box<dyn std::error::Error>> {
        match self {
            #[cfg(target_os = "linux")]
            LegacyPlatform::Linux(p) => p.process_events(),
            #[cfg(target_os = "windows")]
            LegacyPlatform::Windows(p) => {
                p.process_events()?;
                Ok(ProcessResult::Continue)
            }
            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            LegacyPlatform::Unsupported => Ok(ProcessResult::Continue),
        }
    }

    /// Shuts down the platform, releasing all resources.
    ///
    /// This should be called when the daemon is exiting to ensure
    /// proper cleanup of input devices and the system tray.
    #[allow(dead_code)]
    pub fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            #[cfg(target_os = "linux")]
            LegacyPlatform::Linux(p) => p.shutdown(),
            #[cfg(target_os = "windows")]
            LegacyPlatform::Windows(_p) => Ok(()),
            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            LegacyPlatform::Unsupported => Ok(()),
        }
    }
}

impl Default for LegacyPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_core::config::KeyCode;

    /// Mock platform implementation for testing.
    ///
    /// This demonstrates that the Platform trait is object-safe and can be
    /// used for dependency injection in tests.
    struct MockPlatform {
        initialized: bool,
        events: Vec<KeyEvent>,
        event_index: usize,
    }

    impl MockPlatform {
        fn new() -> Self {
            Self {
                initialized: false,
                events: vec![],
                event_index: 0,
            }
        }

        fn with_events(events: Vec<KeyEvent>) -> Self {
            Self {
                initialized: false,
                events,
                event_index: 0,
            }
        }
    }

    impl super::Platform for MockPlatform {
        fn initialize(&mut self) -> PlatformResult<()> {
            self.initialized = true;
            Ok(())
        }

        fn capture_input(&mut self) -> PlatformResult<KeyEvent> {
            if !self.initialized {
                return Err(PlatformError::InitializationFailed {
                    reason: "Platform not initialized".to_string(),
                });
            }

            if self.event_index < self.events.len() {
                let event = self.events[self.event_index].clone();
                self.event_index += 1;
                Ok(event)
            } else {
                Err(PlatformError::DeviceNotFound("No more events".to_string()))
            }
        }

        fn inject_output(&mut self, _event: KeyEvent) -> PlatformResult<()> {
            if !self.initialized {
                return Err(PlatformError::InitializationFailed {
                    reason: "Platform not initialized".to_string(),
                });
            }
            Ok(())
        }

        fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>> {
            Ok(vec![DeviceInfo {
                id: "mock-0".to_string(),
                name: "Mock Keyboard".to_string(),
                path: "/dev/mock/kbd0".to_string(),
                vendor_id: 0x1234,
                product_id: 0x5678,
            }])
        }

        fn shutdown(&mut self) -> PlatformResult<()> {
            self.initialized = false;
            Ok(())
        }
    }

    #[test]
    fn test_platform_trait_is_object_safe() {
        // This test verifies that Platform can be used as a trait object (Box<dyn Platform>)
        let platform: Box<dyn super::Platform> = Box::new(MockPlatform::new());
        let _ = platform; // Compile-time check that trait object works
    }

    #[test]
    fn test_mock_platform_lifecycle() {
        let mut platform = MockPlatform::new();

        // Should not be initialized yet
        assert!(!platform.initialized);

        // Initialize the platform
        platform.initialize().unwrap();
        assert!(platform.initialized);

        // Can inject output after initialization
        let event = KeyEvent::press(KeyCode::A);
        platform.inject_output(event).unwrap();

        // Can list devices
        let devices = platform.list_devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Mock Keyboard");

        // Shutdown
        platform.shutdown().unwrap();
        assert!(!platform.initialized);
    }

    #[test]
    fn test_mock_platform_capture_input() {
        let events = vec![
            KeyEvent::press(KeyCode::A),
            KeyEvent::release(KeyCode::A),
            KeyEvent::press(KeyCode::B),
        ];

        let mut platform = MockPlatform::with_events(events);
        platform.initialize().unwrap();

        // Capture first event
        let event1 = platform.capture_input().unwrap();
        assert_eq!(event1.keycode(), KeyCode::A);
        assert!(event1.is_press());

        // Capture second event
        let event2 = platform.capture_input().unwrap();
        assert_eq!(event2.keycode(), KeyCode::A);
        assert!(event2.is_release());

        // Capture third event
        let event3 = platform.capture_input().unwrap();
        assert_eq!(event3.keycode(), KeyCode::B);
        assert!(event3.is_press());

        // No more events
        let result = platform.capture_input();
        assert!(result.is_err());
    }

    #[test]
    fn test_platform_requires_initialization() {
        let mut platform = MockPlatform::new();

        // capture_input should fail before initialization
        let result = platform.capture_input();
        assert!(matches!(
            result,
            Err(PlatformError::InitializationFailed { .. })
        ));

        // inject_output should fail before initialization
        let result = platform.inject_output(KeyEvent::press(KeyCode::A));
        assert!(matches!(
            result,
            Err(PlatformError::InitializationFailed { .. })
        ));
    }

    #[test]
    fn test_platform_as_trait_object() {
        // Demonstrates dependency injection pattern
        fn run_with_platform(mut platform: Box<dyn super::Platform>) -> PlatformResult<()> {
            platform.initialize()?;
            let devices = platform.list_devices()?;
            assert!(!devices.is_empty());
            platform.shutdown()?;
            Ok(())
        }

        let platform: Box<dyn super::Platform> = Box::new(MockPlatform::new());
        run_with_platform(platform).unwrap();
    }

    #[test]
    fn test_device_info_equality() {
        let device1 = DeviceInfo {
            id: "kbd-0".to_string(),
            name: "Keyboard".to_string(),
            path: "/dev/input/event0".to_string(),
            vendor_id: 0x1234,
            product_id: 0x5678,
        };

        let device2 = DeviceInfo {
            id: "kbd-0".to_string(),
            name: "Keyboard".to_string(),
            path: "/dev/input/event0".to_string(),
            vendor_id: 0x1234,
            product_id: 0x5678,
        };

        assert_eq!(device1, device2);
    }
}
