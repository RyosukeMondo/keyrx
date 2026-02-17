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
/// * `config_path` - Path to configuration file
/// * `debug` - Enable debug logging
/// * `test_mode` - Enable test mode (no keyboard capture)
/// * `container` - Service container with all dependencies wired
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err((exit_code, message))` on failure.
pub fn run_daemon(
    config_path: &Path,
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
        return run_test_mode(config_path, debug, container);
    }

    // Determine config directory (always use standard location for profile management)
    let config_dir = {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("keyrx");
        path
    };

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

    // Check if config file exists, warn if not
    if !config_path.exists() {
        log::warn!(
            "Config file not found: {}. Running in pass-through mode.",
            config_path.display()
        );
    }

    log::info!(
        "Starting keyrx daemon (Windows) with config: {}",
        config_path.display()
    );

    // Create platform instance
    let platform = crate::platform::create_platform().map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to create platform: {}", e),
        )
    })?;

    // Create the daemon
    let mut daemon = Daemon::new(platform, config_path).map_err(daemon_error_to_exit)?;

    // Extract profile name from config path for shared state
    // Example: "C:\Users\user\AppData\Roaming\keyrx\profiles\default.krx" -> "default"
    let profile_name = config_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    // Create shared state for daemon-to-web-server communication (Windows IPC replacement)
    // This enables the web API to query daemon status without Unix sockets
    let daemon_state = Arc::new(DaemonSharedState::from_daemon(&daemon, profile_name));

    // Create broadcast channel for event streaming to WebSocket clients
    let (event_tx, _event_rx) = tokio::sync::broadcast::channel(1000);
    let event_tx_clone = event_tx.clone();
    let event_tx_for_broadcaster = event_tx.clone();

    // Create event broadcaster for real-time updates
    let event_broadcaster = crate::daemon::EventBroadcaster::new(event_tx_for_broadcaster);
    let running_for_broadcaster = daemon.running_flag();
    let latency_recorder_for_broadcaster = daemon.latency_recorder();

    // Wire the event broadcaster into the daemon for real-time event streaming
    daemon.set_event_broadcaster(event_broadcaster.clone());

    // Create DaemonQueryService for REST endpoint metrics
    let daemon_query = Arc::new(crate::services::DaemonQueryService::new(
        daemon.latency_recorder(),
        Arc::clone(&daemon_state),
    ));

    // Create AppState from ServiceContainer with daemon shared state (dependency injection)
    // Clone daemon_state since it will be used later in the event loop
    let app_state = Arc::new(crate::web::AppState::from_container_with_daemon(
        (*container).clone(),
        None, // No test mode socket in production
        Arc::clone(&daemon_state),
        Some(Arc::clone(&daemon_query)),
    ));

    // Get settings service reference for port handling (Windows-specific)
    let settings_service = app_state.settings_service.clone();

    // Find an available port, starting with configured port
    let actual_port = find_available_port(configured_port);

    // If we had to use a different port, save it to settings and notify user
    let port_changed = actual_port != configured_port;
    if port_changed {
        log::warn!(
            "Configured port {} is in use. Using port {} instead.",
            configured_port,
            actual_port
        );
        // Save the new port to settings
        if let Err(e) = settings_service.set_port(actual_port) {
            log::warn!("Failed to save new port to settings: {}", e);
        }
    }

    let actual_port_for_thread = actual_port;
    let port_changed_for_thread = port_changed;
    let configured_port_for_thread = configured_port;
    let daemon_query_for_thread = Arc::clone(&daemon_query);
    let event_rx_for_collector = app_state.event_broadcaster.subscribe();

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
                Some(latency_recorder_for_broadcaster),
            ));

            // Start event collector for REST event log endpoint
            daemon_query_for_thread.spawn_event_collector(event_rx_for_collector);

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

    // Check for administrative privileges
    if !is_admin() {
        log::warn!("Daemon is not running with administrative privileges. Key remapping may not work for elevated applications.");
    }

    // Log hook installation status after daemon initialization
    log_post_init_hook_status();

    log::info!("Daemon initialized. Running message loop...");

    // Build web UI URL with actual port
    let web_ui_url = format!("http://127.0.0.1:{}", actual_port);

    // Track current config path for hot-reload detection
    let mut current_config_path = config_path.to_path_buf();

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

            // Check for explicit reload requests (e.g., profile config saved via web UI)
            if daemon_state.take_reload_request() {
                log::info!("Reload requested, reloading configuration...");
                if let Err(e) = daemon.reload() {
                    log::error!("Failed to reload configuration: {}", e);
                } else {
                    log::info!("Configuration reloaded successfully");
                }
            }

            // Check for profile changes from web API (Windows hot-reload mechanism)
            // On Windows, we can't use Unix signals (SIGHUP), so the web API updates
            // the shared daemon state, and we detect the change here to trigger reload
            let active_profile_from_state = daemon_state.get_active_profile();
            let current_profile_from_path = current_config_path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string());

            if active_profile_from_state != current_profile_from_path {
                // Profile changed via web API - trigger hot reload
                log::info!(
                    "Profile change detected: {:?} -> {:?}, reloading configuration...",
                    current_profile_from_path,
                    active_profile_from_state
                );

                // Build new config path from active profile
                if let Some(new_profile) = &active_profile_from_state {
                    let new_config_path = config_dir
                        .join("profiles")
                        .join(format!("{}.krx", new_profile));

                    // Trigger daemon reload with new config
                    if let Err(e) = daemon.reload() {
                        log::error!("Failed to reload configuration: {}", e);
                        // Reset shared state to previous profile on failure
                        daemon_state.set_active_profile(current_profile_from_path.clone());
                    } else {
                        log::info!(
                            "Configuration reloaded successfully for profile: {}",
                            new_profile
                        );
                        // Update current config path for next iteration
                        current_config_path = new_config_path.clone();
                        // Update shared state config path
                        daemon_state.set_config_path(new_config_path);
                    }
                }
            }

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
                            let _ = daemon.reload();
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
    _config_path: &Path,
    _debug: bool,
    _container: Arc<crate::container::ServiceContainer>,
) -> Result<(), (i32, String)> {
    use crate::config::ProfileManager;
    use crate::daemon::ExitCode;
    use crate::daemon_config::DaemonConfig;
    use crate::ipc::commands::IpcCommandHandler;
    use crate::ipc::server::IpcServer;
    use std::sync::Arc;
    use tokio::sync::{Mutex, RwLock};

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
    let config_dir = {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("keyrx");
        path
    };

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

    // Create daemon running flag
    let daemon_running = Arc::new(RwLock::new(true));

    // Create IPC command handler
    let ipc_handler = Arc::new(IpcCommandHandler::new(
        Arc::clone(&profile_manager),
        Arc::clone(&daemon_running),
    ));

    // Create IPC server with unique socket path (Windows uses named pipes)
    let pid = std::process::id();
    let test_socket_path = PathBuf::from(format!("keyrx-test-{}", pid));
    let mut ipc_server = IpcServer::new(test_socket_path.clone()).map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to create IPC server: {}", e),
        )
    })?;

    // Start IPC server
    ipc_server.start().map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to start IPC server: {}", e),
        )
    })?;

    log::info!("IPC server started on {}", test_socket_path.display());

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to create tokio runtime: {}", e),
        )
    })?;

    // Clone handler for server thread
    let ipc_handler_for_server = Arc::clone(&ipc_handler);

    // Start IPC server connection handler in background
    std::thread::spawn(move || {
        let handler_fn = Arc::new(Mutex::new(
            move |request: crate::ipc::IpcRequest| -> Result<crate::ipc::IpcResponse, String> {
                // Create a new runtime for this handler call
                let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
                let handler = Arc::clone(&ipc_handler_for_server);
                Ok(rt.block_on(async move { handler.handle(request).await }))
            },
        ));

        if let Err(e) = ipc_server.handle_connections(handler_fn) {
            log::error!("IPC server error: {}", e);
        }
    });

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
        &profile_manager,
    )));
    let settings_service = Arc::new(crate::services::SettingsService::new(config_dir.clone()));
    let simulation_service = Arc::new(crate::services::SimulationService::new(
        config_dir.clone(),
        Some(macro_event_tx),
    ));
    let subscription_manager = Arc::new(crate::web::subscriptions::SubscriptionManager::new());

    // Create RPC event broadcaster
    let (rpc_event_tx, _) = tokio::sync::broadcast::channel(1000);

    // Create minimal daemon shared state for test mode
    // Even though no daemon is running, we create a minimal state for API consistency
    use std::sync::atomic::AtomicBool;
    let test_running_flag = Arc::new(AtomicBool::new(false)); // Not running in test mode
    let test_daemon_state = Arc::new(DaemonSharedState::new(
        test_running_flag,
        None, // No active profile in test mode
        PathBuf::from("test.krx"),
        0, // No devices in test mode
    ));

    let app_state = Arc::new(crate::web::AppState::new_with_test_mode(
        macro_recorder.clone(),
        profile_service,
        device_service,
        config_service,
        settings_service,
        simulation_service,
        subscription_manager,
        rpc_event_tx,
        test_socket_path.clone(),
        Some(test_daemon_state),
    ));

    // Start web server
    let addr = config.socket_addr().map_err(|e| {
        (
            ExitCode::ConfigError as i32,
            format!("Failed to create socket address: {}", e),
        )
    })?;
    log::info!("Starting web server on {}", config.web_url());

    rt.block_on(async {
        // Spawn macro recorder event loop inside runtime context
        let recorder_for_loop = (*macro_recorder).clone();
        tokio::spawn(async move {
            recorder_for_loop.run_event_loop(macro_event_rx).await;
        });

        match crate::web::serve(addr, event_tx, app_state).await {
            Ok(()) => {
                log::info!("Web server stopped");
                Ok(())
            }
            Err(e) => {
                log::error!("Web server error: {}", e);
                Err((
                    ExitCode::RuntimeError as i32,
                    format!("Web server error: {}", e),
                ))
            }
        }
    })
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
