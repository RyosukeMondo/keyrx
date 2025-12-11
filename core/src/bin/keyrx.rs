#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
//! KeyRx CLI binary
//!
//! This binary uses println! and eprintln! for user-facing output,
//! which is intentional and distinct from internal logging.
//!
//! # Module Organization
//! - `commands_core`: Core engine commands (run, simulate, check, discover)
//! - `commands_config`: Configuration commands (devices, hardware, layout, keymap, runtime)
//! - `commands_test`: Testing commands (test, replay, analyze, uat, regression, doctor, repl)

mod commands_config;
mod commands_core;
mod commands_test;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use keyrx_core::cli::{CommandContext, CommandResult, OutputFormat, Verbosity};
use keyrx_core::config::{load_config, Config};
use keyrx_core::observability::StructuredLogger;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{debug, error, info};

use commands_config::{
    DeviceCommandAction, HardwareCommandAction, KeymapCommandAction, LayoutCommandAction,
    RuntimeCommandAction,
};
use commands_test::{CiCheckOptions, GoldenCommandAction, UatOptions};

#[derive(Parser)]
#[command(name = "keyrx")]
#[command(about = "KeyRx - The Ultimate Input Remapping Engine")]
#[command(version)]
#[command(after_help = "\
EXIT CODES:
  0 - Success
  1 - General error
  2 - Assertion failed (tests/validation)
  3 - Device not found
  4 - Permission denied
  5 - Timeout
  6 - Invalid argument
  7 - Configuration error
  101 - Panic (internal error)

LOGGING:
  Control logging with environment variables:
    RUST_LOG=<level>           Set log level (trace, debug, info, warn, error)
    KEYRX_LOG_FORMAT=<format>  Set log format (pretty, json)

  Examples:
    RUST_LOG=debug keyrx run --script my_config.rhai
    RUST_LOG=keyrx_core=trace keyrx doctor

For detailed exit code information, run:
  keyrx exit-codes
  keyrx exit-codes --json
