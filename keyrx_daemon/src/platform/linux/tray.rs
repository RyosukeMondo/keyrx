//! Linux system tray implementation using the `appindicator3` crate.
//!
//! This module provides a Linux-specific implementation of the [`SystemTray`] trait,
//! using the AppIndicator library via GTK bindings. This works with GNOME Shell
//! (with AppIndicator extension), KDE Plasma, and other desktop environments.

use std::cell::RefCell;
use std::rc::Rc;

use appindicator3::{prelude::*, Indicator, IndicatorCategory, IndicatorStatus};
use crossbeam_channel::Receiver;
use gtk::prelude::{GtkMenuItemExt, MenuShellExt, WidgetExt};

use crate::platform::{SystemTray, TrayControlEvent, TrayError};

/// Linux system tray controller using the appindicator3 crate.
///
/// Implements [`SystemTray`] for cross-platform compatibility.
/// The tray icon is displayed using the AppIndicator library,
/// which is supported by GNOME (with AppIndicator extension), KDE Plasma,
/// and most modern Linux desktop environments.
///
/// # Menu Items
///
/// - **Reload Config**: Triggers [`TrayControlEvent::Reload`]
/// - **Exit**: Triggers [`TrayControlEvent::Exit`]
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::platform::{SystemTray, TrayControlEvent};
/// use keyrx_daemon::platform::linux::tray::LinuxSystemTray;
///
/// let mut tray = LinuxSystemTray::new()?;
///
/// loop {
///     if let Some(event) = tray.poll_event() {
///         match event {
///             TrayControlEvent::Reload => {
///                 println!("Reloading configuration...");
///             }
///             TrayControlEvent::OpenWebUI => {
///                 println!("Opening web UI...");
///             }
///             TrayControlEvent::Exit => {
///                 println!("Shutting down...");
///                 tray.shutdown()?;
///                 break;
///             }
///         }
///     }
///     std::thread::sleep(std::time::Duration::from_millis(10));
/// }
/// # Ok::<(), keyrx_daemon::platform::TrayError>(())
/// ```
pub struct LinuxSystemTray {
    /// Channel receiver for menu events.
    event_receiver: Receiver<TrayControlEvent>,
    /// AppIndicator instance (kept alive for the lifetime of the tray).
    /// Uses Rc<RefCell<>> since GTK is single-threaded.
    _indicator: Rc<RefCell<Indicator>>,
}

impl SystemTray for LinuxSystemTray {
    fn new() -> Result<Self, TrayError> {
        // Initialize GTK on the main thread if not already initialized
        if gtk::is_initialized() {
            log::debug!("GTK already initialized");
        } else {
            gtk::init().map_err(|_| {
                TrayError::NotAvailable(
                    "Failed to initialize GTK. Ensure you are running in a graphical session."
                        .to_string(),
                )
            })?;
            log::debug!("GTK initialized successfully");
        }

        // Create a channel for communicating events from menu callbacks
        let (event_sender, event_receiver) = crossbeam_channel::unbounded();

        // Create the AppIndicator
        // Use "input-keyboard" as fallback icon name from icon theme
        let indicator = Indicator::new(
            "keyrx-daemon",
            "input-keyboard",
            IndicatorCategory::ApplicationStatus,
        );
        indicator.set_status(IndicatorStatus::Active);
        indicator.set_title(Some("KeyRx Daemon"));

        // Load icon from embedded bytes
        let icon_bytes = include_bytes!("../../../assets/icon.png");
        let icon_path = save_icon_to_temp(icon_bytes)?;
        indicator.set_icon_full(&icon_path, "KeyRx");

        log::debug!("AppIndicator created with ID 'keyrx-daemon', icon theme 'input-keyboard', custom icon at: {}", icon_path);

        // Create menu
        let menu = gtk::Menu::new();

        // Open Web UI menu item
        let webui_item = gtk::MenuItem::with_label("Open Web UI");
        let webui_sender = event_sender.clone();
        webui_item.connect_activate(move |_| {
            log::info!("Tray menu: Open Web UI clicked");
            if let Err(e) = webui_sender.send(TrayControlEvent::OpenWebUI) {
                log::error!("Failed to send OpenWebUI event: {}", e);
            } else {
                log::info!("Tray menu: OpenWebUI event sent successfully");
            }
        });
        menu.append(&webui_item);

        // Reload menu item
        let reload_item = gtk::MenuItem::with_label("Reload Config");
        let reload_sender = event_sender.clone();
        reload_item.connect_activate(move |_| {
            log::info!("Tray menu: Reload Config clicked");
            if let Err(e) = reload_sender.send(TrayControlEvent::Reload) {
                log::error!("Failed to send reload event: {}", e);
            } else {
                log::info!("Tray menu: Reload event sent successfully");
            }
        });
        menu.append(&reload_item);

        // Separator
        let separator = gtk::SeparatorMenuItem::new();
        menu.append(&separator);

        // Exit menu item
        let exit_item = gtk::MenuItem::with_label("Exit");
        let exit_sender = event_sender.clone();
        exit_item.connect_activate(move |_| {
            log::info!("Tray menu: Exit clicked");
            if let Err(e) = exit_sender.send(TrayControlEvent::Exit) {
                log::error!("Failed to send exit event: {}", e);
            } else {
                log::info!("Tray menu: Exit event sent successfully");
            }
        });
        menu.append(&exit_item);

        menu.show_all();
        indicator.set_menu(Some(&menu));

        log::info!("System tray initialized with AppIndicator");

        // Keep indicator alive (single-threaded GTK)
        let indicator = Rc::new(RefCell::new(indicator));

        Ok(Self {
            event_receiver,
            _indicator: indicator,
        })
    }

