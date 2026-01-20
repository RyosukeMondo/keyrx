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

use crossbeam_channel::{unbounded, Receiver, Sender};
use keyrx_core::runtime::KeyEvent;

use crate::platform::{DeviceInfo, Platform, PlatformError, PlatformResult};

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
    sender: Sender<KeyEvent>,
    receiver: Receiver<KeyEvent>,
    initialized: Arc<Mutex<bool>>,
}

#[cfg(target_os = "macos")]
impl MacosPlatform {
    /// Creates a new macOS platform instance.
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            input: MacosInputCapture::new(receiver.clone()),
            output: MacosOutputInjector::new(),
            sender,
            receiver,
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

// SAFETY: MacosPlatform is Send + Sync because:
// - crossbeam_channel is thread-safe
// - Arc<Mutex<>> provides safe concurrent access
// - rdev and enigo operations are thread-safe
#[cfg(target_os = "macos")]
unsafe impl Send for MacosPlatform {}
#[cfg(target_os = "macos")]
unsafe impl Sync for MacosPlatform {}