")]
struct Cli {
    /// Output format (human, json, or yaml)
    #[arg(
        long = "output-format",
        visible_alias = "format",
        default_value = "human",
        value_parser = ["human", "json", "yaml"],
        global = true,
        conflicts_with = "json"
    )]
    output_format: String,

    /// Shortcut for JSON output (equivalent to --output-format json)
    #[arg(long, global = true, conflicts_with = "output_format")]
    json: bool,

    /// Path to configuration file (default: ~/.config/keyrx/config.toml)
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate and lint a Rhai script
    Check {
        /// Path to the script file
        script: PathBuf,
    },

    /// Manage devices and bindings
    Devices {
        #[command(subcommand)]
        command: Option<DeviceCommands>,
    },

    /// Hardware detection, profiles, and calibration
    Hardware {
        #[command(subcommand)]
        command: HardwareCommands,
    },

    /// Manage virtual layouts
    Layout {
        #[command(subcommand)]
        command: Option<LayoutCommands>,
    },

    /// Manage logical keymaps
    Keymap {
        #[command(subcommand)]
        command: Option<KeymapCommands>,
    },

    /// Inspect and modify runtime profile slots
    Runtime {
        #[command(subcommand)]
        command: Option<RuntimeCommands>,
    },

    /// Show all exit codes with descriptions
    ExitCodes,

    /// Generate API documentation
    Docs {
        /// Output format (markdown, html, json)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Output directory (default: ./docs)
        #[arg(short, long, default_value = "docs")]
        output: PathBuf,
    },

    /// Run the engine in headless mode
    Run {
        /// Path to the script file
        #[arg(short, long)]
        script: Option<PathBuf>,

        /// Enable debug output
        #[arg(short, long)]
        debug: bool,

        /// Run without keyboard capture (for CI/daemon mode). Use 'keyrx simulate' for interactive testing.
        #[arg(long, alias = "mock")]
        no_capture: bool,

        /// Load and validate script, then exit immediately
        #[arg(long)]
        validate_only: bool,

        /// Path to keyboard device (e.g., /dev/input/event3). Auto-detects if not specified.
        #[arg(long)]
        device: Option<PathBuf>,

        /// Record session to a .krx file for replay/analysis
        #[arg(long)]
        record: Option<PathBuf>,

        /// Export OpenTelemetry traces to file (requires otel-tracing feature)
        #[arg(long)]
        trace: Option<PathBuf>,

        /// Override tap timeout in milliseconds (valid: 50-1000)
        #[arg(long)]
        tap_timeout: Option<u32>,

        /// Override combo timeout in milliseconds (valid: 10-200)
        #[arg(long)]
        combo_timeout: Option<u32>,

        /// Override hold delay in milliseconds (valid: 0-500)
        #[arg(long)]
        hold_delay: Option<u32>,

        /// Disable compiled script caching for this run
        #[arg(long)]
        no_cache: bool,

        /// Clear the compiled script cache before running
        #[arg(long)]
        clear_cache: bool,
    },

    /// Inspect current engine state
    State {
        /// Show layer information
        #[arg(long)]
        layers: bool,

        /// Show modifier information
        #[arg(long)]
        modifiers: bool,

        /// Show pending tap-hold and combo decisions
        #[arg(long)]
        pending: bool,

        /// Path to the script file to hydrate engine state
        #[arg(short, long)]
        script: Option<PathBuf>,
    },

    /// Run self-diagnostics
    Doctor {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start interactive REPL
    Repl,

    /// Run latency benchmark
    Bench {
        /// Number of iterations
        #[arg(long, default_value = "10000")]
        iterations: usize,

        /// Path to the script file
        #[arg(short, long)]
        script: Option<PathBuf>,

        /// Generate a flame graph SVG (writes bench-flamegraph.svg)
        #[arg(long)]
        flamegraph: bool,

        /// Generate an allocation report JSON (writes bench-allocations.json)
        #[arg(long)]
        allocations: bool,
    },

    /// Simulate key events without real keyboard
    Simulate {
        /// Comma-separated list of keys to simulate (e.g., "A,B,CapsLock")
        #[arg(short, long, required_unless_present = "interactive")]
        input: Option<String>,

        /// Path to the script file
        #[arg(short, long)]
        script: Option<PathBuf>,

        /// Hold duration in milliseconds for each key (overrides default)
        #[arg(long)]
        hold_ms: Option<u64>,

        /// Treat input keys as a simultaneous combo
        #[arg(long)]
        combo: bool,

        /// Start interactive REPL-style simulation mode
        #[arg(long, short = 'I')]
        interactive: bool,
    },

    /// Discover a keyboard layout and write a device profile
    Discover {
        /// Target device vendor:product (hex or decimal). If omitted, auto-detects the first keyboard.
        #[arg(long)]
        device: Option<String>,

        /// Force re-discovery even if a profile already exists.
        #[arg(long)]
        force: bool,

        /// Assume yes for confirmations.
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Run tests in a Rhai script
    Test {
        /// Path to the script file containing test functions
        #[arg(short, long)]
        script: PathBuf,

        /// Filter tests by name pattern (supports * wildcard, e.g., "test_capslock*")
        #[arg(short, long)]
        filter: Option<String>,

        /// Watch the script file and re-run tests on change
        #[arg(short, long)]
        watch: bool,
    },

    /// Replay a recorded session from a .krx file
    Replay {
        /// Path to the .krx session file
        session: PathBuf,

        /// Verify that outputs match the recorded outputs
        #[arg(long)]
        verify: bool,

        /// Replay speed multiplier (0 = instant, 1 = realtime, 2 = 2x speed)
        #[arg(long, default_value = "0")]
        speed: f64,
    },

    /// Analyze a recorded session and generate timing diagrams
    Analyze {
        /// Path to the .krx session file
        session: PathBuf,

        /// Generate ASCII timing diagram
        #[arg(long)]
        diagram: bool,
    },

    /// Run User Acceptance Tests (UAT)
    Uat {
        /// Filter by category (can be specified multiple times)
        #[arg(short, long, value_delimiter = ',')]
        category: Vec<String>,

        /// Filter by priority (P0, P1, P2)
        #[arg(short, long, value_delimiter = ',')]
        priority: Vec<String>,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,

        /// Stop on first failure
        #[arg(long)]
        fail_fast: bool,

        /// Run performance tests
        #[arg(long)]
        perf: bool,

        /// Run fuzz tests
        #[arg(long)]
        fuzz: bool,

        /// Fuzz test duration in seconds
        #[arg(long, default_value = "60")]
        fuzz_duration: u64,

        /// Fuzz test sequence count (overrides duration)
        #[arg(long)]
        fuzz_count: Option<u64>,

        /// Generate coverage report
        #[arg(long)]
        coverage: bool,

        /// Generate full report
        #[arg(long)]
        report: bool,

        /// Report format (html, md, json)
        #[arg(long, default_value = "html")]
        report_format: String,

        /// Report output path
        #[arg(long)]
        report_output: Option<PathBuf>,

        /// Quality gate to enforce (alpha, beta, ga)
        #[arg(long)]
        gate: Option<String>,
    },

    /// Golden session management for regression testing
    Golden {
        #[command(subcommand)]
        command: GoldenCommands,
    },

    /// Verify all golden sessions for regressions
    Regression {
        /// Custom golden sessions directory
        #[arg(long)]
        golden_dir: Option<PathBuf>,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Run complete CI check (all tests + quality gates)
    CiCheck {
        /// Quality gate to enforce (default, alpha, beta, rc, ga)
        #[arg(long)]
        gate: Option<String>,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,

        /// Skip unit tests
        #[arg(long)]
        skip_unit: bool,

        /// Skip integration tests
        #[arg(long)]
        skip_integration: bool,

        /// Skip UAT tests
        #[arg(long)]
        skip_uat: bool,

        /// Skip regression tests
        #[arg(long)]
        skip_regression: bool,

        /// Skip performance tests
        #[arg(long)]
        skip_perf: bool,
    },

    /// Migrate profiles from old version to new version
    Migrate {
        /// Source version to migrate from (only 'v1' is supported)
        #[arg(long, default_value = "v1")]
        from: String,

        /// Create backup of old profiles before migration
        #[arg(long)]
        backup: bool,
    },
}

#[derive(Subcommand, Clone)]
enum HardwareCommands {
    /// List stored hardware wiring profiles
    List,

    /// Create or update a hardware wiring profile from JSON (file path or "-")
    Define {
        /// Path to hardware profile JSON (use "-" to read from stdin)
        #[arg(value_name = "PATH")]
        source: PathBuf,
    },

    /// Wire a scancode to a virtual key within a hardware profile
    Wire {
        /// Hardware profile identifier
        #[arg(long)]
        profile: String,

        /// Scancode to map (hex like 0x04 or decimal)
        #[arg(long, value_parser = parse_hex_or_decimal_u16)]
        scancode: u16,

        /// Virtual key identifier to assign (omit with --clear to remove)
        #[arg(long)]
        virtual_key: Option<String>,

        /// Clear an existing mapping for the scancode
        #[arg(long, action = ArgAction::SetTrue)]
        clear: bool,
    },

    /// Detect connected keyboards and suggest timing profiles
    Detect,

    /// Resolve profiles for detected or specific hardware
    Profile {
        /// Vendor ID (hex like 0x1b1c or decimal)
        #[arg(long, value_parser = parse_hex_or_decimal_u16)]
        vendor_id: Option<u16>,

        /// Product ID (hex like 0x1b2e or decimal)
        #[arg(long, value_parser = parse_hex_or_decimal_u16)]
        product_id: Option<u16>,
    },

    /// Run calibration using latency samples (microseconds)
    Calibrate {
        /// Vendor ID (hex like 0x1b1c or decimal)
        #[arg(long, value_parser = parse_hex_or_decimal_u16)]
        vendor_id: Option<u16>,

        /// Product ID (hex like 0x1b2e or decimal)
        #[arg(long, value_parser = parse_hex_or_decimal_u16)]
        product_id: Option<u16>,

        /// Warmup samples to discard
        #[arg(long, default_value_t = 3)]
        warmup_samples: usize,

        /// Samples to keep for optimization
        #[arg(long, default_value_t = 25)]
        sample_count: usize,

        /// Max duration for calibration run (seconds)
        #[arg(long, default_value_t = 30)]
        max_duration_secs: u64,

        /// Latency samples in microseconds (repeatable)
        #[arg(long = "latency-us")]
        latencies: Vec<u64>,

        /// Path to newline-delimited latency samples (microseconds)
        #[arg(long)]
        samples_file: Option<PathBuf>,
    },
}

#[derive(Subcommand, Clone, Default)]
enum LayoutCommands {
    /// List all virtual layouts
    #[default]
    List,

    /// Show a specific virtual layout by id
    Show {
        /// Layout identifier
        id: String,
    },

    /// Create or update a virtual layout from JSON (file path or "-")
    Create {
        /// Path to layout JSON (use "-" to read from stdin)
        #[arg(value_name = "PATH")]
        source: PathBuf,
    },
}

#[derive(Subcommand, Clone, Default)]
enum KeymapCommands {
    /// List all keymaps
    #[default]
    List,

    /// Show a specific keymap by id
    Show {
        /// Keymap identifier
        id: String,
    },

    /// Set or clear a binding in a keymap layer
    Map {
        /// Keymap identifier
        #[arg(long)]
        keymap: String,

        /// Layer name (creates the layer if missing)
        #[arg(long)]
        layer: String,

        /// Virtual key identifier to bind
        #[arg(long = "virtual-key")]
        virtual_key: String,

        /// Action binding to apply (key:<code>, macro:<text>, layer-toggle:<layer>, or transparent)
        #[arg(long, required_unless_present = "clear")]
        action: Option<String>,

        /// Clear the binding instead of setting it
        #[arg(long, action = ArgAction::SetTrue)]
        clear: bool,
    },
}

#[derive(Subcommand, Clone, Default)]
enum RuntimeCommands {
    /// List runtime devices and active slots
    #[default]
    Devices,

    /// Add or update a runtime slot for a device
    SlotAdd {
        /// Vendor ID (hex like 0x1b1c or decimal)
        #[arg(long, value_parser = parse_hex_or_decimal_u16)]
        vendor_id: u16,

        /// Product ID (hex like 0x1b2e or decimal)
        #[arg(long, value_parser = parse_hex_or_decimal_u16)]
        product_id: u16,

        /// Optional device serial number to disambiguate identical models
        #[arg(long)]
        serial: Option<String>,

        /// Slot identifier
        #[arg(long)]
        slot: String,

        /// Hardware profile id to attach
        #[arg(long = "hardware-profile")]
        hardware_profile: String,

        /// Keymap id to attach
        #[arg(long)]
        keymap: String,

        /// Slot priority (higher wins). Defaults to append order.
        #[arg(long)]
        priority: Option<u32>,

        /// Whether the slot is active
        #[arg(long, default_value_t = true)]
        active: bool,
    },

    /// Remove a runtime slot
    SlotRemove {
        /// Vendor ID (hex like 0x1b1c or decimal)
        #[arg(long, value_parser = parse_hex_or_decimal_u16)]
        vendor_id: u16,

        /// Product ID (hex like 0x1b2e or decimal)
        #[arg(long, value_parser = parse_hex_or_decimal_u16)]
        product_id: u16,

        /// Optional device serial number to disambiguate identical models
        #[arg(long)]
        serial: Option<String>,

        /// Slot identifier to remove
        #[arg(long)]
        slot: String,
    },

    /// Toggle a runtime slot's active flag
    SlotActive {
        /// Vendor ID (hex like 0x1b1c or decimal)
        #[arg(long, value_parser = parse_hex_or_decimal_u16)]
        vendor_id: u16,

        /// Product ID (hex like 0x1b2e or decimal)
        #[arg(long, value_parser = parse_hex_or_decimal_u16)]
        product_id: u16,

        /// Optional device serial number to disambiguate identical models
        #[arg(long)]
        serial: Option<String>,

        /// Slot identifier to toggle
        #[arg(long)]
        slot: String,

        /// Desired active state (true/false)
        #[arg(long)]
        active: bool,
    },
}

#[derive(Subcommand, Clone, Default)]
enum DeviceCommands {
    /// List all persisted device bindings
    #[default]
    List,

    /// Show details for a device identity
    Show {
        /// Device identity key (VID:PID:SERIAL)
        device: String,
    },

    /// Set or clear a user label for a device
    Label {
        /// Device identity key (VID:PID:SERIAL)
        device: String,

        /// New label value
        #[arg(required_unless_present = "clear")]
        label: Option<String>,

        /// Clear the label instead of setting one
        #[arg(long, action = ArgAction::SetTrue)]
        clear: bool,
    },

    /// Toggle remapping for a device
    Remap {
        /// Device identity key (VID:PID:SERIAL)
        device: String,

        /// Desired remap state
        #[arg(value_enum)]
        state: RemapState,
    },

    /// Assign a profile to a device
    Assign {
        /// Device identity key (VID:PID:SERIAL)
        device: String,

        /// Profile ID to assign
        profile: String,
    },

    /// Remove any profile assignment for a device
    Unassign {
        /// Device identity key (VID:PID:SERIAL)
        device: String,
    },
}

#[derive(Clone, ValueEnum)]
enum RemapState {
    On,
    Off,
}

impl RemapState {
    fn enabled(&self) -> bool {
        matches!(self, RemapState::On)
    }
}

/// Golden session subcommands.
#[derive(Subcommand)]
enum GoldenCommands {
    /// Record a new golden session from a script
    Record {
        /// Name of the golden session (alphanumeric, underscores, hyphens)
        name: String,

        /// Path to the script that generates events
        #[arg(short, long)]
        script: PathBuf,
    },

    /// Verify an existing golden session
    Verify {
        /// Name of the golden session to verify
        name: String,

        /// Path to the script to run for verification (optional)
        #[arg(short, long)]
        script: Option<PathBuf>,
    },

    /// Update an existing golden session
    Update {
        /// Name of the golden session to update
        name: String,

        /// Path to the script that generates events
        #[arg(short, long)]
        script: PathBuf,

        /// Confirm the update (required to overwrite existing session)
        #[arg(long)]
        confirm: bool,
    },

    /// List all golden sessions
    List,
}

fn parse_format(s: &str, json_flag: bool) -> OutputFormat {
    if json_flag {
        return OutputFormat::Json;
    }

    match s.to_lowercase().as_str() {
        "json" => OutputFormat::Json,
        "yaml" | "yml" => OutputFormat::Yaml,
        _ => OutputFormat::Human,
    }
}

fn parse_hex_or_decimal_u16(value: &str) -> Result<u16, String> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16).map_err(|err| format!("Invalid hex value '{value}': {err}"))
    } else {
        trimmed.parse::<u16>().or_else(|_| {
            u16::from_str_radix(trimmed, 16)
                .map_err(|err| format!("Invalid number or hex value '{value}': {err}"))
        })
    }
}

