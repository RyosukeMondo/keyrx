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
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,

        /// Enable debug logging for verbose output.
        ///
        /// This increases log verbosity to show individual key events and
        /// detailed processing information. Useful for troubleshooting.
        #[arg(short, long)]
        debug: bool,
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

    let result = match cli.command {
        Commands::Run { config, debug } => handle_run(&config, debug),
        Commands::Devices(args) => {
            match keyrx_daemon::cli::devices::execute(args, None) {
                Ok(()) => Ok(()),
                Err(code) => Err((code, String::new())), // Error already printed by execute
            }
        }
        Commands::Profiles(args) => {
            match keyrx_daemon::cli::profiles::execute(args, None) {
                Ok(()) => Ok(()),
                Err(code) => Err((code, String::new())), // Error already printed by execute
            }
        }
        Commands::Config(args) => {
            match keyrx_daemon::cli::config::execute(args, None) {
                Ok(()) => Ok(()),
                Err(code) => Err((code, String::new())), // Error already printed by execute
            }
        }
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

/// Handles the `run` subcommand - starts the daemon.
#[cfg(target_os = "linux")]
fn handle_run(config_path: &std::path::Path, debug: bool) -> Result<(), (i32, String)> {
    use keyrx_daemon::daemon::Daemon;

    // Initialize logging
    init_logging(debug);

    log::info!(
        "Starting keyrx daemon with config: {}",
        config_path.display()
    );

    // Create and run the daemon
    let mut daemon = Daemon::new(config_path).map_err(daemon_error_to_exit)?;

    log::info!(
        "Daemon initialized with {} device(s)",
        daemon.device_count()
    );

    // Run the event loop
    daemon.run().map_err(daemon_error_to_exit)?;

    log::info!("Daemon stopped gracefully");
    Ok(())
}

#[cfg(target_os = "windows")]
fn handle_run(config_path: &std::path::Path, debug: bool) -> Result<(), (i32, String)> {
    use keyrx_daemon::daemon::Daemon;
    use keyrx_daemon::platform::windows::tray::TrayIconController;
    use keyrx_daemon::platform::{SystemTray, TrayControlEvent};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT,
    };

    // Initialize logging
    init_logging(debug);

    log::info!(
        "Starting keyrx daemon (Windows) with config: {}",
        config_path.display()
    );

    // Create the daemon
    let mut daemon = Daemon::new(config_path).map_err(daemon_error_to_exit)?;

    // Create the tray icon
    let tray = TrayIconController::new()
        .map_err(|e| (exit_codes::RUNTIME_ERROR, format!("Tray error: {}", e)))?;

    // Check for administrative privileges
    if !is_admin() {
        log::warn!("Daemon is not running with administrative privileges. Key remapping may not work for elevated applications.");
    }

    log::info!("Daemon initialized. Running message loop...");

    // Windows low-level hooks REQUIRE a message loop on the thread that installed them.
    // Our Daemon::new() calls grab() which installs the hook.
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        loop {
            // Process ALL available messages
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    return Ok(());
                }

                TranslateMessage(&msg);
                // WIN-BUG #4: Wrap message dispatch in catch_unwind to prevent
                // a panic in wnd_proc from terminating the entire process.
                let _ = std::panic::catch_unwind(|| {
                    DispatchMessageW(&msg);
                });
            }

            // WIN-BUG #4: Wrap event processing in catch_unwind.
            let process_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                daemon.process_pending_events()
            }));

            if let Ok(Err(e)) = process_result {
                log::error!("Error processing events: {}", e);
            } else if process_result.is_err() {
                log::error!("Panic occurred during event processing");
            }

            // Poll tray events
            if let Some(event) = tray.poll_event() {
                match event {
                    TrayControlEvent::Reload => {
                        log::info!("Reloading config...");
                        let _ = daemon.reload();
                    }
                    TrayControlEvent::Exit => {
                        log::info!("Exiting...");
                        return Ok(());
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
fn handle_run(_config_path: &std::path::Path, _debug: bool) -> Result<(), (i32, String)> {
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
        DaemonError::Device(dev_err) => {
            // Check if it's a permission error
            if dev_err.to_string().contains("permission")
                || dev_err.to_string().contains("Permission")
            {
                (exit_codes::PERMISSION_ERROR, error.to_string())
            } else {
                (exit_codes::CONFIG_ERROR, error.to_string())
            }
        }
        DaemonError::DiscoveryError(_) => (exit_codes::CONFIG_ERROR, error.to_string()),
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
