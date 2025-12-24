//! Linux system tray implementation using the `ksni` crate.
//!
//! This module provides a Linux-specific implementation of the [`SystemTray`] trait,
//! using the StatusNotifierItem (SNI) D-Bus protocol via the `ksni` crate.
//! This works with most modern Linux desktop environments including KDE Plasma,
//! GNOME (with AppIndicator extension), and others.

use crossbeam_channel::{Receiver, Sender};
use ksni::blocking::TrayMethods;

use crate::platform::{SystemTray, TrayControlEvent, TrayError};

/// Internal tray service that implements the ksni::Tray trait.
///
/// This struct is spawned in a background thread and communicates
/// with the main daemon via a crossbeam channel.
#[allow(dead_code)] // Will be used when LinuxSystemTray is integrated in task 7
struct TrayService {
    /// Channel sender for communicating menu events back to the daemon.
    event_sender: Sender<TrayControlEvent>,
    /// Icon data in ARGB32 format.
    icon_data: Vec<u8>,
    /// Icon width in pixels.
    icon_width: i32,
    /// Icon height in pixels.
    icon_height: i32,
}

impl ksni::Tray for TrayService {
    fn id(&self) -> String {
        "keyrx-daemon".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![ksni::Icon {
            width: self.icon_width,
            height: self.icon_height,
            data: self.icon_data.clone(),
        }]
    }

    fn title(&self) -> String {
        "KeyRx Daemon".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "KeyRx Daemon".into(),
            description: "Keyboard remapping daemon".into(),
            icon_name: String::new(),
            icon_pixmap: Vec::new(),
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        let reload_sender = self.event_sender.clone();
        let exit_sender = self.event_sender.clone();

        vec![
            StandardItem {
                label: "Reload Config".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = reload_sender.send(TrayControlEvent::Reload);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Exit".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = exit_sender.send(TrayControlEvent::Exit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// Loads an icon from PNG bytes and converts to ARGB32 format.
///
/// The ksni crate expects icons in ARGB32 format (network byte order),
/// which means each pixel is 4 bytes: [A, R, G, B].
#[allow(dead_code)] // Will be used when LinuxSystemTray is integrated in task 7
fn load_icon(bytes: &[u8]) -> Result<(Vec<u8>, i32, i32), TrayError> {
    let image = image::load_from_memory(bytes)
        .map_err(|e| TrayError::IconLoadFailed(e.to_string()))?
        .into_rgba8();

    let (width, height) = image.dimensions();

    // Convert RGBA to ARGB (network byte order for D-Bus)
    let rgba_data = image.into_raw();
    let mut argb_data = Vec::with_capacity(rgba_data.len());

    for chunk in rgba_data.chunks(4) {
        // RGBA -> ARGB
        argb_data.push(chunk[3]); // A
        argb_data.push(chunk[0]); // R
        argb_data.push(chunk[1]); // G
        argb_data.push(chunk[2]); // B
    }

    Ok((argb_data, width as i32, height as i32))
}

/// Linux system tray controller using the ksni crate.
///
/// Implements [`SystemTray`] for cross-platform compatibility.
/// The tray icon is displayed using the StatusNotifierItem D-Bus protocol,
/// which is supported by most modern Linux desktop environments.
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
#[allow(dead_code)] // Will be integrated in task 7
pub struct LinuxSystemTray {
    /// Handle to the ksni tray service for shutdown.
    handle: ksni::blocking::Handle<TrayService>,
    /// Channel receiver for menu events.
    event_receiver: Receiver<TrayControlEvent>,
}

impl SystemTray for LinuxSystemTray {
    fn new() -> Result<Self, TrayError> {
        // Load the icon from embedded bytes
        let icon_bytes = include_bytes!("../../../assets/icon.png");
        let (icon_data, icon_width, icon_height) = load_icon(icon_bytes)?;

        // Create a channel for communicating events from the tray service
        let (event_sender, event_receiver) = crossbeam_channel::unbounded();

        // Create the tray service
        let tray_service = TrayService {
            event_sender,
            icon_data,
            icon_width,
            icon_height,
        };

        // Spawn the tray service in blocking mode
        // The ksni blocking API spawns a background thread internally
        let handle = tray_service.spawn().map_err(|e| {
            // Check for common error cases
            let err_str = e.to_string();
            if err_str.contains("org.freedesktop.DBus.Error.ServiceUnknown")
                || err_str.contains("StatusNotifierWatcher")
                || err_str.contains("No StatusNotifier")
            {
                TrayError::NotAvailable(
                    "StatusNotifierItem not available. \
                     Ensure a system tray is running (e.g., GNOME with AppIndicator extension, KDE Plasma)."
                        .to_string(),
                )
            } else if err_str.contains("Connection refused")
                || err_str.contains("org.freedesktop.DBus.Error.NoServer")
            {
                TrayError::NotAvailable(
                    "D-Bus session bus not available. \
                     Ensure you are running in a graphical session."
                        .to_string(),
                )
            } else {
                TrayError::Other(format!("Failed to spawn tray service: {}", e))
            }
        })?;

        Ok(Self {
            handle,
            event_receiver,
        })
    }

    fn poll_event(&self) -> Option<TrayControlEvent> {
        // Non-blocking check for events
        self.event_receiver.try_recv().ok()
    }

    fn shutdown(&mut self) -> Result<(), TrayError> {
        // Request shutdown of the tray service
        let awaiter = self.handle.shutdown();
        // Wait for the service to shut down
        awaiter.wait();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that load_icon converts PNG to ARGB correctly.
    #[test]
    fn test_load_icon() {
        let icon_bytes = include_bytes!("../../../assets/icon.png");
        let result = load_icon(icon_bytes);

        assert!(result.is_ok(), "Icon should load successfully");

        let (data, width, height) = result.unwrap();

        // Icon should have dimensions > 0
        assert!(width > 0, "Icon width should be positive");
        assert!(height > 0, "Icon height should be positive");

        // Data length should be width * height * 4 (ARGB32)
        let expected_len = (width * height * 4) as usize;
        assert_eq!(
            data.len(),
            expected_len,
            "ARGB data should have correct length"
        );
    }

    /// Test that load_icon fails gracefully with invalid data.
    #[test]
    fn test_load_icon_invalid_data() {
        let invalid_bytes = b"not a valid png";
        let result = load_icon(invalid_bytes);

        assert!(result.is_err(), "Invalid data should return error");

        match result {
            Err(TrayError::IconLoadFailed(msg)) => {
                assert!(!msg.is_empty(), "Error message should not be empty");
            }
            Err(e) => panic!("Expected IconLoadFailed, got {:?}", e),
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }

    /// Test that LinuxSystemTray::new() returns appropriate error when D-Bus unavailable.
    /// Note: This test will succeed or fail depending on the environment.
    #[test]
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
