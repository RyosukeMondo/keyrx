//! keyrx_daemon - OS-level keyboard remapping daemon
//!
//! This binary provides the main daemon interface for keyboard remapping.
//! It intercepts keyboard events via platform-specific APIs and injects
//! remapped events back to the system.
//!
//! # Subcommands
//!
//! - `run`: Start the daemon with a .krx configuration file
//! - `list-devices`: List available input devices
//! - `validate`: Validate configuration and device matching without grabbing

// Hide console window on Windows release builds
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

// Note: platform and web modules are used via the library (keyrx_daemon::platform)

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

/// KeyRx daemon for OS-level keyboard remapping.
///
/// Intercepts keyboard events and applies remapping rules defined in .krx
/// configuration files compiled by keyrx_compiler.
#[derive(Parser)]
#[command(name = "keyrx_daemon")]
#[command(version, about = "OS-level keyboard remapping daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands for the daemon.
#[derive(Subcommand)]
enum Commands {
    /// Start the daemon with the specified configuration file.
    ///
    /// The daemon will intercept keyboard events from matched devices and apply
    /// the remapping rules from the .krx configuration file.
    Run {
        /// Path to the .krx configuration file compiled by keyrx_compiler.
        /// If not specified, uses the active profile from %APPDATA%\keyrx.
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,

        /// Enable debug logging for verbose output.
        ///
        /// This increases log verbosity to show individual key events and
        /// detailed processing information. Useful for troubleshooting.
        #[arg(short, long)]
        debug: bool,

        /// Enable test mode with IPC infrastructure but without keyboard capture.
        ///
        /// Only available in debug builds for security. Enables full IPC
        /// infrastructure for profile activation and daemon status queries.
        #[arg(long)]
        test_mode: bool,
    },

    /// Manage device metadata (rename, set scope, set layout).
    ///
    /// Device management commands for persistent metadata storage.
    Devices(keyrx_daemon::cli::devices::DevicesArgs),

    /// Manage configuration profiles (create, activate, delete, etc.).
    ///
    /// Profile management commands for Rhai configuration files.
    Profiles(keyrx_daemon::cli::profiles::ProfilesArgs),

    /// Manage key mappings and configuration.
    ///
    /// Configuration commands for setting key mappings, tap-hold, macros, etc.
    Config(keyrx_daemon::cli::config::ConfigArgs),

    /// Manage layers (create, rename, delete, show).
    ///
    /// Layer management commands for organizing key mappings.
    Layers(keyrx_daemon::cli::layers::LayersArgs),

    /// Manage keyboard layouts (import, list, show KLE JSON).
    ///
    /// Layout management commands for keyboard layout presets.
    Layouts(keyrx_daemon::cli::layouts::LayoutsArgs),

    /// Run deterministic simulation tests.
    ///
    /// Simulation commands for testing configurations with event replay.
    Simulate(keyrx_daemon::cli::simulate::SimulateArgs),

    /// Run built-in test scenarios.
    ///
    /// Test commands for autonomous configuration validation with scenarios.
    Test(keyrx_daemon::cli::test::TestArgs),

    /// Query daemon status via IPC.
    ///
    /// Displays daemon running state, uptime, active profile, and device count.
    Status(keyrx_daemon::cli::status::StatusArgs),

    /// Inspect runtime state (modifier/lock state).
    ///
    /// Queries the daemon for the current 255-bit modifier/lock state via IPC.
    State(keyrx_daemon::cli::state::StateArgs),

    /// Query daemon performance metrics.
    ///
    /// Provides latency statistics (min, avg, max, p95, p99) and recent event tail.
    Metrics(keyrx_daemon::cli::metrics::MetricsArgs),

    /// List available input devices on the system.
    ///
    /// Displays all input devices with their names, paths, and serial numbers.
    /// Keyboards are clearly marked to help with configuration setup.
    ListDevices,

    /// Validate configuration and device matching without grabbing devices.
    ///
    /// This performs a dry-run that loads the configuration, enumerates devices,
    /// and shows which devices would be matched. No devices are grabbed, so normal
    /// keyboard input continues.
    Validate {
        /// Path to the .krx configuration file to validate.
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
    },

    /// Record input events from a device to a file for replay testing.
    ///
    /// Captures raw input events, converts them to KeyEvents, and saves them
    /// to a JSON file. This file can be used to reproduce bugs or verify behavior
    /// in the test infrastructure.
    Record {
        /// Path to the output JSON file.
        #[arg(short, long)]
        output: PathBuf,

        /// Path to the input device (e.g., /dev/input/event0).
        /// If not provided, lists devices and exits.
        #[arg(short, long)]
        device: Option<PathBuf>,
    },
}

