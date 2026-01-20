//! macOS platform implementation using Accessibility API and IOKit.
//!
//! This module provides macOS-specific keyboard input capture and output injection
//! using the Accessibility API (via rdev) for input events and enigo for output
//! injection. Device enumeration uses IOKit for USB keyboard discovery.
//!
//! # Architecture
//!
//! - [`MacosInputCapture`]: Captures keyboard events using rdev::listen
//! - [`MacosOutputInjector`]: Injects keyboard events using enigo
//! - [`device_discovery`]: Enumerates USB keyboards via IOKit
//! - [`keycode_map`]: Bidirectional CGKeyCode ↔ KeyCode mapping
//! - [`MacosSystemTray`]: System menu bar integration
//! - [`permissions`]: Accessibility permission checking
//!
//! # Permissions
//!
//! macOS requires Accessibility permission for keyboard event capture.
//! The application must be granted permission in System Settings >
//! Privacy & Security > Accessibility.

pub mod device_discovery;
pub mod input_capture;
pub mod keycode_map;
pub mod output_injection;
pub mod permissions;
pub mod tray;

use std::sync::{Arc, Mutex};

use crossbeam_channel::unbounded;
use keyrx_core::runtime::KeyEvent;

use crate::platform::{DeviceInfo, InputDevice, OutputDevice, Platform, PlatformError, PlatformResult};

pub use input_capture::MacosInputCapture;
pub use output_injection::MacosOutputInjector;

/// macOS platform implementation.
///
/// This struct coordinates input capture, output injection, and device
/// enumeration for macOS systems.
#[cfg(target_os = "macos")]
pub struct MacosPlatform {
    input: MacosInputCapture,
    output: MacosOutputInjector,
    initialized: Arc<Mutex<bool>>,
}

#[cfg(target_os = "macos")]
impl MacosPlatform {
    /// Creates a new macOS platform instance.
    ///
    /// # Panics
    ///
    /// Panics if output injector creation fails (e.g., missing Accessibility permissions).
    /// This is checked explicitly in the `initialize()` method, so this should not panic
    /// in normal usage.
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            input: MacosInputCapture::new(receiver, sender),
            output: MacosOutputInjector::new()
                .expect("Failed to create output injector - check Accessibility permissions"),
            initialized: Arc::new(Mutex::new(false)),
        }
    }
}

#[cfg(target_os = "macos")]
impl Default for MacosPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "macos")]
impl Platform for MacosPlatform {
    fn initialize(&mut self) -> PlatformResult<()> {
        log::info!("Initializing macOS platform");

        // Check Accessibility permission
        if !permissions::check_accessibility_permission() {
            let error_message = permissions::get_permission_error_message();
            log::error!("Accessibility permission not granted");
            return Err(PlatformError::PermissionDenied(error_message));
        }

        // Mark as initialized
        {
            let mut initialized = self.initialized.lock().map_err(|e| {
                PlatformError::InitializationFailed {
                    reason: format!("Failed to acquire initialization lock: {}", e),
                }
            })?;
            *initialized = true;
        }

        log::info!("macOS platform initialized successfully");
        Ok(())
    }

    fn capture_input(&mut self) -> PlatformResult<KeyEvent> {
        // Verify initialization
        {
            let initialized = self.initialized.lock().map_err(|e| {
                PlatformError::InitializationFailed {
                    reason: format!("Failed to check initialization state: {}", e),
                }
            })?;
            if !*initialized {
                return Err(PlatformError::InitializationFailed {
                    reason: "Platform not initialized".to_string(),
                });
            }
        }

        // Delegate to input capture
        self.input
            .next_event()
            .map_err(|e| match e {
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
        // Verify initialization
        {
            let initialized = self.initialized.lock().map_err(|e| {
                PlatformError::InitializationFailed {
                    reason: format!("Failed to check initialization state: {}", e),
                }
            })?;
            if !*initialized {
                return Err(PlatformError::InitializationFailed {
                    reason: "Platform not initialized".to_string(),
                });
            }
        }

        // Delegate to output injector
        self.output
            .inject_event(event)
            .map_err(|e| match e {
                crate::platform::DeviceError::InjectionFailed(msg) => {
                    PlatformError::InjectionFailed {
                        reason: msg,
                        suggestion: "Check macOS Accessibility permissions and event structure"
                            .to_string(),
                    }
                }
                crate::platform::DeviceError::Io(io_err) => PlatformError::Io(io_err),
                _ => PlatformError::Io(std::io::Error::other(format!("Injection error: {}", e))),
            })
    }

    fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>> {
        // Delegate to device discovery
        device_discovery::list_keyboard_devices().map_err(|e| PlatformError::Io(
            std::io::Error::other(format!("Failed to enumerate devices: {}", e)),
        ))
    }

    fn shutdown(&mut self) -> PlatformResult<()> {
        log::info!("Shutting down macOS platform");

        // Mark as uninitialized
        {
            let mut initialized = self.initialized.lock().map_err(|e| {
                PlatformError::InitializationFailed {
                    reason: format!("Failed to acquire shutdown lock: {}", e),
                }
            })?;
            *initialized = false;
        }

        log::info!("macOS platform shutdown complete");
        Ok(())
    }
}

// SAFETY: MacosPlatform is Send + Sync because:
// - crossbeam_channel is thread-safe
// - Arc<Mutex<>> provides safe concurrent access
// - rdev and enigo operations are thread-safe
#[cfg(target_os = "macos")]
unsafe impl Send for MacosPlatform {}
#[cfg(target_os = "macos")]
unsafe impl Sync for MacosPlatform {}
