//! keyrx_daemon - OS-level keyboard remapping daemon
//!
//! This binary provides the main daemon interface for keyboard remapping.
//! It intercepts keyboard events via platform-specific APIs and injects
//! remapped events back to the system.
//!
//! # Subcommands
//!
//! - `run`: Start the daemon with a .krx configuration file
//! - `devices`: Manage device metadata (rename, set scope, set layout)
//! - `profiles`: Manage configuration profiles
//! - `config`: Manage key mappings and configuration
//! - `layers`: Manage layers
//! - `layouts`: Manage keyboard layouts
//! - `simulate`: Run deterministic simulation tests
//! - `test`: Run built-in test scenarios
//! - `status`: Query daemon status via IPC
//! - `state`: Inspect runtime state (modifier/lock state)
//! - `metrics`: Query daemon performance metrics
//! - `list-devices`: List available input devices
//! - `doctor`: Diagnose common setup problems and print fixes
//! - `validate`: Validate configuration and device matching
//! - `record`: Record input events to a file for replay testing

// Hide console window on Windows release builds
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use clap::{Parser, Subcommand};
use keyrx_daemon::cli::dispatcher::{self, Command};
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
    /// Linux emergency stop: if the keyboard stops responding, hold Left Ctrl
    /// + Right Ctrl + Escape (or, with one hand, hold Escape alone for 3
    /// seconds) to release every keyboard and stop the daemon.
    Run {
        /// Path to the .krx configuration file compiled by keyrx_compiler.
        /// If not specified, uses the active profile from the keyrx config
        /// directory (Linux: $XDG_CONFIG_HOME/keyrx or ~/.config/keyrx;
        /// Windows: %APPDATA%\keyrx). Override with KEYRX_CONFIG_DIR.
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,

        /// Enable debug logging for verbose output. Never logs which keys
        /// you type; add --log-keys for that.
        #[arg(short, long)]
        debug: bool,

        /// Also log the NAMES of the keys being typed (privacy-sensitive:
        /// the log becomes a keylogger). For debugging a mapping only.
        #[arg(long)]
        log_keys: bool,

        /// Do not watch the loaded profile's .rhai for edits. By default a
        /// saved edit is recompiled and applied within a couple of seconds; one
        /// that does not compile is reported and the running config kept.
        #[arg(long)]
        no_watch: bool,

        /// Enable test mode with IPC infrastructure but without keyboard capture.
        /// Only available in debug builds for security.
        #[arg(long)]
        test_mode: bool,

        #[command(flatten)]
        tuning: keyrx_daemon::daemon::options::OptionOverrides,
    },

    /// Manage device metadata (rename, set scope, set layout).
    Devices(keyrx_daemon::cli::devices::DevicesArgs),

    /// Manage configuration profiles (create, activate, delete, etc.).
    Profiles(keyrx_daemon::cli::profiles::ProfilesArgs),

    /// Manage key mappings and configuration.
    Config(keyrx_daemon::cli::config::ConfigArgs),

    /// Manage layers (create, rename, delete, show).
    Layers(keyrx_daemon::cli::layers::LayersArgs),

    /// Manage keyboard layouts (import, list, show KLE JSON).
    Layouts(keyrx_daemon::cli::layouts::LayoutsArgs),

    /// Run deterministic simulation tests.
    Simulate(keyrx_daemon::cli::simulate::SimulateArgs),

    /// Run built-in test scenarios.
    Test(keyrx_daemon::cli::test::TestArgs),

    /// Query daemon status via IPC.
    Status(keyrx_daemon::cli::status::StatusArgs),

    /// Inspect runtime state (modifier/lock state).
    State(keyrx_daemon::cli::state::StateArgs),

    /// Query daemon performance metrics.
    Metrics(keyrx_daemon::cli::metrics::MetricsArgs),

    /// List available input devices on the system.
    ListDevices {
        /// Also list software keyboards (other tools' virtual devices).
        #[arg(long)]
        all: bool,
    },

    /// Diagnose common setup problems (device access, uinput, udev rules,
    /// config, whether the daemon is running) and print the fix for each.
    Doctor {
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Validate configuration and device matching without grabbing devices.
    Validate {
        /// Path to the .krx configuration file to validate.
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
    },

    /// Record input events from a device to a file for replay testing.
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

fn main() {
    let cli = Cli::parse();

    // Validate test mode early for release builds
    #[cfg(not(debug_assertions))]
    if let Commands::Run {
        test_mode: true, ..
    } = &cli.command
    {
        eprintln!("Error: Test mode is only available in debug builds");
        process::exit(dispatcher::exit_codes::CONFIG_ERROR);
    }

    // Convert CLI commands to dispatcher commands
    let command = match cli.command {
        Commands::Run {
            config,
            debug,
            log_keys,
            no_watch,
            test_mode,
            tuning,
        } => Command::Run {
            config,
            debug,
            log_keys,
            no_watch,
            test_mode,
            tuning,
        },
        Commands::Devices(args) => Command::Devices(args),
        Commands::Profiles(args) => Command::Profiles(args),
        Commands::Config(args) => Command::Config(args),
        Commands::Layers(args) => Command::Layers(args),
        Commands::Layouts(args) => Command::Layouts(args),
        Commands::Simulate(args) => Command::Simulate(args),
        Commands::Test(args) => Command::Test(args),
        Commands::Status(args) => Command::Status(args),
        Commands::State(args) => Command::State(args),
        Commands::Metrics(args) => Command::Metrics(args),
        Commands::ListDevices { all } => Command::ListDevices { all },
        Commands::Doctor { json } => Command::Doctor { json },
        Commands::Validate { config } => Command::Validate { config },
        Commands::Record { output, device } => Command::Record { output, device },
    };

    // Dispatch command to handler
    let result = dispatcher::dispatch(command);

    // Handle result and exit
    match result {
        Ok(()) => process::exit(dispatcher::exit_codes::SUCCESS),
        Err((code, message)) => {
            if !message.is_empty() {
                eprintln!("Error: {}", message);
            }
            process::exit(code);
        }
    }
}