/// Exit codes following Unix conventions.
mod exit_codes {
    /// Successful execution.
    pub const SUCCESS: i32 = 0;
    /// Configuration error (file not found, parse error).
    pub const CONFIG_ERROR: i32 = 1;
    /// Permission error (cannot access devices, cannot create uinput).
    #[cfg(target_os = "linux")]
    pub const PERMISSION_ERROR: i32 = 2;
    #[cfg(target_os = "windows")]
    pub const PERMISSION_ERROR: i32 = 2;
    /// Runtime error (device disconnected with no fallback).
    #[cfg(target_os = "linux")]
    pub const RUNTIME_ERROR: i32 = 3;
    #[cfg(target_os = "windows")]
    pub const RUNTIME_ERROR: i32 = 3;
}

fn main() {
    let cli = Cli::parse();

    // Validate test mode early for release builds
    #[cfg(not(debug_assertions))]
    if let Commands::Run {
        test_mode: true, ..
    } = &cli.command
    {
        eprintln!("Error: Test mode is only available in debug builds");
        process::exit(exit_codes::CONFIG_ERROR);
    }

    let result = match cli.command {
        Commands::Run {
            config,
            debug,
            test_mode,
        } => {
            // If no config specified, use active profile from %APPDATA%\keyrx
            let config_path = match config {
                Some(path) => path,
                None => {
                    let mut default_path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
                    default_path.push("keyrx");
                    default_path.push("default.krx");
                    default_path
                }
            };
            handle_run(&config_path, debug, test_mode)
        }
        Commands::Devices(args) => match keyrx_daemon::cli::devices::execute(args, None) {
            Ok(()) => Ok(()),
            Err(err) => Err((exit_codes::CONFIG_ERROR, err.to_string())),
        },
        Commands::Profiles(args) => handle_profiles_command(args),
        Commands::Config(args) => match keyrx_daemon::cli::config::execute(args, None) {
            Ok(()) => Ok(()),
            Err(e) => Err((exit_codes::CONFIG_ERROR, e.to_string())),
        },
        Commands::Layers(args) => match keyrx_daemon::cli::layers::execute(args) {
            Ok(()) => Ok(()),
            Err(e) => Err((exit_codes::CONFIG_ERROR, e.to_string())),
        },
        Commands::Layouts(args) => match keyrx_daemon::cli::layouts::execute(args) {
            Ok(()) => Ok(()),
            Err(e) => Err((exit_codes::CONFIG_ERROR, e.to_string())),
        },
        Commands::Simulate(args) => match keyrx_daemon::cli::simulate::execute(args) {
            Ok(()) => Ok(()),
            Err(e) => Err((exit_codes::CONFIG_ERROR, e.to_string())),
        },
        Commands::Test(args) => match keyrx_daemon::cli::test::execute(args) {
            Ok(()) => Ok(()),
            Err(e) => Err((exit_codes::CONFIG_ERROR, e.to_string())),
        },
        Commands::Status(args) => match keyrx_daemon::cli::status::execute(args) {
            Ok(()) => Ok(()),
            Err(e) => Err((exit_codes::CONFIG_ERROR, e.to_string())),
        },
        Commands::State(args) => match keyrx_daemon::cli::state::execute(args) {
            Ok(()) => Ok(()),
            Err(e) => Err((exit_codes::CONFIG_ERROR, e.to_string())),
        },
        Commands::Metrics(args) => match keyrx_daemon::cli::metrics::execute(args) {
            Ok(()) => Ok(()),
            Err(e) => Err((exit_codes::CONFIG_ERROR, e.to_string())),
        },
        Commands::ListDevices => handle_list_devices(),
        Commands::Validate { config } => handle_validate(&config),
        Commands::Record { output, device } => handle_record(&output, device.as_deref()),
    };

    match result {
        Ok(()) => process::exit(exit_codes::SUCCESS),
        Err((code, message)) => {
            if !message.is_empty() {
                eprintln!("Error: {}", message);
            }
            process::exit(code);
        }
    }
}

/// Handles the `profiles` command.
fn handle_profiles_command(
    args: keyrx_daemon::cli::profiles::ProfilesArgs,
) -> Result<(), (i32, String)> {
    use keyrx_daemon::config::ProfileManager;
    use keyrx_daemon::services::ProfileService;
    use std::sync::Arc;

    // Determine config directory (default: ~/.config/keyrx)
    let config_dir = {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("keyrx");
        path
    };

    // Initialize ProfileManager
    let manager = match ProfileManager::new(config_dir) {
        Ok(mgr) => Arc::new(mgr),
        Err(e) => {
            return Err((
                exit_codes::CONFIG_ERROR,
                format!("Failed to initialize profile manager: {}", e),
            ));
        }
    };

    // Create ProfileService
    let service = ProfileService::new(manager);

    // Create async runtime for CLI commands
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        (
            exit_codes::RUNTIME_ERROR,
            format!("Failed to create async runtime: {}", e),
        )
    })?;

    // Execute command
    rt.block_on(async {
        match keyrx_daemon::cli::profiles::execute(args, &service).await {
            Ok(()) => Ok(()),
            Err(err) => Err((exit_codes::CONFIG_ERROR, err.to_string())),
        }
    })
}

/// Opens a URL in the default web browser.
#[allow(dead_code)]
fn open_browser(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/c", "start", url])
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    Ok(())
}