fn install_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        // Extract panic location
        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        // Extract panic message
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic message".to_string()
        };

        // Log panic at error level (if logger is initialized)
        error!(
            location = %location,
            message = %message,
            "Panic occurred"
        );

        // Print to stderr for visibility (works even if tracing isn't initialized yet)
        eprintln!("Error: Panic at {}: {}", location, message);
        eprintln!("This is a bug. Please report it at: https://github.com/keyrx/keyrx/issues");

        // Exit with code 101 (Rust panic convention)
        std::process::exit(101);
    }));
}

#[tokio::main]
async fn main() -> ExitCode {
    // Install panic handler to catch panics and return exit code 101
    install_panic_handler();

    let cli = Cli::parse();
    let format = parse_format(&cli.output_format, cli.json);

    // Initialize structured logger
    // Use human-readable format for CLI to make debugging easier
    // Logger level is controlled by RUST_LOG environment variable
    let log_format = match std::env::var("KEYRX_LOG_FORMAT")
        .unwrap_or_else(|_| "pretty".to_string())
        .as_str()
    {
        "json" => keyrx_core::observability::OutputFormat::Json,
        _ => keyrx_core::observability::OutputFormat::Pretty,
    };

    if let Err(e) = StructuredLogger::new().with_format(log_format).init() {
        // If logger init fails, print to stderr but continue
        eprintln!("Warning: Failed to initialize logger: {}", e);
    }

    debug!(
        output_format = ?format,
        config_path = ?cli.config,
        "CLI initialized"
    );

    // Load configuration from file (or use defaults)
    let config = load_config(cli.config.as_deref());

    info!(
        tap_timeout_ms = config.timing.tap_timeout_ms,
        combo_timeout_ms = config.timing.combo_timeout_ms,
        "Configuration loaded"
    );

    // Create command context
    let ctx = CommandContext::with_config(format, Verbosity::Normal, cli.config);

    // Execute command and get result
    let result = run_command(cli.command, &ctx, config).await;

    // Extract exit code and handle errors
    if result.is_success() {
        debug!("Command completed successfully");
        ExitCode::SUCCESS
    } else {
        let exit_code = result.exit_code();
        error!(
            exit_code = exit_code as u8,
            message_count = result.messages().len(),
            "Command failed"
        );

        // Print error messages
        for msg in result.messages() {
            eprintln!("Error: {msg}");
        }
        exit_code.into()
    }
}

