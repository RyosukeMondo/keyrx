//! Windows daemon runner.
//!
//! This module contains the Windows-specific implementation for running the daemon,
//! including message loop handling, web server setup, and system tray integration.
//!
//! # Architecture - Shared State IPC Replacement
//!
//! On Windows, the daemon runs as a single process with two threads:
//! - **Main thread**: Keyboard event processing and Windows message loop
//! - **Web server thread**: Tokio async runtime serving REST API and WebSocket
//!
//! Instead of Unix domain sockets (which don't exist on Windows), these threads
//! communicate via [`DaemonSharedState`] - a thread-safe structure using
//! `Arc<AtomicBool>` and `Arc<RwLock>` for shared state access.
//!
//! The shared state enables the web server to query daemon status (running flag,
//! active profile, device count, uptime) without IPC overhead.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::daemon::DaemonSharedState;

/// Run the daemon on Windows.
///
/// # Arguments
///
/// * `source` - Where the startup configuration comes from (see `daemon::live_config`)
/// * `config_dir` - The keyrx config directory (profiles, `.active`)
/// * `debug` - Enable debug logging
/// * `test_mode` - Enable test mode (no keyboard capture)
/// * `container` - Service container with all dependencies wired
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err((exit_code, message))` on failure.
pub fn run_daemon(
    source: crate::daemon::ConfigSource,
    config_dir: PathBuf,
    debug: bool,
    test_mode: bool,
    container: Arc<crate::container::ServiceContainer>,
) -> Result<(), (i32, String)> {
    use crate::daemon::platform_setup::{
        init_logging, log_post_init_hook_status, log_startup_version_info,
    };
    use crate::daemon::{Daemon, ExitCode};
    use crate::daemon_config::DaemonConfig;
    use crate::platform::windows::tray::TrayIconController;
    use crate::platform::{SystemTray, TrayControlEvent};
    use crate::services::SettingsService;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT,
    };

    // Initialize logging
    init_logging(debug);

    // Log version information on startup
    log_startup_version_info();

    // Load configuration
    let config = DaemonConfig::from_env().map_err(|e| {
        (
            ExitCode::ConfigError as i32,
            format!("Configuration error: {}", e),
        )
    })?;

    config.validate().map_err(|e| {
        (
            ExitCode::ConfigError as i32,
            format!("Invalid configuration: {}", e),
        )
    })?;

    if test_mode {
        log::info!("Test mode enabled - running with IPC infrastructure without keyboard capture");
        return run_test_mode(debug, container);
    }

    // Ensure single instance - kill any existing daemon before starting
    let killed_old = ensure_single_instance(&config_dir);

    // Load settings to get configured port
    let settings_service_for_port = SettingsService::new(config_dir.clone());

    // If we killed an old instance, reset port to default since it should be free now
    let configured_port = if killed_old {
        let default_port = crate::services::DEFAULT_PORT;
        if let Err(e) = settings_service_for_port.set_port(default_port) {
            log::warn!("Failed to reset port to default: {}", e);
        }
        default_port
    } else {
        settings_service_for_port.get_port()
    };
    log::info!("Configured web server port: {}", configured_port);

    log::info!("Starting keyrx daemon (Windows) from {source:?}");

    // Create platform instance
    let platform = crate::platform::create_platform().map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to create platform: {}", e),
        )
    })?;

    // Create the daemon
    let mut daemon =
        Daemon::new(platform, source, config_dir.clone()).map_err(daemon_error_to_exit)?;

    // Status/control shared with the web server thread (Windows has no Unix
    // socket; the web API reads this directly).
    let daemon_state = daemon.shared_state();

    // Create broadcast channel for event streaming to WebSocket clients
    let (event_tx, _event_rx) = tokio::sync::broadcast::channel(1000);
    let event_tx_clone = event_tx.clone();
    let event_tx_for_broadcaster = event_tx.clone();

    // Create event broadcaster for real-time updates
    let event_broadcaster = crate::daemon::EventBroadcaster::new(event_tx_for_broadcaster);
    let running_for_broadcaster = daemon.running_flag();
    let telemetry_for_broadcaster = daemon.telemetry();

    // Wire the event broadcaster into the daemon for real-time event streaming
    daemon.set_event_broadcaster(event_broadcaster.clone());

    // The single read model for the web API (see DaemonQueryService docs).
    let daemon_query = Arc::new(crate::services::DaemonQueryService::new(
        Arc::clone(&daemon_state),
        daemon.telemetry(),
    ));
    let app_state = Arc::new(crate::web::AppState::from_container_with_daemon(
        (*container).clone(),
        daemon_query,
    ));

    // Find an available port, starting with configured port
    let actual_port = find_available_port(configured_port);

    // If we had to use a different port, notify user but don't persist
    // (persisting causes port drift: 9867→9868→9869... on each restart)
    let port_changed = actual_port != configured_port;
    if port_changed {
        log::warn!(
            "Configured port {} is in use. Using port {} instead (not saved to settings).",
            configured_port,
            actual_port
        );
    }

    let actual_port_for_thread = actual_port;
    let port_changed_for_thread = port_changed;
    let configured_port_for_thread = configured_port;

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(e) => {
                log::error!("Failed to create tokio runtime for web server: {}", e);
                log::error!("Web server will not start. Ensure your system has sufficient resources (threads, memory)");
                return;
            }
        };
        rt.block_on(async {
            // Start latency broadcast task with real metrics collection
            tokio::spawn(crate::daemon::start_latency_broadcast_task(
                event_broadcaster,
                running_for_broadcaster,
                Some(telemetry_for_broadcaster),
            ));

            let addr: std::net::SocketAddr = ([127, 0, 0, 1], actual_port_for_thread).into();
            if port_changed_for_thread {
                log::info!(
                    "Port {} was in use. Starting web server on http://{} (saved to settings)",
                    configured_port_for_thread,
                    addr
                );
            } else {
                log::info!("Starting web server on http://{}", addr);
            }
            match crate::web::serve(addr, event_tx_clone, app_state).await {
                Ok(()) => log::info!("Web server stopped"),
                Err(e) => log::error!("Web server error: {}", e),
            }
        });
    });

    // Create the tray icon (optional - may fail in headless/WinRM sessions)
    let tray = match TrayIconController::new() {
        Ok(tray) => {
            log::info!("System tray icon created successfully");
            // Notify user about port if it changed
            if port_changed {
                tray.show_notification(
                    "KeyRx Port Changed",
                    &format!(
                        "Port {} was in use. Now running on port {}.",
                        configured_port, actual_port
                    ),
                );
            }
            Some(tray)
        }
        Err(e) => {
            log::warn!(
                "Failed to create system tray icon (this is normal in headless/WinRM sessions): {}",
                e
            );
            log::info!(
                "Daemon will continue without system tray. Web UI is available at http://127.0.0.1:{}",
                actual_port
            );
            None
        }
    };

    // Wire the suspended flag to the keyboard hook so it can skip
    // blocking when the daemon is suspended (pass-through mode).
    wire_suspended_flag(&daemon_state);

    // Check for administrative privileges
    if !is_admin() {
        log::warn!("Daemon is not running with administrative privileges. Key remapping may not work for elevated applications.");
    }

    // Log hook installation status after daemon initialization
    log_post_init_hook_status();

    log::info!("Daemon initialized. Running message loop...");

    // Build web UI URL with actual port
    let web_ui_url = format!("http://127.0.0.1:{}", actual_port);

    // Track last tap-hold timeout check (must poll every ~10ms)
    let mut last_timeout_check = std::time::Instant::now();

    // Windows low-level hooks REQUIRE a message loop on the thread that installed them.
    // Our Daemon::new() calls grab() which installs the hook.
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        loop {
            // Process ALL available messages
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    cleanup_pid_file(&config_dir);
                    return Ok(());
                }

                TranslateMessage(&msg);
                // WIN-BUG #4: Wrap message dispatch in catch_unwind to prevent
                // a panic in wnd_proc from terminating the entire process.
                let _ = std::panic::catch_unwind(|| {
                    DispatchMessageW(&msg);
                });
            }

            // Process keyboard events from the daemon's event queue
            // This reads events captured by the Windows hooks and:
            // 1. Processes them through the remapping engine
            // 2. Broadcasts them to WebSocket clients for metrics display
            // Process multiple events per iteration to keep up with fast typing
            for _ in 0..10 {
                match daemon.process_one_event() {
                    Ok(true) => {
                        // Event processed, try to get more
                        continue;
                    }
                    Ok(false) => {
                        // No more events available
                        break;
                    }
                    Err(e) => {
                        log::warn!("Error processing event: {}", e);
                        break;
                    }
                }
            }

            // Check tap-hold timeouts every ~10ms so hold-to-activate-layer works
            if last_timeout_check.elapsed() >= std::time::Duration::from_millis(10) {
                daemon.check_tap_hold_timeouts();
                last_timeout_check = std::time::Instant::now();
            }

            // Profile activation, saved config of the loaded profile, reload:
            // all raise the one reload flag; the daemon resolves what to load.
            daemon.service_reload_request();

            // Check if daemon is still running
            if !daemon.is_running() {
                log::info!("Daemon stopped");
                cleanup_pid_file(&config_dir);
                return Ok(());
            }

            // Poll tray events (only if tray was created successfully)
            if let Some(ref tray_controller) = tray {
                if let Some(event) = tray_controller.poll_event() {
                    match event {
                        TrayControlEvent::Reload => {
                            log::info!("Reloading config...");
                            daemon_state.request_reload();
                        }
                        TrayControlEvent::OpenWebUI => {
                            log::info!("Opening web UI at {}...", web_ui_url);
                            if let Err(e) = open_browser(&web_ui_url) {
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
                            tray_controller.update_suspend_state(now_suspended);
                            log::info!(
                                "Daemon {}",
                                if now_suspended {
                                    "suspended"
                                } else {
                                    "resumed"
                                }
                            );
                        }
                        TrayControlEvent::Exit => {
                            log::info!("Exiting...");
                            cleanup_pid_file(&config_dir);
                            return Ok(());
                        }
                    }
                }
            }

            // Small sleep to prevent 100% CPU usage
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

/// Run the daemon in test mode (no keyboard capture).
fn run_test_mode(
    _debug: bool,
    _container: Arc<crate::container::ServiceContainer>,
) -> Result<(), (i32, String)> {
    use crate::config::ProfileManager;
    use crate::daemon::ExitCode;
    use crate::daemon_config::DaemonConfig;
    use crate::ipc::commands::IpcCommandHandler;
    use std::sync::Arc;

    // Load configuration
    let config = DaemonConfig::from_env().map_err(|e| {
        (
            ExitCode::ConfigError as i32,
            format!("Configuration error: {}", e),
        )
    })?;

    config.validate().map_err(|e| {
        (
            ExitCode::ConfigError as i32,
            format!("Invalid configuration: {}", e),
        )
    })?;

    log::info!("Starting daemon in test mode (no keyboard capture)");

    // Determine config directory
    let config_dir = crate::cli::config_dir::get_config_dir().map_err(|e| {
        (
            ExitCode::ConfigError as i32,
            format!("Cannot determine config directory: {e}"),
        )
    })?;

    // Initialize ProfileManager (without RwLock - ProfileManager has internal mutability)
    let profile_manager = match ProfileManager::new(config_dir.clone()) {
        Ok(mgr) => Arc::new(mgr),
        Err(e) => {
            return Err((
                ExitCode::ConfigError as i32,
                format!("Failed to initialize ProfileManager: {}", e),
            ));
        }
    };

    // One read model shared by the IPC handler and the web API. Test mode has
    // no keyboard daemon, so it reports "not running" consistently on both.
    let daemon_query = Arc::new(crate::services::DaemonQueryService::without_daemon());
    let ipc_handler = Arc::new(IpcCommandHandler::new(
        Arc::clone(&profile_manager),
        Arc::clone(&daemon_query),
    ));

    // Windows uses named pipes
    let test_socket_path = PathBuf::from(format!("keyrx-test-{}", std::process::id()));
    crate::ipc::server::spawn(test_socket_path.clone(), ipc_handler).map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to start IPC server: {}", e),
        )
    })?;
    log::info!("IPC server started on {}", test_socket_path.display());

    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to create tokio runtime: {}", e),
        )
    })?;

    // Create broadcast channel for event streaming
    let (event_tx, _event_rx) = tokio::sync::broadcast::channel(1000);

    // Create event bus channel for simulator-to-macro-recorder communication
    let (macro_event_tx, macro_event_rx) =
        tokio::sync::mpsc::channel::<keyrx_core::runtime::KeyEvent>(1000);

    // Create services for web API (test mode — no daemon running, no reload needed)
    let macro_recorder = Arc::new(crate::macro_recorder::MacroRecorder::new());
    let profile_service = Arc::new(crate::services::ProfileService::new(Arc::clone(
        &profile_manager,
    )));
    let device_service = Arc::new(crate::services::DeviceService::new(config_dir.clone()));
    let config_service = Arc::new(crate::services::ConfigService::new(Arc::clone(
        &profile_service,
    )));
    let settings_service = Arc::new(crate::services::SettingsService::new(config_dir.clone()));
    let simulation_service = Arc::new(crate::services::SimulationService::new(
        config_dir.clone(),
        Some(macro_event_tx),
    ));
    let subscription_manager = Arc::new(crate::web::subscriptions::SubscriptionManager::new());

    // Create RPC event broadcaster
    let (rpc_event_tx, _) = tokio::sync::broadcast::channel(1000);

    let app_state = Arc::new(crate::web::AppState::new_with_test_mode(
        macro_recorder.clone(),
        profile_service,
        device_service,
        config_service,
        settings_service,
        simulation_service,
        subscription_manager,
        rpc_event_tx,
        daemon_query,
    ));

    // Start web server in background thread (main thread runs message loop for tray)
    let addr = config.socket_addr().map_err(|e| {
        (
            ExitCode::ConfigError as i32,
            format!("Failed to create socket address: {}", e),
        )
    })?;
    let web_url = config.web_url();
    log::info!("Starting web server on {}", web_url);

    std::thread::spawn(move || {
        rt.block_on(async {
            // Spawn macro recorder event loop inside runtime context
            let recorder_for_loop = (*macro_recorder).clone();
            tokio::spawn(async move {
                recorder_for_loop.run_event_loop(macro_event_rx).await;
            });

            match crate::web::serve(addr, event_tx, app_state).await {
                Ok(()) => log::info!("Web server stopped"),
                Err(e) => log::error!("Web server error: {}", e),
            }
        });
    });

    // Create the tray icon
    use crate::platform::windows::tray::TrayIconController;
    use crate::platform::{SystemTray, TrayControlEvent};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT,
    };

    let tray = match TrayIconController::new() {
        Ok(tray) => {
            log::info!("System tray icon created successfully (test mode)");
            Some(tray)
        }
        Err(e) => {
            log::warn!("Failed to create system tray icon: {}", e);
            None
        }
    };

    log::info!("Test mode running. Web UI at {}", web_url);

    // Message loop (required for tray icon to work on Windows)
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        loop {
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    return Ok(());
                }
                TranslateMessage(&msg);
                let _ = std::panic::catch_unwind(|| {
                    DispatchMessageW(&msg);
                });
            }

            // Poll tray events
            if let Some(ref tray_controller) = tray {
                if let Some(event) = tray_controller.poll_event() {
                    match event {
                        TrayControlEvent::OpenWebUI => {
                            log::info!("Opening web UI at {}...", web_url);
                            if let Err(e) = open_browser(&web_url) {
                                log::error!("Failed to open web UI: {}", e);
                            }
                        }
                        TrayControlEvent::About => {
                            show_about_dialog();
                        }
                        TrayControlEvent::Exit => {
                            log::info!("Exiting test mode...");
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

/// Ensure only one instance of the daemon is running.
/// Kills any existing instance before starting.
/// Returns true if an old instance was killed.
fn ensure_single_instance(config_dir: &Path) -> bool {
    let pid_file = config_dir.join("daemon.pid");
    let mut killed = false;

    // Check if PID file exists and process is running
    if pid_file.exists() {
        if let Ok(contents) = std::fs::read_to_string(&pid_file) {
            if let Ok(old_pid) = contents.trim().parse::<u32>() {
                // Try to kill the old process
                log::info!("Found existing daemon (PID {}), terminating...", old_pid);
                unsafe {
                    use windows_sys::Win32::Foundation::CloseHandle;
                    use windows_sys::Win32::System::Threading::{
                        OpenProcess, TerminateProcess, PROCESS_TERMINATE,
                    };

                    let handle = OpenProcess(PROCESS_TERMINATE, 0, old_pid);
                    if !handle.is_null() {
                        if TerminateProcess(handle, 0) != 0 {
                            log::info!("Terminated previous daemon instance (PID {})", old_pid);
                            killed = true;
                            // Give it a moment to clean up
                            std::thread::sleep(std::time::Duration::from_millis(500));
                        }
                        CloseHandle(handle);
                    }
                }
            }
        }
        // Remove old PID file
        let _ = std::fs::remove_file(&pid_file);
    }

    // Write current PID
    let current_pid = std::process::id();
    if let Err(e) = std::fs::write(&pid_file, current_pid.to_string()) {
        log::warn!("Failed to write PID file: {}", e);
    } else {
        log::debug!("Wrote PID {} to {:?}", current_pid, pid_file);
    }

    killed
}

/// Clean up PID file on exit
fn cleanup_pid_file(config_dir: &Path) {
    let pid_file = config_dir.join("daemon.pid");
    let _ = std::fs::remove_file(&pid_file);
}

/// Find an available port starting from the given port.
/// Tries ports in sequence: port, port+1, port+2, ... up to 10 attempts.
fn find_available_port(start_port: u16) -> u16 {
    use std::net::TcpListener;

    for offset in 0..10 {
        let port = start_port.saturating_add(offset);
        if port == 0 {
            continue;
        }

        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(_listener) => {
                // Port is available (listener is dropped immediately, releasing the port)
                return port;
            }
            Err(_) => {
                // Port is in use, try next
                continue;
            }
        }
    }

    // Fallback: return the original port (will fail with a clear error later)
    start_port
}

/// Check if running with administrative privileges
fn is_admin() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        );

        CloseHandle(token);
        result != 0 && elevation.TokenIsElevated != 0
    }
}

/// Opens a URL in the default web browser.
fn open_browser(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::process::Command::new("cmd")
        .args(["/c", "start", url])
        .spawn()?;
    Ok(())
}

/// Show About dialog with version information
fn show_about_dialog() {
    use crate::version;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let message = format!(
        "KeyRx - Advanced Keyboard Remapping\n\n\
         Version: {}\n\
         Build: {}\n\
         Commit: {}\n\n\
         Copyright © 2024 KeyRx Contributors\n\
         Licensed under AGPL-3.0-or-later",
        version::VERSION,
        version::BUILD_DATE,
        version::GIT_HASH
    );

    let title = "About KeyRx";

    // Convert to UTF-16 for Windows API
    let message_wide: Vec<u16> = OsStr::new(&message)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let title_wide: Vec<u16> = OsStr::new(title)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
            std::ptr::null_mut(), // No parent window
            message_wide.as_ptr(),
            title_wide.as_ptr(),
            windows_sys::Win32::UI::WindowsAndMessaging::MB_OK
                | windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONINFORMATION,
        );
    }
}

/// Wire the suspended flag from DaemonSharedState to the keyboard hook.
///
/// The hook callback reads this flag on every keypress to decide whether
/// to pass the key through unchanged (suspended) or process it normally.
fn wire_suspended_flag(daemon_state: &Arc<DaemonSharedState>) {
    use crate::platform::windows::platform_state::PlatformState;

    if let Some(state_arc) = PlatformState::get() {
        if let Ok(state) = state_arc.lock() {
            if let Some(ref blocker) = state.key_blocker {
                blocker.set_suspended_flag(daemon_state.suspended_flag());
                log::info!("Suspended flag wired to keyboard hook");
            }
        }
    }
}

/// Converts a DaemonError to an exit code and message.
fn daemon_error_to_exit(error: crate::daemon::DaemonError) -> (i32, String) {
    use crate::daemon::{DaemonError, ExitCode};

    match &error {
        DaemonError::Config(_) => (ExitCode::ConfigError as i32, error.to_string()),
        DaemonError::PermissionError(_) => (ExitCode::PermissionError as i32, error.to_string()),
        DaemonError::Platform(plat_err) => {
            // Check if it's a permission error
            if plat_err.to_string().contains("permission")
                || plat_err.to_string().contains("Permission")
            {
                (ExitCode::PermissionError as i32, error.to_string())
            } else {
                (ExitCode::ConfigError as i32, error.to_string())
            }
        }
        _ => (ExitCode::RuntimeError as i32, error.to_string()),
    }
}
