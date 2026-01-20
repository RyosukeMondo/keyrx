//! macOS system tray (menu bar) integration.
//!
//! This module provides a menu bar icon with control menu items for
//! the keyrx daemon on macOS using the `tray-icon` crate.

use crossbeam_channel::Receiver;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

use crate::platform::{SystemTray, TrayControlEvent, TrayError};

/// Loads an icon from PNG bytes.
fn load_icon(bytes: &[u8]) -> Result<Icon, TrayError> {
    let image = image::load_from_memory(bytes)
        .map_err(|e| TrayError::IconLoadFailed(e.to_string()))?
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).map_err(|e| TrayError::IconLoadFailed(e.to_string()))
}

/// macOS system tray (menu bar) controller.
///
/// Wraps the `tray-icon` crate to provide a native macOS menu bar icon with menu.
/// Implements [`SystemTray`] for cross-platform compatibility.
#[allow(dead_code)]
pub struct MacosSystemTray {
    _tray_icon: TrayIcon,
    menu_receiver: Receiver<MenuEvent>,
    reload_id: String,
    webui_id: String,
    exit_id: String,
}

impl MacosSystemTray {
    /// Show a notification via the menu bar icon
    pub fn show_notification(&self, title: &str, message: &str) {
        // tray-icon doesn't have built-in notification support
        // For now, just log the notification
        log::info!("Tray notification: {} - {}", title, message);

        // On macOS, we could use the UserNotifications framework for native notifications
        // For now, we'll rely on the log message being visible to the user
    }
}

impl SystemTray for MacosSystemTray {
    fn new() -> Result<Self, TrayError> {
        let tray_menu = Menu::new();
        let webui_item = MenuItem::new("Open Web UI", true, None);
        let reload_item = MenuItem::new("Reload Config", true, None);
        let exit_item = MenuItem::new("Exit", true, None);

        tray_menu
            .append_items(&[
                &webui_item,
                &PredefinedMenuItem::separator(),
                &reload_item,
                &PredefinedMenuItem::separator(),
                &exit_item,
            ])
            .map_err(|e| TrayError::Other(e.to_string()))?;

        let webui_id = webui_item.id().0.clone();
        let reload_id = reload_item.id().0.clone();
        let exit_id = exit_item.id().0.clone();

        let icon_bytes = include_bytes!("../../../assets/icon.png");
        let icon = load_icon(icon_bytes)?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("KeyRx Daemon")
            .with_icon(icon)
            .build()
            .map_err(|e| TrayError::Other(e.to_string()))?;

        let menu_receiver = MenuEvent::receiver().clone();

        Ok(Self {
            _tray_icon: tray_icon,
            menu_receiver,
            reload_id,
            webui_id,
            exit_id,
        })
    }

    fn poll_event(&self) -> Option<TrayControlEvent> {
        if let Ok(event) = self.menu_receiver.try_recv() {
            if event.id.0 == self.webui_id {
                return Some(TrayControlEvent::OpenWebUI);
            } else if event.id.0 == self.reload_id {
                return Some(TrayControlEvent::Reload);
            } else if event.id.0 == self.exit_id {
                return Some(TrayControlEvent::Exit);
            }
        }
        None
    }

    fn shutdown(&mut self) -> Result<(), TrayError> {
        // tray-icon automatically cleans up when TrayIcon is dropped.
        // No explicit shutdown logic needed.
        Ok(())
    }
}
