//! Linux daemon runner.
//!
//! This module contains the Linux-specific implementation for running the daemon,
//! including web server setup, IPC server initialization, and system tray integration.

#![cfg(target_os = "linux")]

use std::path::PathBuf;
use std::sync::Arc;

/// Run the daemon on Linux.
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
    log: crate::daemon::platform_setup::LogOptions,
    watch: bool,
    test_mode: bool,
    container: Arc<crate::container::ServiceContainer>,
    options: crate::daemon::options::RuntimeOptions,
) -> Result<(), (i32, String)> {
    use crate::daemon::platform_setup::{init_logging, log_startup_version_info};
    use crate::daemon::{Daemon, ExitCode};
    use crate::daemon_config::DaemonConfig;
    use crate::platform::linux::LinuxSystemTray;
    use crate::platform::{SystemTray, TrayControlEvent};

    // Initialize logging
    init_logging(log);

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
        return run_test_mode(config_dir);
    }

    super::ensure_no_other_daemon().map_err(|m| (ExitCode::RuntimeError as i32, m))?;

    log::info!("Starting keyrx daemon from {source:?}");

    // Create platform instance
    let platform: Box<dyn crate::platform::Platform> = Box::new(
        crate::platform::linux::LinuxPlatform::from_env().with_emergency(options.emergency.clone()),
    );

    // Create the daemon
    let mut daemon = Daemon::with_options(platform, source, config_dir, &options)
        .map_err(daemon_error_to_exit)?;

    log::info!(
        "Daemon initialized with {} device(s)",
        daemon.device_count()
    );
    log::info!(
        "Emergency stop - if the keyboard ever stops working: {}. Either releases \
         every keyboard and stops keyrx",
        options.emergency.describe()
    );
    log::info!(
        "Minimum output key-down time: {} ms (a tap shorter than that is stretched so \
         per-frame pollers see it; --min-key-down-ms 0 turns it off)",
        options.min_key_down.as_millis()
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
    let telemetry_for_broadcaster = daemon.telemetry();

    // Wire the event broadcaster into the daemon for real-time event streaming
    daemon.set_event_broadcaster(event_broadcaster.clone());

    // The single read model shared by the web API and the IPC server (same
    // pattern as Windows), so both report identical status.
    let daemon_state = daemon.shared_state();
    if watch {
        crate::daemon::config_watch::spawn(
            Arc::clone(container.profile_service().profile_manager()),
            Arc::clone(&daemon_state),
            daemon.running_flag(),
            crate::daemon::config_watch::POLL_INTERVAL,
        );
    }
    let daemon_query = Arc::new(crate::services::DaemonQueryService::new(
        Arc::clone(&daemon_state),
        daemon.telemetry(),
    ));

    let app_state = Arc::new(crate::web::AppState::from_container_with_daemon(
        (*container).clone(),
        Arc::clone(&daemon_query),
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
                Some(telemetry_for_broadcaster),
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

    // Serve the same read model over IPC for the `status|state|metrics` CLI.
    // Best-effort; the daemon runs regardless.
    let _ipc = super::start_production_ipc_server(&container, daemon_query);

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
                        daemon_state.request_reload();
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
                    TrayControlEvent::Suspend => {
                        log::info!(
                            "Suspend requested via tray menu (not yet implemented on Linux)"
                        );
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
    let joined = result.join();
    match joined {
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
fn run_test_mode(config_dir: PathBuf) -> Result<(), (i32, String)> {
    use crate::config::ProfileManager;
    use crate::daemon::ExitCode;
    use crate::daemon_config::DaemonConfig;
    use crate::ipc::commands::IpcCommandHandler;
    use std::sync::Arc;

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

    let test_endpoint = crate::ipc::IpcEndpoint::test_for_process(std::process::id());
    crate::ipc::server::spawn(test_endpoint.clone(), ipc_handler).map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to start IPC server: {}", e),
        )
    })?;
    log::info!("IPC server started on {test_endpoint}");

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

    // Create services for web API
    let macro_recorder = Arc::new(crate::macro_recorder::MacroRecorder::new());
    // Reuse the same ProfileManager instance for IPC and REST API
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
    (error.exit_code().into(), error.to_string())
}