async fn run_command(command: Commands, ctx: &CommandContext, config: Config) -> CommandResult<()> {
    // Log command execution start
    let command_name = get_command_name(&command);
    debug!(command = command_name, "Executing command");

    match command {
        // Core commands
        Commands::Check { script } => commands_core::execute_check(script, ctx),
        Commands::Run {
            script,
            debug,
            no_capture,
            validate_only,
            device,
            record,
            trace,
            tap_timeout,
            combo_timeout,
            hold_delay,
            no_cache,
            clear_cache,
        } => commands_core::execute_run(
            script,
            debug,
            no_capture,
            validate_only,
            device,
            record,
            trace,
            tap_timeout,
            combo_timeout,
            hold_delay,
            no_cache,
            clear_cache,
            config,
            ctx,
        ),
        Commands::Simulate {
            input,
            script,
            hold_ms,
            combo,
            interactive,
        } => commands_core::execute_simulate(input, script, hold_ms, combo, interactive, ctx).await,
        Commands::Discover { device, force, yes } => {
            commands_core::execute_discover(device, force, yes, ctx).await
        }
        Commands::Docs { format, output } => commands_core::execute_docs(format, output, ctx),
        Commands::State {
            layers,
            modifiers,
            pending,
            script,
        } => commands_core::execute_state(layers, modifiers, pending, script, ctx),
        Commands::Bench {
            iterations,
            script,
            flamegraph,
            allocations,
        } => commands_core::execute_bench(iterations, script, flamegraph, allocations, ctx),
        Commands::ExitCodes => commands_core::execute_exit_codes(ctx),

        // Config commands
        Commands::Devices { command } => {
            let device_cmd = command.unwrap_or_default();
            let action = match device_cmd {
                DeviceCommands::List => DeviceCommandAction::List,
                DeviceCommands::Show { device } => DeviceCommandAction::Show { device },
                DeviceCommands::Label {
                    device,
                    label,
                    clear,
                } => DeviceCommandAction::Label {
                    device,
                    label,
                    clear,
                },
                DeviceCommands::Remap { device, state } => DeviceCommandAction::Remap {
                    device,
                    enabled: state.enabled(),
                },
                DeviceCommands::Assign { device, profile } => {
                    DeviceCommandAction::Assign { device, profile }
                }
                DeviceCommands::Unassign { device } => DeviceCommandAction::Unassign { device },
            };
            commands_config::execute_devices(action, ctx)
        }
        Commands::Hardware { command } => {
            let action = match command {
                HardwareCommands::List => HardwareCommandAction::List,
                HardwareCommands::Define { source } => HardwareCommandAction::Define { source },
                HardwareCommands::Wire {
                    profile,
                    scancode,
                    virtual_key,
                    clear,
                } => HardwareCommandAction::Wire {
                    profile,
                    scancode,
                    virtual_key,
                    clear,
                },
                HardwareCommands::Detect => HardwareCommandAction::Detect,
                HardwareCommands::Profile {
                    vendor_id,
                    product_id,
                } => HardwareCommandAction::Profile {
                    vendor_id,
                    product_id,
                },
                HardwareCommands::Calibrate {
                    vendor_id,
                    product_id,
                    warmup_samples,
                    sample_count,
                    max_duration_secs,
                    latencies,
                    samples_file,
                } => HardwareCommandAction::Calibrate {
                    vendor_id,
                    product_id,
                    warmup_samples,
                    sample_count,
                    max_duration_secs,
                    latencies,
                    samples_file,
                },
            };
            commands_config::execute_hardware(action, ctx)
        }
        Commands::Layout { command } => {
            let layout_cmd = command.unwrap_or_default();
            let action = match layout_cmd {
                LayoutCommands::List => LayoutCommandAction::List,
                LayoutCommands::Show { id } => LayoutCommandAction::Show { id },
                LayoutCommands::Create { source } => LayoutCommandAction::Create { source },
            };
            commands_config::execute_layout(action, ctx)
        }
        Commands::Keymap { command } => {
            let keymap_cmd = command.unwrap_or_default();
            let action = match keymap_cmd {
                KeymapCommands::List => KeymapCommandAction::List,
                KeymapCommands::Show { id } => KeymapCommandAction::Show { id },
                KeymapCommands::Map {
                    keymap,
                    layer,
                    virtual_key,
                    action,
                    clear,
                } => KeymapCommandAction::Map {
                    keymap,
                    layer,
                    virtual_key,
                    action,
                    clear,
                },
            };
            commands_config::execute_keymap(action, ctx)
        }
        Commands::Runtime { command } => {
            let runtime_cmd = command.unwrap_or_default();
            let action = match runtime_cmd {
                RuntimeCommands::Devices => RuntimeCommandAction::Devices,
                RuntimeCommands::SlotAdd {
                    vendor_id,
                    product_id,
                    serial,
                    slot,
                    hardware_profile,
                    keymap,
                    priority,
                    active,
                } => RuntimeCommandAction::SlotAdd {
                    vendor_id,
                    product_id,
                    serial,
                    slot,
                    hardware_profile,
                    keymap,
                    priority,
                    active,
                },
                RuntimeCommands::SlotRemove {
                    vendor_id,
                    product_id,
                    serial,
                    slot,
                } => RuntimeCommandAction::SlotRemove {
                    vendor_id,
                    product_id,
                    serial,
                    slot,
                },
                RuntimeCommands::SlotActive {
                    vendor_id,
                    product_id,
                    serial,
                    slot,
                    active,
                } => RuntimeCommandAction::SlotActive {
                    vendor_id,
                    product_id,
                    serial,
                    slot,
                    active,
                },
            };
            commands_config::execute_runtime(action, ctx)
        }
        Commands::Migrate { from, backup } => {
            commands_config::execute_migrate(from, backup, ctx).await
        }

        // Test commands
        Commands::Doctor { verbose } => commands_test::execute_doctor(verbose, ctx),
        Commands::Repl => commands_test::execute_repl(ctx),
        Commands::Test {
            script,
            filter,
            watch,
        } => commands_test::execute_test(script, filter, watch, ctx),
        Commands::Replay {
            session,
            verify,
            speed,
        } => commands_test::execute_replay(session, verify, speed, ctx).await,
        Commands::Analyze { session, diagram } => {
            commands_test::execute_analyze(session, diagram, ctx)
        }
        Commands::Uat {
            category,
            priority,
            json,
            fail_fast,
            perf,
            fuzz,
            fuzz_duration,
            fuzz_count,
            coverage,
            report,
            report_format,
            report_output,
            gate,
        } => {
            let options = UatOptions {
                category,
                priority,
                json,
                fail_fast,
                perf,
                fuzz,
                fuzz_duration,
                fuzz_count,
                coverage,
                report,
                report_format,
                report_output,
                gate,
            };
            commands_test::execute_uat(options, ctx)
        }
        Commands::Golden { command } => {
            let action = match command {
                GoldenCommands::Record { name, script } => {
                    GoldenCommandAction::Record { name, script }
                }
                GoldenCommands::Verify { name, script } => {
                    GoldenCommandAction::Verify { name, script }
                }
                GoldenCommands::Update {
                    name,
                    script,
                    confirm,
                } => GoldenCommandAction::Update {
                    name,
                    script,
                    confirm,
                },
                GoldenCommands::List => GoldenCommandAction::List,
            };
            commands_test::execute_golden(action, ctx)
        }
        Commands::Regression { golden_dir, json } => {
            commands_test::execute_regression(golden_dir, json, ctx)
        }
        Commands::CiCheck {
            gate,
            json,
            skip_unit,
            skip_integration,
            skip_uat,
            skip_regression,
            skip_perf,
        } => {
            let options = CiCheckOptions {
                gate,
                json,
                skip_unit,
                skip_integration,
                skip_uat,
                skip_regression,
                skip_perf,
            };
            commands_test::execute_ci_check(options, ctx)
        }
    }
}