/// Handles the `run` subcommand in test mode - starts web server and IPC without keyboard capture.
#[cfg(target_os = "linux")]
fn handle_run_test_mode(_config_path: &std::path::Path, _debug: bool) -> Result<(), (i32, String)> {
    use keyrx_daemon::config::ProfileManager;
    use keyrx_daemon::ipc::commands::IpcCommandHandler;
    use keyrx_daemon::ipc::server::IpcServer;
    use std::sync::Arc;
    use tokio::sync::{Mutex, RwLock};

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
                exit_codes::CONFIG_ERROR,
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
            exit_codes::RUNTIME_ERROR,
            format!("Failed to create IPC server: {}", e),
        )
    })?;

    // Start IPC server
    ipc_server.start().map_err(|e| {
        (
            exit_codes::RUNTIME_ERROR,
            format!("Failed to start IPC server: {}", e),
        )
    })?;

    log::info!("IPC server started on {}", test_socket_path.display());

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        (
            exit_codes::RUNTIME_ERROR,
            format!("Failed to create tokio runtime: {}", e),
        )
    })?;

    // Clone handler for server thread
    let ipc_handler_for_server = Arc::clone(&ipc_handler);

    // Start IPC server connection handler in background
    std::thread::spawn(move || {
        let handler_fn = Arc::new(Mutex::new(
            move |request: keyrx_daemon::ipc::IpcRequest| -> Result<keyrx_daemon::ipc::IpcResponse, String> {
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

    // Create services for web API
    let macro_recorder = Arc::new(keyrx_daemon::macro_recorder::MacroRecorder::new());
    // Reuse the same ProfileManager instance for IPC and REST API
    let profile_service = Arc::new(keyrx_daemon::services::ProfileService::new(Arc::clone(
        &profile_manager,
    )));
    let device_service = Arc::new(keyrx_daemon::services::DeviceService::new(
        config_dir.clone(),
    ));
    let config_service = Arc::new(keyrx_daemon::services::ConfigService::new(Arc::clone(
        &profile_manager,
    )));
    let settings_service = Arc::new(keyrx_daemon::services::SettingsService::new(
        config_dir.clone(),
    ));
    let simulation_service = Arc::new(keyrx_daemon::services::SimulationService::new(
        config_dir.clone(),
    ));
    let subscription_manager =
        Arc::new(keyrx_daemon::web::subscriptions::SubscriptionManager::new());

    // Create RPC event broadcaster
    let (rpc_event_tx, _) = tokio::sync::broadcast::channel(1000);

    let app_state = Arc::new(keyrx_daemon::web::AppState::new_with_test_mode(
        macro_recorder,
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
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 9867).into();
    log::info!("Starting web server on http://{}", addr);

    rt.block_on(async {
        match keyrx_daemon::web::serve(addr, event_tx, app_state).await {
            Ok(()) => {
                log::info!("Web server stopped");
                Ok(())
            }
            Err(e) => {
                log::error!("Web server error: {}", e);
                Err((
                    exit_codes::RUNTIME_ERROR,
                    format!("Web server error: {}", e),
                ))
            }
        }
    })
}

/// Handles the `run` subcommand - starts the daemon.
#[cfg(target_os = "linux")]
fn handle_run(
    config_path: &std::path::Path,
    debug: bool,
    test_mode: bool,
) -> Result<(), (i32, String)> {
    use keyrx_daemon::daemon::Daemon;
    use keyrx_daemon::platform::linux::LinuxSystemTray;
    use keyrx_daemon::platform::{SystemTray, TrayControlEvent};

    // Initialize logging
    init_logging(debug);

    if test_mode {
        log::info!("Test mode enabled - running with IPC infrastructure without keyboard capture");
        return handle_run_test_mode(config_path, debug);
    }

    log::info!(
        "Starting keyrx daemon with config: {}",
        config_path.display()
    );

    // Create platform instance
    let platform = keyrx_daemon::platform::create_platform().map_err(|e| {
        (
            keyrx_daemon::daemon::ExitCode::RuntimeError as i32,
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
            log::info!("Daemon will continue without system tray. Web UI is available at http://127.0.0.1:9867");
            None
        }
    };

    // Create broadcast channel for event streaming to WebSocket clients
    let (event_tx, _event_rx) = tokio::sync::broadcast::channel(1000);
    let event_tx_clone = event_tx.clone();
    let event_tx_for_broadcaster = event_tx.clone();

    // Create event broadcaster for real-time updates
    let event_broadcaster = keyrx_daemon::daemon::EventBroadcaster::new(event_tx_for_broadcaster);
    let running_for_broadcaster = daemon.running_flag();
    let latency_recorder_for_broadcaster = daemon.latency_recorder();

    // Wire the event broadcaster into the daemon for real-time event streaming
    daemon.set_event_broadcaster(event_broadcaster.clone());

    // Create AppState with dependencies for web API
    let macro_recorder = std::sync::Arc::new(keyrx_daemon::macro_recorder::MacroRecorder::new());

    // Determine config directory (always use standard location for profile management)
    let config_dir = {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("keyrx");
        path
    };

    // Initialize ProfileManager and ProfileService
    let profile_manager = match keyrx_daemon::config::ProfileManager::new(config_dir.clone()) {
        Ok(mgr) => std::sync::Arc::new(mgr),
        Err(e) => {
            log::warn!("Failed to initialize ProfileManager: {}. Profile operations will not be available.", e);
            // Continue without profile support - use a dummy
            return Err((
                keyrx_daemon::daemon::ExitCode::ConfigError as i32,
                format!("Failed to initialize ProfileManager: {}", e),
            ));
        }
    };
    let profile_service = std::sync::Arc::new(keyrx_daemon::services::ProfileService::new(
        std::sync::Arc::clone(&profile_manager),
    ));
    let device_service = std::sync::Arc::new(keyrx_daemon::services::DeviceService::new(
        config_dir.clone(),
    ));
    let config_service =
        std::sync::Arc::new(keyrx_daemon::services::ConfigService::new(profile_manager));
    let settings_service = std::sync::Arc::new(keyrx_daemon::services::SettingsService::new(
        config_dir.clone(),
    ));
    let simulation_service = std::sync::Arc::new(keyrx_daemon::services::SimulationService::new(
        config_dir.clone(),
    ));

    let subscription_manager =
        std::sync::Arc::new(keyrx_daemon::web::subscriptions::SubscriptionManager::new());

    // Create RPC event broadcaster for WebSocket RPC events (device/profile updates)
    let (rpc_event_tx, _) = tokio::sync::broadcast::channel(1000);

    let app_state = std::sync::Arc::new(keyrx_daemon::web::AppState::new(
        macro_recorder,
        profile_service,
        device_service,
        config_service,
        settings_service,
        simulation_service,
        subscription_manager,
        rpc_event_tx,
    ));

    // Start web server and event broadcasting in background (optional)
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
            tokio::spawn(keyrx_daemon::daemon::start_latency_broadcast_task(
                event_broadcaster,
                running_for_broadcaster,
                Some(latency_recorder_for_broadcaster),
            ));

            let addr: std::net::SocketAddr = ([127, 0, 0, 1], 9867).into();
            log::info!("Starting web server on http://{}", addr);
            match keyrx_daemon::web::serve(addr, event_tx_clone, app_state).await {
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
                        if let Err(e) = open_browser("http://127.0.0.1:9867") {
                            log::error!("Failed to open browser: {}", e);
                        }
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

/// Ensure only one instance of the daemon is running.
/// Kills any existing instance before starting.
/// Returns true if an old instance was killed.
#[cfg(target_os = "windows")]
fn ensure_single_instance(config_dir: &std::path::Path) -> bool {
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
#[cfg(target_os = "windows")]
fn cleanup_pid_file(config_dir: &std::path::Path) {
    let pid_file = config_dir.join("daemon.pid");
    let _ = std::fs::remove_file(&pid_file);
}

/// Handles the `run` subcommand in test mode - starts web server and IPC without keyboard capture.
#[cfg(target_os = "windows")]
fn handle_run_test_mode(_config_path: &std::path::Path, _debug: bool) -> Result<(), (i32, String)> {
    use keyrx_daemon::config::ProfileManager;
    use keyrx_daemon::ipc::commands::IpcCommandHandler;
    use keyrx_daemon::ipc::server::IpcServer;
    use std::sync::Arc;
    use tokio::sync::{Mutex, RwLock};

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
                exit_codes::CONFIG_ERROR,
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
            exit_codes::RUNTIME_ERROR,
            format!("Failed to create IPC server: {}", e),
        )
    })?;

    // Start IPC server
    ipc_server.start().map_err(|e| {
        (
            exit_codes::RUNTIME_ERROR,
            format!("Failed to start IPC server: {}", e),
        )
    })?;

    log::info!("IPC server started on {}", test_socket_path.display());

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        (
            exit_codes::RUNTIME_ERROR,
            format!("Failed to create tokio runtime: {}", e),
        )
    })?;

    // Clone handler for server thread
    let ipc_handler_for_server = Arc::clone(&ipc_handler);

    // Start IPC server connection handler in background
    std::thread::spawn(move || {
        let handler_fn = Arc::new(Mutex::new(
            move |request: keyrx_daemon::ipc::IpcRequest| -> Result<keyrx_daemon::ipc::IpcResponse, String> {
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

    // Create services for web API
    let macro_recorder = Arc::new(keyrx_daemon::macro_recorder::MacroRecorder::new());
    // Reuse the same ProfileManager instance for IPC and REST API
    let profile_service = Arc::new(keyrx_daemon::services::ProfileService::new(Arc::clone(
        &profile_manager,
    )));
    let device_service = Arc::new(keyrx_daemon::services::DeviceService::new(
        config_dir.clone(),
    ));
    let config_service = Arc::new(keyrx_daemon::services::ConfigService::new(Arc::clone(
        &profile_manager,
    )));
    let settings_service = Arc::new(keyrx_daemon::services::SettingsService::new(
        config_dir.clone(),
    ));
    let simulation_service = Arc::new(keyrx_daemon::services::SimulationService::new(
        config_dir.clone(),
    ));
    let subscription_manager =
        Arc::new(keyrx_daemon::web::subscriptions::SubscriptionManager::new());

    // Create RPC event broadcaster
    let (rpc_event_tx, _) = tokio::sync::broadcast::channel(1000);

    let app_state = Arc::new(keyrx_daemon::web::AppState::new_with_test_mode(
        macro_recorder,
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
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 9867).into();
    log::info!("Starting web server on http://{}", addr);

    rt.block_on(async {
        match keyrx_daemon::web::serve(addr, event_tx, app_state).await {
            Ok(()) => {
                log::info!("Web server stopped");
                Ok(())
            }
            Err(e) => {
                log::error!("Web server error: {}", e);
                Err((
                    exit_codes::RUNTIME_ERROR,
                    format!("Web server error: {}", e),
                ))
            }
        }
    })
}

/// Find an available port starting from the given port.
/// Tries ports in sequence: port, port+1, port+2, ... up to 10 attempts.
#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
fn handle_run(
    config_path: &std::path::Path,
    debug: bool,
    test_mode: bool,
) -> Result<(), (i32, String)> {
    use keyrx_daemon::daemon::Daemon;
    use keyrx_daemon::platform::windows::tray::TrayIconController;
    use keyrx_daemon::platform::{SystemTray, TrayControlEvent};
    use keyrx_daemon::services::SettingsService;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT,
    };

    // Initialize logging
    init_logging(debug);

    if test_mode {
        log::info!("Test mode enabled - running with IPC infrastructure without keyboard capture");
        return handle_run_test_mode(config_path, debug);
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
        let default_port = keyrx_daemon::services::DEFAULT_PORT;
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
    let platform = keyrx_daemon::platform::create_platform().map_err(|e| {
        (
            exit_codes::RUNTIME_ERROR,
            format!("Failed to create platform: {}", e),
        )
    })?;

    // Create the daemon
    let mut daemon = Daemon::new(platform, config_path).map_err(daemon_error_to_exit)?;

    // Create broadcast channel for event streaming to WebSocket clients
    let (event_tx, _event_rx) = tokio::sync::broadcast::channel(1000);
    let event_tx_clone = event_tx.clone();
    let event_tx_for_broadcaster = event_tx.clone();

    // Create event broadcaster for real-time updates
    let event_broadcaster = keyrx_daemon::daemon::EventBroadcaster::new(event_tx_for_broadcaster);
    let running_for_broadcaster = daemon.running_flag();
    let latency_recorder_for_broadcaster = daemon.latency_recorder();

    // Wire the event broadcaster into the daemon for real-time event streaming
    daemon.set_event_broadcaster(event_broadcaster.clone());

    // Start web server in background
    let macro_recorder = std::sync::Arc::new(keyrx_daemon::macro_recorder::MacroRecorder::new());

    // Initialize ProfileManager and ProfileService
    let profile_manager = match keyrx_daemon::config::ProfileManager::new(config_dir.clone()) {
        Ok(mgr) => std::sync::Arc::new(mgr),
        Err(e) => {
            log::warn!("Failed to initialize ProfileManager: {}. Profile operations will not be available.", e);
            // Continue without profile support - use a dummy
            return Err((
                keyrx_daemon::daemon::ExitCode::ConfigError as i32,
                format!("Failed to initialize ProfileManager: {}", e),
            ));
        }
    };
    let profile_service = std::sync::Arc::new(keyrx_daemon::services::ProfileService::new(
        std::sync::Arc::clone(&profile_manager),
    ));
    let device_service = std::sync::Arc::new(keyrx_daemon::services::DeviceService::new(
        config_dir.clone(),
    ));
    let config_service =
        std::sync::Arc::new(keyrx_daemon::services::ConfigService::new(profile_manager));
    let settings_service = std::sync::Arc::new(keyrx_daemon::services::SettingsService::new(
        config_dir.clone(),
    ));
    let simulation_service = std::sync::Arc::new(keyrx_daemon::services::SimulationService::new(
        config_dir.clone(),
    ));

    let subscription_manager =
        std::sync::Arc::new(keyrx_daemon::web::subscriptions::SubscriptionManager::new());

    // Create RPC event broadcaster for WebSocket RPC events (device/profile updates)
    let (rpc_event_tx, _) = tokio::sync::broadcast::channel(1000);

    let app_state = std::sync::Arc::new(keyrx_daemon::web::AppState::new(
        macro_recorder,
        profile_service,
        device_service,
        config_service,
        settings_service.clone(),
        simulation_service,
        subscription_manager,
        rpc_event_tx,
    ));

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
            tokio::spawn(keyrx_daemon::daemon::start_latency_broadcast_task(
                event_broadcaster,
                running_for_broadcaster,
                Some(latency_recorder_for_broadcaster),
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
            match keyrx_daemon::web::serve(addr, event_tx_clone, app_state).await {
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

    log::info!("Daemon initialized. Running message loop...");

    // Build web UI URL with actual port
    let web_ui_url = format!("http://127.0.0.1:{}", actual_port);

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

#[cfg(target_os = "windows")]
#[allow(dead_code)]
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

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn handle_run(
    _config_path: &std::path::Path,
    _debug: bool,
    _test_mode: bool,
) -> Result<(), (i32, String)> {
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'run' command is only available on Linux and Windows. \
         Build with --features linux or --features windows to enable."
            .to_string(),
    ))
}

/// Handles the `record` subcommand.
#[cfg(target_os = "linux")]
fn handle_record(
    output_path: &std::path::Path,
    device_path: Option<&std::path::Path>,
) -> Result<(), (i32, String)> {
    use keyrx_daemon::platform::linux::evdev_to_keycode;
    use serde::{Deserialize, Serialize};
    use std::fs::File;
    use std::io::Write;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::SystemTime;

    // If no device provided, list devices and return
    let Some(device_path) = device_path else {
        println!("No input device specified.");
        println!("Please choose a device from the list below and run:");
        println!(
            "  sudo keyrx_daemon record --output {} --device <PATH>",
            output_path.display()
        );
        println!();
        return handle_list_devices();
    };

    println!("Preparing to record from: {}", device_path.display());

    // Open the device
    let mut device = evdev::Device::open(device_path).map_err(|e| {
        (
            exit_codes::PERMISSION_ERROR,
            format!("Failed to open device {}: {}", device_path.display(), e),
        )
    })?;

    // Grab the device to prevent interference (optional, maybe we want non-exclusive?)
    // Spec says: "non-exclusive grab, so the daemon still works".
    // But if the daemon is running, it will grab it exclusively!
    // So we assume the daemon is NOT running when we record raw events?
    // Or we record from the daemon's input?
    // "User runs recorder: sudo keyrx_daemon record ..."
    // If the daemon is running, we can't open it if it grabbed it.
    // So the daemon must be stopped.
    // Therefore, grabbing it here is fine/good to prevent other inputs.
    // However, if we grab it, the OS won't receive keys, so the user can't do anything (like reproduce the bug in an app)!
    // CRITICAL: We must NOT grab the device if we want the user to interact with the OS.
    // BUT if we don't grab, and keyrx_daemon is NOT running, the OS gets raw events (QWERTY).
    // If the user wants to reproduce a bug in their LAYOUT, they need the daemon RUNNING.
    //
    // Re-reading spec: "Recorder Component... non-exclusive grab".
    // If the daemon is running, it has an EXCLUSIVE grab. We cannot open it even for reading usually?
    // `evdev` allows multiple readers if no one has grabbed it?
    //
    // Scenario 1: Reproduce a bug in the *Daemon's processing*.
    // The user runs the daemon. We want to capture what the daemon sees.
    // If the daemon is running, we can't easily snoop on `/dev/input/eventX` if it's grabbed.
    //
    // Scenario 2: Record a sequence to feed into tests.
    // The user stops the daemon. Runs `record`. Types keys. The OS sees raw keys (wrong layout).
    // The user types the *physical* keys that trigger the bug.
    // This is valid. The user physically presses keys. We record them.
    // Later, the test runner feeds these physical keys to the daemon logic.
    //
    // Conclusion: `record` mode assumes daemon is STOPPED. It does NOT grab the device (so user can see what they type, albeit unmapped, or maybe they type blindly/use on-screen keyboard).
    // Actually, if they type unmapped, it might trigger system shortcuts.
    // Best practice: Do NOT grab.
    // Warning: "Ensure keyrx_daemon is stopped before recording."

    println!("Recording started. Press Ctrl+C to stop.");
    println!("Warning: Ensure keyrx_daemon is stopped.");

    // Setup signal handler
    let running = Arc::new(AtomicBool::new(true));

    // We can't use signal-hook simple registration because we are in a loop.
    // We'll check the flag.
    // Need to register signal handler.
    // signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&running)) ...
    // But keyrx_daemon/Cargo.toml has signal-hook as dependency.
    // Let's use simple generic ctrl-c handler or signal-hook.

    // Using a crate-agnostic way or signal-hook since it's available.
    if let Err(e) = signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&running)) {
        eprintln!("Failed to register signal handler: {}", e);
    }

    #[derive(Serialize, Deserialize)]
    struct Metadata {
        version: String,
        timestamp: String,
        device_name: String,
    }

    #[derive(Serialize, Deserialize)]
    struct Recording {
        metadata: Metadata,
        events: Vec<keyrx_core::runtime::KeyEvent>,
    }

    let mut captured_events = Vec::new();
    let start_time = std::time::Instant::now();

    // Event loop
    while running.load(Ordering::SeqCst) {
        // Non-blocking read or read with timeout?
        // evdev crate `fetch_events` is blocking by default?
        // It uses `read`. We can use `nix::poll` (as in EvdevInput) or just blocking read if we handle signals.
        // But blocking read won't return on SIGINT immediately unless it gets EINTR.
        // Let's rely on standard read returning.

        match device.fetch_events() {
            Ok(iterator) => {
                for ev in iterator {
                    // Filter for key events
                    if ev.event_type() == evdev::EventType::KEY {
                        let code = ev.code();
                        let value = ev.value(); // 0=Release, 1=Press, 2=Repeat

                        if value == 2 {
                            continue;
                        } // Ignore repeats for now

                        if let Some(keycode) = evdev_to_keycode(code) {
                            let event_type = if value == 1 {
                                keyrx_core::runtime::KeyEventType::Press
                            } else {
                                keyrx_core::runtime::KeyEventType::Release
                            };

                            // Calculate relative time
                            let timestamp_us = start_time.elapsed().as_micros() as u64;

                            let key_event =
                                keyrx_core::runtime::KeyEvent::press(keycode) // Type fixed below
                                    .with_timestamp(timestamp_us);

                            // Construct properly with correct type
                            let final_event =
                                if event_type == keyrx_core::runtime::KeyEventType::Press {
                                    key_event // Default is press
                                } else {
                                    keyrx_core::runtime::KeyEvent::release(keycode)
                                        .with_timestamp(timestamp_us)
                                };

                            print!("\rCaptured: {:?}     ", final_event.keycode());
                            std::io::stdout().flush().ok();

                            captured_events.push(final_event);
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                // Signal received
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Should not happen in blocking mode
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => {
                eprintln!("\nError reading device: {}", e);
                break;
            }
        }
    }

    println!(
        "\nRecording stopped. Saving {} events...",
        captured_events.len()
    );

    let recording = Recording {
        metadata: Metadata {
            version: "1.0".to_string(),
            timestamp: humantime::format_rfc3339(SystemTime::now()).to_string(),
            device_name: device.name().unwrap_or("Unknown").to_string(),
        },
        events: captured_events,
    };

    let json = serde_json::to_string_pretty(&recording).map_err(|e| {
        (
            exit_codes::RUNTIME_ERROR,
            format!("Failed to serialize recording: {}", e),
        )
    })?;

    let mut file = File::create(output_path).map_err(|e| {
        (
            exit_codes::PERMISSION_ERROR,
            format!("Failed to create output file: {}", e),
        )
    })?;

    file.write_all(json.as_bytes()).map_err(|e| {
        (
            exit_codes::RUNTIME_ERROR,
            format!("Failed to write to file: {}", e),
        )
    })?;

    println!("Saved to {}", output_path.display());
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn handle_record(
    _output: &std::path::Path,
    _device: Option<&std::path::Path>,
) -> Result<(), (i32, String)> {
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'record' command is only available on Linux. \
         Build with --features linux to enable."
            .to_string(),
    ))
}

/// Handles the `list-devices` subcommand - lists input devices.
#[cfg(target_os = "linux")]
fn handle_list_devices() -> Result<(), (i32, String)> {
    use keyrx_daemon::device_manager::enumerate_keyboards;

    // Get all keyboard devices
    let keyboards = enumerate_keyboards().map_err(|e| {
        (
            exit_codes::PERMISSION_ERROR,
            format!("Failed to enumerate devices: {}", e),
        )
    })?;

    if keyboards.is_empty() {
        println!("No keyboard devices found.");
        println!();
        println!("This could mean:");
        println!("  - No keyboards are connected");
        println!("  - Permission denied to read /dev/input/event* devices");
        println!();
        println!("To fix permission issues, either:");
        println!("  - Run as root (for testing only)");
        println!("  - Add your user to the 'input' group: sudo usermod -aG input $USER");
        println!("  - Install the udev rules: see docs/LINUX_SETUP.md");
        return Ok(());
    }

    println!("Available keyboard devices:");
    println!();
    println!("{:<30} {:<25} SERIAL", "PATH", "NAME");
    println!("{}", "-".repeat(80));

    for keyboard in &keyboards {
        let serial_display = keyboard.serial.as_deref().unwrap_or("-");
        println!(
            "{:<30} {:<25} {}",
            keyboard.path.display(),
            truncate_string(&keyboard.name, 24),
            serial_display
        );
    }

    println!();
    println!("Found {} keyboard device(s).", keyboards.len());
    println!();
    println!("Tip: Use patterns in your configuration to match devices:");
    println!("  - \"*\" matches all keyboards");
    println!("  - \"USB*\" matches devices with USB in name/serial");
    println!("  - Exact name match for specific devices");

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn handle_list_devices() -> Result<(), (i32, String)> {
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'list-devices' command is only available on Linux. \
         Build with --features linux to enable."
            .to_string(),
    ))
}

/// Handles the `validate` subcommand - validates config without grabbing.
#[cfg(target_os = "linux")]
fn handle_validate(config_path: &std::path::Path) -> Result<(), (i32, String)> {
    use keyrx_daemon::config_loader::load_config;
    use keyrx_daemon::device_manager::{enumerate_keyboards, match_device};

    println!("Validating configuration: {}", config_path.display());
    println!();

    // Step 1: Load and validate the configuration
    println!("1. Loading configuration...");
    let config = load_config(config_path).map_err(|e| {
        (
            exit_codes::CONFIG_ERROR,
            format!("Failed to load configuration: {}", e),
        )
    })?;

    println!(
        "   Configuration loaded: {} device pattern(s)",
        config.devices.len()
    );

    // Print the device patterns
    for (i, device_config) in config.devices.iter().enumerate() {
        println!(
            "   [{:>2}] Pattern: \"{}\" ({} mapping(s))",
            i + 1,
            device_config.identifier.pattern,
            device_config.mappings.len()
        );
    }
    println!();

    // Step 2: Enumerate keyboard devices
    println!("2. Enumerating keyboard devices...");
    let keyboards = enumerate_keyboards().map_err(|e| {
        (
            exit_codes::PERMISSION_ERROR,
            format!("Failed to enumerate devices: {}", e),
        )
    })?;

    if keyboards.is_empty() {
        println!("   No keyboard devices found.");
        println!();
        println!("This could mean:");
        println!("  - No keyboards are connected");
        println!("  - Permission denied to read /dev/input/event* devices");
        println!();
        println!("To fix permission issues, either:");
        println!("  - Run as root (for testing only)");
        println!("  - Add your user to the 'input' group: sudo usermod -aG input $USER");
        println!("  - Install the udev rules: see docs/LINUX_SETUP.md");
        return Ok(());
    }

    println!("   Found {} keyboard device(s)", keyboards.len());
    println!();

    // Step 3: Match devices against patterns
    println!("3. Matching devices to configuration patterns...");
    println!();

    let mut matched_count = 0;
    let mut unmatched_devices = Vec::new();

    for keyboard in &keyboards {
        // Check each pattern in order (priority)
        let mut matched_pattern: Option<&str> = None;

        for device_config in config.devices.iter() {
            let pattern = device_config.identifier.pattern.as_str();
            if match_device(keyboard, pattern) {
                matched_pattern = Some(pattern);
                break; // First match wins (priority ordering)
            }
        }

        if let Some(pattern) = matched_pattern {
            println!(
                "   [MATCH] {} -> pattern \"{}\"",
                keyboard.path.display(),
                pattern
            );
            println!("           Name: {}", keyboard.name);
            if let Some(ref serial) = keyboard.serial {
                println!("           Serial: {}", serial);
            }
            matched_count += 1;
        } else {
            unmatched_devices.push(keyboard);
        }
    }

    println!();

    // Show unmatched devices as warnings
    if !unmatched_devices.is_empty() {
        println!("   Unmatched devices (will not be remapped):");
        for device in &unmatched_devices {
            println!("   [SKIP]  {}", device.path.display());
            println!("           Name: {}", device.name);
        }
        println!();
    }

    // Final result
    println!("{}", "=".repeat(60));
    if matched_count > 0 {
        println!(
            "RESULT: Configuration is valid. {} of {} device(s) matched.",
            matched_count,
            keyboards.len()
        );
        println!();
        println!(
            "Run 'keyrx_daemon run --config {}' to start remapping.",
            config_path.display()
        );
    } else {
        println!("WARNING: Configuration is valid, but no devices matched any pattern.");
        println!();
        println!(
            "Check your device patterns. Use 'keyrx_daemon list-devices' to see available devices."
        );
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn handle_validate(_config_path: &std::path::Path) -> Result<(), (i32, String)> {
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'validate' command is only available on Linux. \
         Build with --features linux to enable."
            .to_string(),
    ))
}

/// Initializes the logging system.
#[cfg(any(target_os = "linux", target_os = "windows"))]
fn init_logging(debug: bool) {
    use env_logger::Builder;
    use log::LevelFilter;

    let level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    Builder::new()
        .filter_level(level)
        .format_timestamp_millis()
        .init();
}

/// Converts a DaemonError to an exit code and message.
#[cfg(any(target_os = "linux", target_os = "windows"))]
fn daemon_error_to_exit(error: keyrx_daemon::daemon::DaemonError) -> (i32, String) {
    use keyrx_daemon::daemon::DaemonError;

    match &error {
        DaemonError::Config(_) => (exit_codes::CONFIG_ERROR, error.to_string()),
        DaemonError::PermissionError(_) => (exit_codes::PERMISSION_ERROR, error.to_string()),
        DaemonError::Platform(plat_err) => {
            // Check if it's a permission error
            if plat_err.to_string().contains("permission")
                || plat_err.to_string().contains("Permission")
            {
                (exit_codes::PERMISSION_ERROR, error.to_string())
            } else {
                (exit_codes::CONFIG_ERROR, error.to_string())
            }
        }
        _ => (exit_codes::RUNTIME_ERROR, error.to_string()),
    }
}

/// Truncates a string to the specified length, adding "..." if truncated.
#[cfg(target_os = "linux")]
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s[..max_len].to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
