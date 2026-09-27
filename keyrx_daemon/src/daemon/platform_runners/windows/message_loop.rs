//! The Windows message loop and tray handling.
//!
//! Low-level keyboard hooks only fire while the thread that installed them
//! pumps messages, so the daemon's event processing runs inside this loop.

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::shell::{open_browser, show_about_dialog};
use crate::daemon::{Daemon, DaemonSharedState};
use crate::platform::windows::tray::TrayIconController;
use crate::platform::{SystemTray, TrayControlEvent};

/// Tap-hold timeouts must be checked about this often for hold-to-layer.
const TAP_HOLD_CHECK: Duration = Duration::from_millis(10);
/// Keyboard events processed per loop iteration (keeps up with fast typing).
const EVENTS_PER_ITERATION: usize = 10;

/// Dispatches every pending window message. Returns `false` on `WM_QUIT`.
pub(super) fn pump_messages() -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT,
    };

    // SAFETY: plain Win32 message pumping on the current thread with a
    // zero-initialised MSG owned by this frame.
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            if msg.message == WM_QUIT {
                return false;
            }
            TranslateMessage(&msg);
            // WIN-BUG #4: a panic in wnd_proc must not terminate the process.
            let _ = std::panic::catch_unwind(|| {
                DispatchMessageW(&msg);
            });
        }
    }
    true
}

/// Creates the tray icon. `None` in headless/WinRM sessions, which is fine:
/// the web UI stays available.
pub(super) fn create_tray(
    port_note: Option<String>,
    web_ui_url: &str,
) -> Option<TrayIconController> {
    match TrayIconController::new() {
        Ok(tray) => {
            log::info!("System tray icon created successfully");
            if let Some(note) = port_note {
                tray.show_notification("KeyRx Port Changed", &note);
            }
            Some(tray)
        }
        Err(e) => {
            log::warn!(
                "Failed to create system tray icon (this is normal in headless/WinRM sessions): {}",
                e
            );
            log::info!(
                "Daemon will continue without system tray. Web UI is available at {web_ui_url}"
            );
            None
        }
    }
}

/// Runs the production loop until `WM_QUIT`, the daemon stops, or the tray
/// asks to exit.
pub(super) fn run(
    daemon: &mut Daemon,
    daemon_state: &Arc<DaemonSharedState>,
    tray: Option<&TrayIconController>,
    web_ui_url: &str,
) {
    let mut last_timeout_check = Instant::now();
    loop {
        if !pump_messages() {
            return;
        }
        process_events(daemon);

        if last_timeout_check.elapsed() >= TAP_HOLD_CHECK {
            daemon.check_tap_hold_timeouts();
            last_timeout_check = Instant::now();
        }

        // Profile activation, saved config of the loaded profile, reload:
        // all raise the one reload flag; the daemon resolves what to load.
        daemon.service_reload_request();

        if !daemon.is_running() {
            log::info!("Daemon stopped");
            return;
        }

        if let Some(tray) = tray {
            if let Some(event) = tray.poll_event() {
                if !handle_tray_event(event, daemon_state, tray, web_ui_url) {
                    return;
                }
            }
        }

        // Small sleep to prevent 100% CPU usage
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Feeds hook-captured events through the remapping engine (which also
/// broadcasts them to WebSocket clients).
fn process_events(daemon: &mut Daemon) {
    for _ in 0..EVENTS_PER_ITERATION {
        match daemon.process_one_event() {
            Ok(true) => continue,
            Ok(false) => break,
            Err(e) => {
                log::warn!("Error processing event: {}", e);
                break;
            }
        }
    }
}

/// Acts on a tray menu event. Returns `false` when the daemon should exit.
fn handle_tray_event(
    event: TrayControlEvent,
    daemon_state: &Arc<DaemonSharedState>,
    tray: &TrayIconController,
    web_ui_url: &str,
) -> bool {
    match event {
        TrayControlEvent::Reload => {
            log::info!("Reloading config...");
            daemon_state.request_reload();
        }
        TrayControlEvent::OpenWebUI => {
            log::info!("Opening web UI at {}...", web_ui_url);
            if let Err(e) = open_browser(web_ui_url) {
                log::error!("Failed to open web UI: {}", e);
            }
        }
        TrayControlEvent::About => {
            log::info!("About requested via tray menu");
            show_about_dialog();
        }
        TrayControlEvent::Suspend => {
            let now_suspended = !daemon_state.is_suspended();
            daemon_state.set_suspended(now_suspended);
            tray.update_suspend_state(now_suspended);
            let verb = if now_suspended {
                "suspended"
            } else {
                "resumed"
            };
            log::info!("Daemon {verb}");
        }
        TrayControlEvent::Exit => {
            log::info!("Exiting...");
            return false;
        }
    }
    true
}