fn get_command_name(command: &Commands) -> &'static str {
    match command {
        Commands::Check { .. } => "check",
        Commands::Devices { .. } => "devices",
        Commands::Hardware { .. } => "hardware",
        Commands::Layout { .. } => "layout",
        Commands::Keymap { .. } => "keymap",
        Commands::Runtime { .. } => "runtime",
        Commands::ExitCodes => "exit-codes",
        Commands::Docs { .. } => "docs",
        Commands::Run { .. } => "run",
        Commands::State { .. } => "state",
        Commands::Doctor { .. } => "doctor",
        Commands::Repl => "repl",
        Commands::Bench { .. } => "bench",
        Commands::Simulate { .. } => "simulate",
        Commands::Discover { .. } => "discover",
        Commands::Test { .. } => "test",
        Commands::Replay { .. } => "replay",
        Commands::Analyze { .. } => "analyze",
        Commands::Uat { .. } => "uat",
        Commands::Golden { .. } => "golden",
        Commands::Regression { .. } => "regression",
        Commands::CiCheck { .. } => "ci-check",
        Commands::Migrate { .. } => "migrate",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_output_format_flag_and_alias() {
        let cli = Cli::try_parse_from(["keyrx", "check", "--output-format", "yaml", "script.rhai"])
            .expect("output-format flag should parse globally");
        assert_eq!(cli.output_format, "yaml");

        let cli = Cli::try_parse_from(["keyrx", "check", "--format", "json", "script.rhai"])
            .expect("format alias should still work");
        assert_eq!(cli.output_format, "json");
    }

    #[test]
    fn parses_json_shortcut_after_subcommand() {
        let cli = Cli::try_parse_from(["keyrx", "check", "script.rhai", "--json"])
            .expect("--json should be accepted globally");
        assert!(cli.json);
    }

    #[test]
    fn parse_format_defaults_to_human_on_unknown_values() {
        assert_eq!(parse_format("human", false), OutputFormat::Human);
        assert_eq!(parse_format("unknown", false), OutputFormat::Human);
    }

    #[test]
    fn parse_format_respects_json_flag_priority() {
        assert_eq!(parse_format("yaml", true), OutputFormat::Json);
        assert_eq!(parse_format("json", true), OutputFormat::Json);
    }

    #[test]
    fn parses_hex_or_decimal() {
        assert_eq!(parse_hex_or_decimal_u16("0x1b1c").unwrap(), 0x1b1c);
        assert_eq!(parse_hex_or_decimal_u16("1b1c").unwrap(), 0x1b1c);
        assert_eq!(parse_hex_or_decimal_u16("7000").unwrap(), 7000);
    }
}