    fn poll_event(&self) -> Option<TrayControlEvent> {
        // Process pending GTK events (must be called from main thread)
        while gtk::events_pending() {
            gtk::main_iteration();
        }

        // Non-blocking check for events
        self.event_receiver.try_recv().ok()
    }

    fn shutdown(&mut self) -> Result<(), TrayError> {
        log::info!("Shutting down system tray");

        // Hide the tray icon by setting status to Passive
        if let Ok(indicator) = self._indicator.try_borrow_mut() {
            indicator.set_status(IndicatorStatus::Passive);
            log::debug!("Tray icon set to passive (hidden)");
        } else {
            log::warn!("Could not borrow indicator to hide it");
        }

        Ok(())
    }
}

/// Saves icon bytes to a temporary file and returns the path.
///
/// GTK/AppIndicator requires a file path for icons, not raw image data.
fn save_icon_to_temp(bytes: &[u8]) -> Result<String, TrayError> {
    use std::io::Write;

    // Create temp directory if it doesn't exist
    let temp_dir = std::env::temp_dir().join("keyrx");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| TrayError::IconLoadFailed(format!("Failed to create temp dir: {}", e)))?;

    // Write icon to temp file
    let icon_path = temp_dir.join("icon.png");
    let mut file = std::fs::File::create(&icon_path)
        .map_err(|e| TrayError::IconLoadFailed(format!("Failed to create icon file: {}", e)))?;
    file.write_all(bytes)
        .map_err(|e| TrayError::IconLoadFailed(format!("Failed to write icon data: {}", e)))?;

    Ok(icon_path
        .to_str()
        .ok_or_else(|| TrayError::IconLoadFailed("Icon path is not valid UTF-8".to_string()))?
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that save_icon_to_temp works correctly.
    #[test]
    fn test_save_icon_to_temp() {
        let icon_bytes = include_bytes!("../../../assets/icon.png");
        let result = save_icon_to_temp(icon_bytes);

        assert!(result.is_ok(), "Icon should save successfully");

        let path = result.unwrap();
        assert!(
            std::path::Path::new(&path).exists(),
            "Icon file should exist"
        );
    }

    /// Test that LinuxSystemTray::new() returns appropriate error when GTK unavailable.
    /// Note: This test will succeed or fail depending on the environment.
    #[test]
    #[ignore] // Ignored by default as it requires a graphical session
    fn test_linux_system_tray_new() {
        // Try to create a tray - this will fail in headless CI environments
        // but should succeed in a desktop session
        let result = LinuxSystemTray::new();

        match result {
            Ok(mut tray) => {
                // If it succeeded, verify we can poll (should return None initially)
                assert!(tray.poll_event().is_none());
                // Clean up
                let _ = tray.shutdown();
            }
            Err(TrayError::NotAvailable(msg)) => {
                // Expected in headless environments
                println!("Tray not available (expected in CI): {}", msg);
            }
            Err(e) => {
                // Other errors might indicate a bug
                println!("Unexpected error (may be ok in some environments): {:?}", e);
            }
        }
    }
}
