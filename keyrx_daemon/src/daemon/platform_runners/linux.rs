//! Linux daemon runner.
//!
//! This module contains the Linux-specific implementation for running the daemon,
//! including web server setup, IPC server initialization, and system tray integration.

#![cfg(target_os = "linux")]

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Run the daemon on Linux.
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
    use crate::daemon::platform_setup::{init_logging, log_startup_version_info};
    use crate::daemon::{Daemon, ExitCode};
    use crate::daemon_config::DaemonConfig;
    use crate::platform::linux::LinuxSystemTray;
    use crate::platform::{SystemTray, TrayControlEvent};

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

    log::info!(
        "Starting keyrx daemon with config: {}",
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

    log::info!(
        "Daemon initialized with {} device(s)",
        daemon.device_count()
    );

    // Create system tray (optional - continues without it if unavailable)
    let tray = match LinuxSystemTray::new() {
        Ok(tray) => {
            log::info!("System tray created successfully");
            Some(tray)
        }
        Err(e) => {
            log::warn!(
                "Failed to create system tray (this is normal in headless sessions): {}",
                e
            );
            log::info!(
                "Daemon will continue without system tray. Web UI is available at {}",
                config.web_url()
            );
            None
        }
    };

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

    // Create AppState from ServiceContainer (dependency injection)
    // This replaces 50+ lines of manual service instantiation
    let app_state = Arc::new(crate::web::AppState::from_container(
        (*container).clone(),
        None, // No test mode socket in production
    ));

    // Start web server and event broadcasting in background (optional)
    let config_for_web = config.clone();
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
            // Note: Macro recorder event loop is spawned by ServiceContainer when test mode is enabled
            // In production mode (no test socket), the event loop is not needed

            // Start latency broadcast task with real metrics collection
            tokio::spawn(crate::daemon::start_latency_broadcast_task(
                event_broadcaster,
                running_for_broadcaster,
                Some(latency_recorder_for_broadcaster),
            ));

            let addr = match config_for_web.socket_addr() {
                Ok(addr) => addr,
                Err(e) => {
                    log::error!("Invalid socket address configuration: {}", e);
                    return;
                }
            };
            log::info!("Starting web server on {}", config_for_web.web_url());
            match crate::web::serve(addr, event_tx_clone, app_state).await {
                Ok(()) => log::info!("Web server stopped"),
                Err(e) => log::error!("Web server error: {}", e),
            }
        });
    });

    // Run the daemon event loop with tray polling
    let running = daemon.running_flag();
    let result = std::thread::spawn(move || daemon.run());

    // Poll tray in main thread (GTK requires main thread)
    if let Some(mut tray_controller) = tray {
        log::info!("Starting tray event loop");
        while running.load(std::sync::atomic::Ordering::SeqCst) {
            if let Some(event) = tray_controller.poll_event() {
                match event {
                    TrayControlEvent::Reload => {
                        log::info!("Reload requested via tray menu");
                        // TODO: Implement config reload
                    }
                    TrayControlEvent::OpenWebUI => {
                        log::info!("Open Web UI requested via tray menu");
                        if let Err(e) = open_browser(&config.web_url()) {
                            log::error!("Failed to open browser: {}", e);
                        }
                    }
                    TrayControlEvent::About => {
                        log::info!("About requested via tray menu");
                        show_about_dialog();
                    }
                    TrayControlEvent::Exit => {
                        log::info!("Exit requested via tray menu");
                        running.store(false, std::sync::atomic::Ordering::SeqCst);
                        break;
                    }
                }
            }
            // Small sleep to prevent busy loop
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Shutdown tray before exiting
        if let Err(e) = tray_controller.shutdown() {
            log::error!("Failed to shutdown tray: {}", e);
        }
    }

    // Wait for daemon thread to finish
    match result.join() {
        Ok(daemon_result) => daemon_result.map_err(daemon_error_to_exit)?,
        Err(panic_payload) => {
            let panic_msg = panic_payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic_payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "Unknown panic".to_string());
            log::error!("Daemon thread panicked: {}", panic_msg);
            return Err((1, format!("Daemon thread panicked: {}", panic_msg)));
        }
    }

    log::info!("Daemon stopped gracefully");
    Ok(())
}

/// Run the daemon in test mode (no keyboard capture).
fn run_test_mode(
    _config_path: &Path,
    _debug: bool,
    container: Arc<crate::container::ServiceContainer>,
) -> Result<(), (i32, String)> {
    use crate::config::ProfileManager;
    use crate::daemon::ExitCode;
    use crate::daemon_config::DaemonConfig;
    use crate::ipc::commands::IpcCommandHandler;
    use crate::ipc::server::IpcServer;
    use std::sync::Arc;
    use tokio::sync::{Mutex, RwLock};

    log::info!("Starting daemon in test mode (no keyboard capture)");

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

    // Create IPC server with unique socket path
    let pid = std::process::id();
    let test_socket_path = PathBuf::from(format!("/tmp/keyrx-test-{}.sock", pid));
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

    // Create services for web API
    let macro_recorder = Arc::new(crate::macro_recorder::MacroRecorder::new());
    // Reuse the same ProfileManager instance for IPC and REST API
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

/// Opens a URL in the default web browser.
fn open_browser(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}

/// Show About dialog with version information
fn show_about_dialog() {
    use crate::version;

    // For Linux, just log the info
    log::info!("About KeyRx v{}", version::VERSION);
    log::info!("Build: {}", version::BUILD_DATE);
    log::info!("Commit: {}", version::GIT_HASH);
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
